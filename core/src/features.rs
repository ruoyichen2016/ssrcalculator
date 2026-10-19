//! Higher-level features: report generation and iterative solvers.

use crate::model::{ShipModel, WeaponAggregate, WeaponKind, WeaponMount};

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

/// Scale ship hull dimensions (length, beam, draught, belt dimensions) proportionally
/// to achieve a target normal displacement (tons).
pub fn solve_displacement_scale(ship: &mut ShipModel, target_disp: f64) -> f64 {
    if target_disp <= 0.0 || !target_disp.is_finite() {
        return ship.hull.displacement;
    }
    let orig_l = ship.hull.length;
    let orig_b = ship.hull.beam;
    let orig_bb = ship.hull.beam_bulges;
    let orig_t = ship.hull.draught;
    let orig_bl = ship.armour.belt_main_length;
    let orig_bh = ship.armour.belt_main_height;

    let mut lo = 0.2_f64;
    let mut hi = 5.0_f64;
    let mut best_s = 1.0_f64;
    let mut iter = 0_u32;

    while hi - lo > 1e-5 && iter < 100 {
        let mid = (lo + hi) * 0.5;
        ship.hull.length = orig_l * mid;
        ship.hull.beam = orig_b * mid;
        ship.hull.beam_bulges = orig_bb * mid;
        ship.hull.draught = orig_t * mid;
        if orig_bl > 0.0 {
            ship.armour.belt_main_length = orig_bl * mid;
        }
        if orig_bh > 0.0 {
            ship.armour.belt_main_height = orig_bh * mid;
        }
        ship.calculate();
        let disp = ship.hull.displacement;
        if disp.is_finite() {
            best_s = mid;
            if disp < target_disp {
                lo = mid;
            } else {
                hi = mid;
            }
        } else {
            hi = mid;
        }
        iter += 1;
    }

    ship.hull.length = orig_l * best_s;
    ship.hull.beam = orig_b * best_s;
    ship.hull.beam_bulges = orig_bb * best_s;
    ship.hull.draught = orig_t * best_s;
    if orig_bl > 0.0 {
        ship.armour.belt_main_length = orig_bl * best_s;
    }
    if orig_bh > 0.0 {
        ship.armour.belt_main_height = orig_bh * best_s;
    }
    ship.calculate();
    ship.hull.displacement
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
    g1.gun_type = 1; // Breech loading
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 150;
    g1.mounts = 4;
    g1.guns_in_mount_g1 = 3; // Twin mount (4 x 2 = 8 guns)
    g1.layout_g1 = 1; // Centreline ends (fore >= aft: 2 fore, 2 aft)
    g1.mount_super_g1 = 2; // Fore & aft superfiring
    g1.mount_sub_g1 = 0;
    g1.arm_face = 13.0;
    g1.arm_back = 10.0;
    g1.arm_bottom = 10.0;

    let g2 = ship.battery_mut(1);
    g2.barrels = 14;
    g2.calibre = 6.0;
    g2.cal_length = 45.0;
    g2.date_gun = 1915;
    g2.gun_type = 2; // Quick firing
    g2.mount_type = 6; // Casemate
    g2.magazine = 150;
    g2.mounts = 14;
    g2.guns_in_mount_g1 = 0; // Single
    g2.layout_g1 = 9; // Sides
    g2.mount_sub_g1 = 14; // in casemates
    g2.arm_face = 6.0;
    g2.arm_back = 2.0;

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
