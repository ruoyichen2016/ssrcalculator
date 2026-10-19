//! Rule-style scantling checks and structural-weight estimate.
//!
//! Implements the standard classification plate-panel and stiffener formulae
//! (ABS/DNV style) with planning values:
//!
//! - plate thickness `t = s * sqrt(p / (k * sigma))` (panel bending)
//! - plate buckling slenderness `s/t <= c * sqrt(E / sigma_y)`
//! - minimum thickness plus corrosion addition
//! - stiffener section modulus `Z = p * s * l^2 / (m * sigma)`
//! - deflection `delta = 5 p s l^4 / (384 E I) <= l/200`
//!
//! These are *rule-style* planning checks, **not** a class compliance
//! calculation: design pressures, corrosion margins and boundary factors are
//! project assumptions and must be replaced by the actual rule clauses.

/// Steel density (kg/m3).
pub const STEEL_DENSITY_KG_M3: f64 = 7850.0;
/// Young's modulus (MPa).
pub const E_MPA: f64 = 206_000.0;
/// Yield stress (MPa), mild ship steel.
pub const SIGMA_Y_MPA: f64 = 235.0;
/// Allowable bending stress (MPa), 0.6 * sigma_y.
pub const SIGMA_ALLOW_MPA: f64 = 0.6 * SIGMA_Y_MPA;
/// Corrosion addition (mm).
pub const CORROSION_MM: f64 = 1.5;
/// Stiffener spacing (m).
pub const STIFFENER_SPACING_M: f64 = 0.7;
/// Panel bending coefficient.
const PANEL_K: f64 = 3.5;
/// Plate buckling coefficient (simply supported panel).
const BUCKLING_C: f64 = 1.9;
/// Stiffener bending moment coefficient (fixed ends).
const STIFFENER_M: f64 = 8.0;

/// Standard plate thickness series (mm).
pub const THICKNESS_SERIES_MM: [f64; 13] = [
    6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 25.0,
];

#[derive(Clone, Copy, Debug)]
pub struct ScantlingInput {
    pub length_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
    /// Number of deck levels used by the arrangement.
    pub deck_levels: f64,
    /// Longitudinal span of the arrangement (0..1).
    pub span: f64,
    pub transverse_bulkheads: f64,
    /// Deck design pressure (kPa).
    pub deck_pressure_kpa: f64,
}

impl ScantlingInput {
    pub fn from_geometry(
        length_m: f64,
        beam_m: f64,
        depth_m: f64,
        deck_levels: f64,
        span: f64,
    ) -> Self {
        ScantlingInput {
            length_m,
            beam_m,
            depth_m,
            deck_levels,
            span,
            transverse_bulkheads: 5.0,
            deck_pressure_kpa: 12.0,
        }
    }
}

/// One checked panel.
#[derive(Clone, Copy, Debug)]
pub struct PanelCheck {
    pub t_required_mm: f64,
    pub t_offered_mm: f64,
    pub z_required_cm3: f64,
    pub z_offered_cm3: f64,
}

impl PanelCheck {
    pub fn thickness_ok(&self) -> bool {
        self.t_offered_mm + 1e-9 >= self.t_required_mm
    }
    pub fn stiffness_ok(&self) -> bool {
        self.z_offered_cm3 + 1e-9 >= self.z_required_cm3
    }
    pub fn utilization(&self) -> f64 {
        self.t_required_mm / self.t_offered_mm.max(1e-9)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScantlingReport {
    pub deck: PanelCheck,
    pub bulkhead: PanelCheck,
    pub deck_plate_kg: f64,
    pub bulkhead_plate_kg: f64,
    pub stiffener_kg: f64,
    pub girder_kg: f64,
    pub deflection_ratio: f64,
}

impl ScantlingReport {
    /// Total structural steel (tons).
    pub fn weight_tons(&self) -> f64 {
        (self.deck_plate_kg + self.bulkhead_plate_kg + self.stiffener_kg + self.girder_kg) / 1000.0
    }

    pub fn checks(&self) -> Vec<(String, bool)> {
        vec![
            (
                "Deck plate thickness (rule)".to_string(),
                self.deck.thickness_ok(),
            ),
            (
                "Deck stiffener section modulus (rule)".to_string(),
                self.deck.stiffness_ok(),
            ),
            (
                "Bulkhead plate thickness (rule)".to_string(),
                self.bulkhead.thickness_ok(),
            ),
            (
                "Bulkhead stiffener section modulus (rule)".to_string(),
                self.bulkhead.stiffness_ok(),
            ),
            (
                "Stiffener deflection <= l/200 (rule)".to_string(),
                self.deflection_ratio <= 1.0 + 1e-9,
            ),
        ]
    }

    pub fn max_utilization(&self) -> f64 {
        self.deck
            .utilization()
            .max(self.bulkhead.utilization())
            .max(self.deck.z_required_cm3 / self.deck.z_offered_cm3.max(1e-9))
            .max(self.bulkhead.z_required_cm3 / self.bulkhead.z_offered_cm3.max(1e-9))
    }
}

fn next_thickness(required_mm: f64) -> f64 {
    THICKNESS_SERIES_MM
        .iter()
        .copied()
        .find(|t| *t + 1e-9 >= required_mm)
        .unwrap_or(*THICKNESS_SERIES_MM.last().unwrap())
}

fn plate_bending_thickness_mm(pressure_kpa: f64, spacing_m: f64) -> f64 {
    let pressure_mpa = pressure_kpa / 1000.0;
    spacing_m * (pressure_mpa / (PANEL_K * SIGMA_ALLOW_MPA)).sqrt() * 1000.0
}

fn plate_buckling_thickness_mm(spacing_m: f64) -> f64 {
    let limit = BUCKLING_C * (E_MPA / SIGMA_Y_MPA).sqrt();
    spacing_m * 1000.0 / limit
}

fn stiffener_z_required_cm3(pressure_kpa: f64, spacing_m: f64, span_m: f64) -> f64 {
    let pressure_pa = pressure_kpa * 1000.0;
    let sigma_pa = SIGMA_ALLOW_MPA * 1e6;
    let z = pressure_pa * spacing_m * span_m * span_m / (STIFFENER_M * sigma_pa);
    z * 1e6 // m3 -> cm3
}

/// Offered flat-bar section modulus (cm3), attached plating ignored.
fn stiffener_z_offered_cm3(span_m: f64, thickness_mm: f64) -> f64 {
    let web_depth_m = (span_m / 10.0).clamp(0.2, 0.8);
    let t_m = thickness_mm / 1000.0;
    let z_m3 = web_depth_m * web_depth_m * t_m / 6.0;
    z_m3 * 1e6
}

fn plate_panel(pressure_kpa: f64, span_m: f64, minimum_mm: f64) -> PanelCheck {
    let s = STIFFENER_SPACING_M;
    let t_bend = plate_bending_thickness_mm(pressure_kpa, s);
    let t_buckle = plate_buckling_thickness_mm(s);
    let t_required = t_bend.max(t_buckle).max(minimum_mm) + CORROSION_MM;
    let t_offered = next_thickness(t_required);
    let z_required = stiffener_z_required_cm3(pressure_kpa, s, span_m);
    // Increase the flat-bar thickness until the offered Z is sufficient.
    let mut t_stiff = t_offered.max(8.0);
    while stiffener_z_offered_cm3(span_m, t_stiff) < z_required && t_stiff < 30.0 {
        t_stiff += 1.0;
    }
    let z_offered = stiffener_z_offered_cm3(span_m, t_stiff);
    PanelCheck {
        t_required_mm: t_required,
        t_offered_mm: t_offered,
        z_required_cm3: z_required,
        z_offered_cm3: z_offered,
    }
}

/// Evaluate the rule-style scantlings for a deck/bulkhead arrangement.
pub fn evaluate(input: &ScantlingInput) -> ScantlingReport {
    let l = input.length_m.max(1.0);
    let b = input.beam_m.max(1.0);
    let d = input.depth_m.max(1.0);
    let levels = input.deck_levels.max(1.0);
    let a_deck = 0.85 * l * b;
    let a_bhd = 0.85 * b * d;
    let s = STIFFENER_SPACING_M;

    let deck_span = (b / 4.0).clamp(1.5, 6.0);
    let deck_min = 5.0 + 0.01 * l;
    let deck = plate_panel(input.deck_pressure_kpa, deck_span, deck_min);

    let flood_pressure = 1.025 * 9.81 * (0.5 * d); // kPa
    let bhd_span = (d / 2.0).clamp(2.0, 8.0);
    let bhd_min = 6.0 + 0.01 * l;
    let bulkhead = plate_panel(flood_pressure, bhd_span, bhd_min);

    let deck_plate_kg = a_deck * levels * (deck.t_offered_mm / 1000.0) * STEEL_DENSITY_KG_M3;
    let bulkhead_plate_kg =
        input.transverse_bulkheads * a_bhd * (bulkhead.t_offered_mm / 1000.0) * STEEL_DENSITY_KG_M3;

    // Stiffener areas from the offered section moduli.
    let stiff_area = |z_cm3: f64, span_m: f64| -> f64 {
        let web = (span_m / 10.0).clamp(0.2, 0.8);
        (z_cm3 * 1e-6 / (0.85 * web)).clamp(2e-4, 5e-2)
    };
    let deck_stiff_area = stiff_area(deck.z_offered_cm3, deck_span);
    let bhd_stiff_area = stiff_area(bulkhead.z_offered_cm3, bhd_span);
    let stiffener_kg = (a_deck / s) * levels * deck_stiff_area * STEEL_DENSITY_KG_M3
        + input.transverse_bulkheads * (a_bhd / s) * bhd_stiff_area * STEEL_DENSITY_KG_M3;

    // Two main girders per deck level (planning section 0.02 m2).
    let girder_kg = 2.0 * l * 0.02 * STEEL_DENSITY_KG_M3 * levels;
    let span_kg = a_deck * input.span.clamp(0.0, 1.0) * 5.0 * levels;

    // Deflection of the deck stiffener under design pressure.
    let i_m4 = {
        let web = (deck_span / 10.0).clamp(0.2, 0.8);
        let t = (deck.t_offered_mm / 1000.0).max(0.006);
        web.powi(3) * t / 12.0
    };
    let p_pa = input.deck_pressure_kpa * 1000.0;
    let e_pa = E_MPA * 1e6;
    let delta = 5.0 * p_pa * s * deck_span.powi(4) / (384.0 * e_pa * i_m4.max(1e-9));
    let deflection_limit = deck_span / 200.0;
    let deflection_ratio = delta / deflection_limit.max(1e-9);

    ScantlingReport {
        deck,
        bulkhead,
        deck_plate_kg,
        bulkhead_plate_kg,
        stiffener_kg: stiffener_kg + span_kg,
        girder_kg,
        deflection_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo() -> ScantlingInput {
        ScantlingInput::from_geometry(180.0, 24.0, 12.0, 3.0, 0.3)
    }

    #[test]
    fn rule_scantlings_are_physically_reasonable() {
        let r = evaluate(&demo());
        assert!(r.deck.t_offered_mm >= 6.0 && r.deck.t_offered_mm <= 25.0);
        assert!(r.bulkhead.t_offered_mm >= 6.0 && r.bulkhead.t_offered_mm <= 25.0);
        assert!(r.weight_tons() > 500.0 && r.weight_tons() < 20_000.0);
        assert!(r.max_utilization() <= 1.0 + 1e-9);
        for (_, ok) in r.checks() {
            assert!(ok);
        }
    }

    #[test]
    fn thicker_plating_for_greater_depth() {
        let shallow = evaluate(&demo());
        let mut deep = demo();
        deep.depth_m = 20.0;
        let deep_r = evaluate(&deep);
        assert!(deep_r.bulkhead.t_offered_mm >= shallow.bulkhead.t_offered_mm);
        assert!(deep_r.weight_tons() > shallow.weight_tons());
    }
}
