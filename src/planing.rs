//! Experimental planing/semi-planing hull power estimation.
//!
//! This is intentionally a simplified Savitsky-style model for very small,
//! high-speed craft where the displacement Holtrop-Mennen method is invalid.
//! It should be replaced with a full Savitsky implementation or CFD once more
//! data are available.

use crate::accuracy::ExternalCase;

/// Estimate shaft horsepower for a simple planing hull.
///
/// Arguments are in Imperial units: length/beam in feet, displacement in pounds,
/// speed in knots.
pub fn planing_hp_estimate(
    length_ft: f64,
    beam_ft: f64,
    displacement_lbs: f64,
    speed_kn: f64,
) -> f64 {
    if length_ft <= 0.0 || beam_ft <= 0.0 || displacement_lbs <= 0.0 || speed_kn <= 0.0 {
        return 0.0;
    }
    let g = 32.2_f64;
    let v = speed_kn * 1.68781; // ft/s
    let b = beam_ft;
    let _cv = v / (g * b).sqrt();

    // Simplified dynamic lift angle trim, degrees.
    let trim_deg = 4.0_f64;
    let lift_drag = (trim_deg * std::f64::consts::PI / 180.0).tan();

    // Very rough wetted area.
    let wet_area = 0.5 * length_ft * beam_ft;
    let rho = 1.94_f64; // slugs/ft^3 seawater
    let cf = 0.004_f64;
    let friction_lbf = 0.5 * rho * wet_area * v * v * cf;

    let total_lbf = displacement_lbs * lift_drag + friction_lbf;
    total_lbf * v / 550.0
}

/// Default calibration factor estimated from `data/examples/planing_ships.csv`.
pub const DEFAULT_PLANING_CALIBRATION: f64 = 0.738;

/// Calibrated planing power estimate using a multiplicative factor.
pub fn planing_hp_estimate_calibrated(
    length_ft: f64,
    beam_ft: f64,
    displacement_lbs: f64,
    speed_kn: f64,
    factor: f64,
) -> f64 {
    planing_hp_estimate(length_ft, beam_ft, displacement_lbs, speed_kn) * factor
}

/// Compute a planing calibration factor from eligible external cases:
/// `mean(reference_hp / raw_planing_hp)`.
pub fn compute_planing_calibration(cases: &[ExternalCase]) -> f64 {
    let mut sum = 0.0_f64;
    let mut count = 0_u32;
    for case in cases {
        if !is_planing_eligible(case.speed_kt, &case.ship_type) {
            continue;
        }
        let raw = planing_hp_estimate(
            case.length_ft,
            case.beam_ft,
            case.displacement_tons * 2240.0,
            case.speed_kt,
        );
        if raw > 0.0 && case.reference_hp > 0.0 {
            sum += case.reference_hp / raw;
            count += 1;
        }
    }
    if count > 0 {
        sum / count as f64
    } else {
        DEFAULT_PLANING_CALIBRATION
    }
}

/// Compute a combined planing calibration factor from multiple example files.
pub fn compute_planing_calibration_from_files(paths: &[&str]) -> f64 {
    let mut sum = 0.0_f64;
    let mut count = 0_u32;
    for path in paths {
        if let Ok(text) = std::fs::read_to_string(path) {
            if let Ok(cases) = crate::accuracy::parse_external_cases(&text) {
                let factor = compute_planing_calibration(&cases);
                if factor.is_finite() {
                    sum += factor;
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        sum / count as f64
    } else {
        DEFAULT_PLANING_CALIBRATION
    }
}

/// True for cases where the experimental planing model should be preferred over
/// the displacement Holtrop-Mennen model.
pub fn is_planing_eligible(speed_kn: f64, ship_type: &str) -> bool {
    speed_kn >= 25.0 || ship_type.contains("motor")
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibrated_planing_estimate_is_lower() {
        let raw = planing_hp_estimate(80.0, 20.0, 112000.0, 40.0);
        let cal =
            planing_hp_estimate_calibrated(80.0, 20.0, 112000.0, 40.0, DEFAULT_PLANING_CALIBRATION);
        assert!(cal < raw);
        assert!(cal > 0.0);
    }

    #[test]
    fn compute_planing_calibration_from_files_is_finite() {
        let paths = [
            "data/examples/planing_ships.csv",
            "data/examples/more_planing.csv",
        ];
        let factor = compute_planing_calibration_from_files(&paths);
        assert!(factor.is_finite());
        assert!(factor > 0.0);
    }

    #[test]
    fn compute_planing_calibration_empty_falls_back_to_default() {
        let factor = compute_planing_calibration(&[]);
        assert!((factor - DEFAULT_PLANING_CALIBRATION).abs() < 1e-6);
    }

    #[test]
    fn compute_planing_calibration_from_external_cases() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\nS-Boat,motor,115,17,5,92,0.43,39,3000,0.65,approx\n";
        let cases = crate::accuracy::parse_external_cases(csv).unwrap();
        let factor = compute_planing_calibration(&cases);
        assert!(factor.is_finite());
        assert!(factor > 0.0);
    }

    #[test]
    fn planing_eligibility_rules() {
        assert!(is_planing_eligible(40.0, "motor"));
        assert!(is_planing_eligible(30.0, "patrol"));
        assert!(!is_planing_eligible(16.0, "patrol"));
        assert!(!is_planing_eligible(20.0, "frigate"));
    }

    #[test]
    fn pt_boat_estimate_is_finite_and_positive() {
        let hp = planing_hp_estimate(80.0, 20.0, 50.0 * 2240.0, 40.0);
        assert!(hp.is_finite());
        assert!(hp > 0.0);
        // Very loose sanity range: 1000-10000 hp for a PT boat.
        assert!(hp > 500.0 && hp < 10000.0);
    }
}
