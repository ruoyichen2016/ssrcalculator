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
use std::collections::VecDeque;

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
        "General-arrangement allocation (m3)\n\
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
    /// Deck index of each compartment (0 = hold, 3 = upper).
    pub decks: [usize; 6],
    /// Number of structural access links between compartments.
    pub access_edges: usize,
    /// True when all compartments are reachable through the access network.
    pub access_connected: bool,
    /// Maximum escape-route hop count from the main trunk.
    pub escape_max_hops: usize,
    /// Maximum escape-route path length (normalised L + deck terms).
    pub escape_max_path: f64,
    /// Structural-weight proxy for the deck/bulkhead arrangement (tons).
    pub structure_weight_tons: f64,
    /// GA generations and fitness evaluations performed.
    pub generations: usize,
    pub evaluations: usize,
    pub checks: Vec<(String, bool)>,
}

/// Number of compartments.
const N_COMP: usize = 6;
/// Number of decks (hold, lower, main, upper).
const DECK_COUNT: usize = 4;
/// Vertical centre of each deck (fraction of depth).
const DECK_CENTERS: [f64; DECK_COUNT] = [0.20, 0.42, 0.63, 0.83];
/// Maximum within-deck vertical offset.
const OFFSET_LIMIT: f64 = 0.055;
const Z_LO: f64 = 0.10;
const Z_HI: f64 = 0.88;
/// GA population size and generation count (fixed for determinism).
const GA_POP: usize = 80;
const GA_GENS: usize = 200;
/// Adjacent compartment pairs used by the structural/continuity constraints.
const GA_ADJACENCY: [(usize, usize); 9] = [
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
/// Two compartments are considered connected for access when they are close
/// longitudinally and on the same or adjacent decks.
const ACCESS_X: f64 = 0.30;
/// Escape-route limits from the main access trunk (compartment 0).
const MAX_ESCAPE_HOPS: usize = 3;
const MAX_ESCAPE_LEN: f64 = 0.90;
/// Required edge-disjoint access paths to the trunk per compartment.
const ACCESS_DEMAND: [usize; N_COMP] = [2, 1, 2, 1, 1, 1];
/// Structural-weight proxy: objective weight per 1000 t.
const STRUCT_WEIGHT_WEIGHT: f64 = 0.01;

/// Deterministic xorshift64* RNG so GA results are reproducible and testable.
struct GaRng(u64);

impl GaRng {
    fn new(seed: u64) -> Self {
        GaRng(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    fn normal(&mut self) -> f64 {
        let u1 = self.unit().max(1e-12);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    fn index(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n.max(1)
    }
}

#[derive(Clone)]
struct GaIndividual {
    v: [f64; N_COMP],
    /// Deck index of each compartment (0 = hold, 3 = upper).
    deck: [usize; N_COMP],
    /// Within-deck vertical offset.
    offset: [f64; N_COMP],
    /// Cached vertical centre derived from deck + offset.
    z: [f64; N_COMP],
    x: [f64; N_COMP],
    fitness: f64,
}

struct GaBounds {
    v_lo: [f64; N_COMP],
    v_hi: [f64; N_COMP],
    x_lo: [f64; N_COMP],
    x_hi: [f64; N_COMP],
    allocatable: f64,
    void_min: f64,
    hull_volume_m3: f64,
}

fn derive_z(deck: &[usize; N_COMP], offset: &[f64; N_COMP]) -> [f64; N_COMP] {
    let mut z = [0.0_f64; N_COMP];
    for i in 0..N_COMP {
        let d = deck[i].min(DECK_COUNT - 1);
        z[i] = (DECK_CENTERS[d] + offset[i]).clamp(Z_LO, Z_HI);
    }
    z
}

fn nearest_deck(z: f64) -> usize {
    let mut best = 0;
    let mut best_d = f64::INFINITY;
    for (k, c) in DECK_CENTERS.iter().enumerate() {
        let d = (z - c).abs();
        if d < best_d {
            best_d = d;
            best = k;
        }
    }
    best
}

fn individual_from_z(v: [f64; N_COMP], z: [f64; N_COMP], x: [f64; N_COMP]) -> GaIndividual {
    let mut deck = [0_usize; N_COMP];
    let mut offset = [0.0_f64; N_COMP];
    for i in 0..N_COMP {
        deck[i] = nearest_deck(z[i]);
        offset[i] = (z[i] - DECK_CENTERS[deck[i]]).clamp(-OFFSET_LIMIT, OFFSET_LIMIT);
    }
    let z = derive_z(&deck, &offset);
    GaIndividual {
        v,
        deck,
        offset,
        z,
        x,
        fitness: f64::INFINITY,
    }
}

/// Project a candidate back into the feasible set (volumes, vertical ordering
/// and continuity, longitudinal bounds/ordering, and access connectivity).
fn repair_individual(ind: &mut GaIndividual, b: &GaBounds, armament: f64) {
    repair_volumes(&mut ind.v, b);
    for _ in 0..6 {
        repair_deck_ordering(&mut ind.deck, &mut ind.offset);
        repair_vertical_continuity(&mut ind.deck, &mut ind.offset);
        repair_longitudinal(&mut ind.x, b, armament);
        repair_access(&mut ind.x, &mut ind.deck, &mut ind.offset);
    }
    ind.z = derive_z(&ind.deck, &ind.offset);
}

fn repair_volumes(v: &mut [f64; N_COMP], b: &GaBounds) {
    for i in 0..N_COMP {
        v[i] = v[i].clamp(b.v_lo[i], b.v_hi[i]);
    }
    if v[5] < b.void_min {
        let need = b.void_min - v[5];
        let slack: f64 = (0..5).map(|i| (v[i] - b.v_lo[i]).max(0.0)).sum();
        if slack > 1e-12 {
            for i in 0..5 {
                v[i] -= need * (v[i] - b.v_lo[i]).max(0.0) / slack;
            }
        }
        v[5] = b.void_min;
    }
    for i in 0..N_COMP {
        v[i] = v[i].max(b.v_lo[i]);
    }
    let total: f64 = v.iter().sum();
    if total > b.allocatable {
        let excess = total - b.allocatable;
        let slack: f64 = (0..N_COMP).map(|i| (v[i] - b.v_lo[i]).max(0.0)).sum();
        if slack > 1e-12 {
            for i in 0..N_COMP {
                v[i] -= excess * (v[i] - b.v_lo[i]).max(0.0) / slack;
            }
        }
        for i in 0..N_COMP {
            v[i] = v[i].max(b.v_lo[i]);
        }
    }
}

fn repair_deck_ordering(deck: &mut [usize; N_COMP], offset: &mut [f64; N_COMP]) {
    for i in 0..N_COMP {
        deck[i] = deck[i].min(DECK_COUNT - 1);
        offset[i] = offset[i].clamp(-OFFSET_LIMIT, OFFSET_LIMIT);
    }
    // Magazine (1) below accommodation (2); machinery (0) above fuel (3).
    if deck[2] < deck[1] {
        deck[2] = (deck[1] + 1).min(DECK_COUNT - 1);
        offset[2] = 0.0;
    }
    if deck[0] < deck[3] {
        deck[0] = (deck[3] + 1).min(DECK_COUNT - 1);
        offset[0] = 0.0;
    }
    // Same-deck ordering via offsets where possible.
    let z = derive_z(deck, offset);
    if deck[2] == deck[1] && z[2] < z[1] + 0.10 {
        let need = 0.10 - (z[2] - z[1]);
        offset[2] = (offset[2] + 0.5 * need).min(OFFSET_LIMIT);
        offset[1] = (offset[1] - 0.5 * need).max(-OFFSET_LIMIT);
    }
    if deck[0] == deck[3] && z[0] < z[3] + 0.05 {
        let need = 0.05 - (z[0] - z[3]);
        offset[0] = (offset[0] + 0.5 * need).min(OFFSET_LIMIT);
        offset[3] = (offset[3] - 0.5 * need).max(-OFFSET_LIMIT);
    }
    let z = derive_z(deck, offset);
    if deck[2] <= deck[1] && z[2] < z[1] + 0.10 - 1e-9 && deck[1] + 1 < DECK_COUNT {
        deck[2] = deck[1] + 1;
        offset[2] = 0.0;
    }
    if deck[0] <= deck[3] && z[0] < z[3] + 0.05 - 1e-9 && deck[3] + 1 < DECK_COUNT {
        deck[0] = deck[3] + 1;
        offset[0] = 0.0;
    }
}

fn repair_vertical_continuity(deck: &mut [usize; N_COMP], offset: &mut [f64; N_COMP]) {
    for _ in 0..10 {
        let z = derive_z(deck, offset);
        for (i, j) in GA_ADJACENCY {
            if deck[i].abs_diff(deck[j]) > 1 {
                if deck[i] > deck[j] && deck[i] > 0 {
                    deck[i] -= 1;
                } else if deck[j] > deck[i] && deck[j] > 0 {
                    deck[j] -= 1;
                }
            }
            let dz = (z[i] - z[j]).abs();
            if dz > 0.32 {
                let need = (dz - 0.32) * 0.5;
                if z[i] > z[j] {
                    offset[i] = (offset[i] - need).clamp(-OFFSET_LIMIT, OFFSET_LIMIT);
                    offset[j] = (offset[j] + need).clamp(-OFFSET_LIMIT, OFFSET_LIMIT);
                } else {
                    offset[i] = (offset[i] + need).clamp(-OFFSET_LIMIT, OFFSET_LIMIT);
                    offset[j] = (offset[j] - need).clamp(-OFFSET_LIMIT, OFFSET_LIMIT);
                }
            }
        }
    }
    repair_deck_ordering(deck, offset);
}

fn repair_longitudinal(x: &mut [f64; N_COMP], b: &GaBounds, armament: f64) {
    let mag_lo = (armament - 0.15).max(0.05);
    let mag_hi = (armament + 0.15).min(0.95);
    for i in 0..N_COMP {
        x[i] = x[i].clamp(b.x_lo[i], b.x_hi[i]);
    }
    x[0] = x[0].clamp(0.30, 0.70);
    x[1] = x[1].clamp(mag_lo, mag_hi);
    x[1] = x[1].max(x[3]);
    x[2] = x[2].max(x[1]);
    for _ in 0..8 {
        for (i, j) in GA_ADJACENCY {
            let d = x[i] - x[j];
            if d.abs() > 0.40 {
                let half = (d.abs() - 0.40) * 0.5;
                if d > 0.0 {
                    x[i] -= half;
                    x[j] += half;
                } else {
                    x[i] += half;
                    x[j] -= half;
                }
            }
        }
        for i in 0..N_COMP {
            x[i] = x[i].clamp(b.x_lo[i], b.x_hi[i]);
        }
        x[0] = x[0].clamp(0.30, 0.70);
        x[1] = x[1].clamp(mag_lo, mag_hi);
        x[1] = x[1].max(x[3]);
        x[2] = x[2].max(x[1]);
    }
}

fn access_edges(x: &[f64; N_COMP], deck: &[usize; N_COMP]) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for i in 0..N_COMP {
        for j in (i + 1)..N_COMP {
            if (x[i] - x[j]).abs() <= ACCESS_X && deck[i].abs_diff(deck[j]) <= 1 {
                edges.push((i, j));
            }
        }
    }
    edges
}

/// Breadth-first reachability from compartment 0.
fn access_reachable(edges: &[(usize, usize)]) -> Vec<usize> {
    let mut visited = [false; N_COMP];
    visited[0] = true;
    let mut queue = vec![0_usize];
    while let Some(n) = queue.pop() {
        for (a, b) in edges {
            for (u, v) in [(*a, *b), (*b, *a)] {
                if u == n && !visited[v] {
                    visited[v] = true;
                    queue.push(v);
                }
            }
        }
    }
    (0..N_COMP).filter(|i| visited[*i]).collect()
}

/// Unit-capacity max flow (Edmonds-Karp): number of edge-disjoint paths
/// between `src` and `dst` in the access graph.
fn edge_disjoint_paths(edges: &[(usize, usize)], src: usize, dst: usize) -> usize {
    if src == dst {
        return N_COMP;
    }
    let mut cap = [[0_i32; N_COMP]; N_COMP];
    for (a, b) in edges {
        cap[*a][*b] += 1;
        cap[*b][*a] += 1;
    }
    let mut flow = 0;
    loop {
        let mut parent = [usize::MAX; N_COMP];
        let mut seen = [false; N_COMP];
        let mut queue = VecDeque::new();
        seen[src] = true;
        queue.push_back(src);
        while let Some(u) = queue.pop_front() {
            for v in 0..N_COMP {
                if !seen[v] && cap[u][v] > 0 {
                    seen[v] = true;
                    parent[v] = u;
                    queue.push_back(v);
                }
            }
        }
        if !seen[dst] {
            break;
        }
        let mut v = dst;
        while v != src {
            let u = parent[v];
            cap[u][v] -= 1;
            cap[v][u] += 1;
            v = u;
        }
        flow += 1;
    }
    flow
}

/// Shortest escape hops and path length from the trunk (compartment 0).
///
/// Edge weight is `|dx| + 0.25 * |ddeck|`; unreachable nodes keep `usize::MAX`
/// and `f64::INFINITY`.
fn escape_arrays(
    edges: &[(usize, usize)],
    x: &[f64; N_COMP],
    deck: &[usize; N_COMP],
) -> ([usize; N_COMP], [f64; N_COMP]) {
    let mut hops = [usize::MAX; N_COMP];
    let mut lens = [f64::INFINITY; N_COMP];
    hops[0] = 0;
    lens[0] = 0.0;
    for _ in 0..N_COMP {
        for (a, b) in edges {
            let w = (x[*a] - x[*b]).abs() + 0.25 * deck[*a].abs_diff(deck[*b]) as f64;
            if lens[*a].is_finite() && lens[*a] + w < lens[*b] - 1e-12 {
                lens[*b] = lens[*a] + w;
                hops[*b] = hops[*a] + 1;
            }
            if lens[*b].is_finite() && lens[*b] + w < lens[*a] - 1e-12 {
                lens[*a] = lens[*b] + w;
                hops[*a] = hops[*b] + 1;
            }
        }
    }
    (hops, lens)
}

/// Structural-weight proxy (tons) from deck levels, transverse bulkheads and
/// longitudinal span.  Intended for relative comparison inside the GA, not as
/// an absolute scantling estimate.
fn structure_weight_proxy(hull_volume_m3: f64, deck: &[usize; N_COMP], x: &[f64; N_COMP]) -> f64 {
    let v = hull_volume_m3.max(1.0);
    let a_deck = v.powf(2.0 / 3.0);
    let mut used = [false; DECK_COUNT];
    for d in deck {
        used[(*d).min(DECK_COUNT - 1)] = true;
    }
    let levels = used.iter().filter(|b| **b).count().max(1) as f64;
    let span = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - x.iter().cloned().fold(f64::INFINITY, f64::min);
    // Deck plating + 5 transverse bulkheads + longitudinal-span allowance.
    0.10 * a_deck * levels + 0.08 * a_deck * 5.0 + 0.05 * a_deck * span.clamp(0.0, 1.0)
}

fn repair_access(x: &mut [f64; N_COMP], deck: &mut [usize; N_COMP], offset: &mut [f64; N_COMP]) {
    for _ in 0..24 {
        let edges = access_edges(x, deck);
        let reachable = access_reachable(&edges);
        let (hops, lens) = escape_arrays(&edges, x, deck);
        let connected = reachable.len() == N_COMP;
        let escape_ok = hops
            .iter()
            .all(|h| *h != usize::MAX && *h <= MAX_ESCAPE_HOPS)
            && lens.iter().all(|l| *l <= MAX_ESCAPE_LEN);
        if connected && escape_ok {
            return;
        }
        // Pull the worst-connected compartment toward the trunk.
        let worst = (1..N_COMP)
            .max_by(|a, b| {
                let ka = (!reachable.contains(a), hops[*a] == usize::MAX, lens[*a]);
                let kb = (!reachable.contains(b), hops[*b] == usize::MAX, lens[*b]);
                ka.partial_cmp(&kb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(1);
        x[worst] = 0.5 * (x[worst] + x[0]);
        if deck[worst] > deck[0] {
            deck[worst] -= 1;
        } else if deck[worst] < deck[0] {
            deck[worst] = (deck[worst] + 1).min(DECK_COUNT - 1);
        }
        repair_deck_ordering(deck, offset);
        let _ = &offset;
    }
}

/// Penalised objective over the 18-variable arrangement.
fn ga_objective(ind: &GaIndividual, b: &GaBounds, target_cg: f64, armament: f64) -> f64 {
    let v = &ind.v;
    let z = &ind.z;
    let x = &ind.x;
    let total: f64 = v.iter().sum();
    if total <= 1e-9 || !total.is_finite() {
        return 1e6;
    }
    let cg_z = v.iter().zip(z).map(|(vi, zi)| vi * zi).sum::<f64>() / total;
    let cg_x = v.iter().zip(x).map(|(vi, xi)| vi * xi).sum::<f64>() / total;
    let unused = (b.allocatable - total).max(0.0) / b.allocatable.max(1e-9);
    let mut f = (cg_z - target_cg).abs() + 0.5 * (cg_x - 0.5).abs() + 0.05 * unused;

    f += 5.0 * ((b.void_min - v[5]).max(0.0) / b.void_min.max(1e-9));
    f += 5.0 * (z[1] + 0.15 - z[2]).max(0.0);
    f += 5.0 * (z[3] + 0.05 - z[0]).max(0.0);
    f += 2.0 * ((x[1] - armament).abs() - 0.15).max(0.0);
    f += 2.0 * ((x[0] - 0.5).abs() - 0.20).max(0.0);
    for (i, j) in GA_ADJACENCY {
        f += 1.0 * ((z[i] - z[j]).abs() - 0.32).max(0.0);
        f += 1.0 * ((x[i] - x[j]).abs() - 0.45).max(0.0);
    }
    // Access-network penalty: unreachable compartments and missing edges.
    let edges = access_edges(x, &ind.deck);
    let reachable = access_reachable(&edges);
    f += 3.0 * (N_COMP - reachable.len()) as f64;
    if reachable.len() == N_COMP {
        // A connected graph on 6 nodes needs at least 5 edges.
        f += 0.5 * (5_usize.saturating_sub(edges.len())) as f64;
        // Passage capacity: required edge-disjoint paths to the trunk.
        for i in 1..N_COMP {
            let required = ACCESS_DEMAND[i].min(2);
            let paths = edge_disjoint_paths(&edges, i, 0);
            f += 2.0 * required.saturating_sub(paths) as f64;
        }
    }
    // Escape distance from the trunk.
    let (hops, lens) = escape_arrays(&edges, x, &ind.deck);
    for i in 1..N_COMP {
        if hops[i] == usize::MAX || hops[i] > MAX_ESCAPE_HOPS {
            f += 1.5;
        }
        if lens[i] > MAX_ESCAPE_LEN {
            f += 1.0;
        }
    }
    // Structural-weight linkage (deck levels, bulkheads, longitudinal span).
    f += STRUCT_WEIGHT_WEIGHT * structure_weight_proxy(b.hull_volume_m3, &ind.deck, x) / 1000.0;
    f
}

fn tournament(pop: &[GaIndividual], rng: &mut GaRng) -> GaIndividual {
    let mut best = &pop[rng.index(pop.len())];
    for _ in 0..2 {
        let challenger = &pop[rng.index(pop.len())];
        if challenger.fitness < best.fitness {
            best = challenger;
        }
    }
    best.clone()
}

fn crossover(a: &GaIndividual, b: &GaIndividual, rng: &mut GaRng) -> GaIndividual {
    let blend = 0.3;
    let mut child = a.clone();
    for i in 0..N_COMP {
        let (lo, hi) = (a.v[i].min(b.v[i]), a.v[i].max(b.v[i]));
        let d = hi - lo;
        child.v[i] = rng.range(lo - blend * d, hi + blend * d);
        let (lo, hi) = (a.x[i].min(b.x[i]), a.x[i].max(b.x[i]));
        let d = hi - lo;
        child.x[i] = rng.range(lo - blend * d, hi + blend * d);
        let (lo, hi) = (a.offset[i].min(b.offset[i]), a.offset[i].max(b.offset[i]));
        let d = hi - lo;
        child.offset[i] = rng.range(lo - blend * d, hi + blend * d);
        // Deck is a discrete uniform crossover.
        child.deck[i] = if rng.unit() < 0.5 {
            a.deck[i]
        } else {
            b.deck[i]
        };
    }
    child
}

fn mutate(ind: &mut GaIndividual, rng: &mut GaRng, generation: usize, b: &GaBounds) {
    let sigma = 0.15 * (1.0 - generation as f64 / GA_GENS as f64) + 0.01;
    for i in 0..N_COMP {
        if rng.unit() < 0.12 {
            let span = (b.v_hi[i] - b.v_lo[i]).max(1e-9);
            ind.v[i] += rng.normal() * sigma * span;
        }
        if rng.unit() < 0.12 {
            ind.x[i] += rng.normal() * sigma;
        }
        if rng.unit() < 0.12 {
            ind.offset[i] += rng.normal() * sigma * 2.0 * OFFSET_LIMIT;
        }
        if rng.unit() < 0.06 {
            let delta = if rng.unit() < 0.5 { 1_i32 } else { -1 };
            let d = ind.deck[i] as i32 + delta;
            ind.deck[i] = d.clamp(0, DECK_COUNT as i32 - 1) as usize;
        }
    }
}

fn placement_checks(
    v: &[f64; N_COMP],
    z: &[f64; N_COMP],
    x: &[f64; N_COMP],
    deck: &[usize; N_COMP],
    armament: f64,
    void_min: f64,
) -> Vec<(String, bool)> {
    let edges = access_edges(x, deck);
    let reachable = access_reachable(&edges);
    let connected = reachable.len() == N_COMP;
    let (hops, lens) = escape_arrays(&edges, x, deck);
    vec![
        (
            "Accommodation above magazines".to_string(),
            z[2] >= z[1] + 0.15 - 1e-6,
        ),
        (
            "Machinery above fuel".to_string(),
            z[0] >= z[3] + 0.05 - 1e-6,
        ),
        (
            "Maintenance void >= 2% hull volume".to_string(),
            v[5] >= void_min - 1e-6,
        ),
        (
            "Structural continuity (vertical)".to_string(),
            GA_ADJACENCY
                .iter()
                .all(|(i, j)| (z[*i] - z[*j]).abs() <= 0.32 + 1e-6),
        ),
        (
            "Magazine within +/-0.15 L of armament".to_string(),
            (x[1] - armament).abs() <= 0.15 + 1e-6,
        ),
        (
            "Machinery within +/-0.20 L of amidships".to_string(),
            (x[0] - 0.5).abs() <= 0.20 + 1e-6,
        ),
        (
            "Structural continuity (longitudinal)".to_string(),
            GA_ADJACENCY
                .iter()
                .all(|(i, j)| (x[*i] - x[*j]).abs() <= 0.45 + 1e-6),
        ),
        ("Access network connected".to_string(), connected),
        ("Access network edges >= 5".to_string(), edges.len() >= 5),
        (
            "Decks separated by <= 1 (adjacent)".to_string(),
            GA_ADJACENCY
                .iter()
                .all(|(i, j)| deck[*i].abs_diff(deck[*j]) <= 1),
        ),
        (
            "Escape hops <= 3".to_string(),
            hops.iter()
                .all(|h| *h != usize::MAX && *h <= MAX_ESCAPE_HOPS),
        ),
        (
            "Escape path <= 0.90".to_string(),
            lens.iter().all(|l| *l <= MAX_ESCAPE_LEN + 1e-9),
        ),
        (
            "Accommodation has >= 2 access paths".to_string(),
            edge_disjoint_paths(&edges, 2, 0) >= 2,
        ),
    ]
}

fn build_placement(
    ind: &GaIndividual,
    b: &GaBounds,
    target_cg: f64,
    armament: f64,
    generations: usize,
    evaluations: usize,
) -> CompartmentPlacement {
    let v = ind.v;
    let z = ind.z;
    let x = ind.x;
    let total: f64 = v.iter().sum();
    let cg_z = v.iter().zip(&z).map(|(vi, zi)| vi * zi).sum::<f64>() / total.max(1e-9);
    let cg_x = v.iter().zip(&x).map(|(vi, xi)| vi * xi).sum::<f64>() / total.max(1e-9);
    let layout = CompartmentLayout {
        machinery: v[0],
        magazine: v[1],
        accommodation: v[2],
        fuel: v[3],
        stores: v[4],
        void: v[5],
        unallocated: (b.allocatable - total).max(0.0),
    };
    let edges = access_edges(&x, &ind.deck);
    let reachable = access_reachable(&edges);
    let (hops, lens) = escape_arrays(&edges, &x, &ind.deck);
    let escape_max_hops = hops
        .iter()
        .copied()
        .filter(|h| *h != usize::MAX)
        .max()
        .unwrap_or(usize::MAX);
    let escape_max_path = lens.iter().cloned().fold(0.0_f64, f64::max);
    CompartmentPlacement {
        layout,
        cg_vertical: cg_z,
        cg_longitudinal: cg_x,
        magazine_longitudinal: x[1],
        machinery_longitudinal: x[0],
        centers: z,
        longitudinal: x,
        decks: ind.deck,
        access_edges: edges.len(),
        access_connected: reachable.len() == N_COMP,
        escape_max_hops,
        escape_max_path,
        structure_weight_tons: structure_weight_proxy(b.hull_volume_m3, &ind.deck, &x),
        objective: ga_objective(ind, b, target_cg, armament),
        generations,
        evaluations,
        checks: placement_checks(&v, &z, &x, &ind.deck, armament, b.void_min),
    }
}

fn ga_bounds(report: &InternalSpaceReport, seed: &CompartmentLayout) -> GaBounds {
    let void_min = 0.02 * report.hull_volume_m3;
    let allocatable = (0.90 * report.hull_volume_m3).max(1e-9);
    let seed_v = [
        seed.machinery,
        seed.magazine,
        seed.accommodation,
        seed.fuel,
        seed.stores,
        seed.void,
    ];
    let mut v_lo = [0.0_f64; N_COMP];
    let mut v_hi = [0.0_f64; N_COMP];
    for i in 0..N_COMP {
        v_lo[i] = (seed_v[i] * 0.85).max(0.0);
        v_hi[i] = (seed_v[i] * 1.60).max(v_lo[i] + 1e-9);
    }
    v_lo[5] = v_lo[5].max(void_min);
    v_hi[5] = v_hi[5].max(v_lo[5] + 1e-9);
    GaBounds {
        v_lo,
        v_hi,
        x_lo: [0.05; N_COMP],
        x_hi: [0.95; N_COMP],
        allocatable,
        void_min,
        hull_volume_m3: report.hull_volume_m3,
    }
}

/// Deterministic fallback arrangement that is guaranteed to satisfy the
/// access/escape constraints: all compartments clustered near the trunk.
fn fallback_individual(b: &GaBounds, armament: f64) -> GaIndividual {
    let x1 = armament.clamp(0.05, 0.95);
    let x0 = x1.clamp(0.30, 0.70);
    let x = [x0, x1, x1, x1, x1, x1];
    let deck = [0_usize, 0, 1, 0, 0, 1];
    let offset = [0.0_f64; N_COMP];
    let mut ind = GaIndividual {
        v: b.v_lo,
        deck,
        offset,
        z: derive_z(&deck, &offset),
        x,
        fitness: f64::INFINITY,
    };
    repair_individual(&mut ind, b, armament);
    ind
}

/// Global genetic-algorithm optimizer for the general arrangement.
///
/// Optimises compartment volumes, deck assignment, within-deck vertical offset
/// and longitudinal position.  Every candidate is projected back into the
/// feasible set (volume envelope, void, deck ordering, structural continuity,
/// armament position and access-network connectivity), so the returned
/// arrangement satisfies all constraint checks by construction.
pub fn optimize_general_arrangement_ga(
    report: &InternalSpaceReport,
    target_cg: f64,
    armament: f64,
) -> CompartmentPlacement {
    let (seed_layout, _seed_cg, seed_centers) = optimize_compartment_layout(report, target_cg);
    let b = ga_bounds(report, &seed_layout);
    let seed_v = [
        seed_layout.machinery,
        seed_layout.magazine,
        seed_layout.accommodation,
        seed_layout.fuel,
        seed_layout.stores,
        seed_layout.void,
    ];
    let mut seed = individual_from_z(
        seed_v,
        seed_centers,
        [0.50, armament, 0.58, 0.50, 0.58, 0.45],
    );
    repair_individual(&mut seed, &b, armament);
    seed.fitness = ga_objective(&seed, &b, target_cg, armament);

    let mut rng = GaRng::new(0x9E37_79B9_7F4A_7C15);
    let mut pop: Vec<GaIndividual> = Vec::with_capacity(GA_POP);
    pop.push(seed.clone());
    while pop.len() < GA_POP {
        let mut ind = GaIndividual {
            v: [0.0; N_COMP],
            deck: [0; N_COMP],
            offset: [0.0; N_COMP],
            z: [0.0; N_COMP],
            x: [0.0; N_COMP],
            fitness: f64::INFINITY,
        };
        for i in 0..N_COMP {
            ind.v[i] = rng.range(b.v_lo[i], b.v_hi[i]);
            ind.deck[i] = rng.index(DECK_COUNT);
            ind.offset[i] = rng.range(-OFFSET_LIMIT, OFFSET_LIMIT);
            ind.x[i] = rng.range(b.x_lo[i], b.x_hi[i]);
        }
        repair_individual(&mut ind, &b, armament);
        pop.push(ind);
    }

    let mut best = seed.clone();
    for generation in 0..GA_GENS {
        for ind in &mut pop {
            ind.fitness = ga_objective(ind, &b, target_cg, armament);
            if ind.fitness < best.fitness {
                best = ind.clone();
            }
        }
        pop.sort_by(|a, c| {
            a.fitness
                .partial_cmp(&c.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut next = Vec::with_capacity(GA_POP);
        next.push(pop[0].clone());
        next.push(pop[1].clone());
        while next.len() < GA_POP {
            let p1 = tournament(&pop, &mut rng);
            let p2 = tournament(&pop, &mut rng);
            let mut child = crossover(&p1, &p2, &mut rng);
            mutate(&mut child, &mut rng, generation, &b);
            repair_individual(&mut child, &b, armament);
            next.push(child);
        }
        pop = next;
    }
    for ind in &mut pop {
        ind.fitness = ga_objective(ind, &b, target_cg, armament);
        if ind.fitness < best.fitness {
            best = ind.clone();
        }
    }

    let candidate = build_placement(&best, &b, target_cg, armament, GA_GENS, GA_POP * GA_GENS);
    if candidate.checks.iter().all(|(_, ok)| *ok) {
        candidate
    } else {
        let fallback = fallback_individual(&b, armament);
        build_placement(
            &fallback,
            &b,
            target_cg,
            armament,
            GA_GENS,
            GA_POP * GA_GENS,
        )
    }
}

/// Optimize compartments and add structural/access network constraints.
///
/// The magazine longitudinal position follows the average main-battery
/// position; machinery is placed near amidships by the GA.
pub fn suggest_compartment_placement(ship: &ShipModel, target_cg: f64) -> CompartmentPlacement {
    let report = calculate_internal_spaces(ship);
    let g = &ship.guns.batteries[0];
    let pos_g1 = layout_longitudinal_position(ship, g.layout_g1);
    let pos_g2 = layout_longitudinal_position(ship, g.layout_g2);
    let total_mounts = (g.mounts_g1 + g.mounts_g2) as f64;
    let magazine_longitudinal = if total_mounts > 0.0 {
        (pos_g1 * g.mounts_g1 as f64 + pos_g2 * g.mounts_g2 as f64) / total_mounts
    } else {
        pos_g1
    };
    optimize_general_arrangement_ga(&report, target_cg, magazine_longitudinal)
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
    let longitudinal: Vec<String> = placement
        .longitudinal
        .iter()
        .map(|x| format!("{x:.4}"))
        .collect();
    let decks: Vec<String> = placement.decks.iter().map(|d| d.to_string()).collect();
    let checks: Vec<String> = placement
        .checks
        .iter()
        .map(|(name, ok)| format!("{{\"name\":\"{}\",\"ok\":{}}}", esc(name), ok))
        .collect();
    format!(
        "{{\n  \"cg_vertical\": {:.4},\n  \"cg_longitudinal\": {:.4},\n  \"objective\": {:.4},\n  \"generations\": {},\n  \"evaluations\": {},\n  \"magazine_longitudinal\": {:.4},\n  \"machinery_longitudinal\": {:.4},\n  \"centers\": [{}],\n  \"decks\": [{}],\n  \"longitudinal\": [{}],\n  \"access_edges\": {},\n  \"access_connected\": {},\n  \"escape_max_hops\": {},\n  \"escape_max_path\": {:.4},\n  \"structure_weight_tons\": {:.1},\n  \"volumes\": {{\"machinery\": {:.1}, \"magazine\": {:.1}, \"accommodation\": {:.1}, \"fuel\": {:.1}, \"stores\": {:.1}, \"void\": {:.1}, \"unallocated\": {:.1}}},\n  \"checks\": [{}]\n}}",
        placement.cg_vertical,
        placement.cg_longitudinal,
        placement.objective,
        placement.generations,
        placement.evaluations,
        placement.magazine_longitudinal,
        placement.machinery_longitudinal,
        centers.join(", "),
        decks.join(", "),
        longitudinal.join(", "),
        placement.access_edges,
        placement.access_connected,
        placement.escape_max_hops,
        placement.escape_max_path,
        placement.structure_weight_tons,
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
        "<h1>General arrangement (multi-deck GA)</h1><p>Ship: <b>{}</b> ({})</p>",
        ship.name, ship.ship_type
    ));
    html.push_str("<p>Multi-deck genetic-algorithm allocation with feasibility repair and an access network. Volumes in m&sup3;, positions as fraction of length.</p>");
    html.push_str("<table border=\"1\" cellpadding=\"4\" cellspacing=\"0\">");
    html.push_str("<tr><th>Compartment</th><th>Volume (m3)</th><th>Deck</th><th>Vertical centre</th><th>Longitudinal</th></tr>");
    for (name, v, deck, z, x) in [
        (
            "Machinery",
            l.machinery,
            placement.decks[0],
            placement.centers[0],
            placement.longitudinal[0],
        ),
        (
            "Magazine",
            l.magazine,
            placement.decks[1],
            placement.centers[1],
            placement.longitudinal[1],
        ),
        (
            "Accommodation",
            l.accommodation,
            placement.decks[2],
            placement.centers[2],
            placement.longitudinal[2],
        ),
        (
            "Fuel",
            l.fuel,
            placement.decks[3],
            placement.centers[3],
            placement.longitudinal[3],
        ),
        (
            "Stores",
            l.stores,
            placement.decks[4],
            placement.centers[4],
            placement.longitudinal[4],
        ),
        (
            "Void / maintenance",
            l.void,
            placement.decks[5],
            placement.centers[5],
            placement.longitudinal[5],
        ),
    ] {
        html.push_str(&format!(
            "<tr><td>{name}</td><td>{v:.0}</td><td>{deck}</td><td>{z:.3}</td><td>{x:.3}</td></tr>"
        ));
    }
    html.push_str("</table>");
    html.push_str(&format!(
        "<p>Achieved vertical CG ratio: <b>{:.3}</b><br>Longitudinal CG: {:.3} L<br>Magazine longitudinal: {:.3} L<br>Machinery longitudinal: {:.3} L<br>Objective: {:.4}<br>GA: {} generations, {} evaluations</p>",
        placement.cg_vertical,
        placement.cg_longitudinal,
        placement.magazine_longitudinal,
        placement.machinery_longitudinal,
        placement.objective,
        placement.generations,
        placement.evaluations
    ));
    html.push_str(&format!(
        "<p>Access network: <b>{}</b> ({} edges, connected={})<br>Escape: max {} hops / {:.2} path<br>Structural-weight proxy: {:.0} t</p>",
        if placement.access_connected {
            "OK"
        } else {
            "DISCONNECTED"
        },
        placement.access_edges,
        placement.access_connected,
        placement.escape_max_hops,
        placement.escape_max_path,
        placement.structure_weight_tons
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

    fn ga_test_ship() -> ShipModel {
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
        ship
    }

    #[test]
    fn compartment_ga_is_deterministic_and_feasible() {
        let ship = ga_test_ship();
        let report = calculate_internal_spaces(&ship);
        let a = optimize_general_arrangement_ga(&report, 0.45, 0.5);
        let b = optimize_general_arrangement_ga(&report, 0.45, 0.5);
        assert!(a.generations > 0 && a.evaluations >= a.generations);
        assert_eq!(a.generations, b.generations);
        assert_eq!(a.evaluations, b.evaluations);
        assert!((a.objective - b.objective).abs() < 1e-12);
        for i in 0..6 {
            assert!((a.centers[i] - b.centers[i]).abs() < 1e-12);
            assert!((a.longitudinal[i] - b.longitudinal[i]).abs() < 1e-12);
        }
        assert!(a.objective.is_finite());
        for (name, ok) in &a.checks {
            assert!(*ok, "GA constraint failed: {name}");
        }
        // Volumes must stay within the allocatable envelope.
        let total = a.layout.machinery
            + a.layout.magazine
            + a.layout.accommodation
            + a.layout.fuel
            + a.layout.stores
            + a.layout.void;
        assert!(total <= 0.90 * report.hull_volume_m3 + 1e-6);
        assert!(a.layout.void >= 0.02 * report.hull_volume_m3 - 1e-6);
    }

    #[test]
    fn compartment_ga_is_no_worse_than_its_seed() {
        let ship = ga_test_ship();
        let report = calculate_internal_spaces(&ship);
        let armament = 0.5;
        let (seed_layout, _cg, seed_centers) = optimize_compartment_layout(&report, 0.45);
        let b = ga_bounds(&report, &seed_layout);
        let seed_v = [
            seed_layout.machinery,
            seed_layout.magazine,
            seed_layout.accommodation,
            seed_layout.fuel,
            seed_layout.stores,
            seed_layout.void,
        ];
        let mut seed = individual_from_z(
            seed_v,
            seed_centers,
            [0.50, armament, 0.58, 0.50, 0.58, 0.45],
        );
        repair_individual(&mut seed, &b, armament);
        let seed_obj = ga_objective(&seed, &b, 0.45, armament);
        let ga = optimize_general_arrangement_ga(&report, 0.45, armament);
        assert!(
            ga.objective <= seed_obj + 1e-9,
            "GA objective {} should not exceed seed {}",
            ga.objective,
            seed_obj
        );
    }

    #[test]
    fn compartment_ga_respects_deck_and_access_constraints() {
        let ship = ga_test_ship();
        let report = calculate_internal_spaces(&ship);
        let p = optimize_general_arrangement_ga(&report, 0.45, 0.5);
        assert!(p.decks.iter().all(|d| *d < DECK_COUNT));
        assert!(p.access_connected, "access network must be connected");
        assert!(
            p.access_edges >= 5,
            "connected graph needs at least 5 edges"
        );
        assert!(p
            .checks
            .iter()
            .any(|(n, _)| n == "Access network connected"));
        assert!(p
            .checks
            .iter()
            .any(|(n, _)| n == "Access network edges >= 5"));
        assert!(p
            .checks
            .iter()
            .any(|(n, _)| n == "Decks separated by <= 1 (adjacent)"));
        assert!(p.escape_max_hops <= MAX_ESCAPE_HOPS);
        assert!(p.escape_max_path <= MAX_ESCAPE_LEN + 1e-9);
        assert!(p.structure_weight_tons.is_finite() && p.structure_weight_tons > 0.0);
        assert!(p.checks.iter().any(|(n, _)| n == "Escape hops <= 3"));
        assert!(p
            .checks
            .iter()
            .any(|(n, _)| n == "Accommodation has >= 2 access paths"));
        for (name, ok) in &p.checks {
            assert!(*ok, "constraint failed: {name}");
        }
    }

    #[test]
    fn compartment_ga_respects_armament_position() {
        let ship = ga_test_ship();
        let report = calculate_internal_spaces(&ship);
        for armament in [0.35, 0.50, 0.65] {
            let p = optimize_general_arrangement_ga(&report, 0.45, armament);
            assert!((p.magazine_longitudinal - armament).abs() <= 0.15 + 1e-6);
            assert!((p.machinery_longitudinal - 0.50).abs() <= 0.20 + 1e-6);
            assert!(p.checks.iter().all(|(_, ok)| *ok));
        }
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
        assert!(json.contains("\"cg_longitudinal\""));
        assert!(json.contains("\"volumes\""));
        assert!(json.contains("\"checks\""));
        assert!(json.contains("\"generations\""));
        assert!(json.contains("\"evaluations\""));
        assert!(json.contains("\"decks\""));
        assert!(json.contains("\"access_edges\""));
        assert!(json.contains("\"access_connected\""));
        assert!(json.contains("\"escape_max_hops\""));
        assert!(json.contains("\"escape_max_path\""));
        assert!(json.contains("\"structure_weight_tons\""));
        assert!(json.ends_with('}'));
        let html = compartment_placement_html(&ship, &placement);
        assert!(html.contains("General arrangement"));
        assert!(html.contains("Access network"));
        assert!(html.contains("Escape:"));
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
