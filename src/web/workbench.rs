//! Zero-script workbench: submitted inputs become one complete calculation page.
use super::{
    forms::{self, Fields},
    svg, Response,
};
use crate::{
    design::Evaluation,
    io::serialize,
    model::ShipModel,
    presentation::html_escape,
    workbench::{Command, Workbench},
};
use std::fmt::Write;

const PAGE: &str = include_str!("../../web/index.html");
const EDITOR: &str = include_str!("../../web/editor.html");
pub const CSS: &str = include_str!("../../web/styles.css");

fn get<'a>(fields: &'a Fields, key: &str) -> &'a str {
    fields.get(key).map(String::as_str).unwrap_or("")
}
fn number(fields: &Fields, key: &str) -> Result<f64, String> {
    let n = get(fields, key)
        .parse::<f64>()
        .map_err(|_| format!("Invalid {key}"))?;
    if n.is_finite() && n > 0.0 {
        Ok(n)
    } else {
        Err(format!("{key} must be finite and positive"))
    }
}
fn submitted(fields: &Fields, base: &ShipModel) -> Result<ShipModel, String> {
    let mut values = forms::save_fields(&serialize(base));
    for (key, value) in fields {
        if values.contains_key(key) {
            if value.contains(['\r', '\n']) {
                return Err(format!("{key} cannot contain a line break"));
            }
            values.insert(key.clone(), value.clone());
        }
    }
    let mut text = String::from("SpringSharp Rust Save 1.0\n");
    for (key, value) in values {
        let _ = writeln!(text, "{key}={value}");
    }
    Workbench::parse_design(&text)
}
fn defaults(fields: &Fields) -> Fields {
    let mut values = Fields::new();
    for (key, value) in [
        ("target_hp", "50000"),
        ("text_language", "gem"),
        ("target_bunker", "1500"),
        ("target_disp", "35000"),
        ("target_cg", "0.45"),
        ("p_length", "80"),
        ("p_beam", "20"),
        ("p_disp", "112000"),
        ("p_speed", "40"),
        ("p_factor", "0.668"),
    ] {
        values.insert(key.into(), value.into());
    }
    values.extend(fields.clone());
    values
}

pub fn initial() -> Response {
    Response::html(
        200,
        page(
            &Workbench::default(),
            &Fields::new(),
            "Demo design loaded",
            false,
            "",
        ),
    )
}

pub fn upload(body: &str) -> Response {
    let mut work = Workbench::default();
    match forms::uploaded_design(body).and_then(|text| {
        work.execute(Command::Import(text))
            .map(crate::command_message)
    }) {
        Ok(message) => Response::html(200, page(&work, &Fields::new(), &message, false, "")),
        Err(error) => Response::html(400, page(&work, &Fields::new(), &error, true, "")),
    }
}

pub fn submit(path: &str, body: &str) -> Response {
    let mut fields = match forms::decode(body) {
        Ok(fields) => defaults(&fields),
        Err(error) => return Response::text(400, error),
    };
    let mut work = if get(&fields, "base_design").is_empty() {
        Workbench::default()
    } else {
        match Workbench::parse_design(get(&fields, "base_design")) {
            Ok(ship) => Workbench::new(ship),
            Err(error) => return Response::text(400, format!("Invalid base design: {error}")),
        }
    };
    if let Err(error) = work.restore_baseline(get(&fields, "baseline")) {
        return Response::html(
            400,
            page(
                &work,
                &fields,
                &format!("Invalid baseline: {error}"),
                true,
                "",
            ),
        );
    }
    let action = if path == "/workbench" {
        get(&fields, "action").to_string()
    } else {
        "calculate".into()
    };
    if !matches!(action.as_str(), "demo" | "preset" | "raw") {
        match submitted(&fields, &work.design.draft) {
            Ok(ship) => work.design.draft = ship,
            Err(error) => return Response::html(400, page(&work, &fields, &error, true, "")),
        }
    }
    let applied = apply(&action, &mut work, &mut fields);
    let (message, analysis) = match applied {
        Ok(output) => output,
        Err(error) => return Response::html(400, page(&work, &fields, &error, true, "")),
    };
    if path == "/workbench" {
        return Response::html(200, page(&work, &fields, &message, false, &analysis));
    }
    let result = work.design.result();
    let metric = get(&fields, "metric") == "true";
    let arcs = get(&fields, "arcs") == "true";
    match path {
        "/export/design.ssr" => Response::download(
            "text/plain; charset=utf-8",
            work.export_design(),
            "design.ssr",
        ),
        "/export/blueprint.svg" => Response::download(
            "image/svg+xml; charset=utf-8",
            svg::blueprint(&result, metric, arcs),
            "blueprint.svg",
        ),
        "/blueprint" => Response::new(
            200,
            "image/svg+xml; charset=utf-8",
            svg::blueprint(&result, metric, arcs),
        ),
        "/export/report.html" => Response::download(
            "text/html; charset=utf-8",
            export(&result, metric, arcs, &fields),
            "report.html",
        ),
        _ => Response::html(200, page(&work, &fields, &message, false, &analysis)),
    }
}

fn apply(
    action: &str,
    work: &mut Workbench,
    fields: &mut Fields,
) -> Result<(String, String), String> {
    if action == "arcs" {
        let enabled = get(fields, "arcs") != "true";
        fields.insert("arcs".into(), enabled.to_string());
    }
    if action == "superstructure_reset" {
        work.design.draft.superstructure = Default::default();
    }
    let command = match action {
        ""
        | "calculate"
        | "superstructure_reset"
        | "arcs"
        | "external"
        | "external_calibrate"
        | "planing"
        | "planing_calibrate" => Command::Calculate,
        "demo" => Command::Demo,
        "preset" => Command::Preset(get(fields, "preset")),
        "raw" => Command::Import(get(fields, "raw_design")),
        "regime" => Command::Regime,
        "blend" => Command::Blend,
        "reset" => Command::ResetResistance,
        "speed" => Command::Speed(number(fields, "target_hp")?),
        "range" => Command::Range(number(fields, "target_bunker")?),
        "displacement" => Command::Displacement(number(fields, "target_disp")?),
        "pin" => Command::PinBaseline,
        "compare" => Command::ComparePreset(get(fields, "preset")),
        "clear_compare" => Command::ClearComparison,
        "compartments" => Command::Arrange(number(fields, "target_cg")?),
        _ => return Err("Unknown workbench action".into()),
    };
    // Web-only reference analyses must also validate before publishing a design.
    let mut analysis = String::new();
    let mut planing_factor = None;
    let message_override = match action {
        "external" => {
            let cases = crate::parse_external_cases(get(fields, "external"))?;
            analysis = embedded_report("External data", &crate::external_report_html(&cases));
            Some("External references evaluated".into())
        }
        "external_calibrate" => {
            let cases = crate::parse_external_cases(get(fields, "external"))?;
            let rf = crate::fit_reduced_form_with_external(&cases);
            let sf = crate::fit_structural_form_with_external(&cases);
            let cv = crate::run_loocv_with_external_weighted(&cases);
            Some(format!("External calibration: {} cases · reduced MAPE {:.1}% · structural MAPE {:.1}% · weighted LOOCV ensemble {:.1}%",cases.len(),rf.mean_abs_pct_error*100.0,sf.mean_abs_pct_error*100.0,cv.ensemble_mape*100.0))
        }
        "planing" => {
            let hp = crate::planing_hp_estimate_calibrated(
                number(fields, "p_length")?,
                number(fields, "p_beam")?,
                number(fields, "p_disp")?,
                number(fields, "p_speed")?,
                number(fields, "p_factor")?,
            );
            Some(format!("Estimated planing power: {hp:.0} hp"))
        }
        "planing_calibrate" => {
            let cases = crate::parse_external_cases(get(fields, "external"))?;
            let factor = crate::compute_planing_calibration(&cases);
            planing_factor = Some(factor);
            Some(format!("Planing calibration factor: {factor:.3}"))
        }
        _ => None,
    };
    let message = crate::command_message(work.execute(command)?);
    if let Some(factor) = planing_factor {
        fields.insert("p_factor".into(), factor.to_string());
    }
    if let Some(placement) = work.placement() {
        analysis = embedded_report(
            "General arrangement",
            &crate::compartment_placement_html(&work.design.result().ship, placement),
        );
    }
    Ok((message_override.unwrap_or(message), analysis))
}
fn embedded_report(title: &str, html: &str) -> String {
    format!(
        r#"<details open><summary>{}</summary><iframe title="{}" sandbox srcdoc="{}"></iframe></details>"#,
        html_escape(title),
        html_escape(title),
        html_escape(html)
    )
}

fn page(work: &Workbench, fields: &Fields, message: &str, error: bool, analysis: &str) -> String {
    let result = work.design.result();
    let ship = &result.ship;
    let metric = get(fields, "metric") == "true";
    let arcs = get(fields, "arcs") == "true";
    let mut values = defaults(fields);
    let calculated = forms::save_fields(&serialize(ship));
    // Invalid submissions retain exactly the typed fields; results retain the
    // previous valid snapshot. A successful submission publishes normalized input.
    for (key, value) in calculated {
        if !error || !values.contains_key(&key) {
            values.insert(key, value);
        }
    }
    let mut slots = Fields::new();
    slots.insert(
        "editor".into(),
        forms::render(EDITOR, &values, &Fields::new()),
    );
    let language = crate::presentation::fonts::TextLanguage::parse(get(fields, "text_language"));
    slots.insert("language".into(), language.tag().into());
    slots.insert(
        "title_class".into(),
        if get(fields, "serif_titles") == "true" {
            "serif-title"
        } else {
            "lang-text"
        }
        .into(),
    );
    slots.insert(
        "languages".into(),
        crate::presentation::fonts::TextLanguage::ALL
            .into_iter()
            .map(|l| {
                format!(
                    "<option value=\"{}\" {}>{}</option>",
                    l.tag(),
                    if l == language { "selected" } else { "" },
                    l.label()
                )
            })
            .collect(),
    );
    slots.insert("title".into(), html_escape(&ship.name));
    slots.insert(
        "subtitle".into(),
        html_escape(&format!(
            "{} · {} · {}",
            ship.country, ship.ship_type, ship.hull.date_hull
        )),
    );
    slots.insert(
        "stylesheet".into(),
        "<link rel=\"stylesheet\" href=\"/assets/styles.css\">".into(),
    );
    slots.insert("snapshot".into(), html_escape(&serialize(ship)));
    slots.insert(
        "baseline".into(),
        html_escape(&work.baseline().map(serialize).unwrap_or_default()),
    );
    let raw = if error && fields.contains_key("raw_design") {
        get(fields, "raw_design").to_string()
    } else {
        serialize(ship)
    };
    slots.insert("raw".into(), html_escape(&raw));
    slots.insert("metric".into(), if metric { "checked" } else { "" }.into());
    slots.insert("arcs_value".into(), arcs.to_string());
    let mut presets = String::from("<option value=\"\">Historical presets...</option>");
    for preset in crate::PRESET_LIST {
        let _ = write!(
            presets,
            "<option value=\"{}\" {}>{} ({})</option>",
            html_escape(preset.id),
            if get(fields, "preset") == preset.id {
                "selected"
            } else {
                ""
            },
            html_escape(preset.name),
            preset.year
        );
    }
    slots.insert("presets".into(), presets);
    slots.insert(
        "message".into(),
        format!(
            "<p class=\"notice {}\" role=\"status\">{}</p>",
            if error { "error" } else { "success" },
            html_escape(message)
        ),
    );
    let model = if ship.engine.blend_weight.clamp(0.0, 1.0) < 1.0 - 1e-9 {
        "Blended Holtrop / Original"
    } else {
        match ship.engine.resistance_model {
            crate::model::ResistanceModel::Original => "Original SpringSharp model",
            crate::model::ResistanceModel::HoltropMennen => "Holtrop-Mennen model",
        }
    };
    slots.insert(
        "status".into(),
        format!(
            "<div class=\"snapshot-status\"><span class=\"badge\">{model}</span><span class=\"badge {}\">{}</span><span class=\"badge {}\">{}</span></div>",
            if error { "warn" } else { "" },
            if error {
                "Input error — results from last valid design"
            } else {
                "Calculated"
            },
            if ship.converged { "good" } else { "warn" },
            if ship.converged {
                "weights converged"
            } else {
                "weights not converged"
            }
        ),
    );
    let report = crate::generate_report(ship);
    let alerts = report
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("WARNING") || line.starts_with("DESIGN FAILURE"))
        .map(|line| format!("<p class=\"design-alert\">{}</p>", html_escape(line)))
        .collect::<String>();
    slots.insert("alerts".into(), alerts);
    slots.insert("blueprint".into(), svg::blueprint(&result, metric, arcs));
    slots.insert(
        "superstructure_notes".into(),
        crate::presentation::superstructure_text::planning_notes(&result.blueprint.superstructure)
            .iter()
            .map(|note| format!("<p class=\"hint\">{}</p>", html_escape(note)))
            .collect::<String>(),
    );

    slots.insert("metrics".into(), metrics(ship, metric));
    let baseline = work.baseline();
    slots.insert(
        "comparison".into(),
        baseline
            .as_ref()
            .map(|base| comparison(ship, base))
            .unwrap_or_default(),
    );
    slots.insert(
        "speed_chart".into(),
        svg::speed_power(&result, baseline, metric),
    );
    slots.insert("immunity_chart".into(), svg::immunity(&result, metric));
    slots.insert("systems".into(), systems(&result));
    slots.insert("historical".into(), historical(ship));
    slots.insert("spaces".into(), spaces(ship));
    slots.insert("analysis".into(), analysis.into());
    slots.insert(
        "report".into(),
        html_escape(&crate::generate_report(&result.ship)),
    );
    forms::render(PAGE, &values, &slots)
}

fn metrics(ship: &ShipModel, metric: bool) -> String {
    let mut out = String::from("<div class=\"cards\">");
    let mass = if metric { 1.01605 } else { 1.0 };
    let speed = if metric { 1.852 } else { 1.0 };
    let length = if metric { 0.3048 } else { 1.0 };
    for (label, value, unit) in [
        (
            "Normal displacement",
            ship.hull.displacement * mass,
            if metric { "tonne" } else { "t" },
        ),
        (
            "Full load",
            ship.disp_max_load * mass,
            if metric { "tonne" } else { "t" },
        ),
        (
            "Max speed",
            ship.engine.speed_max * speed,
            if metric { "km/h" } else { "kt" },
        ),
        (
            "Standard displacement",
            ship.disp_standard * mass,
            if metric { "tonne" } else { "t" },
        ),
        ("Power", ship.engine.hp_max, "hp"),
        (
            "Broadside",
            ship.guns.broadside * if metric { 0.453592 } else { 1.0 },
            if metric { "kg" } else { "lb" },
        ),
        ("Cost", ship.cost_dollars, "$ million"),
        (
            "Range",
            ship.engine.range * speed,
            if metric { "km" } else { "nm" },
        ),
        (
            "GM",
            ship.metacentre * length,
            if metric { "m" } else { "ft" },
        ),
        ("Stability", ship.stability_adj, ""),
        ("Seakeeping", ship.seakeeping, ""),
        ("Strength", ship.strength_comp, ""),
    ] {
        let _=write!(out,"<div class=\"card\"><span class=\"k\">{label}</span><div class=\"v\">{value:.2} <span class=\"u\">{unit}</span></div></div>");
    }
    out.push_str("</div>");
    let mut weights = vec![
        ("Hull", ship.engine.hull_weight),
        ("Armour", ship.armour.weight),
        ("Machinery", ship.engine.engine_weight),
        ("Armament", ship.guns.borne_weight + ship.weapons.weight),
        ("Magazine", ship.guns.magazine),
        ("Bunker", ship.engine.bunker),
        (
            "Miscellaneous",
            ship.weapons.misc_weight_vital
                + ship.weapons.misc_weight_hull
                + ship.weapons.misc_weight_deck
                + ship.weapons.misc_weight_above
                + ship.weapons.void_weight,
        ),
    ];
    let used: f64 = weights.iter().map(|(_, value)| value.max(0.0)).sum();
    if ship.hull.displacement - used > 0.5 {
        weights.push(("Other load", ship.hull.displacement - used));
    }
    let total = ship.hull.displacement.max(1.0);
    let _ = write!(
        out,
        "<h2>Weight distribution <span class=\"hint\">of {total:.0} t normal</span></h2><div class=\"wbar\" role=\"img\" aria-label=\"Weight distribution\">"
    );
    for (label, value) in &weights {
        if *value <= 0.0 {
            continue;
        }
        let pct = value / total * 100.0;
        let _ = write!(
            out,
            "<span class=\"w-{}\" style=\"width:{:.3}%\" title=\"{label}: {value:.0} t · {pct:.1}%\">{}</span>",
            weight_class(label),
            pct.min(100.0),
            if pct >= 6.0 { format!("{pct:.0}%") } else { String::new() }
        );
    }
    out.push_str("</div><div class=\"wlegend\">");
    for (label, value) in &weights {
        let _ = write!(
            out,
            "<div><i class=\"w-{}\"></i>{label}<span>{value:.0} t · {:.1}%</span></div>",
            weight_class(label),
            value / total * 100.0
        );
    }
    out.push_str("</div>");
    out
}

/// CSS colour class for a weight group (palette matches the desktop GUI).
fn weight_class(label: &str) -> &'static str {
    match label {
        "Hull" => "hull",
        "Armour" => "armour",
        "Machinery" => "machinery",
        "Armament" => "armament",
        "Magazine" => "magazine",
        "Bunker" => "bunker",
        "Miscellaneous" => "misc",
        _ => "other",
    }
}
fn comparison(ship: &ShipModel, base: &ShipModel) -> String {
    let mut out=format!("<details open><summary>Comparison with {}</summary><table><tr><th>Metric</th><th>Submitted</th><th>Baseline</th><th>Δ</th></tr>",html_escape(&base.name));
    for (label, a, b) in [
        (
            "Displacement (t)",
            ship.hull.displacement,
            base.hull.displacement,
        ),
        ("Power (hp)", ship.engine.hp_max, base.engine.hp_max),
        ("Speed (kt)", ship.engine.speed_max, base.engine.speed_max),
        ("Length (ft)", ship.hull.length, base.hull.length),
        ("Beam (ft)", ship.hull.beam, base.hull.beam),
        ("Draught (ft)", ship.hull.draught, base.hull.draught),
        ("Full load (t)", ship.disp_max_load, base.disp_max_load),
        ("Range (nm)", ship.engine.range, base.engine.range),
        ("Deck (in)", ship.armour.deck, base.armour.deck),
        (
            "Main calibre (in)",
            ship.guns.batteries[0].calibre,
            base.guns.batteries[0].calibre,
        ),
        ("Strength", ship.strength_comp, base.strength_comp),
        ("GM (ft)", ship.metacentre, base.metacentre),
        (
            "Main belt (in)",
            ship.armour.belt_main,
            base.armour.belt_main,
        ),
        ("Broadside (lb)", ship.guns.broadside, base.guns.broadside),
    ] {
        let _ = write!(
            out,
            "<tr><th>{label}</th><td>{a:.2}</td><td>{b:.2}</td><td>{:+.2}</td></tr>",
            a - b
        );
    }
    out.push_str("</table></details>");
    out
}
fn systems(result: &Evaluation) -> String {
    let sys = &result.systems;
    let mut out = format!(
        "<p>{}</p><p>{}</p><table>",
        html_escape(&crate::presentation::classification(
            &sys.tactical_affinities
        )),
        html_escape(&crate::presentation::match_explanation(sys))
    );
    for (label, value) in [
        ("Installed electrical power (kW)", sys.total_installed_kw),
        ("Combat peak (kW)", sys.combat_peak_kw),
        ("Electrical margin (%)", sys.electrical_margin_pct),
        ("Systems weight (t)", sys.total_systems_weight_tons),
        ("Estimated ΔGM (ft)", sys.estimated_delta_gm_ft),
    ] {
        let _ = write!(out, "<tr><th>{label}</th><td>{value:.2}</td></tr>");
    }
    out.push_str("</table>");
    out
}
fn historical(ship: &ShipModel) -> String {
    let mut out = String::new();
    if let Some(b) = crate::get_preset_benchmark(&ship.name) {
        let _ = write!(
            out,
            "<p>{}</p><table><tr><th>Metric</th><th>Calculated</th><th>Reference</th></tr>",
            html_escape(b.source)
        );
        for (name, a, r) in [
            (
                "Displacement (t)",
                ship.hull.displacement,
                b.ref_displacement_t,
            ),
            ("Hull (t)", ship.engine.hull_weight, b.ref_hull_t),
            (
                "Machinery (t)",
                ship.engine.engine_weight,
                b.ref_machinery_t,
            ),
            (
                "Armour + barbettes (t)",
                ship.armour.weight
                    + ship
                        .guns
                        .batteries
                        .iter()
                        .map(|gun| gun.arm_weight)
                        .sum::<f64>(),
                b.ref_armour_t,
            ),
            (
                "Armament + ammo (t)",
                ship.guns.borne_weight + ship.guns.magazine + ship.weapons.weight,
                b.ref_armament_t,
            ),
            ("Bunker (t)", ship.engine.bunker, b.ref_bunker_t),
            ("GM (ft)", ship.metacentre, b.ref_gm_ft),
        ] {
            let _ = write!(
                out,
                "<tr><th>{name}</th><td>{a:.2}</td><td>{r:.2}</td></tr>"
            );
        }
        out.push_str("</table>");
    }
    out.push_str("<h2>Nearest historical ships</h2><table><tr><th>Ship</th><th>Displacement (t)</th><th>Speed (kt)</th><th>Power (hp)</th></tr>");
    for c in crate::nearest_historical_cases(ship, 6) {
        let _ = write!(
            out,
            "<tr><th>{}</th><td>{:.0}</td><td>{:.1}</td><td>{:.0}</td></tr>",
            html_escape(c.name),
            c.displacement_tons,
            c.speed_kt,
            c.reference_hp
        );
    }
    out.push_str("</table>");
    out
}
fn spaces(ship: &ShipModel) -> String {
    let s = crate::calculate_internal_spaces(ship);
    let mut out = String::from("<table><tr><th>Category</th><th>Volume (m³)</th></tr>");
    for (name, v) in [
        ("Hull", s.hull_volume_m3),
        ("Machinery", s.machinery_volume_m3),
        ("Magazine", s.magazine_volume_m3),
        ("Crew", s.accommodation_volume_m3),
        ("Fuel", s.fuel_volume_m3),
        ("Stores", s.stores_volume_m3),
        ("Required", s.required_volume_m3),
    ] {
        let _ = write!(out, "<tr><th>{name}</th><td>{v:.0}</td></tr>");
    }
    out.push_str("</table>");
    out
}
fn export(result: &Evaluation, metric: bool, arcs: bool, fields: &Fields) -> String {
    let language =
        crate::presentation::fonts::TextLanguage::parse(get(fields, "text_language")).tag();
    let title_class = if get(fields, "serif_titles") == "true" {
        "serif-title"
    } else {
        "lang-text"
    };
    format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>{CSS}</style><body><article class=\"export-report\"><h1 class=\"{title_class}\" lang=\"{language}\">{}</h1>{}{}{}{}<h2>Report</h2><pre class=\"report\">{}</pre><h2>Design parameters</h2><pre class=\"report\">{}</pre></article></body></html>",html_escape(&result.ship.name),html_escape(&result.ship.name),svg::blueprint(result,metric,arcs),metrics(&result.ship,metric),svg::speed_power(result,None,metric),svg::immunity(result,metric),html_escape(&crate::generate_report(&result.ship)),html_escape(&serialize(&result.ship)))
}
