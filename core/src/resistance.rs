//! Holtrop-Mennen style resistance and power prediction.
//!
//! The implementation follows the classic Holtrop & Mennen approximate power
//! prediction method used for displacement hulls.  It is a direct Rust port of
//! a compact public-domain Python implementation, with the friction correction
//! and bulbous-bow term enabled.

use crate::model::HullValues;

/// Hull parameters needed by the Holtrop-Mennen method (SI units).
#[derive(Clone, Debug)]
pub struct HoltropInput {
    /// Length on waterline, m
    pub lwl: f64,
    /// Length between perpendiculars, m
    pub lbp: f64,
    /// Moulded breadth, m
    pub breadth: f64,
    /// Draught at forward perpendicular, m
    pub draught_fp: f64,
    /// Draught at aft perpendicular, m
    pub draught_ap: f64,
    /// Moulded displacement volume, m^3
    pub volume: f64,
    /// Longitudinal centre of buoyancy, % L forward of half length (positive forward)
    pub lcb_percent: f64,
    /// Transverse bulb area, m^2
    pub bulb_area: f64,
    /// Centre of bulb area above keel, m
    pub bulb_centre_height: f64,
    /// Midship section coefficient
    pub cm: f64,
    /// Waterplane area coefficient
    pub cwp: f64,
    /// Immersed transom area, m^2
    pub transom_area: f64,
    /// Wetted area of appendages, m^2
    pub appendage_area: f64,
    /// Stern shape parameter (Cstern: -25 pram, -10 V, 0 normal, +10 U)
    pub cstern: f64,
    /// Ship speed, m/s
    pub speed_ms: f64,
    /// Block coefficient
    pub cb: f64,
    /// Quasi-propulsive coefficient (total propulsion efficiency), default 0.70
    pub propeller_efficiency: f64,
}

impl HoltropInput {
    pub fn cp(&self) -> f64 {
        if self.cm > 0.0 {
            self.cb / self.cm
        } else {
            self.cb
        }
    }

    pub fn froude(&self) -> f64 {
        self.speed_ms / (9.80665 * self.lwl).sqrt()
    }
}

#[derive(Clone, Debug, Default)]
pub struct HoltropResistance {
    pub friction_n: f64,
    pub appendage_n: f64,
    pub wave_n: f64,
    pub bulb_n: f64,
    pub transom_n: f64,
    pub correlation_n: f64,
    pub total_n: f64,
    pub effective_power_kw: f64,
    pub shaft_hp: f64,
}

/// Propulsion efficiency breakdown.
#[derive(Clone, Copy, Debug)]
pub struct PropulsionEfficiency {
    pub wake_fraction: f64,
    pub thrust_deduction: f64,
    pub hull_efficiency: f64,
    pub open_water_efficiency: f64,
    pub relative_rotative_efficiency: f64,
    pub shaft_efficiency: f64,
    pub quasi_propulsive_coefficient: f64,
}

/// Estimate a warship-like propulsion efficiency from hull proportions.
///
/// This replaces the previous fixed 0.70 QPC assumption.  The estimates are
/// based on common Taylor/Holtrop-style wake and thrust-deduction correlations.
pub fn estimate_propulsion_efficiency(hull: &HullValues, speed_kn: f64) -> PropulsionEfficiency {
    let cb = hull.block.max(0.3);
    // Wake fraction: fuller hulls have higher wake.
    let wake_fraction = 0.10 + 0.10 * cb;
    // Thrust deduction: roughly 1.2-1.4 x wake fraction.
    let thrust_deduction = 0.20 * (1.0 + cb);
    let hull_efficiency = (1.0 - thrust_deduction) / (1.0 - wake_fraction);

    // Open-water efficiency: high-speed slender ships are less efficient than
    // moderate-speed displacement ships.
    let fn_ = speed_kn * 0.514444 / (9.80665 * hull.length * 0.3048).sqrt();
    let open_water_efficiency = if fn_ > 0.35 {
        0.58
    } else if fn_ > 0.28 {
        0.62
    } else if fn_ > 0.20 {
        0.66
    } else {
        0.68
    };

    let relative_rotative_efficiency = 0.98;
    let shaft_efficiency = 0.97;
    let quasi_propulsive_coefficient =
        hull_efficiency * open_water_efficiency * relative_rotative_efficiency * shaft_efficiency;

    PropulsionEfficiency {
        wake_fraction,
        thrust_deduction,
        hull_efficiency,
        open_water_efficiency,
        relative_rotative_efficiency,
        shaft_efficiency,
        quasi_propulsive_coefficient,
    }
}

pub const RHO_SW: f64 = 1025.0;
pub const G: f64 = 9.80665;
pub const NU_SW: f64 = 1.1902e-6;

pub fn holtrop_resistance(input: &HoltropInput) -> HoltropResistance {
    let l = input.lwl;
    let b = input.breadth;
    let t = input.draught_fp;
    let vol = input.volume;
    let v = input.speed_ms;
    let cb = input.cb;
    let cm = input.cm;
    let cwp = input.cwp;
    let cp = input.cp();
    let fn_ = input.froude();

    // Froude number and prismatic coefficient in the Python reference.
    let _fn = fn_;
    let _cp = cp;

    // Longitudinal position of centre of buoyancy, fraction of L.
    let lcb = input.lcb_percent / 100.0;

    // Length of run parameter.
    let lr = (1.0 - cp + (0.06 * cp * lcb / (4.0 * cp - 1.0))) * l;

    // Draft/length coefficient.
    let e29 = if t / l > 0.05 {
        (t / l).powf(0.2228446)
    } else if t / l > 0.02 {
        48.2 * (t / l - 0.02).powf(2.078) + 0.479948
    } else {
        0.479948
    };

    // Afterbody shape coefficient.
    let e30 = 1.0 + 0.003 * input.cstern;

    // Viscous form factor (1+k1).
    let e31 = e30
        * (0.93
            + e29
                * (b / lr).powf(0.92497)
                * (0.95 - cp).powf(-0.521448)
                * (1.0 - cp + 0.0225 * lcb).powf(0.6906));

    // Wetted surface area.
    let surface = l
        * (2.0 * t + b)
        * cm.sqrt()
        * (0.453 + 0.4425 * cb - 0.2862 * cm - 0.003467 * (b / t) + 0.3696 * cwp)
        + 2.38 * input.bulb_area / cb;

    // Frictional resistance.
    let re = l * v / NU_SW;
    let cf = 0.075 / (re.log10() - 2.0).powi(2);
    let friction = 0.5 * RHO_SW * surface * v * v * cf;

    // Appendage resistance (kept as input area; no extra form factor applied).
    let appendage = 0.5 * RHO_SW * v * v * input.appendage_area * cf;

    // Wave resistance coefficients.
    let c7 = if b / l < 0.11 {
        0.229577 * (b / l).powf(0.33333)
    } else if b / l > 0.25 {
        0.5 - 0.0625 * (l / b)
    } else {
        b / l
    };

    // Half angle of entrance.  If no better data is available, use a typical
    // warship-like entrance angle based on hull proportions.
    let i_e = 10.0_f64.max(
        125.67 * (b / l) - 162.25 * cp * cp
            + 234.32 * cp * cp * cp
            + 0.1551 * (input.lcb_percent + 6.8).powi(3),
    );

    let c1 = 2_223_105.0 * c7.powf(3.78613) * (t / b).powf(1.07961) * (90.0 - i_e).powf(-1.37565);
    let c3 = if input.bulb_area > 0.0 {
        0.56 * input.bulb_area.powf(1.5)
            / (b * t * (0.31 * input.bulb_area.sqrt() + t - input.bulb_centre_height))
    } else {
        0.0
    };
    let c2 = if c3 > 0.0 {
        (-1.89 * c3.sqrt()).exp()
    } else {
        1.0
    };
    let c5 = 1.0 - 0.8 * input.transom_area / (b * t * cm);

    let c16 = if cp < 0.8 {
        8.07981 * cp - 13.8673 * cp * cp + 6.984388 * cp * cp * cp
    } else {
        1.73014 - 0.7067 * cp
    };

    let m1 = 0.0140407 * (l / t) - 1.75254 * vol.cbrt() / l - 4.79323 * (b / l) - c16;
    let c15 = if l.powi(3) / vol < 512.0 {
        -1.69385
    } else if l.powi(3) / vol > 1727.0 {
        0.0
    } else {
        -1.69385 + (l / vol.cbrt() - 8.0) / 2.36
    };
    let m2 = c15 * cb * cb * (-0.1 * fn_.powi(-2)).exp();
    let lambda = if l / b < 12.0 {
        1.446 * cp - 0.03 * (l / b)
    } else {
        1.446 * cp - 0.36
    };

    let wave = c1
        * c2
        * c5
        * vol
        * RHO_SW
        * G
        * (m1 * fn_.powf(-0.9) + m2 * (lambda * fn_.powi(-2)).cos()).exp();

    // Bulbous bow resistance.
    let bulb = if input.bulb_area > 0.0 && t - input.bulb_centre_height > 0.0 {
        let p = 0.56 * input.bulb_area.sqrt() / (t - 1.5 * input.bulb_centre_height);
        let fni = v
            / (G * (t - input.bulb_centre_height - 0.25 * input.bulb_area.sqrt()) + 0.15 * v * v)
                .sqrt();
        0.11 * (-3.0 / p.powi(2)).exp() * fni.powi(3) * input.bulb_area.powf(1.5) * RHO_SW * G
            / (1.0 + fni * fni)
    } else {
        0.0
    };

    // Immersed transom resistance.
    let transom = if input.transom_area > 0.0 {
        let fnt = v / (2.0 * G * input.transom_area / (b + b * cwp)).sqrt();
        let c6 = if fnt < 5.0 {
            0.2 * (1.0 - 0.2 * fnt)
        } else {
            0.0
        };
        0.5 * RHO_SW * v * v * input.transom_area * c6
    } else {
        0.0
    };

    // Model-ship correlation resistance.
    let e56 = if t / l > 0.04 { 0.04 } else { t / l };
    let ca = ((0.105 * (3.8742e-4_f64).cbrt() - 0.005579) / l.cbrt()) * e56;
    let correlation = 0.5 * RHO_SW * v * v * ca * surface;

    let total = friction * e31 + appendage + wave + bulb + transom + correlation;
    let effective_w = total * v;
    let effective_kw = effective_w / 1000.0;
    let qpc = if input.propeller_efficiency > 0.0 {
        input.propeller_efficiency
    } else {
        0.70
    };
    let shaft_hp = effective_w / 745.6999 / qpc;

    HoltropResistance {
        friction_n: friction * e31,
        appendage_n: appendage,
        wave_n: wave,
        bulb_n: bulb,
        transom_n: transom,
        correlation_n: correlation,
        total_n: total,
        effective_power_kw: effective_kw,
        shaft_hp,
    }
}

/// Build an approximate Holtrop-Mennen input from the existing SpringSharp hull
/// model.  Several parameters not directly represented by SpringSharp (bulb,
/// transom, appendages) are set to neutral values.
pub fn input_from_hull(hull: &HullValues, speed_kn: f64) -> HoltropInput {
    let l = hull.length * 0.3048;
    let b = hull.beam * 0.3048;
    let t = hull.draught * 0.3048;
    let volume = hull.displacement * 35.0 * 0.0283168466;
    let cb = hull.block;
    // Approximate midship coefficient for typical displacement warships unless
    // the user supplied a better value.
    let cm = if hull.midship_coefficient > 0.0 {
        hull.midship_coefficient
    } else {
        (0.90 + 0.10 * cb).min(0.99)
    };
    let cwp = hull.co_water_plane;
    let sqft2m2 = 0.092903;
    let ft2m = 0.3048;

    HoltropInput {
        lwl: l,
        lbp: l * 0.97,
        breadth: b,
        draught_fp: t,
        draught_ap: t,
        volume,
        lcb_percent: if hull.lcb_percent != 0.0 {
            hull.lcb_percent
        } else {
            -0.75
        },
        bulb_area: hull.bulb_area_sqft * sqft2m2,
        bulb_centre_height: hull.bulb_centre_height_ft * ft2m,
        cm,
        cwp,
        transom_area: hull.transom_area_sqft * sqft2m2,
        appendage_area: hull.appendage_area_sqft * sqft2m2,
        cstern: hull.stern_shape,
        speed_ms: speed_kn * 0.514444,
        cb,
        propeller_efficiency: estimate_propulsion_efficiency(hull, speed_kn)
            .quasi_propulsive_coefficient,
    }
}

pub fn holtrop_power_hp(hull: &HullValues, speed_kn: f64) -> f64 {
    let input = input_from_hull(hull, speed_kn);
    holtrop_resistance(&input).shaft_hp
}

/// Holtrop-Mennen shaft power with structural-form calibration multipliers.
pub fn holtrop_power_hp_calibrated(
    hull: &HullValues,
    speed_kn: f64,
    viscous_factor: f64,
    wave_factor: f64,
    correlation_factor: f64,
) -> f64 {
    let input = input_from_hull(hull, speed_kn);
    let res = holtrop_resistance(&input);
    let v = speed_kn * 0.514444;
    let total = res.friction_n * viscous_factor
        + res.appendage_n * viscous_factor
        + (res.wave_n + res.bulb_n + res.transom_n) * wave_factor
        + res.correlation_n * correlation_factor;
    let qpc = if input.propeller_efficiency > 0.0 {
        input.propeller_efficiency
    } else {
        0.70
    };
    total * v / 745.6999 / qpc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::HullValues;

    #[test]
    fn holtrop_gives_finite_power_for_demo_hull() {
        let hull = HullValues {
            length: 600.0,
            beam: 90.0,
            beam_bulges: 90.0,
            draught: 28.0,
            block: 0.55,
            displacement: 23760.0,
            co_water_plane: 0.85,
            ..Default::default()
        };
        let hp = holtrop_power_hp(&hull, 24.0);
        assert!(hp.is_finite() && hp > 0.0);
    }
}
