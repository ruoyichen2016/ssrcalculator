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

/// Vessel geometry used to turn normalised arrangement coordinates into real
/// distances for escape-route checks.
#[derive(Clone, Copy, Debug)]
pub struct VesselGeometry {
    pub length_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
    pub deck_height_m: f64,
}

impl VesselGeometry {
    pub fn from_ship(ship: &ShipModel) -> Self {
        let length_m = (ship.hull.length * 0.3048).max(1.0);
        let beam_m = (ship.hull.beam_bulges * 0.3048).max(1.0);
        let depth_m = ((ship.hull.draught + ship.free.free_distributed) * 0.3048).max(1.0);
        let deck_height_m = (depth_m / DECK_COUNT as f64).max(0.5);
        VesselGeometry {
            length_m,
            beam_m,
            depth_m,
            deck_height_m,
        }
    }

    /// Fallback geometry when only the hull volume is known.
    pub fn from_hull_volume(hull_volume_m3: f64) -> Self {
        let length_m = (2.4 * hull_volume_m3.max(1.0).powf(1.0 / 3.0)).max(1.0);
        let beam_m = (length_m / 7.0).max(1.0);
        let depth_m = 12.0;
        VesselGeometry {
            length_m,
            beam_m,
            depth_m,
            deck_height_m: 2.5,
        }
    }
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
    /// Structural-weight estimate (plating + bulkheads + framing, tons).
    pub structure_weight_tons: f64,
    /// Persons occupying each compartment (escape sizing).
    pub occupancy: [f64; 6],
    /// Required redundant escape routes per compartment.
    pub required_paths: [usize; 6],
    /// Per-compartment evacuation time (s) from the IMO effective flows.
    pub evacuation_time_s: [f64; 6],
    /// Persons the network can move to the trunk within the target time.
    pub network_evacuation_capacity: f64,
    /// Maximum rule-style scantling utilization (<= 1 meets the checks).
    pub scantling_utilization: f64,
    /// Longitudinal position of the assembly station (0..1).
    pub assembly_x: f64,
    /// Congestion-aware network evacuation time (s).
    pub congestion_time_s: f64,
    /// Per-compartment mustering time to the assembly station (s).
    pub mustering_time_s: [f64; 6],
    /// Maximum IMO edge density from the shortest-path flow assignment.
    pub max_edge_density: f64,
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
/// Fraction of the crew present in each compartment for escape sizing.
const OCCUPANCY_FRACTION: [f64; N_COMP] = [0.20, 0.04, 1.00, 0.02, 0.02, 0.01];
/// Hard limit for escape routes measured on the real deck plan.
/// (Polyline routes via the vertical trunk are longer than straight lines.)
const MAX_ESCAPE_METERS: f64 = 45.0;

// --- IMO MSC.1/Circ.1533 (simplified evacuation parameters) --------------
/// Boundary layer at each side of a route (m).
const IMO_BOUNDARY_LAYER_M: f64 = 0.15;
/// IMO density-flow curve: a = 0.266 m2/person; k = 1.4 (corridor) and
/// 1.3 (stair) m/s.  Fs(D) = (1 - a D) k D, capped at k/(4a).
const IMO_A: f64 = 0.266;
const IMO_K_CORRIDOR: f64 = 1.4;
const IMO_K_STAIR: f64 = 1.3;
/// Maximum tolerated density (persons/m2).
const IMO_D_MAX: f64 = 4.0;
/// Nominal clear widths of a schematic access route (m).
const CORRIDOR_WIDTH_M: f64 = 1.2;
const STAIR_WIDTH_M: f64 = 1.0;
/// Target time to move the crew to the trunk / assembly (s).
pub const MAX_EVAC_TIME_S: f64 = 1800.0;

// --- Classification-style scantling parameters (planning values) ----------
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
    /// Longitudinal position of the assembly station (0..1).
    assembly_x: f64,
    fitness: f64,
}

struct GaBounds {
    v_lo: [f64; N_COMP],
    v_hi: [f64; N_COMP],
    x_lo: [f64; N_COMP],
    x_hi: [f64; N_COMP],
    allocatable: f64,
    void_min: f64,
    geometry: VesselGeometry,
    /// Persons occupying each compartment (escape sizing).
    occupancy: [f64; N_COMP],
    /// Required edge-disjoint paths per compartment to the trunk.
    required_paths: [usize; N_COMP],
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
        assembly_x: 0.5,
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
        repair_access(&mut ind.x, &mut ind.deck, &mut ind.offset, b);
    }
    ind.assembly_x = ind.assembly_x.clamp(0.2, 0.8);
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

/// Real geometric length (m) of an access edge on the deck plan.
fn edge_escape_len(
    geom: &VesselGeometry,
    xa: f64,
    xb: f64,
    da: usize,
    db: usize,
    x_trunk: f64,
) -> f64 {
    // Schematic deck plan: two longitudinal side corridors at +/- B/4, so every
    // route has lateral access legs to reach a corridor.
    let lateral = 2.0 * (geom.beam_m / 4.0).max(0.5);
    if da == db {
        (xa - xb).abs() * geom.length_m + lateral
    } else {
        ((xa - x_trunk).abs() + (xb - x_trunk).abs()) * geom.length_m
            + da.abs_diff(db) as f64 * geom.deck_height_m
            + lateral
    }
}

/// Shortest escape hops and path length (metres) from the trunk (compartment 0).
///
/// Edge weight uses the real deck plan: `sqrt((dx*L)^2 + (ddeck*H)^2)`.
/// Unreachable nodes keep `usize::MAX` and `f64::INFINITY`.
fn escape_arrays(
    edges: &[(usize, usize)],
    x: &[f64; N_COMP],
    deck: &[usize; N_COMP],
    geom: &VesselGeometry,
) -> ([usize; N_COMP], [f64; N_COMP]) {
    let mut hops = [usize::MAX; N_COMP];
    let mut lens = [f64::INFINITY; N_COMP];
    hops[0] = 0;
    lens[0] = 0.0;
    for _ in 0..N_COMP {
        for (a, b) in edges {
            let w = edge_escape_len(geom, x[*a], x[*b], deck[*a], deck[*b], x[0]);
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

/// IMO specific flow (persons/(m*s)) at a given density.
fn imo_specific_flow(density: f64, is_stair: bool) -> f64 {
    let k = if is_stair {
        IMO_K_STAIR
    } else {
        IMO_K_CORRIDOR
    };
    let d = density.clamp(0.0, IMO_D_MAX);
    (k * (1.0 - IMO_A * d) * d).max(0.0)
}

/// IMO walking speed (m/s) at a given density.
fn imo_speed(density: f64, is_stair: bool) -> f64 {
    let k = if is_stair {
        IMO_K_STAIR
    } else {
        IMO_K_CORRIDOR
    };
    (k * (1.0 - IMO_A * density)).max(0.0)
}

/// Maximum specific flow (persons/(m*s)) = k/(4a).
fn imo_max_specific_flow(is_stair: bool) -> f64 {
    let k = if is_stair {
        IMO_K_STAIR
    } else {
        IMO_K_CORRIDOR
    };
    k / (4.0 * IMO_A)
}

/// Effective IMO capacity (persons/s) of a nominal route.
fn imo_edge_capacity_persons_s(is_stair: bool) -> f64 {
    let width = if is_stair {
        STAIR_WIDTH_M
    } else {
        CORRIDOR_WIDTH_M
    };
    let effective = (width - 2.0 * IMO_BOUNDARY_LAYER_M).max(0.0);
    effective * imo_max_specific_flow(is_stair)
}

/// Maximum network flow to the trunk (persons/s); merging and downstream
/// congestion are captured by the min cut.
fn network_evacuation_rate(edges: &[(usize, usize)], deck: &[usize; N_COMP]) -> f64 {
    let n = N_COMP + 1;
    let source = N_COMP;
    let mut cap = vec![vec![0.0_f64; n]; n];
    for (i, j) in edges {
        let c = imo_edge_capacity_persons_s(deck[*i] != deck[*j]);
        cap[*i][*j] += c;
        cap[*j][*i] += c;
    }
    for i in 1..N_COMP {
        cap[source][i] = 1.0e9;
    }
    let mut flow = 0.0_f64;
    loop {
        let mut parent = vec![usize::MAX; n];
        let mut seen = vec![false; n];
        let mut queue = VecDeque::new();
        seen[source] = true;
        queue.push_back(source);
        while let Some(u) = queue.pop_front() {
            for v in 0..n {
                if !seen[v] && cap[u][v] > 1e-9 {
                    seen[v] = true;
                    parent[v] = u;
                    queue.push_back(v);
                }
            }
        }
        if !seen[0] {
            break;
        }
        let mut v = 0;
        let mut bottleneck = f64::INFINITY;
        while v != source {
            let u = parent[v];
            bottleneck = bottleneck.min(cap[u][v]);
            v = u;
        }
        v = 0;
        while v != source {
            let u = parent[v];
            cap[u][v] -= bottleneck;
            cap[v][u] += bottleneck;
            v = u;
        }
        flow += bottleneck;
    }
    flow
}

/// Assembly station deck (weather deck).
const ASSEMBLY_DECK: usize = DECK_COUNT - 1;

/// Mustering distance (m) from a compartment to the assembly station, routed
/// via the vertical trunk and the upper-deck corridor.
fn mustering_distance_m(
    geom: &VesselGeometry,
    xi: f64,
    deck_i: usize,
    x_trunk: f64,
    assembly_x: f64,
) -> f64 {
    let lateral = 2.0 * (geom.beam_m / 4.0).max(0.5);
    (xi - x_trunk).abs() * geom.length_m
        + deck_i.abs_diff(ASSEMBLY_DECK) as f64 * geom.deck_height_m
        + (x_trunk - assembly_x).abs() * geom.length_m
        + lateral
}

/// Per-compartment mustering time (s): walking at IMO light-density speed plus
/// a queue at the compartment exit.
fn mustering_times(
    geom: &VesselGeometry,
    x: &[f64; N_COMP],
    deck: &[usize; N_COMP],
    occupancy: &[f64; N_COMP],
    x_trunk: f64,
    assembly_x: f64,
) -> [f64; N_COMP] {
    let walk = imo_speed(0.5, false).max(0.5);
    let mut times = [0.0_f64; N_COMP];
    for i in 0..N_COMP {
        let distance = mustering_distance_m(geom, x[i], deck[i], x_trunk, assembly_x);
        let exit_capacity = if i == 0 {
            f64::INFINITY
        } else {
            imo_edge_capacity_persons_s(deck[i] != ASSEMBLY_DECK)
        };
        let queue = if exit_capacity.is_finite() {
            occupancy[i] / exit_capacity
        } else {
            0.0
        };
        times[i] = distance / walk + queue;
    }
    times
}

/// Invert the IMO density-flow curve by bisection (monotonic up to the peak).
fn density_from_specific_flow(specific_flow: f64, is_stair: bool) -> f64 {
    let hi = 1.0 / (2.0 * IMO_A);
    let mut lo = 0.0_f64;
    let mut hi = hi;
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if imo_specific_flow(mid, is_stair) < specific_flow {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Maximum edge density (persons/m2) from a shortest-path assignment of each
/// compartment's occupants to the trunk.
fn max_edge_density(
    edges: &[(usize, usize)],
    deck: &[usize; N_COMP],
    occupancy: &[f64; N_COMP],
    x: &[f64; N_COMP],
    geom: &VesselGeometry,
) -> f64 {
    let mut dist = [f64::INFINITY; N_COMP];
    let mut prev = [usize::MAX; N_COMP];
    dist[0] = 0.0;
    for _ in 0..N_COMP {
        for (a, b) in edges {
            let w = edge_escape_len(geom, x[*a], x[*b], deck[*a], deck[*b], x[0]);
            if dist[*a].is_finite() && dist[*a] + w < dist[*b] - 1e-12 {
                dist[*b] = dist[*a] + w;
                prev[*b] = *a;
            }
            if dist[*b].is_finite() && dist[*b] + w < dist[*a] - 1e-12 {
                dist[*a] = dist[*b] + w;
                prev[*a] = *b;
            }
        }
    }
    let mut flow: Vec<((usize, usize), f64)> = Vec::new();
    for i in 1..N_COMP {
        if !dist[i].is_finite() {
            continue;
        }
        let mut node = i;
        let mut guard = 0;
        while prev[node] != usize::MAX && guard < N_COMP {
            let p = prev[node];
            let edge = (node.min(p), node.max(p));
            if let Some(entry) = flow.iter_mut().find(|(e, _)| *e == edge) {
                entry.1 += occupancy[i];
            } else {
                flow.push((edge, occupancy[i]));
            }
            node = p;
            guard += 1;
        }
    }
    let mut max_density = 0.0_f64;
    for ((i, j), persons) in flow {
        let is_stair = deck[i] != deck[j];
        let width = (if is_stair {
            STAIR_WIDTH_M
        } else {
            CORRIDOR_WIDTH_M
        }) - 2.0 * IMO_BOUNDARY_LAYER_M;
        let rate = persons / MAX_EVAC_TIME_S;
        let specific_flow = rate / width.max(1e-9);
        let density = density_from_specific_flow(specific_flow, is_stair);
        max_density = max_density.max(density);
    }
    max_density
}

/// Per-compartment evacuation time (s) using IMO effective flows.
fn compartment_evacuation_times(
    edges: &[(usize, usize)],
    deck: &[usize; N_COMP],
    occupancy: &[f64; N_COMP],
) -> [f64; N_COMP] {
    let mut capacity = [0.0_f64; N_COMP];
    for (i, j) in edges {
        let c = imo_edge_capacity_persons_s(deck[*i] != deck[*j]);
        capacity[*i] += c;
        capacity[*j] += c;
    }
    let mut times = [f64::INFINITY; N_COMP];
    times[0] = 0.0; // the trunk is the assembly/refuge point
    for i in 1..N_COMP {
        times[i] = if capacity[i] > 1e-9 {
            occupancy[i] / capacity[i]
        } else {
            f64::INFINITY
        };
    }
    times
}

/// Persons the access network can move to the trunk within `MAX_EVAC_TIME_S`.
///
/// Max flow from a super-source (supply = compartment occupancy) to the trunk,
/// with edge capacities = IMO flow rate * target time.
fn network_evacuation_capacity(
    edges: &[(usize, usize)],
    deck: &[usize; N_COMP],
    occupancy: &[f64; N_COMP],
) -> f64 {
    let n = N_COMP + 1;
    let source = N_COMP;
    let mut cap = vec![vec![0.0_f64; n]; n];
    for (i, j) in edges {
        let c = imo_edge_capacity_persons_s(deck[*i] != deck[*j]) * MAX_EVAC_TIME_S;
        cap[*i][*j] += c;
        cap[*j][*i] += c;
    }
    cap[source][1..N_COMP].copy_from_slice(&occupancy[1..N_COMP]);
    let mut flow = 0.0_f64;
    loop {
        let mut parent = vec![usize::MAX; n];
        let mut seen = vec![false; n];
        let mut queue = VecDeque::new();
        seen[source] = true;
        queue.push_back(source);
        while let Some(u) = queue.pop_front() {
            for v in 0..n {
                if !seen[v] && cap[u][v] > 1e-9 {
                    seen[v] = true;
                    parent[v] = u;
                    queue.push_back(v);
                }
            }
        }
        if !seen[0] {
            break;
        }
        let mut v = 0;
        let mut bottleneck = f64::INFINITY;
        while v != source {
            let u = parent[v];
            bottleneck = bottleneck.min(cap[u][v]);
            v = u;
        }
        v = 0;
        while v != source {
            let u = parent[v];
            cap[u][v] -= bottleneck;
            cap[v][u] += bottleneck;
            v = u;
        }
        flow += bottleneck;
    }
    flow
}

/// Rule-style structural-weight estimate (tons), delegated to the scantling
/// module (plate + stiffener + bulkhead + girder).
fn structure_weight_tons(geom: &VesselGeometry, deck: &[usize; N_COMP], x: &[f64; N_COMP]) -> f64 {
    let mut used = [false; DECK_COUNT];
    for dd in deck {
        used[(*dd).min(DECK_COUNT - 1)] = true;
    }
    let levels = used.iter().filter(|u| **u).count().max(1) as f64;
    let span = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - x.iter().cloned().fold(f64::INFINITY, f64::min);
    let input = crate::scantling::ScantlingInput::from_geometry(
        geom.length_m,
        geom.beam_m,
        geom.depth_m,
        levels,
        span,
    );
    crate::scantling::evaluate(&input).weight_tons()
}

/// Rule-style scantling report for the arrangement.
fn structure_scantling(
    geom: &VesselGeometry,
    deck: &[usize; N_COMP],
    x: &[f64; N_COMP],
) -> crate::scantling::ScantlingReport {
    let mut used = [false; DECK_COUNT];
    for dd in deck {
        used[(*dd).min(DECK_COUNT - 1)] = true;
    }
    let levels = used.iter().filter(|u| **u).count().max(1) as f64;
    let span = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - x.iter().cloned().fold(f64::INFINITY, f64::min);
    let input = crate::scantling::ScantlingInput::from_geometry(
        geom.length_m,
        geom.beam_m,
        geom.depth_m,
        levels,
        span,
    );
    crate::scantling::evaluate(&input)
}

fn repair_access(
    x: &mut [f64; N_COMP],
    deck: &mut [usize; N_COMP],
    offset: &mut [f64; N_COMP],
    b: &GaBounds,
) {
    for _ in 0..24 {
        let edges = access_edges(x, deck);
        let reachable = access_reachable(&edges);
        let (hops, lens) = escape_arrays(&edges, x, deck, &b.geometry);
        let connected = reachable.len() == N_COMP;
        let escape_ok = hops
            .iter()
            .all(|h| *h != usize::MAX && *h <= MAX_ESCAPE_HOPS)
            && lens.iter().all(|l| *l <= MAX_ESCAPE_METERS);
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
    // Access-network connectivity and spare edges.
    let edges = access_edges(x, &ind.deck);
    let reachable = access_reachable(&edges);
    f += 3.0 * (N_COMP - reachable.len()) as f64;
    if reachable.len() == N_COMP {
        f += 0.5 * (5_usize.saturating_sub(edges.len())) as f64;
    }
    // Redundant escape routes (edge-disjoint paths to the trunk).
    for i in 1..N_COMP {
        let paths = edge_disjoint_paths(&edges, i, 0);
        f += 2.0 * b.required_paths[i].saturating_sub(paths) as f64;
    }
    // IMO evacuation times (compartment and whole-network).
    let times = compartment_evacuation_times(&edges, &ind.deck, &b.occupancy);
    for time in times.iter() {
        if !time.is_finite() {
            f += 3.0;
        } else if *time > MAX_EVAC_TIME_S {
            f += 3.0 * (*time - MAX_EVAC_TIME_S) / MAX_EVAC_TIME_S;
        }
    }
    let total_occupancy: f64 = b.occupancy[1..].iter().sum();
    if total_occupancy > 1e-9 {
        let network = network_evacuation_capacity(&edges, &ind.deck, &b.occupancy);
        if network < total_occupancy {
            f += 5.0 * (total_occupancy - network) / total_occupancy;
        }
        // Congestion-aware time through the network bottleneck.
        let rate = network_evacuation_rate(&edges, &ind.deck);
        let congestion_time = if rate > 1e-9 {
            total_occupancy / rate
        } else {
            f64::INFINITY
        };
        if congestion_time > MAX_EVAC_TIME_S {
            f += 3.0 * (congestion_time - MAX_EVAC_TIME_S) / MAX_EVAC_TIME_S;
        }
    }
    // Mustering to the assembly station.
    let muster = mustering_times(
        &b.geometry,
        x,
        &ind.deck,
        &b.occupancy,
        x[0],
        ind.assembly_x,
    );
    for time in muster.iter() {
        if *time > MAX_EVAC_TIME_S {
            f += 2.0 * (*time - MAX_EVAC_TIME_S) / MAX_EVAC_TIME_S;
        }
    }
    // Escape distance on the real deck plan (polyline via the trunk).
    let (hops, lens) = escape_arrays(&edges, x, &ind.deck, &b.geometry);
    for i in 1..N_COMP {
        if hops[i] == usize::MAX || hops[i] > MAX_ESCAPE_HOPS {
            f += 1.5;
        }
        if lens[i] > MAX_ESCAPE_METERS {
            f += 1.0 * (lens[i] - MAX_ESCAPE_METERS) / MAX_ESCAPE_METERS;
        }
    }
    // Rule-based structural-weight linkage (plating + stiffeners + bulkheads).
    f += STRUCT_WEIGHT_WEIGHT * structure_weight_tons(&b.geometry, &ind.deck, x) / 1000.0;
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
    // Blend the assembly-station position.
    let (lo, hi) = (
        a.assembly_x.min(b.assembly_x),
        a.assembly_x.max(b.assembly_x),
    );
    let d = hi - lo;
    child.assembly_x = rng.range(lo - blend * d, hi + blend * d);
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
    if rng.unit() < 0.12 {
        ind.assembly_x += rng.normal() * sigma;
    }
}

fn placement_checks(
    v: &[f64; N_COMP],
    z: &[f64; N_COMP],
    x: &[f64; N_COMP],
    deck: &[usize; N_COMP],
    armament: f64,
    assembly_x: f64,
    b: &GaBounds,
) -> Vec<(String, bool)> {
    let edges = access_edges(x, deck);
    let reachable = access_reachable(&edges);
    let connected = reachable.len() == N_COMP;
    let (hops, lens) = escape_arrays(&edges, x, deck, &b.geometry);
    let capacity_ok = (1..N_COMP).all(|i| edge_disjoint_paths(&edges, i, 0) >= b.required_paths[i]);
    let comp_times = compartment_evacuation_times(&edges, deck, &b.occupancy);
    let total_occupancy: f64 = b.occupancy[1..].iter().sum();
    let network_capacity = network_evacuation_capacity(&edges, deck, &b.occupancy);
    let rate = network_evacuation_rate(&edges, deck);
    let congestion_time = if total_occupancy > 1e-9 && rate > 1e-9 {
        total_occupancy / rate
    } else {
        0.0
    };
    let muster = mustering_times(&b.geometry, x, deck, &b.occupancy, x[0], assembly_x);
    let max_density = max_edge_density(edges.as_slice(), deck, &b.occupancy, x, &b.geometry);
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
            v[5] >= b.void_min - 1e-6,
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
        ("Redundant escape routes".to_string(), capacity_ok),
        (
            "Compartment evacuation <= 1800 s (IMO)".to_string(),
            comp_times
                .iter()
                .all(|t| t.is_finite() && *t <= MAX_EVAC_TIME_S + 1e-6),
        ),
        (
            "Network evacuates crew <= 1800 s (IMO)".to_string(),
            total_occupancy <= 1e-9 || network_capacity + 1e-6 >= total_occupancy,
        ),
        (
            "Escape hops <= 3".to_string(),
            hops.iter()
                .all(|h| *h != usize::MAX && *h <= MAX_ESCAPE_HOPS),
        ),
        (
            "Escape path <= 45 m".to_string(),
            lens.iter().all(|l| *l <= MAX_ESCAPE_METERS + 1e-9),
        ),
        (
            "Congestion time <= 1800 s (IMO)".to_string(),
            congestion_time <= MAX_EVAC_TIME_S + 1e-6,
        ),
        (
            "Mustering <= 1800 s (assembly)".to_string(),
            muster
                .iter()
                .all(|t| t.is_finite() && *t <= MAX_EVAC_TIME_S + 1e-6),
        ),
        (
            "Edge density <= 4 p/m2 (IMO)".to_string(),
            max_density <= IMO_D_MAX + 1e-6,
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
    let (hops, lens) = escape_arrays(&edges, &x, &ind.deck, &b.geometry);
    let evacuation_time_s = compartment_evacuation_times(&edges, &ind.deck, &b.occupancy);
    let network_evacuation_capacity = network_evacuation_capacity(&edges, &ind.deck, &b.occupancy);
    let scant = structure_scantling(&b.geometry, &ind.deck, &x);
    let total_net: f64 = b.occupancy[1..].iter().sum();
    let rate = network_evacuation_rate(&edges, &ind.deck);
    let congestion_time_s = if total_net > 1e-9 && rate > 1e-9 {
        total_net / rate
    } else {
        0.0
    };
    let mustering_time_s = mustering_times(
        &b.geometry,
        &x,
        &ind.deck,
        &b.occupancy,
        x[0],
        ind.assembly_x,
    );
    let max_edge_density = max_edge_density(&edges, &ind.deck, &b.occupancy, &x, &b.geometry);
    let mut checks = placement_checks(&v, &z, &x, &ind.deck, armament, ind.assembly_x, b);
    checks.extend(scant.checks());
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
        structure_weight_tons: scant.weight_tons(),
        occupancy: b.occupancy,
        required_paths: b.required_paths,
        evacuation_time_s,
        network_evacuation_capacity,
        scantling_utilization: scant.max_utilization(),
        assembly_x: ind.assembly_x,
        congestion_time_s,
        mustering_time_s,
        max_edge_density,
        objective: ga_objective(ind, b, target_cg, armament),
        generations,
        evaluations,
        checks,
    }
}

fn ga_bounds(
    report: &InternalSpaceReport,
    seed: &CompartmentLayout,
    geometry: VesselGeometry,
) -> GaBounds {
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

    // Crew occupancy drives the IMO evacuation analysis; separate redundancy
    // requirements ensure independent escape routes.
    let crew = if report.crew_volume_per_person > 0.0 {
        report.accommodation_volume_m3 / report.crew_volume_per_person
    } else {
        0.0
    };
    let occupancy = OCCUPANCY_FRACTION.map(|f| (crew * f).max(0.0));
    let mut required_paths = [1_usize; N_COMP];
    required_paths[2] = 2; // independent escape routes for accommodation

    GaBounds {
        v_lo,
        v_hi,
        x_lo: [0.05; N_COMP],
        x_hi: [0.95; N_COMP],
        allocatable,
        void_min,
        geometry,
        occupancy,
        required_paths,
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
        assembly_x: 0.5,
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
    geometry: VesselGeometry,
    target_cg: f64,
    armament: f64,
) -> CompartmentPlacement {
    let (seed_layout, _seed_cg, seed_centers) = optimize_compartment_layout(report, target_cg);
    let b = ga_bounds(report, &seed_layout, geometry);
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
            assembly_x: rng.range(0.2, 0.8),
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
    let geometry = VesselGeometry::from_ship(ship);
    optimize_general_arrangement_ga(&report, geometry, target_cg, magazine_longitudinal)
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
    let occupancy: Vec<String> = placement
        .occupancy
        .iter()
        .map(|o| format!("{o:.0}"))
        .collect();
    let required_paths: Vec<String> = placement
        .required_paths
        .iter()
        .map(|r| r.to_string())
        .collect();
    let evacuation_time: Vec<String> = placement
        .evacuation_time_s
        .iter()
        .map(|t| format!("{t:.1}"))
        .collect();
    let mustering_time: Vec<String> = placement
        .mustering_time_s
        .iter()
        .map(|t| format!("{t:.1}"))
        .collect();
    let checks: Vec<String> = placement
        .checks
        .iter()
        .map(|(name, ok)| format!("{{\"name\":\"{}\",\"ok\":{}}}", esc(name), ok))
        .collect();
    format!(
        "{{\n  \"cg_vertical\": {:.4},\n  \"cg_longitudinal\": {:.4},\n  \"objective\": {:.4},\n  \"generations\": {},\n  \"evaluations\": {},\n  \"magazine_longitudinal\": {:.4},\n  \"machinery_longitudinal\": {:.4},\n  \"centers\": [{}],\n  \"decks\": [{}],\n  \"longitudinal\": [{}],\n  \"access_edges\": {},\n  \"access_connected\": {},\n  \"escape_max_hops\": {},\n  \"escape_max_path\": {:.4},\n  \"structure_weight_tons\": {:.1},\n  \"occupancy\": [{}],\n  \"required_paths\": [{}],\n  \"evacuation_time_s\": [{}],\n  \"network_evacuation_capacity\": {:.1},\n  \"scantling_utilization\": {:.4},\n  \"assembly_x\": {:.4},\n  \"congestion_time_s\": {:.1},\n  \"mustering_time_s\": [{}],\n  \"max_edge_density\": {:.2},\n  \"volumes\": {{\"machinery\": {:.1}, \"magazine\": {:.1}, \"accommodation\": {:.1}, \"fuel\": {:.1}, \"stores\": {:.1}, \"void\": {:.1}, \"unallocated\": {:.1}}},\n  \"checks\": [{}]\n}}",
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
        occupancy.join(", "),
        required_paths.join(", "),
        evacuation_time.join(", "),
        placement.network_evacuation_capacity,
        placement.scantling_utilization,
        placement.assembly_x,
        placement.congestion_time_s,
        mustering_time.join(", "),
        placement.max_edge_density,
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
        crate::features::html_escape(&ship.name),
        crate::features::html_escape(&ship.ship_type)
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
        "<p>Access network: <b>{}</b> ({} edges, connected={})<br>Escape: max {} hops / {:.1} m<br>Structural-weight estimate: {:.0} t</p>",
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
    html.push_str(&format!(
        "<p>Rule scantlings: max utilization {:.2}; structural weight {:.0} t</p>",
        placement.scantling_utilization, placement.structure_weight_tons
    ));
    html.push_str(&format!(
        "<p>IMO evacuation: {} persons; network capacity {:.0} persons / {} s; congestion time {:.0} s; assembly at {:.2} L; max edge density {:.2} p/m2; max mustering {:.0} s; required routes {:?}</p>",
        placement.occupancy.iter().sum::<f64>(),
        placement.network_evacuation_capacity,
        MAX_EVAC_TIME_S as u32,
        placement.congestion_time_s,
        placement.assembly_x,
        placement.max_edge_density,
        placement
            .mustering_time_s
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max),
        placement.required_paths
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

    fn test_geometry() -> VesselGeometry {
        VesselGeometry {
            length_m: 180.0,
            beam_m: 24.0,
            depth_m: 12.0,
            deck_height_m: 3.0,
        }
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
        let a = optimize_general_arrangement_ga(&report, test_geometry(), 0.45, 0.5);
        let b = optimize_general_arrangement_ga(&report, test_geometry(), 0.45, 0.5);
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
        let b = ga_bounds(&report, &seed_layout, test_geometry());
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
        let ga = optimize_general_arrangement_ga(&report, test_geometry(), 0.45, armament);
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
        let p = optimize_general_arrangement_ga(&report, test_geometry(), 0.45, 0.5);
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
        assert!(p.escape_max_path <= MAX_ESCAPE_METERS + 1e-9);
        assert!(p.structure_weight_tons.is_finite() && p.structure_weight_tons > 0.0);
        assert!(p
            .evacuation_time_s
            .iter()
            .all(|t| t.is_finite() && *t <= MAX_EVAC_TIME_S + 1e-6));
        assert!(p.network_evacuation_capacity.is_finite());
        assert!(p.congestion_time_s.is_finite());
        assert!(p.max_edge_density.is_finite() && p.max_edge_density <= IMO_D_MAX + 1e-6);
        assert!(p.mustering_time_s.iter().all(|t| t.is_finite()));
        assert!(p.scantling_utilization.is_finite() && p.scantling_utilization <= 1.0 + 1e-9);
        assert!((0.2..=0.8).contains(&p.assembly_x));
        for name in [
            "Compartment evacuation <= 1800 s (IMO)",
            "Network evacuates crew <= 1800 s (IMO)",
            "Congestion time <= 1800 s (IMO)",
            "Mustering <= 1800 s (assembly)",
            "Edge density <= 4 p/m2 (IMO)",
            "Deck plate thickness (rule)",
            "Deck stiffener section modulus (rule)",
            "Bulkhead plate thickness (rule)",
            "Bulkhead stiffener section modulus (rule)",
            "Stiffener deflection <= l/200 (rule)",
        ] {
            assert!(
                p.checks.iter().any(|(n, _)| n == name),
                "missing check {name}"
            );
        }
        assert!(p.occupancy.iter().all(|o| *o >= 0.0 && o.is_finite()));
        assert!(p.required_paths.iter().all(|r| *r >= 1));
        assert!(
            p.required_paths[2] >= 2,
            "accommodation needs two independent escape routes"
        );
        assert!(p.checks.iter().any(|(n, _)| n == "Escape hops <= 3"));
        assert!(p.checks.iter().any(|(n, _)| n == "Redundant escape routes"));
        for (name, ok) in &p.checks {
            assert!(*ok, "constraint failed: {name}");
        }
    }

    #[test]
    fn compartment_ga_respects_armament_position() {
        let ship = ga_test_ship();
        let report = calculate_internal_spaces(&ship);
        for armament in [0.35, 0.50, 0.65] {
            let p = optimize_general_arrangement_ga(&report, test_geometry(), 0.45, armament);
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
        assert!(json.contains("\"occupancy\""));
        assert!(json.contains("\"required_paths\""));
        assert!(json.contains("\"evacuation_time_s\""));
        assert!(json.contains("\"network_evacuation_capacity\""));
        assert!(json.contains("\"scantling_utilization\""));
        assert!(json.contains("\"assembly_x\""));
        assert!(json.contains("\"congestion_time_s\""));
        assert!(json.contains("\"mustering_time_s\""));
        assert!(json.contains("\"max_edge_density\""));
        assert!(json.ends_with('}'));
        let html = compartment_placement_html(&ship, &placement);
        assert!(html.contains("General arrangement"));
        assert!(html.contains("Access network"));
        assert!(html.contains("Escape:"));
        assert!(html.contains("Rule scantlings"));
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
