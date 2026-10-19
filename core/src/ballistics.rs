//! Exterior ballistics, armor penetration (De Marre model), and immunity zone analysis.
//!
//! Provides planning-level exterior ballistics and Krupp-cemented (Class A / Class B)
//! armor penetration estimates across combat ranges (5,000 to 35,000 yards)
//! based on historical naval ordnance empirical formulas.

use crate::model::GunBattery;

/// A single range-step trajectory and armor penetration calculation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallisticPoint {
    pub range_yd: f64,
    pub striking_vel_fps: f64,
    pub fall_angle_deg: f64,
    pub belt_pen_in: f64,
    pub deck_pen_in: f64,
}

/// Immunity zone calculation comparing an attacking gun battery against a target ship's protection.
#[derive(Clone, Debug, PartialEq)]
pub struct ImmunityZone {
    pub gun_calibre: f64,
    pub target_belt_in: f64,
    pub target_belt_incline: f64,
    pub target_deck_in: f64,
    /// Minimum range (yards) where target belt armor successfully resists penetration (inner limit).
    pub inner_limit_yd: Option<f64>,
    /// Maximum range (yards) before plunging fire penetrates target deck armor (outer limit).
    pub outer_limit_yd: Option<f64>,
    /// True if an immunity zone exists (inner_limit <= outer_limit).
    pub has_zone: bool,
    pub curve: Vec<BallisticPoint>,
}

/// Estimate standard AP muzzle velocity (ft/s) based on gun year, barrel length, and shell weight ratio.
pub fn estimate_muzzle_velocity(
    calibre: f64,
    cal_length: f64,
    date_gun: i32,
    shell_weight: f64,
) -> f64 {
    if calibre <= 0.0 || cal_length <= 0.0 {
        return 0.0;
    }
    // Base velocity: historical naval guns (WWI ~2500-2700 fps, WWII ~2500-2900 fps)
    let year_term = ((date_gun - 1900) as f64).clamp(-20.0, 50.0) * 3.5;
    let len_term = (cal_length.clamp(25.0, 60.0) - 45.0) * 16.0;
    let base_v0 = 2600.0 + year_term + len_term;

    // Shell weight factor: heavy shells (e.g. US super-heavy 16" Mk 8 2700 lb) have lower V0
    let std_shell_wt = 0.50 * calibre.powi(3);
    let wt_ratio = if std_shell_wt > 0.0 && shell_weight > 0.0 {
        (shell_weight / std_shell_wt).clamp(0.6, 1.6)
    } else {
        1.0
    };
    // Higher mass -> lower muzzle velocity for same propellant energy: v ~ 1 / sqrt(m)
    let v0 = base_v0 / wt_ratio.powf(0.42);
    v0.clamp(1800.0, 3200.0)
}

/// Compute ballistic trajectory points and De Marre armor penetration from 5,000 to 35,000 yards.
pub fn compute_ballistic_curve(
    calibre: f64,
    cal_length: f64,
    date_gun: i32,
    shell_weight: f64,
    target_belt_incline_deg: f64,
) -> Vec<BallisticPoint> {
    if calibre <= 0.0 {
        return Vec::new();
    }
    let v0 = estimate_muzzle_velocity(calibre, cal_length, date_gun, shell_weight);
    if v0 <= 0.0 {
        return Vec::new();
    }

    let actual_shell_wt = if shell_weight > 0.0 {
        shell_weight
    } else {
        0.50 * calibre.powi(3)
    };

    // Ballistic sectional density factor C: larger/heavier shells retain velocity much better
    // C ~ W / d^2; scaled for historical AP form factors
    let ballistic_coeff = (actual_shell_wt / (calibre * calibre)).clamp(5.0, 20.0);

    let mut points = Vec::with_capacity(13);
    for step in 2..=14 {
        let range_yd = (step * 2500) as f64; // 5,000 to 35,000 yd
        let range_ft = range_yd * 3.0;

        // Velocity decay: exponential retardation
        // Historical naval shells: retardation scale ~ 7500 * ballistic coefficient
        let retard_scale = 7500.0 * ballistic_coeff;
        let v_rel = (-range_yd / retard_scale).exp().clamp(0.25, 0.98);
        let striking_v = v0 * v_rel;

        // Angle of fall theta (degrees from horizontal):
        // Flat at short range (3-6° at 10k yd), plunging at long range (20-45° at 25-35k yd)
        let g = 32.174;
        let v_avg = 0.5 * (v0 + striking_v);
        let tof = range_ft / v_avg.max(1.0); // time of flight in seconds
        let vy = g * tof * 0.65;
        let fall_angle_rad = (vy / striking_v.max(1.0)).atan();
        let fall_angle_deg =
            (fall_angle_rad.to_degrees() * (1.0 + 0.000018 * range_yd)).clamp(1.5, 65.0);

        // Belt impact angle from normal:
        // fall_angle plus belt slope/incline (incline tilts top away, increasing obliquity)
        let belt_obliquity_deg = (fall_angle_deg + target_belt_incline_deg.abs()).clamp(0.0, 80.0);
        let belt_cos = belt_obliquity_deg.to_radians().cos().max(0.12);

        // Deck impact angle from normal:
        // Deck is horizontal, so normal is vertical; obliquity from normal is 90° - fall_angle
        let deck_obliquity_deg = (90.0 - fall_angle_deg).clamp(0.0, 85.0);
        let deck_cos = deck_obliquity_deg.to_radians().cos().max(0.05);

        // De Marre penetration formula:
        // P_norm = calibre * ( (W / d^3)^0.714 ) * ( (V / 1000)^1.428 ) * C_kc
        // Calibrated against Krupp cemented armor ballistic tables
        let wt_per_d3 = (actual_shell_wt / calibre.powi(3)).clamp(0.35, 0.70);
        let p_scale = calibre * (wt_per_d3 / 0.50).powf(0.714) * (striking_v / 1000.0).powf(1.428);

        // Obliquity scaling: penetration drops with cos^1.3(obliquity)
        let belt_pen = (0.390 * p_scale * belt_cos.powf(1.30)).max(0.0);
        // Horizontal deck penetration: plunging velocity component
        let deck_pen = (0.390 * p_scale * deck_cos.powf(1.30)).max(0.0);

        points.push(BallisticPoint {
            range_yd,
            striking_vel_fps: striking_v,
            fall_angle_deg,
            belt_pen_in: belt_pen,
            deck_pen_in: deck_pen,
        });
    }

    points
}

/// Calculate the immunity zone against target belt and deck armor.
pub fn compute_immunity_zone(
    calibre: f64,
    cal_length: f64,
    date_gun: i32,
    shell_weight: f64,
    target_belt_in: f64,
    target_belt_incline: f64,
    target_deck_in: f64,
) -> ImmunityZone {
    let curve = compute_ballistic_curve(
        calibre,
        cal_length,
        date_gun,
        shell_weight,
        target_belt_incline,
    );

    if curve.is_empty() || target_belt_in <= 0.0 || target_deck_in <= 0.0 {
        return ImmunityZone {
            gun_calibre: calibre,
            target_belt_in,
            target_belt_incline,
            target_deck_in,
            inner_limit_yd: None,
            outer_limit_yd: None,
            has_zone: false,
            curve,
        };
    }

    // Inner limit: where belt_pen drops to <= target_belt_in (belt becomes safe)
    let mut inner = None;
    for i in 0..curve.len() - 1 {
        let p1 = &curve[i];
        let p2 = &curve[i + 1];
        if p1.belt_pen_in >= target_belt_in && p2.belt_pen_in <= target_belt_in {
            let frac = (target_belt_in - p1.belt_pen_in) / (p2.belt_pen_in - p1.belt_pen_in);
            inner = Some(p1.range_yd + frac * (p2.range_yd - p1.range_yd));
            break;
        }
    }
    if inner.is_none() && !curve.is_empty() && curve[0].belt_pen_in <= target_belt_in {
        inner = Some(curve[0].range_yd);
    }

    // Outer limit: where deck_pen rises to >= target_deck_in (deck is breached by plunging fire)
    let mut outer = None;
    for i in 0..curve.len() - 1 {
        let p1 = &curve[i];
        let p2 = &curve[i + 1];
        if p1.deck_pen_in <= target_deck_in && p2.deck_pen_in >= target_deck_in {
            let frac = (target_deck_in - p1.deck_pen_in) / (p2.deck_pen_in - p1.deck_pen_in);
            outer = Some(p1.range_yd + frac * (p2.range_yd - p1.range_yd));
            break;
        }
    }
    if outer.is_none()
        && !curve.is_empty()
        && curve
            .last()
            .map(|p| p.deck_pen_in <= target_deck_in)
            .unwrap_or(false)
    {
        outer = Some(curve.last().unwrap().range_yd);
    }

    let has_zone = match (inner, outer) {
        (Some(i), Some(o)) => i <= o,
        _ => false,
    };

    ImmunityZone {
        gun_calibre: calibre,
        target_belt_in,
        target_belt_incline,
        target_deck_in,
        inner_limit_yd: inner,
        outer_limit_yd: outer,
        has_zone,
        curve,
    }
}

/// Compute self-immunity zone for a gun battery against own ship's belt and deck protection.
pub fn compute_self_immunity_zone(
    battery: &GunBattery,
    belt_thick: f64,
    belt_incline: f64,
    deck_thick: f64,
) -> ImmunityZone {
    compute_immunity_zone(
        battery.calibre,
        battery.cal_length,
        battery.date_gun,
        battery.shell_weight,
        belt_thick,
        belt_incline,
        deck_thick,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ballistic_curve_produces_plausible_penetrations() {
        // 16"/50 cal gun (Iowa type: 2700 lb shell, 1943)
        let curve = compute_ballistic_curve(16.0, 50.0, 1943, 2700.0, 19.0);
        assert_eq!(curve.len(), 13);
        assert_eq!(curve[0].range_yd, 5000.0);
        assert_eq!(curve.last().unwrap().range_yd, 35000.0);

        // At 5,000 yd, striking velocity should be close to muzzle velocity
        assert!(curve[0].striking_vel_fps > 2200.0);
        // Belt penetration should exceed 20 inches at close range
        assert!(curve[0].belt_pen_in > 20.0);
        // Deck penetration at 5,000 yd should be low (< 3 inches)
        assert!(curve[0].deck_pen_in < 3.0);

        // As range increases, belt penetration decreases and deck penetration increases
        for i in 0..curve.len() - 1 {
            assert!(curve[i].belt_pen_in >= curve[i + 1].belt_pen_in);
            assert!(curve[i].deck_pen_in <= curve[i + 1].deck_pen_in);
            assert!(curve[i].fall_angle_deg <= curve[i + 1].fall_angle_deg);
        }
    }

    #[test]
    fn immunity_zone_finds_valid_range_bracket() {
        // Iowa vs 16"/50 gun: Belt ~12.1" inclined 19 deg, Deck ~6.0"
        let iz_heavy = compute_immunity_zone(16.0, 50.0, 1943, 2700.0, 12.1, 19.0, 6.0);
        assert!(iz_heavy.inner_limit_yd.is_some());
        assert!(iz_heavy.outer_limit_yd.is_some());
        let inner = iz_heavy.inner_limit_yd.unwrap();
        let outer = iz_heavy.outer_limit_yd.unwrap();
        assert!(
            (18000.0..=26000.0).contains(&inner),
            "inner limit was {inner}"
        );
        assert!(
            (18000.0..=26000.0).contains(&outer),
            "outer limit was {outer}"
        );

        // Iowa vs standard 16"/45 (2240 lb shell, 1923): broad immunity zone
        let iz_std = compute_immunity_zone(16.0, 45.0, 1923, 2240.0, 12.1, 19.0, 6.0);
        assert!(iz_std.has_zone);
        assert!(iz_std.inner_limit_yd.unwrap() < iz_std.outer_limit_yd.unwrap());
    }

    #[test]
    fn cruiser_gun_has_sensible_values() {
        // 8"/55 cruiser gun vs 5.0" belt, 2.5" deck
        let iz = compute_immunity_zone(8.0, 55.0, 1940, 335.0, 5.0, 0.0, 2.5);
        assert_eq!(iz.curve.len(), 13);
        assert!(iz.curve[0].belt_pen_in > 7.0);
        assert!(iz.curve.last().unwrap().belt_pen_in < 6.0);
    }
}
