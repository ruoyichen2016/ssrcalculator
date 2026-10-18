//! File save/load support for `ShipModel`.
//!
//! This is a simple, human-readable line-oriented format.  It is intentionally
//! independent from the original WinForms `.ssf` format so that the Rust port
//! can store every input field without depending on the old UI order.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::model::{GunBattery, ShipModel, WeaponKind, WeaponMount};

pub fn serialize(ship: &ShipModel) -> String {
    let mut lines = vec!["SpringSharp Rust Save 1.0".to_string()];
    let mut push = |k: &str, v: String| lines.push(format!("{k}={v}"));

    push("meta.name", ship.name.clone());
    push("meta.country", ship.country.clone());
    push("meta.type", ship.ship_type.clone());

    push("hull.length", ship.hull.length.to_string());
    push("hull.beam", ship.hull.beam.to_string());
    push("hull.beam_bulges", ship.hull.beam_bulges.to_string());
    push("hull.draught", ship.hull.draught.to_string());
    push("hull.block", ship.hull.block.to_string());
    push("hull.date_hull", ship.hull.date_hull.to_string());
    push("hull.date_engine", ship.hull.date_engine.to_string());
    push("hull.lcb_percent", ship.hull.lcb_percent.to_string());
    push(
        "hull.midship_coefficient",
        ship.hull.midship_coefficient.to_string(),
    );
    push("hull.bulb_area_sqft", ship.hull.bulb_area_sqft.to_string());
    push(
        "hull.bulb_centre_height_ft",
        ship.hull.bulb_centre_height_ft.to_string(),
    );
    push(
        "hull.transom_area_sqft",
        ship.hull.transom_area_sqft.to_string(),
    );
    push(
        "hull.appendage_area_sqft",
        ship.hull.appendage_area_sqft.to_string(),
    );
    push("hull.stern_shape", ship.hull.stern_shape.to_string());

    push("free.fc_length", ship.free.fc_length.to_string());
    push("free.fd_length", ship.free.fd_length.to_string());
    push("free.qd_length", ship.free.qd_length.to_string());
    push("free.free_fc_for", ship.free.free_fc_for.to_string());
    push("free.free_fc_aft", ship.free.free_fc_aft.to_string());
    push("free.free_fd_for", ship.free.free_fd_for.to_string());
    push("free.free_fd_aft", ship.free.free_fd_aft.to_string());
    push("free.free_ad_for", ship.free.free_ad_for.to_string());
    push("free.free_ad_aft", ship.free.free_ad_aft.to_string());
    push("free.free_qd_for", ship.free.free_qd_for.to_string());
    push("free.free_qd_aft", ship.free.free_qd_aft.to_string());
    push("free.length_bow", ship.free.length_bow.to_string());
    push("free.length_ram", ship.free.length_ram.to_string());
    push("free.length_stern", ship.free.length_stern.to_string());
    push("free.bow", ship.free.bow.to_string());
    push("free.stern", ship.free.stern.to_string());
    push("free.transom_width", ship.free.transom_width.to_string());

    push("arm.belt_main", ship.armour.belt_main.to_string());
    push(
        "arm.belt_main_length",
        ship.armour.belt_main_length.to_string(),
    );
    push(
        "arm.belt_main_height",
        ship.armour.belt_main_height.to_string(),
    );
    push("arm.belt_ends", ship.armour.belt_ends.to_string());
    push(
        "arm.belt_ends_length",
        ship.armour.belt_ends_length.to_string(),
    );
    push(
        "arm.belt_ends_height",
        ship.armour.belt_ends_height.to_string(),
    );
    push("arm.belt_upper", ship.armour.belt_upper.to_string());
    push(
        "arm.belt_upper_length",
        ship.armour.belt_upper_length.to_string(),
    );
    push(
        "arm.belt_upper_height",
        ship.armour.belt_upper_height.to_string(),
    );
    push("arm.belts_incline", ship.armour.belts_incline.to_string());
    push("arm.bulkhead", ship.armour.bulkhead.to_string());
    push(
        "arm.bulkhead_length",
        ship.armour.bulkhead_length.to_string(),
    );
    push(
        "arm.bulkhead_height",
        ship.armour.bulkhead_height.to_string(),
    );
    push("arm.bulkhead_beam", ship.armour.bulkhead_beam.to_string());
    push("arm.bulkhead_type", ship.armour.bulkhead_type.to_string());
    push("arm.bulge", ship.armour.bulge.to_string());
    push("arm.bulge_length", ship.armour.bulge_length.to_string());
    push("arm.bulge_height", ship.armour.bulge_height.to_string());
    push("arm.deck", ship.armour.deck.to_string());
    push("arm.deck_fc", ship.armour.deck_fc.to_string());
    push("arm.deck_qd", ship.armour.deck_qd.to_string());
    push("arm.deck_coverage", ship.armour.deck_coverage.to_string());
    push("arm.conning_for", ship.armour.conning_for.to_string());
    push("arm.conning_aft", ship.armour.conning_aft.to_string());

    for (i, g) in ship.guns.batteries.iter().enumerate() {
        let p = format!("gun{i}.");
        push(&format!("{p}barrels"), g.barrels.to_string());
        push(&format!("{p}calibre"), g.calibre.to_string());
        push(&format!("{p}cal_length"), g.cal_length.to_string());
        push(&format!("{p}date_gun"), g.date_gun.to_string());
        push(&format!("{p}gun_type"), g.gun_type.to_string());
        push(&format!("{p}mount_type"), g.mount_type.to_string());
        push(&format!("{p}magazine"), g.magazine.to_string());
        push(&format!("{p}mounts"), g.mounts.to_string());
        push(
            &format!("{p}guns_in_mount_g1"),
            g.guns_in_mount_g1.to_string(),
        );
        push(
            &format!("{p}guns_in_mount_g2"),
            g.guns_in_mount_g2.to_string(),
        );
        push(&format!("{p}layout_g1"), g.layout_g1.to_string());
        push(&format!("{p}layout_g2"), g.layout_g2.to_string());
        push(&format!("{p}mount_super_g1"), g.mount_super_g1.to_string());
        push(&format!("{p}mount_sub_g1"), g.mount_sub_g1.to_string());
        push(&format!("{p}mount_super_g2"), g.mount_super_g2.to_string());
        push(&format!("{p}mount_deck_g2"), g.mount_deck_g2.to_string());
        push(&format!("{p}mount_sub_g2"), g.mount_sub_g2.to_string());
        push(
            &format!("{p}super_super_chk"),
            g.super_super_chk.to_string(),
        );
        push(&format!("{p}sub_sub_chk"), g.sub_sub_chk.to_string());
        push(&format!("{p}arm_face"), g.arm_face.to_string());
        push(&format!("{p}arm_back"), g.arm_back.to_string());
        push(&format!("{p}arm_bottom"), g.arm_bottom.to_string());
        push(&format!("{p}weight_adj_g1"), g.weight_adj_g1.to_string());
        push(&format!("{p}weight_adj_g2"), g.weight_adj_g2.to_string());
    }

    let weapons = [
        &ship.weapons.torpedo1,
        &ship.weapons.torpedo2,
        &ship.weapons.mine,
        &ship.weapons.dc1,
        &ship.weapons.dc2,
    ];
    for (i, w) in weapons.iter().enumerate() {
        let p = format!("weap{i}.");
        let kind = match w.kind {
            WeaponKind::Torpedo => "torpedo",
            WeaponKind::Mine => "mine",
            WeaponKind::DepthCharge => "dc",
        };
        push(&format!("{p}kind"), kind.to_string());
        push(&format!("{p}number"), w.number.to_string());
        push(&format!("{p}loads"), w.loads.to_string());
        push(&format!("{p}size"), w.size.to_string());
        push(&format!("{p}length"), w.length.to_string());
        push(&format!("{p}location"), w.location.to_string());
    }

    push("misc.vital", ship.weapons.misc_weight_vital.to_string());
    push("misc.hull", ship.weapons.misc_weight_hull.to_string());
    push("misc.deck", ship.weapons.misc_weight_deck.to_string());
    push("misc.above", ship.weapons.misc_weight_above.to_string());
    push("misc.void", ship.weapons.void_weight.to_string());

    push("engine.speed_max", ship.engine.speed_max.to_string());
    push("engine.speed_cruise", ship.engine.speed_cruise.to_string());
    push("engine.range", ship.engine.range.to_string());
    push("engine.pct_coal", ship.engine.pct_coal.to_string());
    push("engine.screws", ship.engine.screws.to_string());
    push("engine.power_factor", ship.engine.power_factor.to_string());
    push(
        "engine.original_power_factor",
        ship.engine.original_power_factor.to_string(),
    );
    push("engine.blend_weight", ship.engine.blend_weight.to_string());
    push(
        "engine.resistance_model",
        match ship.engine.resistance_model {
            crate::model::ResistanceModel::Original => "original",
            crate::model::ResistanceModel::HoltropMennen => "holtrop",
        }
        .to_string(),
    );
    push(
        "engine.holtrop_viscous_factor",
        ship.engine.holtrop_viscous_factor.to_string(),
    );
    push(
        "engine.holtrop_wave_factor",
        ship.engine.holtrop_wave_factor.to_string(),
    );
    push(
        "engine.holtrop_correlation_factor",
        ship.engine.holtrop_correlation_factor.to_string(),
    );
    push(
        "engine.ensemble_reduced_weight",
        ship.engine.ensemble_reduced_weight.to_string(),
    );
    push(
        "engine.ensemble_structural_weight",
        ship.engine.ensemble_structural_weight.to_string(),
    );
    push(
        "engine.machinery_weight_factor",
        ship.engine.machinery_weight_factor.to_string(),
    );
    for (i, v) in ship.engine.fuel.iter().enumerate() {
        push(&format!("engine.fuel{i}"), v.to_string());
    }
    for (i, v) in ship.engine.engine_types.iter().enumerate() {
        push(&format!("engine.engine{i}"), v.to_string());
    }
    for (i, v) in ship.engine.drive.iter().enumerate() {
        push(&format!("engine.drive{i}"), v.to_string());
    }

    push("perf.steadiness", ship.steadiness.to_string());
    push(
        "spaces.crew_volume_per_person",
        ship.space_crew_volume_per_person.to_string(),
    );
    push(
        "spaces.machinery_volume_per_kw",
        ship.space_machinery_volume_per_kw.to_string(),
    );
    push(
        "spaces.stores_volume_per_ton",
        ship.space_stores_volume_per_ton.to_string(),
    );
    push(
        "planing.calibration_factor",
        ship.planing_calibration_factor.to_string(),
    );
    push(
        "cfd_proxy.enabled",
        ship.engine.cfd_proxy_enabled.to_string(),
    );
    push(
        "cfd_proxy.viscous",
        ship.engine.cfd_proxy_viscous.to_string(),
    );
    push("cfd_proxy.wave", ship.engine.cfd_proxy_wave.to_string());
    push("cfd_proxy.fn_ref", ship.engine.cfd_proxy_fn_ref.to_string());

    push(
        "systems.doctrine",
        ship.systems.doctrine.as_str().to_string(),
    );
    push(
        "systems.tactical_echelon",
        ship.systems.tactical_echelon.as_str().to_string(),
    );
    push(
        "systems.combat_modality",
        ship.systems.combat_modality.as_str().to_string(),
    );
    push(
        "systems.mobility_regime",
        ship.systems.mobility_regime.as_str().to_string(),
    );
    push(
        "systems.main_director",
        ship.systems.main_director.as_str().to_string(),
    );
    push(
        "systems.aa_director",
        ship.systems.aa_director.as_str().to_string(),
    );
    push(
        "systems.ballistic_computer",
        ship.systems.ballistic_computer.as_str().to_string(),
    );
    push(
        "systems.radar_suite",
        ship.systems.radar_suite.as_str().to_string(),
    );
    push(
        "systems.sonar_suite",
        ship.systems.sonar_suite.as_str().to_string(),
    );
    push(
        "systems.electrical_plant",
        ship.systems.electrical_plant.as_str().to_string(),
    );
    if let Some(kw) = ship.systems.custom_turbo_kw {
        push("systems.custom_turbo_kw", kw.to_string());
    }
    if let Some(kw) = ship.systems.custom_diesel_kw {
        push("systems.custom_diesel_kw", kw.to_string());
    }
    push(
        "systems.couple_weights",
        ship.systems.couple_weights.to_string(),
    );

    lines.join("\n") + "\n"
}

pub fn save_to_file(ship: &ShipModel, path: &str) -> io::Result<()> {
    let mut f = fs::File::create(Path::new(path))?;
    f.write_all(serialize(ship).as_bytes())?;
    Ok(())
}

pub fn load_from_file(ship: &mut ShipModel, path: &str) -> Result<(), String> {
    let text = fs::read_to_string(Path::new(path)).map_err(|e| e.to_string())?;
    if text.starts_with("SpringSharp Version") {
        load_legacy_springsharp(ship, &text)
    } else {
        deserialize(ship, &text)
    }
}

pub fn deserialize(ship: &mut ShipModel, text: &str) -> Result<(), String> {
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            if line.starts_with("SpringSharp Rust Save") {
                continue;
            }
            return Err(format!("malformed line: {line}"));
        };
        let key = key.trim();
        let value = value.trim();
        set_value(ship, key, value)?;
    }
    Ok(())
}

fn parse_f64(v: &str) -> Result<f64, String> {
    let parsed: f64 = v
        .parse()
        .map_err(|e| format!("invalid number '{v}': {e}"))?;
    if !parsed.is_finite() {
        return Err(format!(
            "number '{v}' must be finite (NaN/inf are not allowed)"
        ));
    }
    Ok(parsed)
}

/// Validate a parsed value against an inclusive range.
fn check_range(value: f64, lo: f64, hi: f64, field: &str) -> Result<f64, String> {
    if value < lo || value > hi {
        return Err(format!("{field} = {value} is out of range [{lo}, {hi}]"));
    }
    Ok(value)
}

fn parse_i32(v: &str) -> Result<i32, String> {
    v.parse::<i32>()
        .map_err(|e| format!("invalid integer '{v}': {e}"))
}

fn parse_bool(v: &str) -> Result<bool, String> {
    match v.to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(format!("invalid boolean '{v}'")),
    }
}

fn set_guns(ship: &mut ShipModel, idx: usize, key: &str, value: &str) -> Result<(), String> {
    let g: &mut GunBattery = &mut ship.guns.batteries[idx];
    match key {
        "barrels" => g.barrels = parse_i32(value)?,
        "calibre" => g.calibre = parse_f64(value)?,
        "cal_length" => g.cal_length = parse_f64(value)?,
        "date_gun" => g.date_gun = parse_i32(value)?,
        "gun_type" => g.gun_type = parse_i32(value)?,
        "mount_type" => g.mount_type = parse_i32(value)?,
        "magazine" => g.magazine = parse_i32(value)?,
        "mounts" => g.mounts = parse_i32(value)?,
        "guns_in_mount_g1" => g.guns_in_mount_g1 = parse_i32(value)?,
        "guns_in_mount_g2" => g.guns_in_mount_g2 = parse_i32(value)?,
        "layout_g1" => g.layout_g1 = parse_i32(value)?,
        "layout_g2" => g.layout_g2 = parse_i32(value)?,
        "mount_super_g1" => g.mount_super_g1 = parse_i32(value)?,
        "mount_sub_g1" => g.mount_sub_g1 = parse_i32(value)?,
        "mount_super_g2" => g.mount_super_g2 = parse_i32(value)?,
        "mount_deck_g2" => g.mount_deck_g2 = parse_i32(value)?,
        "mount_sub_g2" => g.mount_sub_g2 = parse_i32(value)?,
        "super_super_chk" => g.super_super_chk = parse_bool(value)?,
        "sub_sub_chk" => g.sub_sub_chk = parse_bool(value)?,
        "arm_face" => g.arm_face = parse_f64(value)?,
        "arm_back" => g.arm_back = parse_f64(value)?,
        "arm_bottom" => g.arm_bottom = parse_f64(value)?,
        "weight_adj_g1" => g.weight_adj_g1 = parse_f64(value)?,
        "weight_adj_g2" => g.weight_adj_g2 = parse_f64(value)?,
        _ => return Err(format!("unknown gun{idx} key '{key}'")),
    }
    Ok(())
}

fn set_weapon(ship: &mut ShipModel, idx: usize, key: &str, value: &str) -> Result<(), String> {
    let w: &mut WeaponMount = match idx {
        0 => &mut ship.weapons.torpedo1,
        1 => &mut ship.weapons.torpedo2,
        2 => &mut ship.weapons.mine,
        3 => &mut ship.weapons.dc1,
        4 => &mut ship.weapons.dc2,
        _ => return Err(format!("invalid weapon index {idx}")),
    };
    match key {
        "kind" => {
            w.kind = match value {
                "torpedo" => WeaponKind::Torpedo,
                "mine" => WeaponKind::Mine,
                "dc" => WeaponKind::DepthCharge,
                _ => return Err(format!("invalid weapon kind '{value}'")),
            }
        }
        "number" => w.number = parse_i32(value)?,
        "loads" => w.loads = parse_i32(value)?,
        "size" => w.size = parse_f64(value)?,
        "length" => w.length = parse_f64(value)?,
        "location" => w.location = parse_i32(value)?,
        _ => return Err(format!("unknown weapon{idx} key '{key}'")),
    }
    Ok(())
}

fn set_value(ship: &mut ShipModel, key: &str, value: &str) -> Result<(), String> {
    // Metadata
    match key {
        "meta.name" => {
            ship.name = value.to_string();
            return Ok(());
        }
        "meta.country" => {
            ship.country = value.to_string();
            return Ok(());
        }
        "meta.type" => {
            ship.ship_type = value.to_string();
            return Ok(());
        }
        _ => {}
    }

    let handled = match key {
        "hull.length" => {
            ship.hull.length = check_range(parse_f64(value)?, 1.0, 2000.0, "hull.length")?;
            true
        }
        "hull.beam" => {
            ship.hull.beam = check_range(parse_f64(value)?, 1.0, 500.0, "hull.beam")?;
            true
        }
        "hull.beam_bulges" => {
            ship.hull.beam_bulges = check_range(parse_f64(value)?, 1.0, 500.0, "hull.beam_bulges")?;
            true
        }
        "hull.draught" => {
            ship.hull.draught = check_range(parse_f64(value)?, 0.1, 200.0, "hull.draught")?;
            true
        }
        "hull.block" => {
            ship.hull.block = check_range(parse_f64(value)?, 0.05, 0.95, "hull.block")?;
            true
        }
        "hull.date_hull" => {
            ship.hull.date_hull = parse_i32(value)?;
            true
        }
        "hull.date_engine" => {
            ship.hull.date_engine = parse_i32(value)?;
            true
        }
        "hull.lcb_percent" => {
            ship.hull.lcb_percent =
                check_range(parse_f64(value)?, -100.0, 100.0, "hull.lcb_percent")?;
            true
        }
        "hull.midship_coefficient" => {
            ship.hull.midship_coefficient =
                check_range(parse_f64(value)?, 0.0, 1.0, "hull.midship_coefficient")?;
            true
        }
        "hull.bulb_area_sqft" => {
            ship.hull.bulb_area_sqft = parse_f64(value)?;
            true
        }
        "hull.bulb_centre_height_ft" => {
            ship.hull.bulb_centre_height_ft = parse_f64(value)?;
            true
        }
        "hull.transom_area_sqft" => {
            ship.hull.transom_area_sqft = parse_f64(value)?;
            true
        }
        "hull.appendage_area_sqft" => {
            ship.hull.appendage_area_sqft = parse_f64(value)?;
            true
        }
        "hull.stern_shape" => {
            ship.hull.stern_shape = parse_f64(value)?;
            true
        }
        "free.fc_length" => {
            ship.free.fc_length = parse_f64(value)?;
            true
        }
        "free.fd_length" => {
            ship.free.fd_length = parse_f64(value)?;
            true
        }
        "free.qd_length" => {
            ship.free.qd_length = parse_f64(value)?;
            true
        }
        "free.free_fc_for" => {
            ship.free.free_fc_for = parse_f64(value)?;
            true
        }
        "free.free_fc_aft" => {
            ship.free.free_fc_aft = parse_f64(value)?;
            true
        }
        "free.free_fd_for" => {
            ship.free.free_fd_for = parse_f64(value)?;
            true
        }
        "free.free_fd_aft" => {
            ship.free.free_fd_aft = parse_f64(value)?;
            true
        }
        "free.free_ad_for" => {
            ship.free.free_ad_for = parse_f64(value)?;
            true
        }
        "free.free_ad_aft" => {
            ship.free.free_ad_aft = parse_f64(value)?;
            true
        }
        "free.free_qd_for" => {
            ship.free.free_qd_for = parse_f64(value)?;
            true
        }
        "free.free_qd_aft" => {
            ship.free.free_qd_aft = parse_f64(value)?;
            true
        }
        "free.length_bow" => {
            ship.free.length_bow = parse_f64(value)?;
            true
        }
        "free.length_ram" => {
            ship.free.length_ram = parse_f64(value)?;
            true
        }
        "free.length_stern" => {
            ship.free.length_stern = parse_f64(value)?;
            true
        }
        "free.bow" => {
            ship.free.bow = parse_i32(value)?;
            true
        }
        "free.stern" => {
            ship.free.stern = parse_i32(value)?;
            true
        }
        "free.transom_width" => {
            ship.free.transom_width = parse_f64(value)?;
            true
        }
        "arm.belt_main" => {
            ship.armour.belt_main = parse_f64(value)?;
            true
        }
        "arm.belt_main_length" => {
            ship.armour.belt_main_length = parse_f64(value)?;
            true
        }
        "arm.belt_main_height" => {
            ship.armour.belt_main_height = parse_f64(value)?;
            true
        }
        "arm.belt_ends" => {
            ship.armour.belt_ends = parse_f64(value)?;
            true
        }
        "arm.belt_ends_length" => {
            ship.armour.belt_ends_length = parse_f64(value)?;
            true
        }
        "arm.belt_ends_height" => {
            ship.armour.belt_ends_height = parse_f64(value)?;
            true
        }
        "arm.belt_upper" => {
            ship.armour.belt_upper = parse_f64(value)?;
            true
        }
        "arm.belt_upper_length" => {
            ship.armour.belt_upper_length = parse_f64(value)?;
            true
        }
        "arm.belt_upper_height" => {
            ship.armour.belt_upper_height = parse_f64(value)?;
            true
        }
        "arm.belts_incline" => {
            ship.armour.belts_incline = parse_f64(value)?;
            true
        }
        "arm.bulkhead" => {
            ship.armour.bulkhead = parse_f64(value)?;
            true
        }
        "arm.bulkhead_length" => {
            ship.armour.bulkhead_length = parse_f64(value)?;
            true
        }
        "arm.bulkhead_height" => {
            ship.armour.bulkhead_height = parse_f64(value)?;
            true
        }
        "arm.bulkhead_beam" => {
            ship.armour.bulkhead_beam = parse_f64(value)?;
            true
        }
        "arm.bulkhead_type" => {
            ship.armour.bulkhead_type = parse_i32(value)?;
            true
        }
        "arm.bulge" => {
            ship.armour.bulge = parse_f64(value)?;
            true
        }
        "arm.bulge_length" => {
            ship.armour.bulge_length = parse_f64(value)?;
            true
        }
        "arm.bulge_height" => {
            ship.armour.bulge_height = parse_f64(value)?;
            true
        }
        "arm.deck" => {
            ship.armour.deck = parse_f64(value)?;
            true
        }
        "arm.deck_fc" => {
            ship.armour.deck_fc = parse_f64(value)?;
            true
        }
        "arm.deck_qd" => {
            ship.armour.deck_qd = parse_f64(value)?;
            true
        }
        "arm.deck_coverage" => {
            ship.armour.deck_coverage = parse_i32(value)?;
            true
        }
        "arm.conning_for" => {
            ship.armour.conning_for = parse_f64(value)?;
            true
        }
        "arm.conning_aft" => {
            ship.armour.conning_aft = parse_f64(value)?;
            true
        }
        "misc.vital" => {
            ship.weapons.misc_weight_vital = parse_f64(value)?;
            true
        }
        "misc.hull" => {
            ship.weapons.misc_weight_hull = parse_f64(value)?;
            true
        }
        "misc.deck" => {
            ship.weapons.misc_weight_deck = parse_f64(value)?;
            true
        }
        "misc.above" => {
            ship.weapons.misc_weight_above = parse_f64(value)?;
            true
        }
        "misc.void" => {
            ship.weapons.void_weight = parse_f64(value)?;
            true
        }
        "engine.speed_max" => {
            ship.engine.speed_max = parse_f64(value)?;
            true
        }
        "engine.speed_cruise" => {
            ship.engine.speed_cruise = parse_f64(value)?;
            true
        }
        "engine.range" => {
            ship.engine.range = parse_f64(value)?;
            true
        }
        "engine.pct_coal" => {
            ship.engine.pct_coal = parse_f64(value)?;
            true
        }
        "engine.screws" => {
            ship.engine.screws = parse_i32(value)?;
            true
        }
        "engine.power_factor" => {
            ship.engine.power_factor = parse_f64(value)?;
            true
        }
        "engine.original_power_factor" => {
            ship.engine.original_power_factor = parse_f64(value)?;
            true
        }
        "engine.blend_weight" => {
            ship.engine.blend_weight = parse_f64(value)?;
            true
        }
        "engine.resistance_model" => {
            ship.engine.resistance_model = match value {
                "original" => crate::model::ResistanceModel::Original,
                "holtrop" => crate::model::ResistanceModel::HoltropMennen,
                _ => return Err(format!("invalid resistance model '{value}'")),
            };
            true
        }
        "engine.holtrop_viscous_factor" => {
            ship.engine.holtrop_viscous_factor = parse_f64(value)?;
            true
        }
        "engine.holtrop_wave_factor" => {
            ship.engine.holtrop_wave_factor = parse_f64(value)?;
            true
        }
        "engine.holtrop_correlation_factor" => {
            ship.engine.holtrop_correlation_factor = parse_f64(value)?;
            true
        }
        "engine.ensemble_reduced_weight" => {
            ship.engine.ensemble_reduced_weight = parse_f64(value)?;
            true
        }
        "engine.ensemble_structural_weight" => {
            ship.engine.ensemble_structural_weight = parse_f64(value)?;
            true
        }
        "engine.machinery_weight_factor" => {
            ship.engine.machinery_weight_factor = parse_f64(value)?;
            true
        }
        "perf.steadiness" => {
            ship.steadiness = parse_i32(value)?;
            true
        }
        "spaces.crew_volume_per_person" => {
            ship.space_crew_volume_per_person = parse_f64(value)?;
            true
        }
        "spaces.machinery_volume_per_kw" => {
            ship.space_machinery_volume_per_kw = parse_f64(value)?;
            true
        }
        "spaces.stores_volume_per_ton" => {
            ship.space_stores_volume_per_ton = parse_f64(value)?;
            true
        }
        "planing.calibration_factor" => {
            ship.planing_calibration_factor = parse_f64(value)?;
            true
        }
        "planing.min_speed_kt" => {
            ship.engine.planing_min_speed_kt = parse_f64(value)?;
            true
        }
        "planing.max_displacement_tons" => {
            ship.engine.planing_max_displacement_tons = parse_f64(value)?;
            true
        }
        "cfd_proxy.enabled" => {
            ship.engine.cfd_proxy_enabled = parse_bool(value)?;
            true
        }
        "cfd_proxy.viscous" => {
            ship.engine.cfd_proxy_viscous = parse_f64(value)?;
            true
        }
        "cfd_proxy.wave" => {
            ship.engine.cfd_proxy_wave = parse_f64(value)?;
            true
        }
        "cfd_proxy.fn_ref" => {
            ship.engine.cfd_proxy_fn_ref = parse_f64(value)?;
            true
        }
        "systems.doctrine" => {
            ship.systems.doctrine = crate::systems::StyleDoctrine::from_str_name(value);
            true
        }
        "systems.tactical_echelon" => {
            ship.systems.tactical_echelon = crate::systems::TacticalEchelon::from_str_name(value);
            true
        }
        "systems.combat_modality" => {
            ship.systems.combat_modality = crate::systems::CombatModality::from_str_name(value);
            true
        }
        "systems.mobility_regime" => {
            ship.systems.mobility_regime = crate::systems::MobilityRegime::from_str_name(value);
            true
        }
        "systems.main_director" => {
            ship.systems.main_director = crate::systems::MainDirector::from_str_name(value);
            true
        }
        "systems.aa_director" => {
            ship.systems.aa_director = crate::systems::AaDirector::from_str_name(value);
            true
        }
        "systems.ballistic_computer" => {
            ship.systems.ballistic_computer =
                crate::systems::BallisticComputer::from_str_name(value);
            true
        }
        "systems.radar_suite" => {
            ship.systems.radar_suite = crate::systems::RadarSuite::from_str_name(value);
            true
        }
        "systems.sonar_suite" => {
            ship.systems.sonar_suite = crate::systems::SonarSuite::from_str_name(value);
            true
        }
        "systems.electrical_plant" => {
            ship.systems.electrical_plant =
                crate::systems::ElectricalPlantStyle::from_str_name(value);
            true
        }
        "systems.custom_turbo_kw" => {
            ship.systems.custom_turbo_kw = Some(parse_f64(value)?);
            true
        }
        "systems.custom_diesel_kw" => {
            ship.systems.custom_diesel_kw = Some(parse_f64(value)?);
            true
        }
        "systems.couple_weights" => {
            ship.systems.couple_weights = parse_bool(value)?;
            true
        }
        _ => false,
    };
    if handled {
        return Ok(());
    }

    if let Some(rest) = key.strip_prefix("gun") {
        let Some((idx_str, sub)) = rest.split_once('.') else {
            return Err(format!("malformed gun key '{key}'"));
        };
        let idx: usize = idx_str
            .parse()
            .map_err(|_| format!("bad gun index in '{key}'"))?;
        if idx < 5 {
            return set_guns(ship, idx, sub, value);
        }
    }
    if let Some(rest) = key.strip_prefix("weap") {
        let Some((idx_str, sub)) = rest.split_once('.') else {
            return Err(format!("malformed weapon key '{key}'"));
        };
        let idx: usize = idx_str
            .parse()
            .map_err(|_| format!("bad weapon index in '{key}'"))?;
        if idx < 5 {
            return set_weapon(ship, idx, sub, value);
        }
    }
    if let Some(rest) = key.strip_prefix("engine.fuel") {
        let idx: usize = rest
            .parse()
            .map_err(|_| format!("bad fuel index in '{key}'"))?;
        if idx < 5 {
            ship.engine.fuel[idx] = parse_bool(value)?;
            return Ok(());
        }
    }
    if let Some(rest) = key.strip_prefix("engine.engine") {
        let idx: usize = rest
            .parse()
            .map_err(|_| format!("bad engine index in '{key}'"))?;
        if idx < 3 {
            ship.engine.engine_types[idx] = parse_bool(value)?;
            return Ok(());
        }
    }
    if let Some(rest) = key.strip_prefix("engine.drive") {
        let idx: usize = rest
            .parse()
            .map_err(|_| format!("bad drive index in '{key}'"))?;
        if idx < 4 {
            ship.engine.drive[idx] = parse_bool(value)?;
            return Ok(());
        }
    }

    if !key.starts_with("meta.") {
        return Err(format!("unknown key '{key}'"));
    }
    Ok(())
}

/// Load a file written by the original SpringSharp 3.0 WinForms program.
///
/// The original save format is a header line followed by 280 fixed-position
/// fields.  This function maps the indices used by `MainForm.OpenFileCalc`
/// into the Rust model.  Version 2.0 files are not supported yet.
pub fn load_legacy_springsharp(ship: &mut ShipModel, text: &str) -> Result<(), String> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| "empty SpringSharp file".to_string())?;
    if header != "SpringSharp Version 3.0" && header != "SpringSharp Version 3,0" {
        return Err(format!("unsupported SpringSharp header '{header}'"));
    }

    let fields: Vec<&str> = lines.take(280).collect();
    if fields.len() < 280 {
        return Err(format!(
            "old SpringSharp file has {} fields, expected 280",
            fields.len()
        ));
    }

    let pf = |i: usize| -> Result<f64, String> {
        fields
            .get(i)
            .ok_or_else(|| format!("missing field {i}"))?
            .trim()
            .parse::<f64>()
            .map_err(|_e| format!("field {i} is not a number: {}", fields[i]))
    };
    let pi = |i: usize| -> Result<i32, String> {
        fields
            .get(i)
            .ok_or_else(|| format!("missing field {i}"))?
            .trim()
            .parse::<i32>()
            .map_err(|_e| format!("field {i} is not an integer: {}", fields[i]))
    };
    let pb = |i: usize| -> Result<bool, String> {
        let v = fields
            .get(i)
            .ok_or_else(|| format!("missing field {i}"))?
            .trim();
        match v {
            "True" | "true" | "1" => Ok(true),
            "False" | "false" | "0" => Ok(false),
            _ => Err(format!("field {i} is not a boolean: {v}")),
        }
    };

    ship.name = fields[0].trim().to_string();
    ship.country = fields[1].trim().to_string();
    ship.ship_type = fields[2].trim().to_string();

    ship.hull.date_hull = pi(11)?;
    ship.hull.date_engine = pi(128)?;
    ship.hull.length = pf(13)?;
    ship.hull.beam = pf(14)?;
    ship.hull.draught = pf(15)?;
    ship.hull.beam_bulges = pf(127)?;
    ship.hull.block = pf(17)?;

    ship.free.fc_length = pf(26)? / 100.0;
    ship.free.fd_length = pf(23)? / 100.0;
    ship.free.qd_length = pf(20)? / 100.0;
    ship.free.free_fc_for = pf(29)?;
    ship.free.free_fc_aft = pf(28)?;
    ship.free.free_fd_for = pf(27)?;
    ship.free.free_fd_aft = pf(25)?;
    ship.free.free_ad_for = pf(24)?;
    ship.free.free_ad_aft = pf(22)?;
    ship.free.free_qd_for = pf(21)?;
    ship.free.free_qd_aft = pf(18)?;
    ship.free.length_bow = pf(30)?;
    ship.free.length_ram = pf(135)?;
    ship.free.length_stern = pf(19)?;
    ship.free.bow = pi(134)?;
    ship.free.stern = pi(16)?;
    ship.free.transom_width = pf(247)? / 100.0;

    for (i, g) in ship.guns.batteries.iter_mut().enumerate() {
        let base = 31 + 6 * i;
        g.barrels = pi(base)?;
        g.calibre = pf(base + 1)?;
        g.gun_type = pi(base + 2)?;
        g.mount_super_g1 = pi(base + 3)?;
        g.mount_sub_g1 = pi(base + 4)?;
        // Shell weight is computed by the engine; original file contains it for
        // lock-state compatibility, so it is intentionally ignored here.
        g.date_gun = pi(129 + i)?;
        g.cal_length = pf(140 + i)?;
        g.magazine = if i == 0 { pi(61)? } else { pi(144 + i)? };
        g.mounts = pi(62 + 3 * i)?;
        g.mount_type = pi(63 + 3 * i)?;
        g.layout_g1 = pi(64 + 3 * i)?;
        g.arm_face = pf(92 + 3 * i)?;
        g.arm_back = pf(93 + 3 * i)?;
        g.arm_bottom = pf(94 + 3 * i)?;
        g.layout_g2 = pi(149 + i)?;
        g.guns_in_mount_g1 = pi(236 + i)?;
        g.guns_in_mount_g2 = pi(241 + i)?;
        g.mount_super_g2 = pi(154 + i)?;
        g.super_super_chk = pb(159 + i)?;
        g.mount_deck_g2 = pi(164 + i)?;
        g.mount_sub_g2 = pi(169 + i)?;
        g.sub_sub_chk = pb(174 + i)?;
    }

    ship.weapons.torpedo1.number = pi(77)?;
    ship.weapons.torpedo2.number = pi(78)?;
    ship.weapons.torpedo1.loads = pi(179)?;
    ship.weapons.torpedo2.loads = pi(180)?;
    ship.weapons.torpedo1.size = pf(79)?;
    ship.weapons.torpedo2.size = pf(181)?;
    ship.weapons.torpedo1.length = pf(182)?;
    ship.weapons.torpedo2.length = pf(183)?;
    ship.weapons.torpedo1.location = pi(184)?;
    ship.weapons.torpedo2.location = pi(185)?;

    ship.weapons.mine.kind = WeaponKind::Mine;
    ship.weapons.mine.number = pi(186)?;
    ship.weapons.mine.loads = pi(187)?;
    ship.weapons.mine.size = pf(188)?;
    ship.weapons.mine.location = pi(189)?;

    ship.weapons.dc1.kind = WeaponKind::DepthCharge;
    ship.weapons.dc2.kind = WeaponKind::DepthCharge;
    ship.weapons.dc1.number = pi(190)?;
    ship.weapons.dc2.number = pi(191)?;
    ship.weapons.dc1.loads = pi(192)?;
    ship.weapons.dc2.loads = pi(193)?;
    ship.weapons.dc1.size = pf(194)?;
    ship.weapons.dc2.size = pf(195)?;
    ship.weapons.dc1.location = pi(196)?;
    ship.weapons.dc2.location = pi(197)?;

    ship.weapons.misc_weight_vital = pf(12)?;
    ship.weapons.misc_weight_hull = pf(198)?;
    ship.weapons.misc_weight_deck = pf(199)?;
    ship.weapons.misc_weight_above = pf(200)?;
    ship.weapons.void_weight = pf(246)?;

    ship.armour.belt_main = pf(80)?;
    ship.armour.belt_main_length = pf(81)?;
    ship.armour.belt_main_height = pf(82)?;
    ship.armour.belt_ends = pf(83)?;
    ship.armour.belt_ends_length = pf(84)?;
    ship.armour.belt_ends_height = pf(85)?;
    ship.armour.belt_upper = pf(86)?;
    ship.armour.belt_upper_length = pf(87)?;
    ship.armour.belt_upper_height = pf(88)?;
    ship.armour.bulkhead = pf(89)?;
    ship.armour.bulkhead_length = pf(90)?;
    ship.armour.bulkhead_height = pf(91)?;
    ship.armour.deck = pf(107)?;
    ship.armour.conning_for = pf(108)?;
    ship.armour.conning_aft = pf(210)?;
    ship.armour.belts_incline = pf(201)?;
    ship.armour.bulge = pf(202)?;
    ship.armour.bulge_length = pf(203)?;
    ship.armour.bulge_height = pf(204)?;
    ship.armour.bulkhead_type = pi(205)?;
    ship.armour.bulkhead_beam = pf(206)?;
    ship.armour.deck_fc = pf(207)?;
    ship.armour.deck_qd = pf(208)?;
    ship.armour.deck_coverage = pi(209)?;

    ship.engine.speed_max = pf(109)?;
    ship.engine.speed_cruise = pf(110)?;
    ship.engine.range = pf(111)?;
    ship.engine.screws = pi(112)?;
    ship.engine.pct_coal = pf(113)?;
    for j in 0..5 {
        ship.engine.fuel[j] = pb(114 + j)?;
    }
    for j in 0..3 {
        ship.engine.engine_types[j] = pb(119 + j)?;
    }
    for j in 0..4 {
        ship.engine.drive[j] = pb(122 + j)?;
    }
    ship.steadiness = pi(126)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ShipModel;

    #[test]
    fn save_load_round_trip() {
        let mut ship = ShipModel::new();
        ship.name = "Test Ship".into();
        ship.hull.length = 500.0;
        ship.hull.beam = 80.0;
        ship.hull.beam_bulges = 82.0;
        ship.hull.draught = 25.0;
        ship.hull.block = 0.5;
        ship.hull.date_hull = 1910;
        ship.hull.date_engine = 1912;
        ship.free.fc_length = 0.2;
        ship.free.fd_length = 0.3;
        ship.free.qd_length = 0.15;
        ship.engine.screws = 3;
        ship.engine.original_power_factor = 0.9;
        ship.engine.blend_weight = 0.6;
        ship.systems.tactical_echelon = crate::systems::TacticalEchelon::FastWingVanguard;
        ship.systems.combat_modality = crate::systems::CombatModality::AviationDeckProjection;
        ship.systems.mobility_regime = crate::systems::MobilityRegime::SlenderWaveCutting;

        let text = serialize(&ship);
        let mut loaded = ShipModel::new();
        deserialize(&mut loaded, &text).unwrap();

        assert_eq!(loaded.name, "Test Ship");
        assert_eq!(loaded.hull.length, 500.0);
        assert_eq!(loaded.hull.beam_bulges, 82.0);
        assert_eq!(loaded.hull.date_engine, 1912);
        assert_eq!(loaded.engine.screws, 3);
        assert_eq!(loaded.free.qd_length, 0.15);
        assert_eq!(loaded.engine.original_power_factor, 0.9);
        assert_eq!(loaded.engine.blend_weight, 0.6);
        assert_eq!(loaded.engine.planing_min_speed_kt, 25.0);
        assert_eq!(loaded.engine.planing_max_displacement_tons, 3000.0);
        assert_eq!(
            loaded.systems.tactical_echelon,
            crate::systems::TacticalEchelon::FastWingVanguard
        );
        assert_eq!(
            loaded.systems.combat_modality,
            crate::systems::CombatModality::AviationDeckProjection
        );
        assert_eq!(
            loaded.systems.mobility_regime,
            crate::systems::MobilityRegime::SlenderWaveCutting
        );
    }

    #[test]
    fn loads_legacy_springsharp_v3_format() {
        let mut lines = vec!["SpringSharp Version 3.0".to_string()];
        for _ in 0..280 {
            lines.push("0".to_string());
        }
        // Set a handful of representative fields.
        lines[1] = "Legacy Ship".to_string();
        lines[2] = "Britain".to_string();
        lines[3] = "Cruiser".to_string();
        lines[12] = "1915".to_string();
        lines[14] = "500".to_string();
        lines[15] = "75".to_string();
        lines[16] = "25".to_string();
        lines[18] = "0.52".to_string();
        lines[27] = "20".to_string();
        lines[24] = "30".to_string();
        lines[21] = "15".to_string();
        lines[32] = "8".to_string();
        lines[33] = "12".to_string();
        lines[64] = "5".to_string();
        lines[63] = "4".to_string();
        lines[110] = "22".to_string();
        lines[113] = "2".to_string();
        lines[115] = "True".to_string();
        lines[121] = "True".to_string();
        lines[123] = "True".to_string();
        lines[127] = "50".to_string();

        let mut ship = ShipModel::new();
        load_legacy_springsharp(&mut ship, &lines.join("\n")).unwrap();
        assert_eq!(ship.name, "Legacy Ship");
        assert_eq!(ship.hull.length, 500.0);
        assert_eq!(ship.hull.block, 0.52);
        assert_eq!(ship.free.fc_length, 0.2);
        assert_eq!(ship.free.fd_length, 0.3);
        assert_eq!(ship.free.qd_length, 0.15);
        assert_eq!(ship.guns.batteries[0].barrels, 8);
        assert_eq!(ship.guns.batteries[0].calibre, 12.0);
        assert_eq!(ship.guns.batteries[0].mount_type, 5);
        assert_eq!(ship.guns.batteries[0].mounts, 4);
        assert_eq!(ship.engine.speed_max, 22.0);
        assert_eq!(ship.engine.screws, 2);
        assert!(ship.engine.fuel[0]);
        assert!(ship.engine.engine_types[1]);
        assert!(ship.engine.drive[0]);
    }

    #[test]
    fn rejects_non_finite_and_out_of_range_values() {
        let mut ship = ShipModel::new();
        assert!(deserialize(&mut ship, "hull.length=NaN\n").is_err());
        assert!(deserialize(&mut ship, "hull.beam=inf\n").is_err());
        assert!(deserialize(&mut ship, "hull.length=-5\n").is_err());
        assert!(deserialize(&mut ship, "hull.block=1.5\n").is_err());
        assert!(deserialize(&mut ship, "hull.draught=0\n").is_err());
        // Valid values still parse.
        assert!(deserialize(&mut ship, "hull.length=500\nhull.block=0.5\n").is_ok());
        assert_eq!(ship.hull.length, 500.0);
    }

    #[test]
    fn accepts_header_without_equals() {
        let mut ship = ShipModel::new();
        let text = "SpringSharp Rust Save 1.0\nhull.length=123.5\n";
        deserialize(&mut ship, text).unwrap();
        assert_eq!(ship.hull.length, 123.5);
    }
}
