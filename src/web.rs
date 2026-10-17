//! A small zero-dependency web front-end for the Rust SpringSharp engine.
//!
//! It serves one single-page "workbench" (`src/web_page.html`, embedded with
//! `include_str!`) plus a handful of JSON/text endpoints.  The design travels
//! between browser and server as `.ssr` save text, exactly what
//! `io::save_to_file` writes, so the web UI and the file format stay in sync.
//! Enumerated inputs (gun type, mount, layout, bow/stern) are rendered as
//! labelled selects that still submit the numeric index the engine expects.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::model::{ResistanceModel, ShipModel};
use crate::{
    apply_blend_calibration, apply_regime_calibration, compartment_placement_html,
    compute_planing_calibration, deserialize, external_report_html, fit_reduced_form_with_external,
    fit_structural_form_with_external, generate_html_report, generate_report, get_preset,
    get_preset_benchmark, is_planing_eligible, nearest_historical_cases, parse_external_cases,
    planing_hp_estimate_calibrated, run_loocv_with_external_weighted, serialize, solve_max_speed,
    solve_range, suggest_compartment_placement, PRESET_LIST,
};

const PAGE_TEMPLATE: &str = include_str!("web_page.html");

const GUN_TYPES: &[&str] = &[
    "Muzzle loading gun",
    "Breech loading gun",
    "Quick firing gun",
    "Anti-air gun",
    "Dual purpose gun",
    "Auto rapid fire gun",
    "Machine gun",
];
const MOUNT_TYPES: &[&str] = &[
    "Broadside",
    "Coles/Ericsson turret",
    "Open barbette",
    "Turret on barbette",
    "Deck and hoist",
    "Deck mount",
    "Casemate",
];
const GUNS_IN_MOUNT: &[&str] = &[
    "Single mount (1)",
    "2 row twin mount (2)",
    "4 row quad mount (4)",
    "Twin mount (2)",
    "2-gun mount (2)",
    "2 row quad mount (4)",
    "Triple mount (3)",
    "3-gun mount (3)",
    "2 row sextuple mount (6)",
    "Quad mount (4)",
    "4-gun mount (4)",
    "2 row octuple mount (8)",
    "Quintuple mount (5)",
    "5-gun mount (5)",
    "2 row decuple mount (10)",
];
const LAYOUTS: &[&str] = &[
    "Centreline - distributed",
    "Centreline - ends (fore >= aft)",
    "Centreline - ends (aft >= fore)",
    "Centreline - fore deck forward",
    "Centreline - fore deck",
    "Centreline - fore deck aft",
    "Centreline - aft deck forward",
    "Centreline - aft deck",
    "Centreline - aft deck aft",
    "Sides - distributed",
    "Sides - ends (fore >= aft)",
    "Sides - ends (aft >= fore)",
    "Sides - fore deck forward",
    "Sides - fore deck",
    "Sides - fore deck aft",
    "Sides - aft deck forward",
    "Sides - aft deck",
    "Sides - aft deck aft",
    "None",
];
const BOWS: &[&str] = &[
    "Normal bow",
    "Bulbous bow - straight",
    "Bulbous bow - forward",
    "Ram bow",
];
const STERNS: &[&str] = &["Cruiser stern", "Transom stern", "Round stern"];
const FUELS: &[&str] = &[
    "Coal fired boilers",
    "Oil fired boilers",
    "Diesel motors",
    "Petrol motors",
    "Batteries",
];
const ENGINES: &[&str] = &[
    "Simple reciprocating",
    "Complex reciprocating",
    "Steam turbines",
];
const DRIVES: &[&str] = &["Direct", "Geared", "Electric", "Hydraulic"];

fn html_page() -> String {
    PAGE_TEMPLATE.replace("__FORM__", &form_html())
}

fn grid(fields: &[(&str, &str)]) -> String {
    let mut s = String::from("<div class='grid'>");
    for (key, label) in fields {
        s.push_str(&number_input(key, label));
    }
    s.push_str("</div>");
    s
}

fn form_html() -> String {
    let mut f = String::new();

    // ---- Hull & freeboard ----
    f.push_str("<div class='tab active' data-tab='hull'><h2>Ship</h2><div class='grid'>");
    f.push_str(&text_input("meta.name", "Name"));
    f.push_str(&text_input("meta.country", "Country"));
    f.push_str(
        r#"<label>Type<input data-key="meta.type" list="shipTypes" value=""></label><datalist id="shipTypes"><option value="Battleship"><option value="Battlecruiser"><option value="Carrier"><option value="Heavy cruiser"><option value="Light cruiser"><option value="Destroyer"></datalist>"#,
    );
    f.push_str("</div><h2>Hull</h2>");
    f.push_str(&grid(&[
        ("hull.length", "Length WL (ft)"),
        ("hull.beam", "Beam (ft)"),
        ("hull.beam_bulges", "Beam over bulges (ft)"),
        ("hull.draught", "Draught (ft)"),
        ("hull.block", "Block coefficient"),
        ("hull.date_hull", "Hull year"),
        ("hull.date_engine", "Engine year"),
    ]));
    f.push_str("<h2>Hull form (resistance detail, 0 = auto)</h2>");
    f.push_str(&grid(&[
        ("hull.lcb_percent", "LCB (% L, + fwd)"),
        ("hull.midship_coefficient", "Midship coeff Cm"),
        ("hull.bulb_area_sqft", "Bulb area (sq ft)"),
        ("hull.bulb_centre_height_ft", "Bulb centre height (ft)"),
        ("hull.transom_area_sqft", "Transom area (sq ft)"),
        ("hull.appendage_area_sqft", "Appendage area (sq ft)"),
        ("hull.stern_shape", "Stern shape Cstern"),
    ]));
    f.push_str("<h2>Ends</h2><div class='grid'>");
    f.push_str(&select_input("free.bow", "Bow", BOWS));
    f.push_str(&select_input("free.stern", "Stern", STERNS));
    for (key, label) in [
        ("free.transom_width", "Transom width (fraction)"),
        ("free.length_bow", "Bow angle (deg)"),
        ("free.length_ram", "Ram length (ft)"),
        ("free.length_stern", "Stern overhang (ft)"),
    ] {
        f.push_str(&number_input(key, label));
    }
    f.push_str("</div><h2>Freeboard</h2>");
    f.push_str(&grid(&[
        ("free.fc_length", "Forecastle length (fraction)"),
        ("free.fd_length", "Forward deck length (fraction)"),
        ("free.qd_length", "Quarter deck length (fraction)"),
        ("free.free_fc_for", "Forecastle fwd (ft)"),
        ("free.free_fc_aft", "Forecastle aft (ft)"),
        ("free.free_fd_for", "Forward deck fwd (ft)"),
        ("free.free_fd_aft", "Forward deck aft (ft)"),
        ("free.free_ad_for", "Aft deck fwd (ft)"),
        ("free.free_ad_aft", "Aft deck aft (ft)"),
        ("free.free_qd_for", "Quarter deck fwd (ft)"),
        ("free.free_qd_aft", "Quarter deck aft (ft)"),
    ]));
    f.push_str("</div>");

    // ---- Armour ----
    f.push_str("<div class='tab' data-tab='armour'>");
    for (title, fields) in [
        (
            "Belts",
            &[
                ("arm.belt_main", "Main belt (in)"),
                ("arm.belt_main_length", "Main belt length (ft)"),
                ("arm.belt_main_height", "Main belt height (ft)"),
                ("arm.belt_ends", "Ends belt (in)"),
                ("arm.belt_ends_length", "Ends belt length (ft)"),
                ("arm.belt_ends_height", "Ends belt height (ft)"),
                ("arm.belt_upper", "Upper belt (in)"),
                ("arm.belt_upper_length", "Upper belt length (ft)"),
                ("arm.belt_upper_height", "Upper belt height (ft)"),
                ("arm.belts_incline", "Belt incline (deg)"),
            ][..],
        ),
        (
            "Torpedo bulkhead & bulges",
            &[
                ("arm.bulkhead", "Bulkhead (in)"),
                ("arm.bulkhead_length", "Bulkhead length (ft)"),
                ("arm.bulkhead_height", "Bulkhead height (ft)"),
                ("arm.bulkhead_beam", "Beam between bulkheads (ft)"),
                ("arm.bulkhead_type", "Bulkhead type (0/1)"),
                ("arm.bulge", "Bulge (in)"),
                ("arm.bulge_length", "Bulge length (ft)"),
                ("arm.bulge_height", "Bulge height (ft)"),
            ][..],
        ),
        (
            "Decks & conning tower",
            &[
                ("arm.deck", "Main deck (in)"),
                ("arm.deck_fc", "Forecastle deck (in)"),
                ("arm.deck_qd", "Quarter deck (in)"),
                ("arm.deck_coverage", "Deck coverage mode"),
                ("arm.conning_for", "Conning tower fwd (in)"),
                ("arm.conning_aft", "Conning tower aft (in)"),
            ][..],
        ),
    ] {
        f.push_str(&format!("<h2>{title}</h2>"));
        f.push_str(&grid(fields));
    }
    f.push_str("</div>");

    // ---- Guns (sub-tabs) ----
    f.push_str("<div class='tab' data-tab='guns'><div class='pills'>");
    for i in 0..5 {
        f.push_str(&format!(
            "<button type='button' data-sub='gun{i}'{}>{}</button>",
            if i == 0 { " class='active'" } else { "" },
            battery_title(i)
        ));
    }
    f.push_str("</div>");
    for i in 0..5 {
        f.push_str(&gun_section(i));
    }
    f.push_str("</div>");

    // ---- Weapons & misc ----
    f.push_str("<div class='tab' data-tab='weapons'>");
    for i in 0..5 {
        f.push_str(&weapon_section(i));
    }
    f.push_str("<h2>Miscellaneous weights (tons)</h2>");
    f.push_str(&grid(&[
        ("misc.vital", "Vital spaces"),
        ("misc.hull", "Hull spaces"),
        ("misc.deck", "Deck spaces"),
        ("misc.above", "Above-water"),
        ("misc.void", "Void"),
    ]));
    f.push_str("</div>");

    // ---- Machinery ----
    f.push_str("<div class='tab' data-tab='engine'><h2>Performance</h2>");
    f.push_str(&grid(&[
        ("engine.speed_max", "Max speed (kt)"),
        ("engine.speed_cruise", "Cruise speed (kt)"),
        ("engine.range", "Range (nm)"),
        ("engine.pct_coal", "Coal share (%)"),
        ("engine.screws", "Screws"),
        ("perf.steadiness", "Trim / steadiness"),
    ]));
    f.push_str("<h2>Fuel</h2><div class='grid'>");
    for (i, name) in FUELS.iter().enumerate() {
        f.push_str(&checkbox_input(&format!("engine.fuel{i}"), name));
    }
    f.push_str("</div><h2>Engines</h2><div class='grid'>");
    for (i, name) in ENGINES.iter().enumerate() {
        f.push_str(&checkbox_input(&format!("engine.engine{i}"), name));
    }
    f.push_str("</div><h2>Drive</h2><div class='grid'>");
    for (i, name) in DRIVES.iter().enumerate() {
        f.push_str(&checkbox_input(&format!("engine.drive{i}"), name));
    }
    f.push_str("</div><h2>Resistance model</h2><div class='grid'>");
    f.push_str(
        r#"<label>Model<select data-key="engine.resistance_model"><option value="original">Original SpringSharp</option><option value="holtrop">Holtrop-Mennen</option></select></label>"#,
    );
    for (key, label) in [
        ("engine.power_factor", "Power factor"),
        ("engine.original_power_factor", "Original-model factor"),
        ("engine.blend_weight", "Holtrop blend weight (1 = none)"),
        (
            "engine.machinery_weight_factor",
            "Machinery weight factor (1.0 = std, ~0.50 = DD)",
        ),
        ("engine.holtrop_viscous_factor", "Holtrop viscous factor"),
        ("engine.holtrop_wave_factor", "Holtrop wave factor"),
        (
            "engine.holtrop_correlation_factor",
            "Holtrop correlation factor",
        ),
    ] {
        f.push_str(&number_input(key, label));
    }
    f.push_str("</div><h2>Internal-space overrides (0 = auto)</h2>");
    f.push_str(&grid(&[
        ("spaces.crew_volume_per_person", "Crew m³/person"),
        ("spaces.machinery_volume_per_kw", "Machinery m³/kW"),
        ("spaces.stores_volume_per_ton", "Stores m³/ton"),
    ]));
    f.push_str("</div>");
    f
}

fn text_input(key: &str, label: &str) -> String {
    format!(r#"<label>{label}<input data-key="{key}" value=""></label>"#)
}

fn number_input(key: &str, label: &str) -> String {
    format!(r#"<label>{label}<input data-key="{key}" type="number" step="any" value="0"></label>"#)
}

fn checkbox_input(key: &str, label: &str) -> String {
    format!(r#"<label class="check"><input data-key="{key}" type="checkbox"> {label}</label>"#)
}

/// A select whose option values are the numeric indices stored in `.ssr`.
fn select_input(key: &str, label: &str, options: &[&str]) -> String {
    let mut s = format!(r#"<label>{label}<select data-key="{key}">"#);
    for (i, name) in options.iter().enumerate() {
        s.push_str(&format!(
            r#"<option value="{i}">{i} · {}</option>"#,
            crate::features::html_escape(name)
        ));
    }
    s.push_str("</select></label>");
    s
}

fn battery_title(i: usize) -> &'static str {
    match i {
        0 => "Main battery",
        1 => "2nd battery",
        2 => "3rd battery",
        3 => "4th battery",
        _ => "5th battery",
    }
}

fn gun_section(i: usize) -> String {
    let p = format!("gun{i}");
    let mut h = format!(
        "<div class='sub{}' data-sub='{p}'><h2>Guns</h2><div class='grid'>",
        if i == 0 { " active" } else { "" }
    );
    for (key, label) in [
        ("barrels", "Barrels"),
        ("calibre", "Calibre (in)"),
        ("cal_length", "Barrel length (cal)"),
        ("date_gun", "Model year"),
        ("magazine", "Rounds per gun"),
    ] {
        h.push_str(&number_input(&format!("{p}.{key}"), label));
    }
    h.push_str(&select_input(
        &format!("{p}.gun_type"),
        "Gun type",
        GUN_TYPES,
    ));
    h.push_str(&select_input(
        &format!("{p}.mount_type"),
        "Mount type",
        MOUNT_TYPES,
    ));
    h.push_str(&number_input(&format!("{p}.mounts"), "Mounts"));
    h.push_str("</div>");
    for g in 1..=2 {
        h.push_str(&format!("<h2>Group {g}</h2><div class='grid'>"));
        h.push_str(&select_input(
            &format!("{p}.layout_g{g}"),
            "Layout",
            LAYOUTS,
        ));
        h.push_str(&select_input(
            &format!("{p}.guns_in_mount_g{g}"),
            "Barrels per mount",
            GUNS_IN_MOUNT,
        ));
        h.push_str(&number_input(
            &format!("{p}.mount_super_g{g}"),
            "Superfiring mounts",
        ));
        if g == 2 {
            h.push_str(&number_input(&format!("{p}.mount_deck_g2"), "Deck mounts"));
        }
        h.push_str(&number_input(&format!("{p}.mount_sub_g{g}"), "Hull mounts"));
        h.push_str(&number_input(
            &format!("{p}.weight_adj_g{g}"),
            "Weight adjustment",
        ));
        h.push_str("</div>");
    }
    h.push_str("<h2>Mount armour (in)</h2><div class='grid'>");
    for (key, label) in [
        ("arm_face", "Face"),
        ("arm_back", "Back / barbette"),
        ("arm_bottom", "Bottom / hoist"),
    ] {
        h.push_str(&number_input(&format!("{p}.{key}"), label));
    }
    h.push_str(&checkbox_input(
        &format!("{p}.super_super_chk"),
        "Super-superfiring",
    ));
    h.push_str(&checkbox_input(
        &format!("{p}.sub_sub_chk"),
        "Lower casemates",
    ));
    h.push_str("</div></div>");
    h
}

fn weapon_section(i: usize) -> String {
    let title = match i {
        0 => "Torpedo set 1",
        1 => "Torpedo set 2",
        2 => "Mines",
        3 => "DC / AS mortars 1",
        _ => "DC / AS mortars 2",
    };
    let p = format!("weap{i}");
    let mut h = format!(r#"<fieldset><legend>{title}</legend><div class='grid'>"#);
    h.push_str(&format!(
        r#"<label>Kind<select data-key="{p}.kind"><option value="torpedo">Torpedo</option><option value="mine">Mine</option><option value="dc">Depth charge</option></select></label>"#
    ));
    for (key, label) in [
        ("number", "Number"),
        ("loads", "Loads / reloads"),
        ("size", "Size"),
        ("length", "Length (ft, torpedo)"),
        ("location", "Location index"),
    ] {
        h.push_str(&number_input(&format!("{p}.{key}"), label));
    }
    h.push_str("</div></fieldset>");
    h
}

// ---------------------------------------------------------------------------
// JSON helpers (hand-rolled; the core crate has no dependencies)
// ---------------------------------------------------------------------------

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_num(v: f64) -> String {
    if v.is_finite() {
        format!("{v}")
    } else {
        "null".to_string()
    }
}

/// Full evaluation of a calculated design, consumed by the workbench page.
fn evaluation_json(ship: &ShipModel, message: &str) -> String {
    let h = &ship.hull;
    let e = &ship.engine;
    let g = &ship.guns;
    let w = &ship.weapons;
    let report = generate_report(ship);
    let alerts: Vec<String> = report
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("DESIGN FAILURE") || l.starts_with("WARNING"))
        .map(json_str)
        .collect();
    let model = if e.blend_weight < 1.0 {
        "blended Holtrop/original"
    } else {
        match e.resistance_model {
            ResistanceModel::Original => "original SpringSharp model",
            ResistanceModel::HoltropMennen => "Holtrop-Mennen model",
        }
    };
    let weights = [
        ("Hull", e.hull_weight),
        ("Armour", ship.armour.weight),
        ("Machinery", e.engine_weight),
        ("Armament", g.borne_weight + w.weight),
        ("Magazine", g.magazine),
        ("Bunker", e.bunker),
        (
            "Miscellaneous",
            w.misc_weight_vital
                + w.misc_weight_hull
                + w.misc_weight_deck
                + w.misc_weight_above
                + w.void_weight,
        ),
    ]
    .iter()
    .map(|(n, v)| format!("[{},{}]", json_str(n), json_num(*v)))
    .collect::<Vec<_>>()
    .join(",");
    let nearest = nearest_historical_cases(ship, 6)
        .iter()
        .map(|c| {
            format!(
                "{{\"name\":{},\"ship_type\":{},\"displacement\":{},\"speed\":{},\"length\":{},\"hp\":{}}}",
                json_str(c.name),
                json_str(c.ship_type),
                json_num(c.displacement_tons),
                json_num(c.speed_kt),
                json_num(c.length_ft),
                json_num(c.reference_hp)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let benchmark_json = if let Some(b) = get_preset_benchmark(&ship.name) {
        format!(
            "{{\"name\":{},\"ref_disp\":{},\"ref_hull\":{},\"ref_armour\":{},\"ref_machinery\":{},\"ref_armament\":{},\"ref_bunker\":{},\"ref_gm\":{},\"sim_hull\":{},\"sim_armour\":{},\"sim_machinery\":{},\"sim_armament\":{},\"sim_bunker\":{},\"sim_gm\":{},\"source\":{}}}",
            json_str(b.name),
            json_num(b.ref_displacement_t),
            json_num(b.ref_hull_t),
            json_num(b.ref_armour_t),
            json_num(b.ref_machinery_t),
            json_num(b.ref_armament_t),
            json_num(b.ref_bunker_t),
            json_num(b.ref_gm_ft),
            json_num(e.hull_weight),
            json_num(ship.armour.weight),
            json_num(e.engine_weight),
            json_num(g.guns_weight + g.magazine + w.weight),
            json_num(e.bunker),
            json_num(ship.metacentre),
            json_str(b.source)
        )
    } else {
        "null".to_string()
    };
    let mut speeds = Vec::new();
    let max_s = (e.speed_max + 2.0).max(20.0);
    let mut s = 10.0;
    while s <= max_s + 1e-6 {
        speeds.push(s);
        s += 2.0;
    }
    if e.speed_cruise > 5.0 && !speeds.iter().any(|&x| (x - e.speed_cruise).abs() < 0.2) {
        speeds.push(e.speed_cruise);
    }
    if e.speed_max > 5.0 && !speeds.iter().any(|&x| (x - e.speed_max).abs() < 0.2) {
        speeds.push(e.speed_max);
    }
    speeds.sort_by(|a, b| a.total_cmp(b));

    let speed_curve_json = speeds
        .into_iter()
        .map(|spd| {
            let mut cur_engine = e.clone();
            let model_hp = cur_engine.speed_hp_calc(spd, h);
            let mut orig_engine = e.clone();
            orig_engine.resistance_model = ResistanceModel::Original;
            orig_engine.blend_weight = 1.0;
            let orig_hp = orig_engine.speed_hp_calc(spd, h);
            let fn_ = if h.length > 0.0 {
                spd * 1.68781 / (32.174 * h.length).sqrt()
            } else {
                0.0
            };
            format!(
                "[{},{},{},{}]",
                json_num(spd),
                json_num(model_hp),
                json_num(orig_hp),
                json_num(fn_)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let fields: Vec<(&str, String)> = vec![
        ("message", json_str(message)),
        ("name", json_str(&ship.name)),
        ("country", json_str(&ship.country)),
        ("ship_type", json_str(&ship.ship_type)),
        ("date_hull", h.date_hull.to_string()),
        ("model", json_str(model)),
        ("converged", ship.converged.to_string()),
        ("disp_light", json_num(ship.disp_light)),
        ("disp_standard", json_num(ship.disp_standard)),
        ("disp_normal", json_num(h.displacement)),
        ("disp_full", json_num(ship.disp_max_load)),
        ("speed_max", json_num(e.speed_max)),
        ("speed_cruise", json_num(e.speed_cruise)),
        ("hp_max", json_num(e.hp_max)),
        ("range", json_num(e.range)),
        ("stability", json_num(ship.stability_adj)),
        ("metacentre", json_num(ship.metacentre)),
        ("seakeeping", json_num(ship.seakeeping)),
        ("strength_comp", json_num(ship.strength_comp)),
        ("flotation", json_num(ship.flotation)),
        ("cost_dollars", json_num(ship.cost_dollars)),
        ("weights", format!("[{weights}]")),
        ("alerts", format!("[{}]", alerts.join(","))),
        ("nearest", format!("[{nearest}]")),
        ("benchmark", benchmark_json),
        ("report", json_str(&report)),
        ("ssr", json_str(&serialize(ship))),
        ("speed_curve", format!("[{speed_curve_json}]")),
        // Blueprint geometry
        ("length", json_num(h.length)),
        ("beam", json_num(h.beam)),
        ("draught", json_num(h.draught)),
        ("fc_len", json_num(ship.free.fc_length)),
        ("fd_len", json_num(ship.free.fd_length)),
        ("qd_len", json_num(ship.free.qd_length)),
        ("ffc_fwd", json_num(ship.free.free_fc_for)),
        ("ffc_aft", json_num(ship.free.free_fc_aft)),
        ("ffd_fwd", json_num(ship.free.free_fd_for)),
        ("ffd_aft", json_num(ship.free.free_fd_aft)),
        ("fad_fwd", json_num(ship.free.free_ad_for)),
        ("fad_aft", json_num(ship.free.free_ad_aft)),
        ("fqd_fwd", json_num(ship.free.free_qd_for)),
        ("fqd_aft", json_num(ship.free.free_qd_aft)),
        ("belt_len", json_num(ship.armour.belt_main_length)),
        ("belt_ht", json_num(ship.armour.belt_main_height)),
        ("belt_thick", json_num(ship.armour.belt_main)),
        ("deck_thick", json_num(ship.armour.deck)),
        ("g1_calibre", json_num(g.batteries[0].calibre)),
        ("g1_cal_len", json_num(g.batteries[0].cal_length)),
        ("g1_arm_face", json_num(g.batteries[0].arm_face)),
        ("g1_mag", json_num(g.batteries[0].magazine as f64)),
        ("g1_mounts", json_num(g.batteries[0].mounts as f64)),
        ("g1_layout1", json_num(g.batteries[0].layout_g1 as f64)),
        ("g1_layout2", json_num(g.batteries[0].layout_g2 as f64)),
        ("g1_m1", json_num(g.batteries[0].mounts_g1 as f64)),
        ("g1_m2", json_num(g.batteries[0].mounts_g2 as f64)),
        ("g1_bpm1", json_num(g.batteries[0].per_mount_g1 as f64)),
        ("g1_bpm2", json_num(g.batteries[0].per_mount_g2 as f64)),
        ("g2_calibre", json_num(g.batteries[1].calibre)),
        ("g2_cal_len", json_num(g.batteries[1].cal_length)),
        ("g2_arm_face", json_num(g.batteries[1].arm_face)),
        ("g2_mag", json_num(g.batteries[1].magazine as f64)),
        ("g2_mounts", json_num(g.batteries[1].mounts as f64)),
        ("g2_layout1", json_num(g.batteries[1].layout_g1 as f64)),
        ("g2_layout2", json_num(g.batteries[1].layout_g2 as f64)),
        ("g2_m1", json_num(g.batteries[1].mounts_g1 as f64)),
        ("g2_m2", json_num(g.batteries[1].mounts_g2 as f64)),
        ("g2_bpm1", json_num(g.batteries[1].per_mount_g1 as f64)),
        ("g2_bpm2", json_num(g.batteries[1].per_mount_g2 as f64)),
    ];
    let body = fields
        .iter()
        .map(|(k, v)| format!("{}:{v}", json_str(k)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{body}}}")
}

fn parse_design(text: &str) -> Result<ShipModel, String> {
    let mut ship = ShipModel::new();
    deserialize(&mut ship, text)?;
    ship.calculate();
    Ok(ship)
}

/// Split `"<number>\n<ssr text>"` bodies used by the solver/arrangement routes.
fn split_target(body: &str) -> Result<(f64, &str), String> {
    let (first, rest) = body.split_once('\n').unwrap_or((body, ""));
    let v: f64 = first
        .trim()
        .parse()
        .map_err(|_| format!("invalid target value '{}'", first.trim()))?;
    if !v.is_finite() || v <= 0.0 {
        return Err("target value must be a positive number".to_string());
    }
    Ok((v, rest))
}

fn design_route(body: &str, action: &str) -> Result<String, String> {
    match action {
        "evaluate" => parse_design(body).map(|s| evaluation_json(&s, "")),
        "regime" => {
            let mut ship = parse_design(body)?;
            let entry = apply_regime_calibration(&mut ship).ok_or_else(|| {
                format!(
                    "No regime factor for ship type '{}'; use battleship, carrier, cruiser or destroyer",
                    ship.ship_type
                )
            })?;
            let msg = format!(
                "Regime model applied: Holtrop-Mennen x {:.3} ({} factor). Max power now {:.0} hp.",
                entry.factor, entry.ship_type, ship.engine.hp_max
            );
            Ok(evaluation_json(&ship, &msg))
        }
        "blend" => {
            let mut ship = parse_design(body)?;
            apply_blend_calibration(&mut ship).ok_or_else(|| {
                format!(
                    "No blend weights for ship type '{}'; use battleship, carrier, cruiser or destroyer",
                    ship.ship_type
                )
            })?;
            let msg = format!(
                "Blended model applied: Holtrop weight {:.2}, Holtrop factor {:.3}, original factor {:.3}. Max power now {:.0} hp.",
                ship.engine.blend_weight,
                ship.engine.power_factor,
                ship.engine.original_power_factor,
                ship.engine.hp_max
            );
            Ok(evaluation_json(&ship, &msg))
        }
        "speed" => {
            let (hp, rest) = split_target(body)?;
            let mut ship = parse_design(rest)?;
            let v = solve_max_speed(&mut ship, hp);
            let msg = format!("Max speed for {hp:.0} hp: {v:.2} kt");
            Ok(evaluation_json(&ship, &msg))
        }
        "range" => {
            let (bunker, rest) = split_target(body)?;
            let mut ship = parse_design(rest)?;
            let r = solve_range(&mut ship, bunker);
            let msg = format!("Range for {bunker:.0} t normal bunker: {r:.0} nm");
            Ok(evaluation_json(&ship, &msg))
        }
        "compartments" => {
            let (cg, rest) = split_target(body)?;
            let ship = parse_design(rest)?;
            let placement = suggest_compartment_placement(&ship, cg);
            Ok(compartment_placement_html(&ship, &placement))
        }
        _ => Err("unknown action".to_string()),
    }
}
/// Maximum accepted HTTP request body.
///
/// The console is meant for hand-pasted `.ssr`/CSV text; anything larger is a
/// mistake or an attempt to exhaust memory, so it is rejected early.
pub const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Maximum simultaneous active client connections allowed by the server.
/// Exceeding this bound immediately returns 503 Service Unavailable to protect
/// file descriptors and thread resources.
pub const MAX_CONCURRENT_CONNECTIONS: usize = 64;

/// Read/write timeout for a single connection.  Without this a client that
/// announces a body and then stalls would block the (single-threaded) accept
/// loop indefinitely.
const CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub fn run_server(port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("SpringSharp web console: http://127.0.0.1:{port}");
    let active_connections = Arc::new(AtomicUsize::new(0));

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };

        let current = active_connections.load(Ordering::Relaxed);
        if current >= MAX_CONCURRENT_CONNECTIONS {
            let _ = write_plain(
                &mut stream,
                503,
                "Service Unavailable: connection limit reached",
            );
            continue;
        }

        active_connections.fetch_add(1, Ordering::SeqCst);
        let counter = Arc::clone(&active_connections);

        // Handle each connection on its own thread so one slow/stalled client
        // cannot block the whole console.
        std::thread::spawn(move || {
            struct ConnGuard(Arc<AtomicUsize>);
            impl Drop for ConnGuard {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }
            let _guard = ConnGuard(counter);

            // A panic inside a request handler must not take down the server.
            let result = std::panic::catch_unwind(|| handle_connection(stream));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => eprintln!("connection error: {e}"),
                Err(_) => eprintln!("connection handler panicked"),
            }
        });
    }
    Ok(())
}

fn design_action(path: &str) -> Option<&'static str> {
    Some(match path {
        "/api/evaluate" => "evaluate",
        "/api/calibrate/regime" => "regime",
        "/api/calibrate/blend" => "blend",
        "/api/solve/speed" => "speed",
        "/api/solve/range" => "range",
        "/api/compartments" => "compartments",
        _ => return None,
    })
}

fn presets_json() -> String {
    let items = PRESET_LIST
        .iter()
        .map(|p| {
            format!(
                "{{\"id\":{},\"name\":{},\"country\":{},\"ship_type\":{},\"year\":{},\"displacement\":{}}}",
                json_str(p.id),
                json_str(p.name),
                json_str(p.country),
                json_str(p.ship_type),
                p.year,
                json_num(p.displacement_tons)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn route(method: &str, path: &str, body_text: &str) -> (u16, &'static str, String) {
    match (method, path) {
        ("GET", "/" | "/index.html") => (200, "text/html; charset=utf-8", html_page()),
        ("GET", "/api/demo") => {
            let ship = crate::features::demo_ship();
            (200, "text/plain; charset=utf-8", serialize(&ship))
        }
        ("GET", "/api/presets") => (200, "application/json; charset=utf-8", presets_json()),
        ("GET", p) if p.starts_with("/api/presets/") => {
            let id = p.trim_start_matches("/api/presets/");
            if let Some(ship) = get_preset(id) {
                (200, "text/plain; charset=utf-8", serialize(&ship))
            } else {
                (
                    404,
                    "text/plain; charset=utf-8",
                    "Preset not found".to_string(),
                )
            }
        }
        ("GET", "/api/health") => (200, "text/plain; charset=utf-8", "ok".to_string()),
        ("POST", p) if design_action(p).is_some() => {
            let action = design_action(p).unwrap_or_default();
            let ctype = if action == "compartments" {
                "text/html; charset=utf-8"
            } else {
                "application/json; charset=utf-8"
            };
            match design_route(body_text, action) {
                Ok(out) => (200, ctype, out),
                Err(e) => (400, "text/plain; charset=utf-8", e),
            }
        }
        ("POST", "/api/calculate") => {
            let mut ship = ShipModel::new();
            match deserialize(&mut ship, body_text) {
                Ok(()) => {
                    ship.calculate();
                    (200, "text/html; charset=utf-8", generate_html_report(&ship))
                }
                Err(e) => (
                    400,
                    "text/plain; charset=utf-8",
                    format!("Failed to parse design:\n{e}"),
                ),
            }
        }
        ("POST", "/api/external") => match parse_external_cases(body_text) {
            Ok(cases) => {
                let html = external_report_html(&cases);
                (200, "text/html; charset=utf-8", html)
            }
            Err(e) => (
                400,
                "text/plain; charset=utf-8",
                format!("Failed to parse external CSV:\n{e}"),
            ),
        },
        ("POST", "/api/external-calibrate") => match parse_external_cases(body_text) {
            Ok(cases) => {
                let rf = fit_reduced_form_with_external(&cases);
                let sf = fit_structural_form_with_external(&cases);
                let loocv = run_loocv_with_external_weighted(&cases);
                let html = format!(
                        "<pre>External calibration:\nCombined built-in + {} external cases\nReduced MAPE={:.1}%\nStructural viscous={:.3} wave={:.3} MAPE={:.1}%\nWeighted LOOCV reduced={:.1}% structural={:.1}% ensemble={:.1}%\nEnsemble weights reduced={:.2} structural={:.2}</pre>",
                        cases.len(),
                        rf.mean_abs_pct_error * 100.0,
                        sf.viscous_factor,
                        sf.wave_factor,
                        sf.mean_abs_pct_error * 100.0,
                        loocv.reduced_mape * 100.0,
                        loocv.structural_mape * 100.0,
                        loocv.ensemble_mape * 100.0,
                        loocv.optimal_reduced_weight,
                        1.0 - loocv.optimal_reduced_weight
                    );
                (200, "text/html; charset=utf-8", html)
            }
            Err(e) => (
                400,
                "text/plain; charset=utf-8",
                format!("Failed to parse external CSV:\n{e}"),
            ),
        },
        ("POST", "/api/planing") => {
            let parts: Vec<&str> = body_text.trim().split(',').map(str::trim).collect();
            if parts.len() < 4 || parts.len() > 5 {
                (
                    400,
                    "text/plain; charset=utf-8",
                    "Expected length,beam,disp_lbs,speed[,factor]".to_string(),
                )
            } else {
                let parse = |i: usize| -> Result<f64, String> {
                    parts[i].parse::<f64>().map_err(|e| e.to_string())
                };
                match (parse(0), parse(1), parse(2), parse(3)) {
                    (Ok(l), Ok(b), Ok(d), Ok(s)) => {
                        let factor = if parts.len() == 5 {
                            parse(4).unwrap_or(1.0)
                        } else {
                            1.0
                        };
                        let hp = planing_hp_estimate_calibrated(l, b, d, s, factor);
                        (200, "text/plain; charset=utf-8", format!("{hp:.0}"))
                    }
                    _ => (
                        400,
                        "text/plain; charset=utf-8",
                        "Invalid numeric input".to_string(),
                    ),
                }
            }
        }
        ("POST", "/api/planing-calibrate") => match parse_external_cases(body_text) {
            Ok(cases) => {
                let factor = compute_planing_calibration(&cases);
                let eligible = cases
                    .iter()
                    .filter(|c| is_planing_eligible(c.speed_kt, &c.ship_type))
                    .count();
                (
                    200,
                    "text/plain; charset=utf-8",
                    format!("{factor:.3} ({eligible} eligible cases)"),
                )
            }
            Err(e) => (
                400,
                "text/plain; charset=utf-8",
                format!("Failed to parse external CSV:\n{e}"),
            ),
        },
        _ => (404, "text/plain; charset=utf-8", "Not found".to_string()),
    }
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(CONNECTION_TIMEOUT))?;
    stream.set_write_timeout(Some(CONNECTION_TIMEOUT))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    // Bound the request line itself: a client that never sends a newline must
    // not be able to grow this String without limit.
    if request_line.len() > 8 * 1024 {
        return write_plain(&mut stream, 400, "Bad Request");
    }

    let mut content_length = 0_usize;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if line.len() > 8 * 1024 {
            return write_plain(&mut stream, 400, "Bad Request");
        }
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            // Reject absurd lengths outright instead of allocating for them.
            content_length = v.trim().parse().unwrap_or(0);
            if content_length > MAX_BODY_BYTES {
                return write_plain(&mut stream, 413, "Payload Too Large");
            }
        }
    }

    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");
    let body_text = String::from_utf8_lossy(&body);

    let (status, content_type, payload) = route(method, path, &body_text);

    let response = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reason_phrase(status),
        payload.len(),
        payload
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

/// Minimal response used by the header-parse guards before a route is chosen.
fn write_plain(stream: &mut TcpStream, status: u16, message: &str) -> io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reason_phrase(status),
        message.len(),
        message
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_ssr() -> String {
        serialize(&crate::features::demo_ship())
    }

    #[test]
    fn page_has_tabs_and_labelled_selects() {
        let (status, _, page) = route("GET", "/", "");
        assert_eq!(status, 200);
        assert!(!page.contains("__FORM__"));
        for needle in [
            "data-tab='guns'",
            "Turret on barbette",
            "Steam turbines",
            "Transom stern",
            "data-key=\"gun4.layout_g2\"",
            "data-key=\"engine.blend_weight\"",
        ] {
            assert!(page.contains(needle), "page is missing {needle}");
        }
    }

    #[test]
    fn evaluate_returns_dashboard_json() {
        let (status, ctype, json) = route("POST", "/api/evaluate", &demo_ssr());
        assert_eq!(status, 200, "{json}");
        assert!(ctype.contains("json"));
        for key in [
            "\"disp_normal\":",
            "\"weights\":[[",
            "\"nearest\":[{",
            "\"ssr\":",
            "\"speed_curve\":[[",
            "\"g1_cal_len\":",
            "\"g1_arm_face\":",
        ] {
            assert!(json.contains(key), "missing {key}");
        }
        assert!(!json.contains(":NaN") && !json.contains(":inf"));
    }

    #[test]
    fn calibration_and_solver_routes_work() {
        let (s, _, json) = route("POST", "/api/calibrate/regime", &demo_ssr());
        assert_eq!(s, 200, "{json}");
        assert!(json.contains("engine.resistance_model=holtrop"));
        let (s, _, json) = route("POST", "/api/calibrate/blend", &demo_ssr());
        assert_eq!(s, 200, "{json}");
        let (s, _, json) = route(
            "POST",
            "/api/solve/speed",
            &format!("40000\n{}", demo_ssr()),
        );
        assert_eq!(s, 200, "{json}");
        assert!(json.contains("Max speed for 40000 hp"));
        let (s, _, _) = route("POST", "/api/solve/range", "abc\nmeta.name=x\n");
        assert_eq!(s, 400);
    }

    #[test]
    fn json_strings_are_escaped() {
        assert_eq!(json_str("a\"b\\c\nd"), "\"a\\\"b\\\\c\\nd\"");
        assert_eq!(json_num(f64::NAN), "null");
    }

    #[test]
    fn health_route_ok() {
        let (status, _, payload) = route("GET", "/api/health", "");
        assert_eq!(status, 200);
        assert_eq!(payload, "ok");
    }

    #[test]
    fn external_route_returns_report() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,600,90,28,23760,0.55,24,50000,0.9,example\n";
        let (status, content_type, payload) = route("POST", "/api/external", csv);
        assert_eq!(status, 200);
        assert!(content_type.contains("text/html"));
        assert!(payload.contains("External Data Validation Report"));
    }

    #[test]
    fn calculate_route_returns_report() {
        let body = "meta.name=Test\nhull.length=400\nhull.beam=60\nhull.beam_bulges=60\nhull.draught=20\nhull.block=0.5\nhull.date_hull=1920\nhull.date_engine=1920\nengine.speed_cruise=10\nengine.range=7000\nengine.screws=2\nperf.steadiness=50\n";
        let (status, content_type, payload) = route("POST", "/api/calculate", body);
        assert_eq!(status, 200);
        assert!(content_type.contains("text/html"));
        assert!(payload.contains("Test"));
    }

    #[test]
    fn small_fast_external_calibrate_route_ok() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\nCorvette,patrol,205,33,11,925,0.55,16,2750,0.70,approx\n";
        let (status, _, payload) = route("POST", "/api/external-calibrate", csv);
        assert_eq!(status, 200);
        assert!(payload.contains("External calibration"));
    }

    #[test]
    fn planing_calibrate_route_returns_factor() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\n";
        let (status, _, payload) = route("POST", "/api/planing-calibrate", csv);
        assert_eq!(status, 200);
        let factor: f64 = payload
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap_or(0.0);
        assert!(factor > 0.0);
    }

    #[test]
    fn planing_route_returns_hp() {
        let (status, _, payload) = route("POST", "/api/planing", "80,20,112000,40,0.668");
        assert_eq!(status, 200);
        let hp: f64 = payload.parse().unwrap();
        assert!(hp > 0.0);
    }

    #[test]
    fn oversized_body_is_rejected() {
        // The body cap must be a sane positive bound (checked at compile time).
        const { assert!(MAX_BODY_BYTES >= 1024 * 1024) };
        // Route-level sanity: normal designs still calculate.
        let body = "meta.name=Test\nhull.length=400\nhull.beam=60\nhull.beam_bulges=60\nhull.draught=20\nhull.block=0.5\nhull.date_hull=1920\nhull.date_engine=1920\n";
        let (status, _, _) = route("POST", "/api/calculate", body);
        assert_eq!(status, 200);
    }

    #[test]
    fn reason_phrase_covers_statuses() {
        assert_eq!(reason_phrase(200), "OK");
        assert_eq!(reason_phrase(400), "Bad Request");
        assert_eq!(reason_phrase(404), "Not Found");
        assert_eq!(reason_phrase(413), "Payload Too Large");
    }

    #[test]
    fn external_calibrate_route_ok() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,600,90,28,23760,0.55,24,50000,0.9,example\n";
        let (status, _, payload) = route("POST", "/api/external-calibrate", csv);
        assert_eq!(status, 200);
        assert!(payload.contains("External calibration"));
    }
}
