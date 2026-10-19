//! Shared, normalized concept layout. Dimensions are fractions of hull length;
//! transverse coordinates and widths are fractions of beam. No mass model.
use crate::{
    blueprint::BlueprintMount,
    model::ShipModel,
    systems::{CombatModality, MachineryArrangement, TacticalEchelon},
};

/// Pure Weberian engineering ideal types for warship superstructure layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Archetype {
    #[default]
    Auto,
    /// Type I: Early twin-citadel distributed bridges & tall dispersed exhaust stacks.
    LowDistributed,
    /// Type II: Divided unit-machinery arrangement with amidships separation / gap.
    Split,
    /// Type III: Concentrated tall multi-tier optical tower / pagoda citadel.
    Tower,
    /// Type IV: Armoured Queen-Anne citadel block & compact central superstructure.
    Compact,
    /// Type V: All-forward main battery with aft-shifted central citadel tower.
    AftCitadel,
    /// Type VI: Low-windage streamlined flotilla bridge & shelter deck for destroyers.
    FlotillaStreamlined,
    /// Type VII: Offset flight-deck starboard island for aircraft carriers.
    Island,
}

impl Archetype {
    pub const ALL: [Self; 8] = [
        Self::Auto,
        Self::LowDistributed,
        Self::Split,
        Self::Tower,
        Self::Compact,
        Self::AftCitadel,
        Self::FlotillaStreamlined,
        Self::Island,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::LowDistributed => "low",
            Self::Split => "split",
            Self::Tower => "tower",
            Self::Compact => "compact",
            Self::AftCitadel => "aft_citadel",
            Self::FlotillaStreamlined => "flotilla",
            Self::Island => "island",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "low" | "low_distributed" | "type_1" | "type1" | "twin_citadel" => {
                Some(Self::LowDistributed)
            }
            "split" | "divided" | "type_2" | "type2" | "amidships_gap" => Some(Self::Split),
            "tower" | "pagoda" | "type_3" | "type3" | "tall_tower" => Some(Self::Tower),
            "compact" | "queen_anne" | "type_4" | "type4" | "central_block" => Some(Self::Compact),
            "aft_citadel" | "aft" | "all_forward" | "nelson" | "richelieu" => {
                Some(Self::AftCitadel)
            }
            "flotilla" | "flotilla_streamlined" | "destroyer" | "streamlined" => {
                Some(Self::FlotillaStreamlined)
            }
            "island" | "carrier_island" | "side_island" => Some(Self::Island),
            _ => None,
        }
    }
}

/// Naval mast and upper sensor platform construction style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MastStyle {
    #[default]
    Auto,
    /// Lightweight pole signal mast.
    Pole,
    /// Heavy tripod mast with optical rangefinder top.
    Tripod,
    /// Hyperboloid cage / lattice trellis fire-control mast.
    Lattice,
    /// Multi-tiered stepped optical pagoda mast platform.
    Pagoda,
    /// Fully enclosed pyramidal tower or integrated mast/funnel (Mack).
    Integrated,
}

impl MastStyle {
    pub const ALL: [Self; 6] = [
        Self::Auto,
        Self::Pole,
        Self::Tripod,
        Self::Lattice,
        Self::Pagoda,
        Self::Integrated,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Pole => "pole",
            Self::Tripod => "tripod",
            Self::Lattice => "lattice",
            Self::Pagoda => "pagoda",
            Self::Integrated => "integrated",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "pole" => Some(Self::Pole),
            "tripod" => Some(Self::Tripod),
            "lattice" | "cage" => Some(Self::Lattice),
            "pagoda" => Some(Self::Pagoda),
            "integrated" | "mack" | "tower" => Some(Self::Integrated),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Correction {
    pub locked: bool,
    pub shift: f64,
    pub length_scale: f64,
    pub height_scale: f64,
    pub x: f64,
    pub length: f64,
    pub height: f64,
}

impl Default for Correction {
    fn default() -> Self {
        Self {
            locked: false,
            shift: 0.0,
            length_scale: 1.0,
            height_scale: 1.0,
            x: 0.35,
            length: 0.05,
            height: 0.025,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub archetype: Archetype,
    pub mast: MastStyle,
    /// 0 selects the inferred exhaust count; 1..4 overrides it.
    pub funnel_count: usize,
    pub funnel_spacing: f64,
    pub bridge: Correction,
    pub aft: Correction,
    pub funnels: Correction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartKind {
    Bridge,
    AftDeckhouse,
    Funnel,
}

#[derive(Clone, Debug)]
pub struct Part {
    pub kind: PartKind,
    pub x: f64,
    pub length: f64,
    pub height: f64,
    pub y: f64,
    pub width: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Conflict {
    DeckBoundary(usize),
    PartOverlap(usize, usize),
    BatteryClearance(usize),
    ExhaustNearBridge,
    HeightLimit(usize),
    IslandOnGunship,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub archetype: Archetype,
    pub mast: MastStyle,
    pub mast_lines: Vec<[[f64; 2]; 2]>,
    pub mast_tip: [f64; 2],
    pub parts: Vec<Part>,
    pub conflicts: Vec<Conflict>,
}

fn correct(p: &mut Part, c: &Correction) {
    if c.locked {
        p.x = c.x;
        p.length = c.length;
        p.height = c.height;
    } else {
        p.x += c.shift;
        p.length *= c.length_scale;
        p.height *= c.height_scale;
    }
}

impl Plan {
    pub fn solve(ship: &ShipModel, batteries: &[Vec<BlueprintMount>; 5], carrier: bool) -> Self {
        let cfg = &ship.superstructure;
        let sys = ship.systems.evaluate(ship);
        let separated = matches!(
            sys.resolved_machinery_arrangement,
            MachineryArrangement::UnitAlternatingMachinery
                | MachineryArrangement::TurboElectricDistributed
                | MachineryArrangement::SandwichEngineBoilerEngine
        );

        let main_mounts = &batteries[0];
        let has_main_mounts = !main_mounts.is_empty();
        let all_forward = has_main_mounts
            && main_mounts.iter().all(|m| m.fwd && m.x <= 0.52)
            && !main_mounts.iter().any(|m| m.x > 0.55);

        let has_amidships_main = has_main_mounts
            && !all_forward
            && main_mounts.iter().any(|m| m.x >= 0.42 && m.x <= 0.58);

        let is_flotilla = sys.resolved_echelon == TacticalEchelon::FlotillaAssaultLine
            || (sys.resolved_echelon == TacticalEchelon::InnerFleetScreen
                && sys.features.slenderness_ratio >= 8.2)
            || (sys.features.slenderness_ratio >= 8.6
                && sys.features.froude_number >= 0.36
                && ship.hull.displacement < 10_000.0);

        let archetype = if cfg.archetype != Archetype::Auto {
            cfg.archetype
        } else if carrier || sys.resolved_modality == CombatModality::AviationDeckProjection {
            Archetype::Island
        } else if all_forward {
            Archetype::AftCitadel
        } else if is_flotilla {
            Archetype::FlotillaStreamlined
        } else if has_amidships_main {
            Archetype::Split
        } else if sys.optical_baseline_m >= 8.0
            || (ship.hull.date_hull >= 1940 && sys.features.main_caliber_scale >= 1.5)
        {
            Archetype::Tower
        } else if separated {
            Archetype::Split
        } else if ship.hull.date_hull < 1910 {
            Archetype::LowDistributed
        } else {
            Archetype::Compact
        };

        let (bl, bh, with_aft, default_origin) = match archetype {
            Archetype::LowDistributed => (0.065, 0.018, true, 0.32),
            Archetype::Split => (0.055, 0.026, true, 0.33),
            Archetype::Tower => (0.048, 0.046, false, 0.34),
            Archetype::Compact => (0.068, 0.030, false, 0.35),
            Archetype::AftCitadel => (0.058, 0.038, false, 0.50),
            Archetype::FlotillaStreamlined => (0.045, 0.022, true, 0.32),
            Archetype::Island => (0.065, 0.030, false, 0.44),
            Archetype::Auto => (0.065, 0.025, false, 0.35),
        };

        let count = if cfg.funnel_count > 0 {
            cfg.funnel_count
        } else if archetype == Archetype::FlotillaStreamlined {
            if ship.hull.date_hull < 1935 || separated {
                2
            } else {
                1
            }
        } else if archetype == Archetype::AftCitadel {
            if ship.hull.date_engine >= 1935 {
                1
            } else {
                2
            }
        } else if separated {
            2
        } else if ship.hull.date_engine < 1910 {
            3
        } else {
            1
        };
        let count = count.clamp(1, 4);

        let demand = (ship.engine.hp_max.max(0.0) / ship.hull.displacement.max(1.0) / 5.0)
            .sqrt()
            .clamp(0.5, 2.0);
        let fl = (0.022 * demand).clamp(0.015, 0.045);
        let spacing = if cfg.funnel_spacing > 0.0 {
            cfg.funnel_spacing
        } else if separated {
            0.065
        } else {
            0.035
        };
        let span = fl + (count - 1) as f64 * spacing;
        let island = archetype == Archetype::Island;
        let y = if island { -0.35 } else { 0.0 };
        let width = if island { 0.16 } else { 0.42 };

        let assemble = |origin: f64| {
            let mut bridge = Part {
                kind: PartKind::Bridge,
                x: origin,
                length: bl,
                height: bh,
                y,
                width,
            };
            let mut group = Part {
                kind: PartKind::Funnel,
                x: origin + bl + 0.035,
                length: span,
                height: 0.032,
                y,
                width: width * 0.7,
            };
            let aft_x = if archetype == Archetype::Split {
                group.x + span + 0.060
            } else {
                group.x + span + 0.035
            };
            let mut aft = Part {
                kind: PartKind::AftDeckhouse,
                x: aft_x,
                length: if archetype == Archetype::FlotillaStreamlined {
                    0.032
                } else {
                    0.040
                },
                height: if archetype == Archetype::FlotillaStreamlined {
                    0.015
                } else {
                    0.018
                },
                y,
                width: width * 0.8,
            };
            correct(&mut bridge, &cfg.bridge);
            correct(&mut group, &cfg.funnels);
            correct(&mut aft, &cfg.aft);
            let mut parts = vec![bridge];
            if with_aft {
                parts.push(aft);
            }
            let scale = group.length / span;
            for i in 0..count {
                parts.push(Part {
                    x: group.x + i as f64 * spacing * scale,
                    length: fl * scale,
                    ..group.clone()
                });
            }
            parts
        };

        let score = |parts: &[Part], origin: f64| {
            let mut cost = (origin - default_origin).abs() * 2.0;
            for (i, p) in parts.iter().enumerate() {
                cost += ((0.05 - p.x).max(0.0) + (p.x + p.length - 0.95).max(0.0)) * 1000.0;
                for m in batteries.iter().flatten() {
                    if (m.side as f64 * 0.35 - p.y).abs() < p.width / 2.0 + 0.12
                        && m.x > p.x - 0.035
                        && m.x < p.x + p.length + 0.035
                    {
                        cost += 10.0;
                    }
                }
                for q in parts.iter().take(i) {
                    if (p.y - q.y).abs() < (p.width + q.width) / 2.0 {
                        cost += ((p.x + p.length).min(q.x + q.length) - p.x.max(q.x) + 0.01)
                            .max(0.0)
                            * 500.0;
                    }
                }
            }
            cost
        };

        let mut parts = assemble(default_origin);
        let mut best = score(&parts, default_origin);
        for i in 15..=68 {
            let origin = i as f64 / 100.0;
            let candidate = assemble(origin);
            let cost = score(&candidate, origin);
            if cost < best {
                best = cost;
                parts = candidate;
            }
        }

        let mut conflicts = Vec::new();
        if island && !carrier {
            conflicts.push(Conflict::IslandOnGunship);
        }
        for (i, p) in parts.iter().enumerate() {
            if p.x < 0.05 || p.x + p.length > 0.95 || p.y.abs() + p.width / 2.0 > 0.5 {
                conflicts.push(Conflict::DeckBoundary(i));
            }
            if p.height > 0.07 {
                conflicts.push(Conflict::HeightLimit(i));
            }
            if batteries.iter().flatten().any(|m| {
                (m.side as f64 * 0.35 - p.y).abs() < p.width / 2.0 + 0.12
                    && m.x > p.x - 0.035
                    && m.x < p.x + p.length + 0.035
            }) {
                conflicts.push(Conflict::BatteryClearance(i));
            }
            for (j, q) in parts.iter().enumerate().take(i) {
                if p.x < q.x + q.length
                    && q.x < p.x + p.length
                    && (p.y - q.y).abs() < (p.width + q.width) / 2.0
                {
                    conflicts.push(Conflict::PartOverlap(j, i));
                }
            }
        }
        if parts
            .iter()
            .filter(|p| p.kind == PartKind::Funnel)
            .any(|p| {
                (p.x - (parts[0].x + parts[0].length)).abs() < 0.025 && p.height >= parts[0].height
            })
        {
            conflicts.push(Conflict::ExhaustNearBridge);
        }

        let mast = if cfg.mast != MastStyle::Auto {
            cfg.mast
        } else if archetype == Archetype::Tower {
            if sys.optical_baseline_m >= 10.0 {
                MastStyle::Pagoda
            } else {
                MastStyle::Integrated
            }
        } else if archetype == Archetype::AftCitadel || archetype == Archetype::Compact {
            if ship.hull.date_hull >= 1935 {
                MastStyle::Integrated
            } else {
                MastStyle::Tripod
            }
        } else if archetype == Archetype::FlotillaStreamlined {
            if ship.hull.date_hull >= 1940 {
                MastStyle::Pole
            } else {
                MastStyle::Tripod
            }
        } else if ship.hull.date_hull < 1930 && sys.optical_baseline_m >= 5.0 {
            MastStyle::Tripod
        } else {
            MastStyle::Pole
        };

        let center = parts[0].x + parts[0].length * 0.5;
        let base = parts[0].height;
        let top = base + 0.022;
        let mut mast_lines = vec![[[center, base], [center, top]]];
        match mast {
            MastStyle::Tripod => {
                for side in [-1.0, 1.0] {
                    mast_lines.push([[center + side * 0.012, base], [center, top]]);
                }
            }
            MastStyle::Pagoda => {
                for side in [-1.0, 1.0] {
                    mast_lines.push([[center + side * 0.014, base], [center + side * 0.006, top]]);
                }
                for step in 1..=4 {
                    let y = base + (step as f64 / 5.0) * 0.022;
                    let hw = 0.014 - (step as f64 / 5.0) * 0.008;
                    mast_lines.push([[center - hw, y], [center + hw, y]]);
                }
            }
            MastStyle::Lattice | MastStyle::Integrated => {
                for side in [-1.0, 1.0] {
                    mast_lines.push([[center + side * 0.012, base], [center + side * 0.005, top]]);
                }
                mast_lines.push([[center - 0.005, top], [center + 0.005, top]]);
                if mast == MastStyle::Lattice {
                    for step in 0..3 {
                        let y = base + step as f64 * 0.022 / 3.0;
                        mast_lines.push([[center - 0.008, y], [center + 0.008, y + 0.022 / 3.0]]);
                        mast_lines.push([[center + 0.008, y], [center - 0.008, y + 0.022 / 3.0]]);
                    }
                }
            }
            _ => {}
        }

        Self {
            archetype,
            mast,
            mast_lines,
            mast_tip: [center, top],
            parts,
            conflicts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        blueprint::BlueprintLayout,
        io::{deserialize, serialize},
    };

    #[test]
    fn locked_geometry_roundtrips_and_reports_conflicts() {
        let mut ship = crate::get_preset("dreadnought").unwrap();
        ship.superstructure.archetype = Archetype::Split;
        ship.superstructure.funnel_count = 4;
        ship.superstructure.bridge.locked = true;
        ship.superstructure.bridge.x = 0.93;
        ship.superstructure.bridge.length = 0.1;
        ship.superstructure.bridge.height = 0.08;
        ship.superstructure.funnels.shift = -0.1;
        let saved = serialize(&ship);
        let mut restored = ShipModel::new();
        deserialize(&mut restored, &saved).unwrap();
        assert_eq!(serialize(&restored), saved);
        let plan = BlueprintLayout::from_ship(&restored).superstructure;
        assert_eq!(plan.parts[0].x, 0.93);
        assert_eq!(plan.parts[0].height, 0.08);
        assert!(plan.conflicts.contains(&Conflict::DeckBoundary(0)));
        assert_eq!(
            plan.parts
                .iter()
                .filter(|p| p.kind == PartKind::Funnel)
                .count(),
            4
        );
        assert!(deserialize(&mut restored, "superstructure.funnel_count=1.5").is_err());
    }

    #[test]
    fn automatic_and_relative_layouts_are_deterministic() {
        let mut ship = crate::get_preset("dreadnought").unwrap();
        ship.superstructure.archetype = Archetype::Tower;
        let base = BlueprintLayout::from_ship(&ship).superstructure;
        ship.superstructure.bridge.height_scale = 1.5;
        let changed = BlueprintLayout::from_ship(&ship).superstructure;
        assert_eq!(changed.parts[0].height, base.parts[0].height * 1.5);
        for kind in Archetype::ALL {
            ship.superstructure.archetype = kind;
            let a = BlueprintLayout::from_ship(&ship).superstructure;
            let b = BlueprintLayout::from_ship(&ship).superstructure;
            assert_eq!(a.parts.len(), b.parts.len());
            for (p, q) in a.parts.iter().zip(&b.parts) {
                assert_eq!(p.x, q.x);
                assert!(p.x.is_finite() && p.length > 0.0 && p.height > 0.0);
            }
        }
    }

    #[test]
    fn historical_presets_infer_appropriate_archetypes() {
        // Nelson: all-forward battery -> AftCitadel
        let nelson = crate::get_preset("nelson").unwrap();
        let plan_nelson = BlueprintLayout::from_ship(&nelson).superstructure;
        assert_eq!(plan_nelson.archetype, Archetype::AftCitadel);

        // Fletcher / Shimakaze: flotilla echelon / high slenderness -> FlotillaStreamlined
        let fletcher = crate::get_preset("fletcher").unwrap();
        let plan_fletcher = BlueprintLayout::from_ship(&fletcher).superstructure;
        assert_eq!(plan_fletcher.archetype, Archetype::FlotillaStreamlined);

        // Essex: carrier modality -> Island
        let essex = crate::get_preset("essex").unwrap();
        let plan_essex = BlueprintLayout::from_ship(&essex).superstructure;
        assert_eq!(plan_essex.archetype, Archetype::Island);

        // Mikasa / Olympia: pre-1910 -> LowDistributed
        let mikasa = crate::get_preset("mikasa").unwrap();
        let plan_mikasa = BlueprintLayout::from_ship(&mikasa).superstructure;
        assert_eq!(plan_mikasa.archetype, Archetype::LowDistributed);

        // Yamato: huge optical baseline / super-capital -> Tower
        let yamato = crate::get_preset("yamato").unwrap();
        let plan_yamato = BlueprintLayout::from_ship(&yamato).superstructure;
        assert_eq!(plan_yamato.archetype, Archetype::Tower);
        assert_eq!(plan_yamato.mast, MastStyle::Pagoda);
    }
}
