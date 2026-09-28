//! Improved internal-space volume model.
//!
//! The original SpringSharp `RoomHullCalc` is a simple weight/space ratio.  This
//! module adds a more transparent volume-budget model based on common naval
//! architecture practice:
//!
//! - machinery volume scales with installed power and engine type
//! - magazines scale with shell/magazine weights
//! - accommodation scales with crew size
//! - fuel/stores/void spaces are estimated explicitly
//!
//! These are first-principles planning estimates, not a replacement for a real
//! shipyard general-arrangement design.

use crate::model::{metric_calc, Conversion, ShipModel};

#[derive(Clone, Debug, Default)]
pub struct InternalSpaceReport {
    pub hull_volume_m3: f64,
    pub machinery_volume_m3: f64,
    pub magazine_volume_m3: f64,
    pub accommodation_volume_m3: f64,
    pub fuel_volume_m3: f64,
    pub stores_volume_m3: f64,
    pub void_volume_m3: f64,
    pub required_volume_m3: f64,
    pub room_hull_improved: f64,
    pub ship_type: String,
    pub crew_volume_per_person: f64,
    pub machinery_volume_per_kw: f64,
    pub stores_volume_per_ton: f64,
}

/// Specific volumes can be calibrated from ship studies.  The current values
/// are deliberately conservative planning factors.
pub const MACHINERY_M3_PER_KW: f64 = 0.020;
pub const CREW_M3_PER_PERSON: f64 = 12.0;
pub const MAGAZINE_M3_PER_TON: f64 = 1.3;
pub const FUEL_M3_PER_TON: f64 = 1.2;
pub const STORES_M3_PER_TON_DISPLACEMENT: f64 = 0.015;
pub const VOID_FRACTION: f64 = 0.03;

/// Magazine volume per ton of magazine weight, scaled by main-battery calibre.
fn magazine_m3_per_ton(ship: &ShipModel) -> f64 {
    let calibre = ship.guns.batteries[0].calibre.max(0.0);
    let total_rounds: f64 = ship
        .guns
        .batteries
        .iter()
        .map(|b| (b.magazine.max(0) as f64) * (b.barrels.max(0) as f64))
        .sum();
    let total_mounts: f64 = ship
        .guns
        .batteries
        .iter()
        .map(|b| b.mounts.max(0) as f64)
        .sum();
    // Handling/storage overhead grows with rounds and mount count.
    let handling = 1.0 + 0.05 * (total_rounds / 2000.0) + 0.02 * total_mounts;
    ((1.3 + 0.02 * calibre) * handling).clamp(1.2, 3.0)
}

/// Crew volume factor by broad ship category.
fn crew_volume_per_person(ship_type: &str) -> f64 {
    if ship_type.contains("destroyer") {
        8.0
    } else if ship_type.contains("cruiser") {
        10.0
    } else if ship_type.contains("carrier") {
        14.0
    } else {
        12.0
    }
}

/// Machinery volume per kW, dependent on era and powerplant type.
fn machinery_volume_per_kw(ship: &ShipModel) -> f64 {
    let base = if ship.hull.date_engine < 1900 {
        0.030
    } else if ship.hull.date_engine < 1925 {
        0.025
    } else {
        0.020
    };
    // Steam turbines tend to be more compact than reciprocating engines;
    // diesel-electric and high-speed plants are also compact.
    if ship.engine.engine_types[2] {
        base * 0.85
    } else if ship.engine.engine_types[0] || ship.engine.engine_types[1] {
        base
    } else {
        base * 0.9
    }
}

/// Stores volume per ton of displacement; longer-range ships need more stores.
fn stores_volume_per_ton(ship: &ShipModel) -> f64 {
    let range_factor = (ship.engine.range / 7000.0).clamp(0.5, 2.0);
    STORES_M3_PER_TON_DISPLACEMENT * range_factor
}

pub fn calculate_internal_spaces(ship: &ShipModel) -> InternalSpaceReport {
    let hull_vol_ft3 =
        ship.hull.length * ship.hull.beam_bulges * ship.hull.draught * ship.hull.block;
    let hull_volume_m3 = hull_vol_ft3 * 0.0283168466;
    let power_kw = metric_calc(ship.engine.hp_max, Conversion::Hp2Kw);
    let crew_per_person = if ship.space_crew_volume_per_person > 0.0 {
        ship.space_crew_volume_per_person
    } else {
        crew_volume_per_person(&ship.ship_type.to_ascii_lowercase())
    };
    let mach_per_kw = if ship.space_machinery_volume_per_kw > 0.0 {
        ship.space_machinery_volume_per_kw
    } else {
        machinery_volume_per_kw(ship)
    };
    let stores_per_ton = if ship.space_stores_volume_per_ton > 0.0 {
        ship.space_stores_volume_per_ton
    } else {
        stores_volume_per_ton(ship)
    };
    let machinery_volume_m3 = power_kw * mach_per_kw;
    let magazine_volume_m3 = ship.guns.magazine * magazine_m3_per_ton(ship);
    let accommodation_volume_m3 = ship.hull.crew_min.max(1) as f64 * crew_per_person;
    let fuel_volume_m3 = ship.engine.bunker * FUEL_M3_PER_TON;
    let stores_volume_m3 = ship.hull.displacement * stores_per_ton;
    let void_volume_m3 = ship.weapons.void_weight * FUEL_M3_PER_TON;

    let required = machinery_volume_m3
        + magazine_volume_m3
        + accommodation_volume_m3
        + fuel_volume_m3
        + stores_volume_m3
        + void_volume_m3;

    let room_hull_improved = if required > 0.0 {
        hull_volume_m3 / required
    } else {
        0.0
    };

    InternalSpaceReport {
        hull_volume_m3,
        machinery_volume_m3,
        magazine_volume_m3,
        accommodation_volume_m3,
        fuel_volume_m3,
        stores_volume_m3,
        void_volume_m3,
        required_volume_m3: required,
        room_hull_improved,
        ship_type: ship.ship_type.clone(),
        crew_volume_per_person: crew_per_person,
        machinery_volume_per_kw: mach_per_kw,
        stores_volume_per_ton: stores_per_ton,
    }
}

pub fn format_space_report(report: &InternalSpaceReport) -> String {
    format!(
        "Improved internal space model\n\
         Hull volume: {:.0} m3\n\
         Machinery: {:.0} m3\n\
         Magazines: {:.0} m3\n\
         Accommodation: {:.0} m3\n\
         Fuel: {:.0} m3\n\
         Stores: {:.0} m3\n\
         Void: {:.0} m3\n\
         Required: {:.0} m3\n\
         Room ratio (hull/required): {:.3}\n         Ship type: {}\n         Crew volume/person: {:.1} m3\n         Machinery volume/kW: {:.3} m3\n         Stores volume/ton: {:.3} m3",
        report.hull_volume_m3,
        report.machinery_volume_m3,
        report.magazine_volume_m3,
        report.accommodation_volume_m3,
        report.fuel_volume_m3,
        report.stores_volume_m3,
        report.void_volume_m3,
        report.required_volume_m3,
        report.room_hull_improved,
        report.ship_type,
        report.crew_volume_per_person,
        report.machinery_volume_per_kw,
        report.stores_volume_per_ton,
    )
}

/// Simple prototype compartment-allocation result.
#[derive(Clone, Debug, Default)]
pub struct CompartmentLayout {
    pub machinery: f64,
    pub magazine: f64,
    pub accommodation: f64,
    pub fuel: f64,
    pub stores: f64,
    pub void: f64,
    pub unallocated: f64,
}

/// Suggest a first-pass compartment volume allocation.
///
/// This is a proportional allocation with a 10% reserve, intended as a
/// prototype for a future constraint optimizer.
pub fn suggest_compartment_layout(report: &InternalSpaceReport) -> CompartmentLayout {
    let reserve = 0.10 * report.hull_volume_m3;
    let allocatable = (report.hull_volume_m3 - reserve).max(0.0);
    let required = report.required_volume_m3.max(1e-9);
    let scale = (allocatable / required).min(1.0);
    CompartmentLayout {
        machinery: report.machinery_volume_m3 * scale,
        magazine: report.magazine_volume_m3 * scale,
        accommodation: report.accommodation_volume_m3 * scale,
        fuel: report.fuel_volume_m3 * scale,
        stores: report.stores_volume_m3 * scale,
        void: report.void_volume_m3 * scale,
        unallocated: (allocatable - required * scale).max(0.0),
    }
}

pub fn format_compartment_report(layout: &CompartmentLayout) -> String {
    format!(
        "Prototype compartment allocation (m3)\n\
         Machinery:     {:.0}\n\
         Magazine:      {:.0}\n\
         Accommodation: {:.0}\n\
         Fuel:          {:.0}\n\
         Stores:        {:.0}\n\
         Void:          {:.0}\n\
         Unallocated:   {:.0}",
        layout.machinery,
        layout.magazine,
        layout.accommodation,
        layout.fuel,
        layout.stores,
        layout.void,
        layout.unallocated,
    )
}

/// Constraint-aware prototype compartment optimizer.
///
/// Variables are the six compartment volumes. The objective is a combination of
/// vertical centre-of-gravity error and unused volume. This is still a
/// simplified model, but it demonstrates the optimisation path for P3.
pub fn optimize_compartment_layout(
    report: &InternalSpaceReport,
    target_cg_ratio: f64,
) -> (CompartmentLayout, f64, [f64; 6]) {
    let initial_centers = [0.35_f64, 0.30, 0.65, 0.25, 0.45, 0.20];
    let lower_centers = [0.30_f64, 0.20, 0.55, 0.15, 0.30, 0.10];
    let upper_centers = [0.50_f64, 0.40, 0.85, 0.45, 0.55, 0.40];
    let void_min = 0.02 * report.hull_volume_m3;
    let required = [
        report.machinery_volume_m3,
        report.magazine_volume_m3,
        report.accommodation_volume_m3,
        report.fuel_volume_m3,
        report.stores_volume_m3,
        report.void_volume_m3.max(void_min),
    ];
    let allocatable = (0.90 * report.hull_volume_m3).max(0.0);
    let required_total: f64 = required.iter().sum();
    let lower: Vec<f64> = required.iter().map(|v| 0.90 * v).collect();
    let upper: Vec<f64> = required.iter().map(|v| 1.50 * v).collect();

    let mut best_volumes = required;
    let mut best_centers = initial_centers;
    let mut best_obj = objective_with_required(
        &best_volumes,
        &best_centers,
        allocatable,
        target_cg_ratio,
        required_total,
    );

    let mut rng = 0x1234_5678_9abc_def0_u64;
    let mut current_volumes = required;
    let mut current_centers = initial_centers;
    let mut current_obj = best_obj;

    for _ in 0..8000 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let choose_center = (rng >> 40) & 1 == 1;
        if choose_center {
            let idx = (rng >> 33) as usize % 6;
            let delta = ((rng >> 11) as f64 / u64::MAX as f64 - 0.5) * 0.2;
            let mut candidate = current_centers;
            candidate[idx] = (candidate[idx] + delta).clamp(lower_centers[idx], upper_centers[idx]);
            let candidate_obj = objective_with_required(
                &current_volumes,
                &candidate,
                allocatable,
                target_cg_ratio,
                required_total,
            );
            if candidate_obj <= current_obj {
                current_centers = candidate;
                current_obj = candidate_obj;
            }
        } else {
            let idx = (rng >> 33) as usize % 6;
            let delta = ((rng >> 11) as f64 / u64::MAX as f64 - 0.5) * required[idx].max(1.0) * 0.3;
            let mut candidate = current_volumes;
            candidate[idx] = (candidate[idx] + delta).clamp(lower[idx], upper[idx]);
            let candidate_obj = objective_with_required(
                &candidate,
                &current_centers,
                allocatable,
                target_cg_ratio,
                required_total,
            );
            if candidate_obj <= current_obj {
                current_volumes = candidate;
                current_obj = candidate_obj;
            }
        }
        if current_obj < best_obj {
            best_volumes = current_volumes;
            best_centers = current_centers;
            best_obj = current_obj;
        }
    }

    // Project adjacent centers to satisfy the continuity constraint.
    for _ in 0..20 {
        for (i, j) in [
            (0, 1),
            (1, 4),
            (0, 3),
            (4, 5),
            (2, 4),
            (0, 2),
            (1, 3),
            (3, 4),
            (2, 5),
        ] {
            let diff = best_centers[i] - best_centers[j];
            if diff.abs() > 0.25 {
                let half = (diff.abs() - 0.25) * 0.5;
                if diff > 0.0 {
                    best_centers[i] -= half;
                    best_centers[j] += half;
                } else {
                    best_centers[i] += half;
                    best_centers[j] -= half;
                }
                best_centers[i] = best_centers[i].clamp(lower_centers[i], upper_centers[i]);
                best_centers[j] = best_centers[j].clamp(lower_centers[j], upper_centers[j]);
            }
        }
    }

    let total: f64 = best_volumes.iter().sum();
    let cg = if total > 0.0 {
        best_volumes
            .iter()
            .zip(best_centers.iter())
            .map(|(v, c)| v * c)
            .sum::<f64>()
            / total
    } else {
        0.0
    };

    let layout = CompartmentLayout {
        machinery: best_volumes[0],
        magazine: best_volumes[1],
        accommodation: best_volumes[2],
        fuel: best_volumes[3],
        stores: best_volumes[4],
        void: best_volumes[5],
        unallocated: (allocatable - total).max(0.0),
    };
    (layout, cg, best_centers)
}

fn objective_with_required(
    volumes: &[f64; 6],
    centers: &[f64; 6],
    allocatable: f64,
    target_cg: f64,
    required_total: f64,
) -> f64 {
    let base = objective(volumes, centers, allocatable, target_cg);
    let total: f64 = volumes.iter().sum();
    let unmet = if required_total > 0.0 {
        ((required_total - total).max(0.0) / required_total).min(1.0)
    } else {
        0.0
    };
    // Engineering placement constraints:
    // - accommodation should be above the magazines
    // - machinery should be above the fuel spaces
    // - some unallocated volume should remain for access/void
    let accommodation_above = (centers[2] - centers[1] - 0.15).max(0.0);
    let machinery_above_fuel = (centers[0] - centers[3] - 0.05).max(0.0);
    let access_ratio = if allocatable > 0.0 {
        ((allocatable - total).max(0.0) / allocatable).min(1.0)
    } else {
        0.0
    };
    let access_penalty = (0.03 - access_ratio).max(0.0) * 2.0;
    // Structural continuity: adjacent functional spaces should not be too far
    // apart vertically.
    let adjacency = [
        (0, 1),
        (1, 4),
        (0, 3),
        (4, 5),
        (2, 4),
        (0, 2),
        (1, 3),
        (3, 4),
        (2, 5),
    ];
    let continuity_penalty: f64 = adjacency
        .iter()
        .map(|(i, j)| (centers[*i] - centers[*j]).abs() - 0.25)
        .map(|v| v.max(0.0))
        .sum();
    // Maintenance/access space: keep some void volume proportional to the
    // machinery and accommodation spaces.
    let maintenance_need = 0.05 * (volumes[0] + volumes[2]);
    let maintenance_gap = (maintenance_need - volumes[5]).max(0.0);
    let maintenance_penalty = if allocatable > 0.0 {
        maintenance_gap / allocatable
    } else {
        0.0
    };
    base + 0.5 * unmet
        + 0.3 * accommodation_above
        + 0.3 * machinery_above_fuel
        + access_penalty
        + 1.0 * continuity_penalty
        + 0.2 * maintenance_penalty
}

fn objective(volumes: &[f64; 6], centers: &[f64; 6], allocatable: f64, target_cg: f64) -> f64 {
    let total: f64 = volumes.iter().sum();
    if total > allocatable && allocatable > 0.0 {
        return 1e6 + (total - allocatable);
    }
    let cg = if total > 0.0 {
        volumes
            .iter()
            .zip(centers.iter())
            .map(|(v, c)| v * c)
            .sum::<f64>()
            / total
    } else {
        0.0
    };
    let cg_err = (cg - target_cg).abs();
    let unused = (allocatable - total).max(0.0) / allocatable.max(1.0);
    cg_err + 0.05 * unused
}

/// Compartment placement including longitudinal hints from the armament layout.
#[derive(Clone, Debug, Default)]
pub struct CompartmentPlacement {
    pub layout: CompartmentLayout,
    pub cg_vertical: f64,
    pub cg_longitudinal: f64,
    pub magazine_longitudinal: f64,
    pub machinery_longitudinal: f64,
    pub centers: [f64; 6],
    /// Longitudinal position of each compartment (fraction of length from stern).
    pub longitudinal: [f64; 6],
    /// Combined objective value of the constraint optimizer (lower is better).
    pub objective: f64,
    pub checks: Vec<(String, bool)>,
}

/// Optimize compartments and add simple longitudinal placement hints.
///
/// The magazine longitudinal position follows the average main-battery
/// position; machinery is assumed amidships.
pub fn suggest_compartment_placement(ship: &ShipModel, target_cg: f64) -> CompartmentPlacement {
    let report = calculate_internal_spaces(ship);
    let (layout, cg, centers) = optimize_compartment_layout(&report, target_cg);
    let g = &ship.guns.batteries[0];
    let pos_g1 = layout_longitudinal_position(ship, g.layout_g1);
    let pos_g2 = layout_longitudinal_position(ship, g.layout_g2);
    let total_mounts = (g.mounts_g1 + g.mounts_g2) as f64;
    let magazine_longitudinal = if total_mounts > 0.0 {
        (pos_g1 * g.mounts_g1 as f64 + pos_g2 * g.mounts_g2 as f64) / total_mounts
    } else {
        pos_g1
    };
    let adjacency = [
        (0, 1),
        (1, 4),
        (0, 3),
        (4, 5),
        (2, 4),
        (0, 2),
        (1, 3),
        (3, 4),
        (2, 5),
    ];
    let continuity_ok = adjacency
        .iter()
        .all(|(i, j)| (centers[*i] - centers[*j]).abs() <= 0.26);

    // --- Longitudinal placement (multi-constraint coordinate descent) ---
    let vols = [
        layout.machinery,
        layout.magazine,
        layout.accommodation,
        layout.fuel,
        layout.stores,
        layout.void,
    ];
    let targets = [0.50, magazine_longitudinal, 0.58, 0.50, 0.58, 0.45];
    let weights = [1.0, 2.5, 1.0, 0.5, 0.5, 0.4];
    let mut x = targets;
    let mut step = 0.08_f64;
    let mut objective = longitudinal_objective(&x, &targets, &weights, magazine_longitudinal);
    for _ in 0..120 {
        for i in 0..6 {
            for delta in [step, -step] {
                let mut trial = x;
                trial[i] = (trial[i] + delta).clamp(0.05, 0.95);
                let value =
                    longitudinal_objective(&trial, &targets, &weights, magazine_longitudinal);
                if value < objective {
                    x = trial;
                    objective = value;
                }
            }
        }
        step *= 0.97;
    }
    let total_v: f64 = vols.iter().sum::<f64>().max(1e-9);
    let cg_longitudinal = x
        .iter()
        .zip(vols.iter())
        .map(|(xi, vi)| xi * vi)
        .sum::<f64>()
        / total_v;

    let longitudinal_continuity_ok = adjacency.iter().all(|(i, j)| (x[*i] - x[*j]).abs() <= 0.40);
    let checks = vec![
        (
            "Accommodation above magazines".to_string(),
            centers[2] >= centers[1] + 0.15 - 1e-6,
        ),
        (
            "Machinery above fuel".to_string(),
            centers[0] >= centers[3] + 0.05 - 1e-6,
        ),
        (
            "Maintenance void >= 2% hull volume".to_string(),
            layout.void >= 0.02 * report.hull_volume_m3 - 1e-6,
        ),
        (
            "Structural continuity (vertical)".to_string(),
            continuity_ok,
        ),
        (
            "Magazine within +/-0.15 L of armament".to_string(),
            (x[1] - magazine_longitudinal).abs() <= 0.15 + 1e-9,
        ),
        (
            "Machinery within +/-0.20 L of amidships".to_string(),
            (x[0] - 0.50).abs() <= 0.20 + 1e-9,
        ),
        (
            "Structural continuity (longitudinal)".to_string(),
            longitudinal_continuity_ok,
        ),
    ];
    CompartmentPlacement {
        layout,
        cg_vertical: cg,
        cg_longitudinal,
        magazine_longitudinal: x[1],
        machinery_longitudinal: x[0],
        centers,
        longitudinal: x,
        objective,
        checks,
    }
}

/// Penalised longitudinal-placement objective.
fn longitudinal_objective(
    x: &[f64; 6],
    targets: &[f64; 6],
    weights: &[f64; 6],
    armament: f64,
) -> f64 {
    let mut value = 0.0;
    for i in 0..6 {
        value += weights[i] * (x[i] - targets[i]).abs();
    }
    // Functional bounds and ordering constraints as penalties.
    if (x[1] - armament).abs() > 0.15 {
        value += 5.0 * ((x[1] - armament).abs() - 0.15);
    }
    if (x[0] - 0.50).abs() > 0.20 {
        value += 5.0 * ((x[0] - 0.50).abs() - 0.20);
    }
    // Magazine forward of (larger x than) fuel and machinery, access-friendly.
    if x[1] < x[3] {
        value += 2.0 * (x[3] - x[1]);
    }
    if x[2] < x[1] {
        value += 1.0 * (x[1] - x[2]);
    }
    value
}

/// Machine-readable general-arrangement summary (hand-rolled JSON, no deps).
pub fn compartment_placement_json(placement: &CompartmentPlacement) -> String {
    fn esc(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }
    let l = &placement.layout;
    let centers: Vec<String> = placement
        .centers
        .iter()
        .map(|c| format!("{c:.4}"))
        .collect();
    let checks: Vec<String> = placement
        .checks
        .iter()
        .map(|(name, ok)| format!("{{\"name\":\"{}\",\"ok\":{}}}", esc(name), ok))
        .collect();
    format!(
        "{{\n  \"cg_vertical\": {:.4},\n  \"cg_longitudinal\": {:.4},\n  \"objective\": {:.4},\n  \"magazine_longitudinal\": {:.4},\n  \"machinery_longitudinal\": {:.4},\n  \"centers\": [{}],\n  \"longitudinal\": [{}],\n  \"volumes\": {{\"machinery\": {:.1}, \"magazine\": {:.1}, \"accommodation\": {:.1}, \"fuel\": {:.1}, \"stores\": {:.1}, \"void\": {:.1}, \"unallocated\": {:.1}}},\n  \"checks\": [{}]\n}}",
        placement.cg_vertical,
        placement.cg_longitudinal,
        placement.objective,
        placement.magazine_longitudinal,
        placement.machinery_longitudinal,
        centers.join(", "),
        placement
            .longitudinal
            .iter()
            .map(|x| format!("{x:.4}"))
            .collect::<Vec<_>>()
            .join(", "),
        l.machinery,
        l.magazine,
        l.accommodation,
        l.fuel,
        l.stores,
        l.void,
        l.unallocated,
        checks.join(", ")
    )
}

/// Printable HTML general-arrangement report.
pub fn compartment_placement_html(ship: &ShipModel, placement: &CompartmentPlacement) -> String {
    let l = &placement.layout;
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>General Arrangement</title></head><body>",
    );
    html.push_str(&format!(
        "<h1>General arrangement prototype</h1><p>Ship: <b>{}</b> ({})</p>",
        ship.name, ship.ship_type
    ));
    html.push_str("<p>Prototype constraint-aware allocation. Volumes in m&sup3; positions as fraction of length.</p>");
    html.push_str("<table border=\"1\" cellpadding=\"4\" cellspacing=\"0\">");
    html.push_str("<tr><th>Compartment</th><th>Volume (m3)</th></tr>");
    for (name, v) in [
        ("Machinery", l.machinery),
        ("Magazine", l.magazine),
        ("Accommodation", l.accommodation),
        ("Fuel", l.fuel),
        ("Stores", l.stores),
        ("Void / maintenance", l.void),
        ("Unallocated", l.unallocated),
    ] {
        html.push_str(&format!("<tr><td>{name}</td><td>{v:.0}</td></tr>"));
    }
    html.push_str("</table>");
    html.push_str(&format!(
        "<p>Achieved vertical CG ratio: <b>{:.3}</b><br>Longitudinal CG: {:.3} L<br>Magazine longitudinal: {:.3} L<br>Machinery longitudinal: {:.3} L<br>Objective: {:.4}</p>",
        placement.cg_vertical, placement.cg_longitudinal, placement.magazine_longitudinal,
        placement.machinery_longitudinal, placement.objective
    ));
    html.push_str("<h2>Constraint checks</h2><ul>");
    for (name, ok) in &placement.checks {
        html.push_str(&format!(
            "<li>{} {}</li>",
            if *ok { "PASS" } else { "FAIL" },
            name
        ));
    }
    html.push_str("</ul></body></html>");
    html
}

fn layout_longitudinal_position(ship: &ShipModel, layout: i32) -> f64 {
    let fc = ship.free.fc_length;
    let fd = ship.free.fd_length;
    let qd = ship.free.qd_length;
    let ad = (1.0 - fc - fd - qd).max(0.0);
    match layout {
        0 | 9 => 0.5,
        1 | 10 => 0.4,
        2 | 11 => 0.6,
        3 | 12 => fc + 0.25 * fd,
        4 | 13 => fc + 0.5 * fd,
        5 | 14 => fc + 0.75 * fd,
        6 | 15 => 1.0 - (qd + 0.75 * ad),
        7 | 16 => 1.0 - (qd + 0.5 * ad),
        8 | 17 => 1.0 - (qd + 0.25 * ad),
        _ => 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ShipModel;

    #[test]
    fn compartment_placement_uses_armament_position() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.free.free_fd_for = 18.0;
        ship.free.free_fd_aft = 14.0;
        ship.free.free_ad_for = 14.0;
        ship.free.free_ad_aft = 12.0;
        ship.engine.fuel[0] = true;
        ship.engine.engine_types[1] = true;
        ship.engine.drive[0] = true;
        ship.engine.screws = 4;
        ship.engine.speed_max = 24.0;
        ship.engine.speed_cruise = 12.0;
        ship.engine.range = 7000.0;
        ship.guns.batteries[0].layout_g1 = 4;
        ship.guns.batteries[0].layout_g2 = 7;
        ship.guns.batteries[0].mounts = 4;
        ship.guns.batteries[0].mount_super_g1 = 2;
        ship.guns.batteries[0].mount_sub_g1 = 0;
        ship.guns.batteries[0].mount_super_g2 = 2;
        ship.calculate();
        let placement = suggest_compartment_placement(&ship, 0.45);
        let forward = layout_longitudinal_position(&ship, 4);
        let aft = layout_longitudinal_position(&ship, 7);
        assert!(placement.magazine_longitudinal > forward.min(aft));
        assert!(placement.magazine_longitudinal < forward.max(aft));
        assert!(placement.cg_vertical > 0.0);
    }

    #[test]
    fn compartment_json_and_html_are_well_formed() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.engine.screws = 4;
        ship.calculate();
        let placement = suggest_compartment_placement(&ship, 0.45);
        let json = compartment_placement_json(&placement);
        assert!(json.contains("\"cg_vertical\""));
        assert!(json.contains("\"volumes\""));
        assert!(json.contains("\"checks\""));
        assert!(json.ends_with('}'));
        let html = compartment_placement_html(&ship, &placement);
        assert!(html.contains("General arrangement"));
        assert!(html.contains("Constraint checks"));
    }

    #[test]
    fn compartment_optimizer_satisfies_checks() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.free.free_fd_for = 18.0;
        ship.free.free_fd_aft = 14.0;
        ship.free.free_ad_for = 14.0;
        ship.free.free_ad_aft = 12.0;
        ship.engine.fuel[0] = true;
        ship.engine.engine_types[1] = true;
        ship.engine.drive[0] = true;
        ship.engine.screws = 4;
        ship.engine.speed_max = 24.0;
        ship.engine.speed_cruise = 12.0;
        ship.engine.range = 7000.0;
        ship.calculate();
        let placement = suggest_compartment_placement(&ship, 0.45);
        for (name, ok) in &placement.checks {
            assert!(*ok, "constraint failed: {name}");
        }
    }

    #[test]
    fn compartment_optimizer_runs() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.free.free_fd_for = 18.0;
        ship.free.free_fd_aft = 14.0;
        ship.free.free_ad_for = 14.0;
        ship.free.free_ad_aft = 12.0;
        ship.engine.fuel[0] = true;
        ship.engine.engine_types[1] = true;
        ship.engine.drive[0] = true;
        ship.engine.screws = 4;
        ship.engine.speed_max = 24.0;
        ship.engine.speed_cruise = 12.0;
        ship.engine.range = 7000.0;
        ship.calculate();
        let report = calculate_internal_spaces(&ship);
        let (layout, cg, _) = optimize_compartment_layout(&report, 0.45);
        assert!(layout.accommodation > 0.0);
        assert!(cg > 0.0 && cg < 1.0);
        assert!(layout.unallocated > 0.03 * report.hull_volume_m3);
    }

    #[test]
    fn magazine_volume_scales_with_calibre() {
        let mut ship = ShipModel::new();
        ship.guns.batteries[0].calibre = 6.0;
        let small = magazine_m3_per_ton(&ship);
        ship.guns.batteries[0].calibre = 16.0;
        let large = magazine_m3_per_ton(&ship);
        assert!(large > small);
    }

    #[test]
    fn magazine_volume_scales_with_rounds() {
        let mut ship = ShipModel::new();
        ship.guns.batteries[0].calibre = 8.0;
        ship.guns.batteries[0].barrels = 4;
        ship.guns.batteries[0].magazine = 100;
        let small = magazine_m3_per_ton(&ship);
        ship.guns.batteries[0].magazine = 300;
        let large = magazine_m3_per_ton(&ship);
        assert!(large > small);
    }

    #[test]
    fn compartment_layout_is_proportional() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.free.free_fd_for = 18.0;
        ship.free.free_fd_aft = 14.0;
        ship.free.free_ad_for = 14.0;
        ship.free.free_ad_aft = 12.0;
        ship.engine.fuel[0] = true;
        ship.engine.engine_types[1] = true;
        ship.engine.drive[0] = true;
        ship.engine.screws = 4;
        ship.engine.speed_max = 24.0;
        ship.engine.speed_cruise = 12.0;
        ship.engine.range = 7000.0;
        ship.calculate();
        let report = calculate_internal_spaces(&ship);
        let layout = suggest_compartment_layout(&report);
        assert!(layout.accommodation > 0.0);
        assert!(layout.machinery > 0.0);
    }

    #[test]
    fn internal_space_overrides_are_used() {
        let mut ship = ShipModel::new();
        ship.ship_type = "Cruiser".into();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.engine.screws = 4;
        ship.calculate();
        let auto = calculate_internal_spaces(&ship);
        ship.space_crew_volume_per_person = 5.0;
        ship.space_machinery_volume_per_kw = 0.01;
        ship.space_stores_volume_per_ton = 0.005;
        let over = calculate_internal_spaces(&ship);
        assert!(over.crew_volume_per_person < auto.crew_volume_per_person);
        assert!(over.machinery_volume_per_kw < auto.machinery_volume_per_kw);
        assert!(over.stores_volume_per_ton < auto.stores_volume_per_ton);
    }

    #[test]
    fn internal_space_model_runs() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.engine.screws = 4;
        ship.calculate();
        let report = calculate_internal_spaces(&ship);
        assert!(report.hull_volume_m3 > 0.0);
        assert!(report.required_volume_m3 > 0.0);
        assert!(report.room_hull_improved > 0.0);
    }
}
