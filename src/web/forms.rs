//! Native HTML form transport and deliberately small, escaping template helpers.
use crate::presentation::html_escape;
use std::collections::BTreeMap;

pub type Fields = BTreeMap<String, String>;

pub fn decode(body: &str) -> Result<Fields, String> {
    fn component(value: &str) -> Result<String, String> {
        let bytes = value.as_bytes();
        let mut decoded = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'+' => decoded.push(b' '),
                b'%' => {
                    let hex = bytes.get(i + 1..i + 3).ok_or("Truncated form escape")?;
                    let digit = |v: u8| (v as char).to_digit(16).ok_or("Invalid form escape");
                    decoded.push((digit(hex[0])? * 16 + digit(hex[1])?) as u8);
                    i += 2;
                }
                byte => decoded.push(byte),
            }
            i += 1;
        }
        String::from_utf8(decoded).map_err(|_| "Form must be UTF-8".to_string())
    }
    let mut fields = Fields::new();
    for pair in body.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        // A checkbox follows its hidden false value; the final value wins.
        fields.insert(component(key)?, component(value)?);
    }
    Ok(fields)
}

pub fn save_fields(text: &str) -> Fields {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().into(), value.trim().into()))
        .collect()
}

/// `<option>` list for an integer enum field. Out-of-range stored values are
/// kept as an extra "Other" option so a submit never silently rewrites them.
fn option_list(key: &str, fields: &Fields) -> String {
    use crate::model::WeaponKind;
    use crate::presentation::fonts::TextLanguage;
    use crate::presentation::options;
    let lang = match fields.get("text_language").map(String::as_str) {
        Some("und-Hani") => TextLanguage::Han,
        Some("und-Latn") => TextLanguage::Italic,
        _ => TextLanguage::Germanic,
    };
    let catalog = match key.strip_suffix(".location") {
        Some(prefix) if prefix.starts_with("weap") => {
            let kind = match fields.get(&format!("{prefix}.kind")).map(String::as_str) {
                Some("mine") => WeaponKind::Mine,
                Some("dc") => WeaponKind::DepthCharge,
                _ => WeaponKind::Torpedo,
            };
            options::weapon_locations_lang(kind, lang)
        }
        _ => match options::for_field_lang(key, lang) {
            Some(catalog) => catalog,
            None => return String::new(),
        },
    };
    let current = fields.get(key).map(|v| v.trim()).unwrap_or("");
    let mut html = String::new();
    let mut matched = false;
    for (i, label) in catalog.iter().enumerate() {
        let selected = current == i.to_string();
        matched |= selected;
        html.push_str(&format!(
            "<option value=\"{i}\"{}>{i} · {}</option>",
            if selected { " selected" } else { "" },
            html_escape(label)
        ));
    }
    if !matched && !current.is_empty() {
        let v = html_escape(current);
        html.push_str(&format!(
            "<option value=\"{v}\" selected>{v} · Other</option>"
        ));
    }
    html
}

/// Tokens are processed once from the template, never recursively from values.
/// `slots` contains only trusted server-rendered HTML. All field values escape.
pub fn render(template: &str, fields: &Fields, slots: &Fields) -> String {
    let mut result = String::with_capacity(template.len());
    let mut tail = template;
    while let Some((before, rest)) = tail.split_once("{{") {
        result.push_str(before);
        let Some((token, after)) = rest.split_once("}}") else {
            result.push_str("{{");
            result.push_str(rest);
            return result;
        };
        if let Some(key) = token.strip_prefix("label:") {
            let lang = match fields.get("text_language").map(String::as_str) {
                Some("und-Hani") => crate::presentation::fonts::TextLanguage::Han,
                Some("und-Latn") => crate::presentation::fonts::TextLanguage::Italic,
                _ => crate::presentation::fonts::TextLanguage::Germanic,
            };
            result.push_str(&html_escape(crate::presentation::fields::label_for(
                key, lang,
            )));
        } else if let Some(key) = token.strip_prefix("value:") {
            result.push_str(&html_escape(
                fields.get(key).map(String::as_str).unwrap_or(""),
            ));
        } else if let Some(key) = token.strip_prefix("checked:") {
            if fields
                .get(key)
                .is_some_and(|v| v.eq_ignore_ascii_case("true"))
            {
                result.push_str("checked");
            }
        } else if let Some((key, value)) = token
            .strip_prefix("selected:")
            .and_then(|t| t.split_once(':'))
        {
            if fields.get(key).is_some_and(|v| v == value) {
                result.push_str("selected");
            }
        } else if let Some(key) = token.strip_prefix("options:") {
            result.push_str(&option_list(key, fields));
        } else if let Some(value) = slots.get(token) {
            result.push_str(value);
        }
        tail = after;
    }
    result.push_str(tail);
    result
}

/// The only uploaded part accepted is a UTF-8 .ssr file. No files are written.
pub fn uploaded_design(body: &str) -> Result<&str, String> {
    let (boundary, rest) = body.split_once("\r\n").ok_or("Invalid upload framing")?;
    if !boundary.starts_with("--") || !(3..=72).contains(&boundary.len()) {
        return Err("Invalid upload boundary".into());
    }
    let delimiter = format!("\r\n{boundary}");
    for part in rest.split(&delimiter) {
        let part = part.strip_prefix("\r\n").unwrap_or(part);
        let Some((headers, text)) = part.split_once("\r\n\r\n") else {
            continue;
        };
        if headers.lines().any(|line| {
            line.to_ascii_lowercase()
                .starts_with("content-disposition:")
                && line.contains("name=\"design_file\"")
                && line.contains("filename=\"")
        }) {
            if text.is_empty() {
                return Err("Select a non-empty .ssr file".into());
            }
            return Ok(text);
        }
    }
    Err("No design file in upload".into())
}
