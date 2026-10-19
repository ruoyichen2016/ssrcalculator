use eframe::egui::{self, FontFamily};
use springsharp::presentation::fonts::{self, TextLanguage};

pub fn install(ctx: &egui::Context, language: TextLanguage) -> Vec<String> {
    let mut definitions = egui::FontDefinitions::default();
    let mut missing = Vec::new();
    for (name, file) in fonts::FONTS {
        match fonts::font_path(file).and_then(|p| std::fs::read(p).ok()) {
            Some(bytes) => {
                definitions
                    .font_data
                    .insert(name.into(), egui::FontData::from_owned(bytes).into());
            }
            None => missing.push(file.to_string()),
        }
    }
    let available = |name: &&str| definitions.font_data.contains_key(*name);
    let proportional: Vec<String> = language
        .sans()
        .into_iter()
        .filter(available)
        .map(|s| s.to_string())
        .collect();
    let mono: Vec<String> = ["Noto Sans Mono", "Noto Sans CJK SC", "Noto Sans"]
        .into_iter()
        .filter(available)
        .map(|s| s.to_string())
        .collect();
    let mut serif: Vec<String> = [
        "Noto Serif",
        "Noto Serif CJK JP",
        "Noto Sans",
        "Noto Sans CJK SC",
    ]
    .into_iter()
    .filter(available)
    .map(|s| s.to_string())
    .collect();
    serif.extend(definitions.families[&FontFamily::Proportional].clone());
    definitions
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .splice(0..0, proportional);
    definitions
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .splice(0..0, mono);
    definitions
        .families
        .insert(FontFamily::Name("serif".into()), serif);
    ctx.set_fonts(definitions);
    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrifa::MetadataProvider;
    #[test]
    fn bundled_fonts_cover_workbench_languages() {
        let ctx = egui::Context::default();
        assert!(install(&ctx, TextLanguage::Han).is_empty());
        let sample = "English Deutsch ÄÖÜ äöü ß ẞ Français Élévation œuvre Æ āēīōūȳ e\u{301} 中文舰船 日本語あいうえおカタカナ Δρ m³";
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.ctx().fonts_mut(|fonts| {
                for family in [
                    FontFamily::Proportional,
                    FontFamily::Monospace,
                    FontFamily::Name("serif".into()),
                ] {
                    let id = egui::FontId::new(16.0, family);
                    for ch in sample.chars().filter(|ch| !ch.is_whitespace()) {
                        let definitions = fonts.definitions();
                        let supporting = definitions.families[&id.family]
                            .iter()
                            .filter(|name| name.starts_with("Noto "))
                            .find(|name| {
                                let data = &definitions.font_data[*name];
                                skrifa::FontRef::from_index(&data.font, data.index)
                                    .unwrap()
                                    .charmap()
                                    .map(ch)
                                    .is_some()
                            });
                        assert!(supporting.is_some(), "Missing {ch} in {:?}", id.family);
                        if ch == '船' {
                            let expected = if id.family == FontFamily::Name("serif".into()) {
                                "Noto Serif CJK JP"
                            } else {
                                "Noto Sans CJK SC"
                            };
                            assert_eq!(supporting.unwrap(), expected);
                        }
                    }
                    let galley = fonts.layout_no_wrap(sample.into(), id, egui::Color32::WHITE);
                    assert!(galley.size().x > 0.0);
                }
            });
        });
        output.textures_delta.clear();
    }
}
