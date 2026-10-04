//! Higher-level features: report generation and iterative solvers.

use crate::model::{
    metric_calc, Conversion, ShipModel, WeaponAggregate, WeaponKind, WeaponMount, POUNDS2KG,
    SQFEET2SQMETRES,
};

/// Search for the maximum speed that requires no more than `target_power` HP.
///
/// The original WinForms code walks the speed upward/downward with ever smaller
/// steps.  This version uses the same physics but a deterministic binary search,
/// which is faster and gives a more stable result.
pub fn solve_max_speed(ship: &mut ShipModel, target_power: f64) -> f64 {
    let mut lo = 0.0_f64;
    let mut hi = 60.0_f64;
    let mut best = 0.0_f64;
    let mut iter = 0_u32;

    // Binary search with tolerance termination (as recommended by
    // Introduction-to-Algorithms style correctness arguments).
    while hi - lo > 1e-4 && iter < 200 {
        let mid = (lo + hi) * 0.5;
        ship.engine.speed_max = mid;
        ship.calculate();
        let hp = ship.engine.hp_max;
        if hp.is_finite() && hp <= target_power {
            best = mid;
            lo = mid;
        } else {
            hi = mid;
        }
        iter += 1;
    }

    ship.engine.speed_max = best;
    ship.calculate();
    best
}

/// Search for the range in nautical miles that gives no more than
/// `target_bunker` tons of normal bunkerage.
pub fn solve_range(ship: &mut ShipModel, target_bunker: f64) -> f64 {
    let mut lo = 0.0_f64;
    let mut hi = 100_000.0_f64;
    let mut best = 0.0_f64;
    let mut iter = 0_u32;

    while hi - lo > 1e-3 && iter < 200 {
        let mid = (lo + hi) * 0.5;
        ship.engine.range = mid;
        ship.calculate();
        let bunker = ship.engine.bunker;
        if bunker.is_finite() && bunker <= target_bunker {
            best = mid;
            lo = mid;
        } else {
            hi = mid;
        }
        iter += 1;
    }

    ship.engine.range = best;
    ship.calculate();
    best
}

fn gun_type_name(idx: i32) -> &'static str {
    match idx {
        0 => "Muzzle loading gun",
        1 => "Breech loading gun",
        2 => "Quick firing gun",
        3 => "Anti-air gun",
        4 => "Dual purpose gun",
        5 => "Auto rapid fire gun",
        6 => "Machine gun",
        _ => "gun",
    }
}

fn mount_type_name(idx: i32) -> &'static str {
    match idx {
        0 => "in broadside mount",
        1 => "in Coles/Ericsson turret mount",
        2 => "in open barbette mount",
        3 => "in turret on barbette mount",
        4 => "in deck and hoist mount",
        5 => "in deck mount",
        6 => "in casemate mount",
        _ => "mount",
    }
}

fn layout_name(idx: i32) -> &'static str {
    match idx {
        0 => "Centreline - distributed",
        1 => "Centreline - ends (fore >= aft)",
        2 => "Centreline - ends (aft >= fore)",
        3 => "Centreline - fore deck forward",
        4 => "Centreline - fore deck",
        5 => "Centreline - fore deck aft",
        6 => "Centreline - aft deck forward",
        7 => "Centreline - aft deck",
        8 => "Centreline - aft deck aft",
        9 => "Sides - distributed",
        10 => "Sides - ends (fore >= aft)",
        11 => "Sides - ends (aft >= fore)",
        12 => "Sides - fore deck forward",
        13 => "Sides - fore deck",
        14 => "Sides - fore deck aft",
        15 => "Sides - aft deck forward",
        16 => "Sides - aft deck",
        17 => "Sides - aft deck aft",
        _ => "None",
    }
}

fn gun_description(b: &crate::model::GunBattery, main: bool) -> String {
    if b.barrels <= 0 || b.calibre <= 0.0 {
        return format!(
            "      No {} battery",
            if main { "main" } else { "secondary" }
        );
    }
    let mm = b.calibre * 25.4;
    let mm_text = if mm < 100.0 {
        format!("{mm:.1}")
    } else {
        format!("{mm:.0}")
    };
    let mut d = format!(
        "      {} - {:.2}\" / {} mm {:.1} cal gun{}",
        b.barrels,
        b.calibre,
        mm_text,
        b.cal_length,
        if b.barrels > 1 { "s" } else { "" }
    );
    d.push_str(&format!(
        " - {:.2}lbs / {:.2}kg shells, {} per gun",
        b.shell_weight,
        metric_calc(b.shell_weight, Conversion::Pounds2Kg),
        b.magazine
    ));
    d.push_str(&format!(
        "\n\t  {} {}{}, {} Model",
        gun_type_name(b.gun_type),
        mount_type_name(b.mount_type),
        if b.mounts_g1 + b.mounts_g2 > 1 {
            "s"
        } else {
            ""
        },
        b.date_gun
    ));
    if b.mounts_g1 > 0 {
        d.push_str(&format!(
            "\n\t  {} x {} {}",
            b.mounts_g1,
            b.per_mount_g1,
            layout_name(b.layout_g1)
        ));
        if b.mount_super_g1 > 0 {
            d.push_str(&format!(
                "\n\t\t{} raised mount{}",
                b.mount_super_g1,
                if b.mount_super_g1 > 1 { "s" } else { "" }
            ));
        }
        if b.mount_sub_g1 > 0 {
            let placement = if b.mount_type == 0 {
                " on upper deck"
            } else if b.mount_sub_g1 > 1 {
                " in casemates"
            } else {
                " in casemate"
            };
            d.push_str(&format!(
                "\n\t\t{} hull mount{}{placement}",
                b.mount_sub_g1,
                if b.mount_sub_g1 > 1 { "s" } else { "" }
            ));
        }
    }
    if b.mounts_g2 > 0 {
        d.push_str(&format!(
            "\n\t  {} x {} {}",
            b.mounts_g2,
            b.per_mount_g2,
            layout_name(b.layout_g2)
        ));
        if b.mount_super_g2 > 0 {
            d.push_str(&format!(
                "\n\t\t{} raised mount{}",
                b.mount_super_g2,
                if b.mount_super_g2 > 1 { "s" } else { "" }
            ));
        }
        if b.mount_sub_g2 > 0 {
            let placement = if b.mount_type == 0 {
                if b.sub_sub_chk {
                    " on gun deck"
                } else {
                    " on upper deck"
                }
            } else if b.sub_sub_chk {
                " in lower casemates"
            } else if b.mount_sub_g2 > 1 {
                " in casemates"
            } else {
                " in casemate"
            };
            d.push_str(&format!(
                "\n\t\t{} hull mount{}{placement}",
                b.mount_sub_g2,
                if b.mount_sub_g2 > 1 { "s" } else { "" }
            ));
        }
    }
    if b.free_gun > 0.0 {
        let use_note = if b.free_gun < 12.0 {
            "Limited use in any sea"
        } else if b.free_gun < 16.0 {
            "Limited use in all but light seas"
        } else {
            "Limited use in heavy seas"
        };
        d.push_str(&format!("\n\t\t{use_note}"));
    }
    d
}

fn ship_classification(ship: &ShipModel) -> Vec<String> {
    let h = &ship.hull;
    let a = &ship.armour;
    let g = &ship.guns;
    let p = ship;
    let mut lines = Vec::new();

    if g.batteries[0].mount_type == 2 || g.batteries[1].mount_type == 2 {
        lines.push("Barbette ship".to_string());
    }
    if (g.batteries[0].layout_g1 == 4 || g.batteries[0].layout_g1 == 10)
        && g.batteries[0].mount_adj > 0.6
        && h.date_hull < 1895
    {
        lines.push("Central citadel ship".to_string());
    }
    let has_old_mount = (0..3).any(|i| g.batteries[i].mount_type == 0);
    if has_old_mount {
        if a.weight_belts > 0.0 {
            let has_arm_back =
                (0..3).any(|i| g.batteries[i].mount_type == 0 && g.batteries[i].arm_back > 0.0);
            let has_sub = (0..3).any(|i| {
                g.batteries[i].mount_type == 0
                    && g.batteries[i].gun_sub_g1 + g.batteries[i].gun_sub_g2 > 0
            });
            lines.push(if has_arm_back {
                "Armoured Casemate Ship".to_string()
            } else if ship.free.fc_length + ship.free.qd_length < 0.5 {
                if has_sub {
                    "Armoured Frigate (Broadside ironclad)".to_string()
                } else {
                    "Armoured Corvette (Broadside ironclad)".to_string()
                }
            } else if has_sub {
                "Armoured Frigate (Central battery ironclad)".to_string()
            } else {
                "Armoured Corvette (Central battery ironclad)".to_string()
            });
        } else {
            let has_sub = (0..3).any(|i| {
                g.batteries[i].mount_type == 0
                    && g.batteries[i].gun_sub_g1 + g.batteries[i].gun_sub_g2 > 0
            });
            lines.push(if has_sub {
                "Frigate (Unarmoured)".to_string()
            } else {
                "Corvette (Unarmoured)".to_string()
            });
        }
    }
    if h.block <= 0.0 || h.block > 1.0 {
        lines.push("DESIGN FAILURE: Displacement impossible with given dimensions".to_string());
    }
    if h.displacement > 0.0 && h.displacement <= g.broadside / 4.0 {
        lines.push("DESIGN FAILURE: Gun weight too much for hull".to_string());
    }
    if h.displacement > 0.0 && a.weight > h.displacement {
        lines.push("DESIGN FAILURE: Armour weight too much for hull".to_string());
    }
    if p.strength_comp < 0.5 {
        lines.push("DESIGN FAILURE: Overall load weight too much for hull".to_string());
    }
    if p.metacentre <= 0.0 {
        lines.push("DESIGN FAILURE: Ship will capsize".to_string());
    }
    lines
}

/// Escape text for safe inclusion in HTML element content or attribute values.
///
/// Rust's `format!` gives no automatic escaping, and several report generators
/// interpolate user-supplied strings (ship names, external CSV fields, source
/// labels).  Everything that lands in HTML must go through this helper.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn generate_report(ship: &ShipModel) -> String {
    let mut r = String::new();
    let h = &ship.hull;
    let a = &ship.armour;
    let g = &ship.guns;
    let e = &ship.engine;
    let w = &ship.weapons;
    let p = &ship;

    r.push_str(&format!(
        "{}, {} {} laid down {}\n",
        ship.name, ship.country, ship.ship_type, h.date_hull
    ));
    if h.date_hull != h.date_engine {
        r.push_str(&format!("(Engine {})\n", h.date_engine));
    }
    for line in ship_classification(ship) {
        r.push_str(&format!("\n{line}"));
    }

    r.push_str("\nDisplacement:\n");
    r.push_str(&format!(
        "\t{:.0} t light; {:.0} t standard; {:.0} t normal; {:.0} t full load\n",
        p.disp_light, p.disp_standard, h.displacement, p.disp_max_load
    ));

    r.push_str("\nDimensions:\n");
    r.push_str(&format!(
        "\t({:.1} ft / {:.1} ft) x {:.1} ft x ({:.1} / {:.1} ft)\n",
        h.length_oa, h.length, h.beam, h.draught, h.draught_deep
    ));
    r.push_str(&format!(
        "\t({:.1} m / {:.1} m) x {:.1} m x ({:.1} / {:.1} m)\n",
        metric_calc(h.length_oa, Conversion::Feet2Metres),
        metric_calc(h.length, Conversion::Feet2Metres),
        metric_calc(h.beam, Conversion::Feet2Metres),
        metric_calc(h.draught, Conversion::Feet2Metres),
        metric_calc(h.draught_deep, Conversion::Feet2Metres),
    ));
    if h.beam_bulges > h.beam {
        r.push_str(&format!(
            "\tBulges: {:.1} ft / {:.1} m\n",
            h.beam_bulges,
            metric_calc(h.beam_bulges, Conversion::Feet2Metres)
        ));
    }

    r.push_str("\nArmament:\n");
    for (i, b) in g.batteries.iter().enumerate() {
        if b.barrels <= 0 {
            continue;
        }
        r.push_str(&format!("\n{}", gun_description(b, i == 0)));
    }
    r.push_str(&format!(
        "\n\tBroadside: {:.0} lbs / {:.0} kg; Magazine: {:.0} tons\n",
        g.broadside,
        metric_calc(g.broadside, Conversion::Pounds2Kg),
        g.magazine
    ));

    let all_weapons = [&w.torpedo1, &w.torpedo2, &w.mine, &w.dc1, &w.dc2];
    for wp in all_weapons.iter() {
        if wp.number <= 0 {
            continue;
        }
        let kind = match wp.kind {
            WeaponKind::Torpedo => "torpedo",
            WeaponKind::Mine => "mine",
            WeaponKind::DepthCharge => "depth charge/AS mortar",
        };
        r.push_str(&format!(
            "\t{} x {:.1} {} (location {}), {:.2} tons\n",
            wp.number,
            wp.size,
            kind,
            wp.location,
            wp.weapon_weight + wp.mount_weight
        ));
    }

    r.push_str("\nArmour:\n");
    r.push_str(&format!(
        "\tMain belt: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        a.belt_main, a.belt_main_length, a.belt_main_height, a.weight_belt_main
    ));
    r.push_str(&format!(
        "\tBelt ends: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        a.belt_ends, a.belt_ends_length, a.belt_ends_height, a.weight_belt_ends
    ));
    r.push_str(&format!(
        "\tUpper belt: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        a.belt_upper, a.belt_upper_length, a.belt_upper_height, a.weight_belt_upper
    ));
    r.push_str(&format!(
        "\tBulkhead: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        a.bulkhead, a.bulkhead_length, a.bulkhead_height, a.weight_bulkhead
    ));
    r.push_str(&format!(
        "\tBulge: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        a.bulge, a.bulge_length, a.bulge_height, a.weight_bulge
    ));
    r.push_str(&format!(
        "\tDecks: {:.1}\" (FC {:.1}\", QD {:.1}\"), {:.0} tons\n",
        a.deck, a.deck_fc, a.deck_qd, a.weight_deck
    ));
    r.push_str(&format!(
        "\tConning: {:.1}\" + {:.1}\", {:.0} tons\n",
        a.conning_for, a.conning_aft, a.weight_conning
    ));
    r.push_str(&format!("\tTotal armour: {:.0} tons\n", a.weight));

    r.push_str("\nMachinery:\n");
    let fuel_desc = describe_fuel(e);
    r.push_str(&format!("\tFuel: {fuel_desc}\n"));
    r.push_str(&format!(
        "\tMax speed: {:.1} kt, cruise: {:.1} kt\n",
        e.speed_max, e.speed_cruise
    ));
    r.push_str(&format!(
        "\tMax power: {:.0} hp / {:.0} kW; cruise power: {:.0} hp / {:.0} kW\n",
        e.hp_max,
        metric_calc(e.hp_max, Conversion::Hp2Kw),
        e.hp_cruise,
        metric_calc(e.hp_cruise, Conversion::Hp2Kw)
    ));
    r.push_str(if e.resistance_model == crate::model::ResistanceModel::Original {
        "\tNote: power uses the original SpringSharp empirical model (reconstructed, unimproved).\n"
    } else {
        "\tNote: power uses the improved Holtrop-Mennen-style model.\n"
    });
    r.push_str(&format!(
        "\tEngine weight: {:.0} tons; bunker normal: {:.0} tons / max {:.0} tons\n",
        e.engine_weight,
        e.bunker,
        e.bunker * 1.8
    ));
    if e.qpc_applicable {
        r.push_str(&format!(
            "\tPropulsion: QPC={:.3} (used in power calculation), hull={:.3}, open-water={:.3}, shaft={:.3}\n",
            e.qpc, e.hull_efficiency, e.open_water_efficiency, e.shaft_efficiency
        ));
    } else {
        r.push_str("\tPropulsion: original empirical shaft-power fit; no explicit QPC applies.\n");
    }
    r.push_str(&format!(
        "\tScrews: {}; range: {:.0} nm\n",
        e.screws, e.range
    ));

    r.push_str("\nPerformance:\n");
    r.push_str(&format!(
        "\tRoom: hull {:.1}%, deck {:.1}%\n",
        p.room_hull * 100.0,
        p.room_deck * 100.0
    ));
    r.push_str(&format!(
        "\tStability: {:.2}; metacentre: {:.2} ft\n",
        p.stability_adj, p.metacentre
    ));
    r.push_str(&format!(
        "\tSeakeeping: {:.2}; steadiness: {:.0}\n",
        p.seakeeping, p.steadiness_adj
    ));
    r.push_str(&format!("\tRecoil: {:.2}\n", p.recoil));
    r.push_str(&format!(
        "\tStructural weight: {:.0} lb/sq ft / {:.0} kg/sq m\n",
        p.struct_weight,
        p.struct_weight / SQFEET2SQMETRES * POUNDS2KG
    ));
    r.push_str(&format!(
        "\tStrength cross/long/composite: {:.2}/{:.2}/{:.2}\n",
        p.strength_cross, p.strength_long, p.strength_comp
    ));
    r.push_str(&format!(
        "\tCost: £{:.3}M / ${:.3}M\n",
        p.cost_pounds, p.cost_dollars
    ));

    if p.strength_comp < 0.5 {
        r.push_str("\nDESIGN FAILURE: Hull structure insufficient to carry required loads!\n");
    } else if p.strength_comp < 0.995 {
        r.push_str("\nWARNING: Hull structure strained by open sea conditions!\n");
    }
    // Same convergence caveat as `ShipModel::summary`: the report is the other
    // place a user reads the weight breakdown, so it must carry the warning too.
    if p.iterations > 0 && !p.converged {
        r.push_str(&format!(
            "\nWARNING: armour/machinery weight iteration did not converge after {} iterations (relative residual {:.2e}); the weight breakdown is approximate.\n",
            p.iterations, p.iteration_residual
        ));
    }
    r
}

pub fn generate_html_report(ship: &ShipModel) -> String {
    let text = generate_report(ship);
    let body = html_escape(&text).replace('\n', "<br>\n");
    format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>{}</title></head><body>\n<pre>{body}</pre>\n</body></html>\n",
        html_escape(&ship.name)
    )
}

fn describe_fuel(e: &crate::model::EngineValues) -> String {
    let fuel_names = [
        "coal-fired boilers",
        "oil-fired boilers",
        "diesel motors",
        "petrol motors",
        "batteries",
    ];
    let selected: Vec<&str> = e
        .fuel
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| if v { Some(fuel_names[i]) } else { None })
        .collect();
    if selected.is_empty() {
        "none".to_string()
    } else {
        selected.join(", ")
    }
}

pub fn demo_ship() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "HMS Example".into();
    ship.country = "United Kingdom".into();
    ship.ship_type = "Battleship".into();

    ship.hull.length = 600.0;
    ship.hull.beam = 90.0;
    ship.hull.beam_bulges = 90.0;
    ship.hull.draught = 28.0;
    ship.hull.block = 0.55;
    ship.hull.date_hull = 1915;
    ship.hull.date_engine = 1915;

    ship.free.fc_length = 0.2;
    ship.free.fd_length = 0.3;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 22.0;
    ship.free.free_fc_aft = 18.0;
    ship.free.free_fd_for = 18.0;
    ship.free.free_fd_aft = 15.0;
    ship.free.free_ad_for = 15.0;
    ship.free.free_ad_aft = 12.0;
    ship.free.free_qd_for = 12.0;
    ship.free.free_qd_aft = 10.0;

    let g1 = ship.battery_mut(0);
    g1.barrels = 8;
    g1.calibre = 15.0;
    g1.cal_length = 45.0;
    g1.date_gun = 1915;
    g1.gun_type = 1;
    g1.mount_type = 5;
    g1.magazine = 150;
    g1.mounts = 4;
    g1.guns_in_mount_g1 = 1;
    g1.layout_g1 = 0;
    g1.mount_super_g1 = 2;
    g1.mount_sub_g1 = 2;
    g1.arm_face = 12.0;
    g1.arm_back = 8.0;
    g1.arm_bottom = 4.0;

    let g2 = ship.battery_mut(1);
    g2.barrels = 12;
    g2.calibre = 6.0;
    g2.cal_length = 50.0;
    g2.date_gun = 1915;
    g2.gun_type = 1;
    g2.mount_type = 5;
    g2.magazine = 150;
    g2.mounts = 12;
    g2.guns_in_mount_g1 = 0;
    g2.layout_g1 = 9;
    g2.mount_super_g1 = 12;
    g2.arm_face = 2.0;
    g2.arm_back = 1.0;

    ship.armour.belt_main = 12.0;
    ship.armour.belt_main_length = 380.0;
    ship.armour.belt_main_height = 15.0;
    ship.armour.belt_ends = 5.0;
    ship.armour.belt_ends_length = 60.0;
    ship.armour.belt_ends_height = 15.0;
    ship.armour.belt_upper = 6.0;
    ship.armour.belt_upper_length = 300.0;
    ship.armour.belt_upper_height = 10.0;
    ship.armour.deck = 3.0;
    ship.armour.deck_fc = 1.0;
    ship.armour.deck_qd = 1.0;
    ship.armour.deck_coverage = 0;
    ship.armour.bulkhead = 1.0;
    ship.armour.bulkhead_length = 300.0;
    ship.armour.bulkhead_height = 20.0;
    ship.armour.bulkhead_beam = 70.0;
    ship.armour.bulge = 0.0;
    ship.armour.conning_for = 12.0;
    ship.armour.conning_aft = 12.0;

    ship.engine.fuel[0] = true;
    ship.engine.engine_types[1] = true;
    ship.engine.drive[0] = true;
    ship.engine.speed_max = 24.0;
    ship.engine.speed_cruise = 12.0;
    ship.engine.range = 7000.0;
    ship.engine.pct_coal = 20.0;
    ship.engine.screws = 4;

    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 4,
            loads: 2,
            size: 21.0,
            length: 24.0,
            location: 4,
            ..Default::default()
        },
        ..Default::default()
    };

    ship
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ShipModel;

    #[test]
    fn report_contains_ship_name() {
        let mut ship = ShipModel::new();
        ship.name = "Report Test".into();
        ship.hull.length = 400.0;
        ship.hull.beam = 60.0;
        ship.hull.beam_bulges = 60.0;
        ship.hull.draught = 20.0;
        ship.hull.block = 0.5;
        ship.hull.date_hull = 1920;
        ship.hull.date_engine = 1920;
        ship.engine.screws = 2;
        ship.calculate();
        let report = generate_report(&ship);
        assert!(report.contains("Report Test"));
        assert!(report.contains("Displacement"));
    }

    #[test]
    fn report_has_classification_and_gun_details() {
        let mut ship = ShipModel::new();
        ship.name = "Ironclad Test".into();
        ship.hull.length = 300.0;
        ship.hull.beam = 50.0;
        ship.hull.beam_bulges = 50.0;
        ship.hull.draught = 18.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1875;
        ship.hull.date_engine = 1875;
        ship.free.fc_length = 0.2;
        ship.free.fd_length = 0.3;
        ship.free.qd_length = 0.15;
        ship.engine.screws = 2;
        let g = &mut ship.guns.batteries[0];
        g.barrels = 6;
        g.calibre = 10.0;
        g.cal_length = 15.0;
        g.date_gun = 1875;
        g.gun_type = 0;
        g.mount_type = 0;
        g.mount_super_g1 = 0;
        g.mount_sub_g1 = 3;
        g.mounts = 3;
        g.guns_in_mount_g1 = 1;
        g.layout_g1 = 9;
        ship.armour.belt_main = 8.0;
        ship.armour.belt_main_length = 200.0;
        ship.armour.belt_main_height = 10.0;
        ship.calculate();
        let classes = ship_classification(&ship);
        assert!(classes.iter().any(|c| c.contains("Armoured")));
        let report = generate_report(&ship);
        assert!(report.contains("Muzzle loading gun"));
        assert!(report.contains("hull mounts"));
    }

    #[test]
    fn max_speed_solver_respects_power_limit() {
        let mut ship = ShipModel::new();
        ship.hull.length = 600.0;
        ship.hull.beam = 90.0;
        ship.hull.beam_bulges = 90.0;
        ship.hull.draught = 28.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1915;
        ship.hull.date_engine = 1915;
        ship.free.free_fd_for = 20.0;
        ship.free.free_fd_aft = 16.0;
        ship.free.free_ad_for = 16.0;
        ship.free.free_ad_aft = 14.0;
        ship.engine.fuel[0] = true;
        ship.engine.engine_types[1] = true;
        ship.engine.drive[0] = true;
        ship.engine.screws = 4;
        ship.engine.speed_cruise = 12.0;
        ship.engine.range = 7000.0;

        let speed = solve_max_speed(&mut ship, 50_000.0);
        assert!(speed > 0.0 && speed <= 60.0);
        assert!(ship.engine.hp_max <= 50_000.0 + 1.0);
    }
}
