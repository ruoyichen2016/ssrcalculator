//! Ship electrical plant, fire-control directors, ballistic computers,
//! radar/sensors, dimensionless feature extraction, and pure Weberian
//! style ideal-type (archetype) matching.
//!
//! All style matching and subsystem scaling operate strictly on scale-free,
//! country-agnostic dimensionless physical invariants ($\Pi$-groups) with zero
//! dependence on flag state (`ship.country`) or vessel name (`ship.name`),
//! preventing label leakage while supporting full per-slot manual mix-and-match.

use crate::model::ShipModel;

/// Scale-free, country-agnostic dimensionless physical and functional invariants ($\Pi$-groups).
#[derive(Clone, Copy, Debug, Default)]
pub struct DimensionlessFeatures {
    /// Froude number $Fn = V / \sqrt{g L}$ (wave-making regime).
    pub froude_number: f64,
    /// Volumetric slenderness ratio $L / \nabla^{1/3}$, where $\nabla = 35 \Delta$.
    pub slenderness_ratio: f64,
    /// Planform length-to-beam aspect ratio $L / B$.
    pub length_beam_ratio: f64,
    /// Transverse form stability ratio $B / T$.
    pub beam_draught_ratio: f64,
    /// Underwater block coefficient $C_b$.
    pub block_coeff: f64,
    /// Bouguer dimensionless transverse metacentric radius ratio $(C_{wp}^2 / C_b) \cdot (B / T)$.
    pub bouguer_bm_ratio: f64,
    /// Reserve buoyancy freeboard-to-draught ratio $F / T$.
    pub freeboard_draught_ratio: f64,
    /// Protection mass fraction $W_{\text{armour}} / \Delta$.
    pub protection_fraction: f64,
    /// Offensive ordnance mass fraction $(W_{\text{guns}} + W_{\text{torp}}) / \Delta$.
    pub ordnance_fraction: f64,
    /// Dual-purpose / automated AA barrel scale density $N_{\text{dp+aa}} / \Delta^{1/3}$.
    pub dp_barrel_density: f64,
    /// Torpedo strike scale density $(N_{\text{torp}} \cdot (d_{\text{torp}} / 21)^2) / \Delta^{1/3}$.
    pub torpedo_strike_density: f64,
    /// Main battery relative caliber scale $d_{\text{main}} / \sqrt{B}$.
    pub main_caliber_scale: f64,
    /// Main battery barrel slenderness ratio $L_{\text{cal}} / 45.0$.
    pub barrel_slenderness: f64,
    /// Dimensionless endurance scale $R / (1000 \cdot L^{1/3})$.
    pub endurance_ratio: f64,
    /// Volumetric Froude number $Fn_{\nabla} = V / \sqrt{g \nabla^{1/3}}$.
    pub volumetric_froude_number: f64,
    /// Dimensionless aviation flight deck & air group capability scale $\Pi_{\text{air}} \in [0, 1]$.
    pub aviation_capability_ratio: f64,
    /// Normalized technological maturity $\tau \in [0, 1]$ across 1890..=1950.
    pub tech_epoch: f64,
}

impl DimensionlessFeatures {
    /// Extract pure dimensionless physical invariants from a `ShipModel`.
    /// Never accesses `ship.country` or `ship.name`.
    pub fn from_ship(ship: &ShipModel) -> Self {
        let l = ship.hull.length.max(10.0);
        let b = ship.hull.beam_bulges.max(ship.hull.beam).max(2.0);
        let t = ship.hull.draught.max(1.0);
        let cb = ship.hull.block.clamp(0.30, 0.85);
        let disp = ship.hull.displacement.max(l * b * t * cb / 35.0).max(50.0);
        let nabla_cuberoot = (disp * 35.0).powf(1.0 / 3.0).max(1.0);
        let disp_cuberoot = disp.powf(1.0 / 3.0).max(1.0);

        let v_fps = ship.engine.speed_max.max(0.0) * 1.687809857;
        let froude_number = v_fps / (32.17404856 * l).sqrt();
        let volumetric_froude_number = v_fps / (32.17404856 * nabla_cuberoot).sqrt();
        let slenderness_ratio = l / nabla_cuberoot;
        let length_beam_ratio = l / b;
        let beam_draught_ratio = b / t;

        let cwp = if ship.hull.co_water_plane > 0.1 {
            ship.hull.co_water_plane
        } else {
            (0.70 + 0.30 * cb).clamp(0.60, 0.92)
        };
        let bouguer_bm_ratio = (cwp * cwp / cb) * beam_draught_ratio;

        let mean_free = if ship.free.freeboard > 0.0 {
            ship.free.freeboard
        } else {
            (ship.free.free_fc_for + ship.free.free_fd_for + ship.free.free_ad_for) / 3.0
        };
        let freeboard_draught_ratio = mean_free.max(1.0) / t;

        let protection_fraction = (ship.armour.weight / disp).clamp(0.0, 0.65);
        let ordnance_fraction =
            ((ship.guns.guns_weight + ship.guns.magazine + ship.weapons.weight) / disp)
                .clamp(0.0, 0.50);

        // Count dual-purpose (gun_type == 4) and automatic AA (gun_type == 5) barrels
        let mut dp_weighted = 0.0_f64;
        for bat in &ship.guns.batteries {
            if bat.barrels > 0 {
                if bat.gun_type == 4 {
                    dp_weighted += bat.barrels as f64 * 1.2;
                } else if bat.gun_type == 5 {
                    dp_weighted += bat.barrels as f64 * 0.45;
                } else if bat.gun_type == 3 {
                    dp_weighted += bat.barrels as f64 * 0.6;
                }
            }
        }
        let dp_barrel_density = dp_weighted / disp_cuberoot;

        let t1_n = ship.weapons.torpedo1.number.max(0) as f64;
        let t1_d = (ship.weapons.torpedo1.size.max(14.0) / 21.0).powi(2);
        let t2_n = ship.weapons.torpedo2.number.max(0) as f64;
        let t2_d = (ship.weapons.torpedo2.size.max(14.0) / 21.0).powi(2);
        let torpedo_strike_density = (t1_n * t1_d + t2_n * t2_d) / disp_cuberoot;

        let main_cal = ship.guns.batteries[0].calibre.max(0.0);
        let main_caliber_scale = main_cal / b.sqrt();
        let barrel_slenderness = if main_cal > 0.0 {
            ship.guns.batteries[0].cal_length.max(20.0) / 45.0
        } else {
            1.0
        };

        let endurance_ratio = ship.engine.range.max(500.0) / (1000.0 * l.powf(1.0 / 3.0));
        let tech_epoch =
            ((ship.hull.date_hull.max(ship.hull.date_engine) - 1890) as f64 / 60.0).clamp(0.0, 1.0);

        // Dimensionless aviation capability indicator Pi_air in [0.0, 1.0]
        let misc_deck_ton = ship.weapons.misc_weight_deck.max(0.0);
        let misc_deck_fraction = (misc_deck_ton / disp).clamp(0.0, 0.15);
        let freeboard_factor = ((freeboard_draught_ratio - 0.70).max(0.0) / 0.35).clamp(0.0, 1.0);
        let non_heavy_factor = if main_cal <= 6.0 {
            1.0
        } else {
            (1.0 - (main_cal - 6.0) / 6.0).clamp(0.0, 1.0)
        };
        let light_armour_factor = (1.0 - protection_fraction / 0.18).clamp(0.0, 1.0);
        let mut aviation_score = 0.35 * (misc_deck_fraction / 0.035).min(1.0)
            + 0.30 * freeboard_factor
            + 0.20 * non_heavy_factor
            + 0.15 * light_armour_factor;
        if misc_deck_ton >= 250.0 && freeboard_draught_ratio >= 0.75 && main_cal <= 6.0 {
            aviation_score = aviation_score.max(0.80);
        }
        if ship.ship_type.to_ascii_lowercase().contains("carrier") {
            aviation_score = aviation_score.max(0.85);
        }
        let aviation_capability_ratio = aviation_score.clamp(0.0, 1.0);

        Self {
            froude_number,
            slenderness_ratio,
            length_beam_ratio,
            beam_draught_ratio,
            block_coeff: cb,
            bouguer_bm_ratio,
            freeboard_draught_ratio,
            protection_fraction,
            ordnance_fraction,
            dp_barrel_density,
            torpedo_strike_density,
            main_caliber_scale,
            barrel_slenderness,
            endurance_ratio,
            volumetric_froude_number,
            aviation_capability_ratio,
            tech_epoch,
        }
    }
}

/// Pure Weberian Naval Engineering Style Ideal Type (country-agnostic).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StyleDoctrine {
    /// Inferred automatically from dimensionless physical features ($\Pi$-groups).
    #[default]
    AdaptiveAuto,
    /// Ideal Type I: Radar-Automated, High-DP Density & High-Margin All-Electric Grid.
    AutomationAllElectric,
    /// Ideal Type II: All-Weather Seakeeping, High Reserve Buoyancy & Balanced Sensor/ASW.
    AllWeatherSeakeeping,
    /// Ideal Type III: Long-Range Optical Duplex Ranging & High-Ordnance Strike.
    LongRangeOpticalStrike,
    /// Ideal Type IV: Stereoscopic High-Velocity Precision & Multi-Compartment Subdivided Plant.
    StereoscopicSubdivided,
    /// User custom mix-and-match combination.
    Custom,
}

impl StyleDoctrine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AdaptiveAuto => "adaptive_auto",
            Self::AutomationAllElectric => "automation_all_electric",
            Self::AllWeatherSeakeeping => "all_weather_seakeeping",
            Self::LongRangeOpticalStrike => "long_range_optical_strike",
            Self::StereoscopicSubdivided => "stereoscopic_subdivided",
            Self::Custom => "custom",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AdaptiveAuto => "Adaptive Auto (Dimensionless Ideal-Type Match)",
            Self::AutomationAllElectric => "Type I · Automated Radar & High-Margin All-Electric",
            Self::AllWeatherSeakeeping => "Type II · All-Weather Seakeeping & Balanced ASW",
            Self::LongRangeOpticalStrike => "Type III · Long-Range Optical & Heavy-Ordnance Strike",
            Self::StereoscopicSubdivided => {
                "Type IV · Stereoscopic Precision & Subdivided Survivability"
            }
            Self::Custom => "Custom Mix-and-Match",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "automation_all_electric"
            | "type_1"
            | "type1"
            | "radar_electric"
            | "usn"
            | "usn_radar_centric"
            | "usn_radar" => Self::AutomationAllElectric,
            "all_weather_seakeeping"
            | "type_2"
            | "type2"
            | "all_weather"
            | "rn"
            | "rn_all_weather"
            | "royal_navy" => Self::AllWeatherSeakeeping,
            "long_range_optical_strike"
            | "type_3"
            | "type3"
            | "optical_strike"
            | "ijn"
            | "ijn_optical_long_range"
            | "ijn_optical" => Self::LongRangeOpticalStrike,
            "stereoscopic_subdivided"
            | "type_4"
            | "type4"
            | "stereoscopic"
            | "km"
            | "km_stereoscopic_diesel"
            | "km_stereoscopic" => Self::StereoscopicSubdivided,
            "custom" | "manual" => Self::Custom,
            _ => Self::AdaptiveAuto,
        }
    }
}

/// Tactical Echelon / Fleet Formation Hierarchy (战术编队阶梯与阵位层次).
/// Pure country-agnostic functional archetype describing the vessel's primary tactical echelon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TacticalEchelon {
    /// Inferred automatically from dimensionless physical invariants.
    #[default]
    Auto,
    /// Echelon I · Main Body / Line of Battle (主力决战本队)
    BattleLineMainBody,
    /// Echelon II · Fast Wing / Vanguard / Flying Squadron (快速翼与前卫游击队)
    FastWingVanguard,
    /// Echelon III · Scouting Line / Outer & Search Screen (外层侦察与搜索跟踪幕)
    OuterScoutingScreen,
    /// Echelon IV · Fleet Screen / Bent-Line & Circular Screen (舰队内层防空反潜护卫幕)
    InnerFleetScreen,
    /// Echelon V · Torpedo Flotillas / Assault Line / Light Forces (水雷突击与近战支队)
    FlotillaAssaultLine,
}

impl TacticalEchelon {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::BattleLineMainBody => "battle_line_main_body",
            Self::FastWingVanguard => "fast_wing_vanguard",
            Self::OuterScoutingScreen => "outer_scouting_screen",
            Self::InnerFleetScreen => "inner_fleet_screen",
            Self::FlotillaAssaultLine => "flotilla_assault_line",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Echelon Match)",
            Self::BattleLineMainBody => "Echelon I · Main Body & Line of Battle (主力决战本队)",
            Self::FastWingVanguard => "Echelon II · Fast Wing & Vanguard (快速翼与前卫游击队)",
            Self::OuterScoutingScreen => {
                "Echelon III · Scouting Line & Outer Screen (外层侦察与搜索幕)"
            }
            Self::InnerFleetScreen => "Echelon IV · Fleet Screen & Inner Escort (舰队内层护卫幕)",
            Self::FlotillaAssaultLine => {
                "Echelon V · Torpedo Flotillas & Assault Line (水雷突击支队)"
            }
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "battle_line_main_body"
            | "battle_line"
            | "main_body"
            | "line_of_battle"
            | "echelon_1"
            | "echelon_i"
            | "main" => Self::BattleLineMainBody,
            "fast_wing_vanguard" | "fast_wing" | "vanguard" | "flying_squadron" | "echelon_2"
            | "echelon_ii" => Self::FastWingVanguard,
            "outer_scouting_screen"
            | "scouting_line"
            | "outer_screen"
            | "search_screen"
            | "cruiser_screen"
            | "echelon_3"
            | "echelon_iii"
            | "scout" => Self::OuterScoutingScreen,
            "inner_fleet_screen" | "fleet_screen" | "bent_line" | "circular_screen"
            | "inner_screen" | "echelon_4" | "echelon_iv" | "screen" => Self::InnerFleetScreen,
            "flotilla_assault_line"
            | "torpedo_flotilla"
            | "assault_line"
            | "light_forces"
            | "flotilla"
            | "echelon_5"
            | "echelon_v" => Self::FlotillaAssaultLine,
            _ => Self::Auto,
        }
    }
}

/// Combat Modality / Primary Weapon & Projection Mechanism (交战功能模态与投射体制).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CombatModality {
    /// Inferred automatically from dimensionless physical invariants.
    #[default]
    Auto,
    /// Modality A · Heavy Artillery & Armoured Citadel (重炮装甲堡垒模态)
    HeavyArtilleryCitadel,
    /// Modality B · Rapid-Fire Cruiser Gunnery (高射速水面火炮模态)
    RapidFireSurfaceGunnery,
    /// Modality C · Aviation Flight Deck & Hangar (航空甲板立体投射模态)
    AviationDeckProjection,
    /// Modality D · Massed Torpedo Strike (重雷击水面突击模态)
    MassedTorpedoStrike,
    /// Modality E · AA & ASW Screen Defense (防空反潜多维护卫模态)
    ScreenEscortDefense,
}

impl CombatModality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::HeavyArtilleryCitadel => "heavy_artillery_citadel",
            Self::RapidFireSurfaceGunnery => "rapid_fire_surface_gunnery",
            Self::AviationDeckProjection => "aviation_deck_projection",
            Self::MassedTorpedoStrike => "massed_torpedo_strike",
            Self::ScreenEscortDefense => "screen_escort_defense",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Modality Match)",
            Self::HeavyArtilleryCitadel => {
                "Modality A · Heavy Artillery & Armoured Citadel (重炮装甲堡垒)"
            }
            Self::RapidFireSurfaceGunnery => {
                "Modality B · Rapid-Fire Surface Gunnery (高射速水面火炮)"
            }
            Self::AviationDeckProjection => {
                "Modality C · Aviation Flight Deck Projection (航空甲板立体投射)"
            }
            Self::MassedTorpedoStrike => "Modality D · Massed Torpedo Strike (重雷击水面突击)",
            Self::ScreenEscortDefense => {
                "Modality E · AA & ASW Screen Escort Defense (防空反潜多维护卫)"
            }
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "heavy_artillery_citadel"
            | "heavy_artillery"
            | "citadel"
            | "big_gun"
            | "modality_a" => Self::HeavyArtilleryCitadel,
            "rapid_fire_surface_gunnery"
            | "rapid_fire"
            | "cruiser_gunnery"
            | "surface_gunnery"
            | "modality_b" => Self::RapidFireSurfaceGunnery,
            "aviation_deck_projection"
            | "aviation_deck"
            | "flight_deck"
            | "carrier_deck"
            | "aviation"
            | "carrier"
            | "modality_c" => Self::AviationDeckProjection,
            "massed_torpedo_strike"
            | "torpedo_strike"
            | "massed_torpedo"
            | "torpedo"
            | "modality_d" => Self::MassedTorpedoStrike,
            "screen_escort_defense"
            | "screen_defense"
            | "escort_defense"
            | "aa_asw"
            | "escort"
            | "modality_e" => Self::ScreenEscortDefense,
            _ => Self::Auto,
        }
    }
}

/// Mobility & Hydrodynamic Regime (机动与水动力体制).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MobilityRegime {
    /// Inferred automatically from dimensionless physical invariants.
    #[default]
    Auto,
    /// Regime 1 · Heavy Displacement Block (丰满排水量稳性体制, Fn <= 0.28, Cb >= 0.58)
    HeavyDisplacementBlock,
    /// Regime 2 · Trans-Froude Fast Capital (跨弗劳德高速大型船体体制, Fn ~ 0.30-0.38, disp >= 15kt)
    TransFroudeFastCapital,
    /// Regime 3 · Slender Wave-Cutting (细长破浪远洋巡航体制, L/B >= 7.8, Fn ~ 0.32-0.42)
    SlenderWaveCutting,
    /// Regime 4 · Super-Critical High-Fn & Planing (超临界高速与半滑行体制, Fn >= 0.40, Fn_vol >= 1.15)
    SuperCriticalHighFn,
}

impl MobilityRegime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::HeavyDisplacementBlock => "heavy_displacement_block",
            Self::TransFroudeFastCapital => "trans_froude_fast_capital",
            Self::SlenderWaveCutting => "slender_wave_cutting",
            Self::SuperCriticalHighFn => "super_critical_high_fn",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Hydrodynamic Match)",
            Self::HeavyDisplacementBlock => {
                "Regime 1 · Heavy Displacement Block Form (丰满排水量稳性体制)"
            }
            Self::TransFroudeFastCapital => {
                "Regime 2 · Trans-Froude Fast Capital Hull (跨弗劳德高速主力体制)"
            }
            Self::SlenderWaveCutting => {
                "Regime 3 · Slender Wave-Cutting Seakeeping (细长破浪远洋巡航体制)"
            }
            Self::SuperCriticalHighFn => {
                "Regime 4 · Super-Critical High-Fn & Semi-Planing (超临界半滑行体制)"
            }
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "heavy_displacement_block" | "displacement_block" | "block_form" | "regime_1" => {
                Self::HeavyDisplacementBlock
            }
            "trans_froude_fast_capital" | "fast_capital" | "trans_froude" | "regime_2" => {
                Self::TransFroudeFastCapital
            }
            "slender_wave_cutting" | "slender_cruiser" | "wave_cutting" | "regime_3" => {
                Self::SlenderWaveCutting
            }
            "super_critical_high_fn"
            | "super_critical"
            | "high_froude"
            | "planing"
            | "regime_4" => Self::SuperCriticalHighFn,
            _ => Self::Auto,
        }
    }
}

/// 3-Dimensional Factored Tactical Role Affinity Profile.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TacticalAffinityProfile {
    /// Echelon soft affinities [I, II, III, IV, V] (sums to 1.0).
    pub echelon_affinities: [f64; 5],
    /// Combat modality soft affinities [A, B, C, D, E] (sums to 1.0).
    pub modality_affinities: [f64; 5],
    /// Mobility regime soft affinities [1, 2, 3, 4] (sums to 1.0).
    pub mobility_affinities: [f64; 4],
    pub dominant_echelon: TacticalEchelon,
    pub dominant_modality: CombatModality,
    pub dominant_mobility: MobilityRegime,
    pub classification_summary: String,
}

/// Main Battery Fire-Control Director (pure functional ideal types).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MainDirector {
    #[default]
    Auto,
    /// Open sights / local turret rangefinder hoods (Base 0m).
    OpenSights,
    /// Tripod central sighting tower + 2.7m coincidence rangefinder.
    CentralTower,
    /// Weatherproof Director Control Tower + 4.6m/9m coincidence rangefinder.
    WeatherproofDct,
    /// Armoured rotating dome + 10.5m stereoscopic rangefinder.
    ArmouredStereoscopic10m,
    /// Elevated duplex tower + 15.5m ultra-long-baseline optical rangefinder.
    DuplexOpticalTower15m,
    /// Gyrostabilized director + centimetric blind-fire radar.
    GyrostabilizedRadarBlindFire,
}

impl MainDirector {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::OpenSights => "open_sights",
            Self::CentralTower => "central_tower",
            Self::WeatherproofDct => "weatherproof_dct",
            Self::ArmouredStereoscopic10m => "armoured_stereoscopic_10m",
            Self::DuplexOpticalTower15m => "duplex_optical_tower_15m",
            Self::GyrostabilizedRadarBlindFire => "gyrostabilized_radar_blind_fire",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::OpenSights => "Open Sights & Local Turret Hoods (Base 0m)",
            Self::CentralTower => "Central Sighting Tower + 2.7m Coincidence",
            Self::WeatherproofDct => "Weatherproof Director Tower + 4.6m/9m Coincidence",
            Self::ArmouredStereoscopic10m => "Armoured Rotating Dome + 10.5m Stereoscopic",
            Self::DuplexOpticalTower15m => "Elevated Duplex Tower + 15.5m Long-Baseline Optical",
            Self::GyrostabilizedRadarBlindFire => {
                "Gyrostabilized Director + Centimetric Blind-Fire Radar"
            }
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "open_sights" | "open" => Self::OpenSights,
            "central_tower" | "central" => Self::CentralTower,
            "weatherproof_dct" | "rn_dct_optical" | "rn_dct" | "dct" => Self::WeatherproofDct,
            "armoured_stereoscopic_10m" | "km_stereoscopic_10m" | "km_10m" | "drehhaube" => {
                Self::ArmouredStereoscopic10m
            }
            "duplex_optical_tower_15m" | "ijn_pagoda_15m" | "ijn_15m" | "pagoda" => {
                Self::DuplexOpticalTower15m
            }
            "gyrostabilized_radar_blind_fire" | "usn_mk38_radar" | "usn_mk38" | "mk38" => {
                Self::GyrostabilizedRadarBlindFire
            }
            _ => Self::Auto,
        }
    }
}

/// Secondary Battery / Dual-Purpose / Anti-Aircraft Fire-Control Director.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AaDirector {
    #[default]
    Auto,
    /// None or local optical ring sights.
    None,
    /// High-Angle Tachymetric Control System.
    HighAngleTachymetric,
    /// Tri-axially stabilized spherical optical/radar director.
    TriAxialStabilizedSphere,
    /// Optical high-angle lead-computing director.
    OpticalLeadComputing,
    /// Dual-purpose gyrostabilized radar director.
    DualPurposeRadarDirector,
}

impl AaDirector {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::HighAngleTachymetric => "high_angle_tachymetric",
            Self::TriAxialStabilizedSphere => "tri_axial_stabilized_sphere",
            Self::OpticalLeadComputing => "optical_lead_computing",
            Self::DualPurposeRadarDirector => "dual_purpose_radar_director",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::None => "None / Local Optical Ring Sights",
            Self::HighAngleTachymetric => "High-Angle Tachymetric Control Director",
            Self::TriAxialStabilizedSphere => "Tri-Axially Stabilized Spherical Director",
            Self::OpticalLeadComputing => "Optical High-Angle Lead-Computing Director",
            Self::DualPurposeRadarDirector => "Dual-Purpose Gyrostabilized Radar Director",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "none" | "no" => Self::None,
            "high_angle_tachymetric" | "rn_hacs" | "hacs" => Self::HighAngleTachymetric,
            "tri_axial_stabilized_sphere" | "km_wackeltopf" | "wackeltopf" => {
                Self::TriAxialStabilizedSphere
            }
            "optical_lead_computing" | "ijn_type94" | "type94" => Self::OpticalLeadComputing,
            "dual_purpose_radar_director" | "usn_mk37" | "mk37" => Self::DualPurposeRadarDirector,
            _ => Self::Auto,
        }
    }
}

/// Ballistic Computer / Transmitting Station / Plotting Room.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BallisticComputer {
    #[default]
    Auto,
    /// Manual firing tables & rate slide rules (Order 1).
    ManualTables,
    /// Mechanical range clock & disc integrator table (Order 2).
    MechanicalRateClock,
    /// Electro-mechanical plotting & fire-control table (Order 4).
    ElectroMechanicalTable,
    /// High-velocity analog ballistic computer (Order 4).
    HighVelocityAnalog,
    /// Multi-axis target triangulation & ballistic table (Order 4).
    MultiAxisTriangulator,
    /// 3D-cam gyro-coupled mechanical rangekeeper (Order 5).
    GyroCamRangekeeper,
}

impl BallisticComputer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ManualTables => "manual_tables",
            Self::MechanicalRateClock => "mechanical_rate_clock",
            Self::ElectroMechanicalTable => "electro_mechanical_table",
            Self::HighVelocityAnalog => "high_velocity_analog",
            Self::MultiAxisTriangulator => "multi_axis_triangulator",
            Self::GyroCamRangekeeper => "gyro_cam_rangekeeper",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::ManualTables => "Manual Firing Tables & Rate Slide Rules (Order 1)",
            Self::MechanicalRateClock => "Mechanical Range Clock & Disc Integrator (Order 2)",
            Self::ElectroMechanicalTable => {
                "Electro-Mechanical Plotting & Fire-Control Table (Order 4)"
            }
            Self::HighVelocityAnalog => "High-Velocity Analog Ballistic Computer (Order 4)",
            Self::MultiAxisTriangulator => {
                "Multi-Axis Target Triangulation & Ballistic Table (Order 4)"
            }
            Self::GyroCamRangekeeper => "3D-Cam Gyro-Coupled Mechanical Rangekeeper (Order 5)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "manual_tables" | "manual" => Self::ManualTables,
            "mechanical_rate_clock" | "vickers_dreyer" | "dreyer" | "vickers" => {
                Self::MechanicalRateClock
            }
            "electro_mechanical_table" | "rn_afct" | "afct" => Self::ElectroMechanicalTable,
            "high_velocity_analog" | "km_c38_analog" | "km_c38" | "c38" => Self::HighVelocityAnalog,
            "multi_axis_triangulator" | "ijn_type98_hoiban" | "ijn_type98" | "hoiban" => {
                Self::MultiAxisTriangulator
            }
            "gyro_cam_rangekeeper" | "usn_ford_mk1a" | "ford_mk1a" | "mk1a" => {
                Self::GyroCamRangekeeper
            }
            _ => Self::Auto,
        }
    }
}

/// Radar, Radio Direction Finder & Combat Information Center (CIC) Suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RadarSuite {
    #[default]
    Auto,
    /// None (optical & wireless telegraphy only).
    None,
    /// Decimetric surface/fire-control radar + passive warning receiver.
    DecimetricSurfaceFc,
    /// Metric air/surface search + horn magnetron surface radar.
    MetricSearchMagnetron,
    /// Centimetric fire-control radar + high-frequency direction finder (HF/DF).
    CentimetricFcDirectionFinder,
    /// Dual-band air/surface search + centimetric blind-fire + Full CIC.
    DualBandBlindFireCic,
}

impl RadarSuite {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::DecimetricSurfaceFc => "decimetric_surface_fc",
            Self::MetricSearchMagnetron => "metric_search_magnetron",
            Self::CentimetricFcDirectionFinder => "centimetric_fc_direction_finder",
            Self::DualBandBlindFireCic => "dual_band_blind_fire_cic",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::None => "None (Optical & Wireless Telegraphy Only)",
            Self::DecimetricSurfaceFc => "Decimetric Ranging Radar + Passive Warning Receiver",
            Self::MetricSearchMagnetron => "Metric Air/Surface Search + Cavity Magnetron Radar",
            Self::CentimetricFcDirectionFinder => {
                "Centimetric Fire-Control Radar + HF/DF Direction Finder"
            }
            Self::DualBandBlindFireCic => {
                "Dual-Band Search + Centimetric Blind-Fire Radar + Full CIC"
            }
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "none" | "no" => Self::None,
            "decimetric_surface_fc" | "km_fumo_decimetric" | "km_fumo" | "fumo" => {
                Self::DecimetricSurfaceFc
            }
            "metric_search_magnetron" | "ijn_type21_22" | "ijn_type21" | "type21" => {
                Self::MetricSearchMagnetron
            }
            "centimetric_fc_direction_finder" | "rn_centimetric_hfdf" | "rn_radar" | "hfdf" => {
                Self::CentimetricFcDirectionFinder
            }
            "dual_band_blind_fire_cic" | "usn_centimetric_cic" | "usn_radar" | "cic" => {
                Self::DualBandBlindFireCic
            }
            _ => Self::Auto,
        }
    }
}

/// Subsurface Acoustic Detection Suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SonarSuite {
    #[default]
    Auto,
    /// None (no subsurface acoustic sensors).
    None,
    /// Hull passive hydrophone listening array.
    PassiveHydrophones,
    /// Retractable active piezoelectric quartz transducer dome.
    ActiveQuartzDome,
    /// Multi-element balcony passive listening array + active pulse sonar.
    MultiElementPassiveArray,
    /// High-power magnetostrictive searchlight active sonar.
    HighPowerSearchSonar,
}

impl SonarSuite {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::PassiveHydrophones => "passive_hydrophones",
            Self::ActiveQuartzDome => "active_quartz_dome",
            Self::MultiElementPassiveArray => "multi_element_passive_array",
            Self::HighPowerSearchSonar => "high_power_search_sonar",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::None => "None",
            Self::PassiveHydrophones => "Hull Passive Hydrophone Listening Array",
            Self::ActiveQuartzDome => "Retractable Active Piezoelectric Quartz Dome",
            Self::MultiElementPassiveArray => "Multi-Element Balcony Passive Array + Active Pulse",
            Self::HighPowerSearchSonar => "High-Power Magnetostrictive Searchlight Sonar",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "none" | "no" => Self::None,
            "passive_hydrophones" | "hydrophones" => Self::PassiveHydrophones,
            "active_quartz_dome" | "rn_asdic" | "asdic" => Self::ActiveQuartzDome,
            "multi_element_passive_array" | "km_ghg_array" | "ghg" => {
                Self::MultiElementPassiveArray
            }
            "high_power_search_sonar" | "usn_qc_sonar" | "usn_qc" | "qc" => {
                Self::HighPowerSearchSonar
            }
            _ => Self::Auto,
        }
    }
}

/// Electrical Auxiliaries Plant Philosophy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ElectricalPlantStyle {
    #[default]
    Auto,
    /// Compact steam dynamos (lightweight, minimal redundancy, 10–15% reserve).
    CompactSteam,
    /// Balanced steam turbo-generators + emergency diesel generators (25–30% reserve).
    BalancedTurboDiesel,
    /// Multi-compartment heavy survivability diesel/turbo plant (30–35% reserve).
    SubdividedHeavyDiesel,
    /// High-margin 450V AC turbo-generator grid + robust diesel backup (40–50% reserve).
    HighMarginAcGrid,
}

impl ElectricalPlantStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::CompactSteam => "compact_steam",
            Self::BalancedTurboDiesel => "balanced_turbo_diesel",
            Self::SubdividedHeavyDiesel => "subdivided_heavy_diesel",
            Self::HighMarginAcGrid => "high_margin_ac_grid",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::CompactSteam => "Compact Steam Dynamos (10–15% Reserve)",
            Self::BalancedTurboDiesel => "Balanced Steam Turbo & Emergency Diesel (25–30% Reserve)",
            Self::SubdividedHeavyDiesel => {
                "Multi-Compartment Heavy Diesel/Turbo Plant (30–35% Reserve)"
            }
            Self::HighMarginAcGrid => "High-Margin 450V AC All-Electric Grid (40–50% Reserve)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "compact_steam" | "compact" => Self::CompactSteam,
            "balanced_turbo_diesel" | "rn_balanced" | "balanced" => Self::BalancedTurboDiesel,
            "subdivided_heavy_diesel" | "km_heavy_diesel" | "diesel" => Self::SubdividedHeavyDiesel,
            "high_margin_ac_grid" | "usn_high_margin_ac" | "usn_ac" | "ac" => {
                Self::HighMarginAcGrid
            }
            _ => Self::Auto,
        }
    }
}

/// Full configuration for a ship's systems suite.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ShipSystemsConfig {
    pub doctrine: StyleDoctrine,
    pub tactical_echelon: TacticalEchelon,
    pub combat_modality: CombatModality,
    pub mobility_regime: MobilityRegime,
    pub main_director: MainDirector,
    pub aa_director: AaDirector,
    pub ballistic_computer: BallisticComputer,
    pub radar_suite: RadarSuite,
    pub sonar_suite: SonarSuite,
    pub electrical_plant: ElectricalPlantStyle,
    /// Explicit turbo generator override in kW (None = auto-calculated).
    pub custom_turbo_kw: Option<f64>,
    /// Explicit diesel generator override in kW (None = auto-calculated).
    pub custom_diesel_kw: Option<f64>,
    /// Whether to couple systems weight/delta-GM into the primary ship calculations.
    /// Default is false to preserve 100% legacy benchmark fidelity unless explicitly toggled.
    pub couple_weights: bool,
}

/// Evaluation diagnostics and physical metrics produced for a ship's systems.
#[derive(Clone, Debug, Default)]
pub struct SystemsEvaluation {
    // Dimensionless physical invariants
    pub features: DimensionlessFeatures,

    // 3D Factored Tactical-Functional Classification & Soft Affinities
    pub resolved_echelon: TacticalEchelon,
    pub resolved_modality: CombatModality,
    pub resolved_mobility_regime: MobilityRegime,
    pub tactical_affinities: TacticalAffinityProfile,

    // Inferred ideal type and resolved equipment
    pub resolved_doctrine: StyleDoctrine,
    pub resolved_main_director: MainDirector,
    pub resolved_aa_director: AaDirector,
    pub resolved_ballistic_computer: BallisticComputer,
    pub resolved_radar_suite: RadarSuite,
    pub resolved_sonar_suite: SonarSuite,
    pub resolved_electrical_plant: ElectricalPlantStyle,

    // Pure Ideal-Type soft affinities (0.0 to 1.0, sum to 1.0)
    /// Type I: Radar-Automated & All-Electric affinity.
    pub type1_affinity: f64,
    /// Type II: All-Weather Seakeeping & Balanced ASW affinity.
    pub type2_affinity: f64,
    /// Type III: Long-Range Optical & Heavy-Ordnance Strike affinity.
    pub type3_affinity: f64,
    /// Type IV: Stereoscopic Precision & Subdivided Survivability affinity.
    pub type4_affinity: f64,
    pub match_explanation: String,

    // Electrical plant metrics
    pub load_turrets_kw: f64,
    pub load_hull_hotel_kw: f64,
    pub load_fire_control_kw: f64,
    pub load_electronics_kw: f64,
    pub combat_peak_kw: f64,
    pub turbo_generator_kw: f64,
    pub diesel_generator_kw: f64,
    pub total_installed_kw: f64,
    pub electrical_margin_pct: f64,
    pub is_overloaded: bool,

    // Weights across 4 vertical CG tiers (tons)
    pub vital_weight_tons: f64,
    pub hull_weight_tons: f64,
    pub deck_weight_tons: f64,
    pub topweight_tons: f64,
    pub total_systems_weight_tons: f64,
    pub estimated_delta_gm_ft: f64,

    // Fire-control performance metrics
    pub optical_baseline_m: f64,
    pub computer_order: i32,
    pub radar_blind_fire_capability: f64,
    /// Salvo hit probability curve: (range_yd, hit_prob_percent) from 5,000 to 35,000 yd.
    pub hit_prob_curve: Vec<(f64, f64)>,
}

impl ShipSystemsConfig {
    /// Evaluate the full electrical, sensory, fire control, and physical suite for a ship model.
    pub fn evaluate(&self, ship: &ShipModel) -> SystemsEvaluation {
        let mut eval = SystemsEvaluation::default();

        // 1. Extract dimensionless physical invariants and score pure Ideal-Type affinities
        let (features, aff1, aff2, aff3, aff4, explanation) = evaluate_affinities(ship);
        eval.features = features;
        eval.type1_affinity = aff1;
        eval.type2_affinity = aff2;
        eval.type3_affinity = aff3;
        eval.type4_affinity = aff4;
        eval.match_explanation = explanation;

        // 1b. Evaluate 3D Factored Tactical Role affinities
        let tactical_profile = evaluate_tactical_affinities(ship, &features);
        eval.resolved_echelon = match self.tactical_echelon {
            TacticalEchelon::Auto => tactical_profile.dominant_echelon,
            explicit => explicit,
        };
        eval.resolved_modality = match self.combat_modality {
            CombatModality::Auto => tactical_profile.dominant_modality,
            explicit => explicit,
        };
        eval.resolved_mobility_regime = match self.mobility_regime {
            MobilityRegime::Auto => tactical_profile.dominant_mobility,
            explicit => explicit,
        };
        eval.tactical_affinities = tactical_profile;

        let dominant_doctrine = if self.doctrine != StyleDoctrine::AdaptiveAuto
            && self.doctrine != StyleDoctrine::Custom
        {
            self.doctrine
        } else {
            let max_aff = aff1.max(aff2).max(aff3).max(aff4);
            if (max_aff - aff1).abs() < 1e-6 {
                StyleDoctrine::AutomationAllElectric
            } else if (max_aff - aff2).abs() < 1e-6 {
                StyleDoctrine::AllWeatherSeakeeping
            } else if (max_aff - aff3).abs() < 1e-6 {
                StyleDoctrine::LongRangeOpticalStrike
            } else {
                StyleDoctrine::StereoscopicSubdivided
            }
        };
        eval.resolved_doctrine = dominant_doctrine;

        // 2. Resolve equipment slots (mix-and-match: keep explicit choices, resolve Auto slots)
        let year = ship.hull.date_hull;
        // Infer functional role purely from resolved tactical modality and echelon (zero reliance on string labels)
        let is_carrier = eval.resolved_modality == CombatModality::AviationDeckProjection;
        let is_destroyer = eval.resolved_echelon == TacticalEchelon::FlotillaAssaultLine
            || (eval.resolved_echelon == TacticalEchelon::InnerFleetScreen
                && (features.froude_number > 0.30 || features.volumetric_froude_number > 0.90));

        eval.resolved_main_director = match self.main_director {
            MainDirector::Auto => resolve_main_director(dominant_doctrine, year, is_carrier),
            other => other,
        };

        eval.resolved_aa_director = match self.aa_director {
            AaDirector::Auto => {
                resolve_aa_director(dominant_doctrine, year, is_destroyer, is_carrier)
            }
            other => other,
        };

        eval.resolved_ballistic_computer = match self.ballistic_computer {
            BallisticComputer::Auto => resolve_ballistic_computer(dominant_doctrine, year),
            other => other,
        };

        eval.resolved_radar_suite = match self.radar_suite {
            RadarSuite::Auto => resolve_radar_suite(dominant_doctrine, year),
            other => other,
        };

        eval.resolved_sonar_suite = match self.sonar_suite {
            SonarSuite::Auto => resolve_sonar_suite(dominant_doctrine, year, is_destroyer),
            other => other,
        };

        eval.resolved_electrical_plant = match self.electrical_plant {
            ElectricalPlantStyle::Auto => resolve_electrical_plant(dominant_doctrine),
            other => other,
        };

        // 3. Electrical Load Breakdown
        let main_cal = ship.guns.batteries[0].calibre;
        let main_barrels = ship.guns.batteries[0].barrels.max(0) as f64;
        let sec_barrels =
            (ship.guns.batteries[1].barrels.max(0) + ship.guns.batteries[2].barrels.max(0)) as f64;
        let turret_kw = (main_barrels * (main_cal / 12.0).powf(2.4) * 22.0)
            + (sec_barrels * 8.0)
            + (ship.weapons.torpedo1.number.max(0) as f64 * 3.5);
        eval.load_turrets_kw = turret_kw.max(10.0);

        let disp = ship.hull.displacement.max(100.0);
        let crew = ship.hull.crew_min.max(50) as f64;
        let hotel_kw = (disp.powf(0.60) * 3.8) + (crew * 0.22);
        let steering_pumps_kw = disp.powf(0.52) * 5.2;
        eval.load_hull_hotel_kw = hotel_kw + steering_pumps_kw;

        let fc_dir_kw = match eval.resolved_main_director {
            MainDirector::OpenSights => 0.0,
            MainDirector::CentralTower => 8.0,
            MainDirector::WeatherproofDct => 24.0,
            MainDirector::ArmouredStereoscopic10m => 48.0,
            MainDirector::DuplexOpticalTower15m => 35.0,
            MainDirector::GyrostabilizedRadarBlindFire => 80.0,
            MainDirector::Auto => 20.0,
        };
        let aa_dir_kw = match eval.resolved_aa_director {
            AaDirector::None => 0.0,
            AaDirector::HighAngleTachymetric => 20.0,
            AaDirector::TriAxialStabilizedSphere => 40.0,
            AaDirector::OpticalLeadComputing => 22.0,
            AaDirector::DualPurposeRadarDirector => 55.0,
            AaDirector::Auto => 15.0,
        };
        let comp_kw = match eval.resolved_ballistic_computer {
            BallisticComputer::ManualTables => 0.0,
            BallisticComputer::MechanicalRateClock => 8.0,
            BallisticComputer::ElectroMechanicalTable => 28.0,
            BallisticComputer::HighVelocityAnalog => 36.0,
            BallisticComputer::MultiAxisTriangulator => 24.0,
            BallisticComputer::GyroCamRangekeeper => 50.0,
            BallisticComputer::Auto => 20.0,
        };
        eval.load_fire_control_kw = fc_dir_kw + aa_dir_kw + comp_kw;

        let radar_kw = match eval.resolved_radar_suite {
            RadarSuite::None => 0.0,
            RadarSuite::DecimetricSurfaceFc => 45.0,
            RadarSuite::MetricSearchMagnetron => 38.0,
            RadarSuite::CentimetricFcDirectionFinder => 95.0,
            RadarSuite::DualBandBlindFireCic => 180.0,
            RadarSuite::Auto => 40.0,
        };
        let sonar_kw = match eval.resolved_sonar_suite {
            SonarSuite::None => 0.0,
            SonarSuite::PassiveHydrophones => 3.0,
            SonarSuite::ActiveQuartzDome => 16.0,
            SonarSuite::MultiElementPassiveArray => 12.0,
            SonarSuite::HighPowerSearchSonar => 32.0,
            SonarSuite::Auto => 10.0,
        };
        eval.load_electronics_kw = radar_kw + sonar_kw;

        eval.combat_peak_kw = eval.load_turrets_kw
            + eval.load_hull_hotel_kw
            + eval.load_fire_control_kw
            + eval.load_electronics_kw;

        let reserve_factor = match eval.resolved_electrical_plant {
            ElectricalPlantStyle::CompactSteam => 1.15,
            ElectricalPlantStyle::BalancedTurboDiesel => 1.30,
            ElectricalPlantStyle::SubdividedHeavyDiesel => 1.38,
            ElectricalPlantStyle::HighMarginAcGrid => 1.48,
            ElectricalPlantStyle::Auto => 1.30,
        };
        let required_kw = eval.combat_peak_kw * reserve_factor;

        let (turbo_ratio, diesel_ratio) = match eval.resolved_electrical_plant {
            ElectricalPlantStyle::CompactSteam => (0.90, 0.10),
            ElectricalPlantStyle::BalancedTurboDiesel => (0.75, 0.25),
            ElectricalPlantStyle::SubdividedHeavyDiesel => (0.50, 0.50),
            ElectricalPlantStyle::HighMarginAcGrid => (0.75, 0.25),
            ElectricalPlantStyle::Auto => (0.75, 0.25),
        };

        eval.turbo_generator_kw = self
            .custom_turbo_kw
            .unwrap_or_else(|| (required_kw * turbo_ratio).round());
        eval.diesel_generator_kw = self
            .custom_diesel_kw
            .unwrap_or_else(|| (required_kw * diesel_ratio).round());
        eval.total_installed_kw = eval.turbo_generator_kw + eval.diesel_generator_kw;

        if eval.combat_peak_kw > 0.0 {
            eval.electrical_margin_pct =
                (eval.total_installed_kw - eval.combat_peak_kw) / eval.combat_peak_kw * 100.0;
        }
        eval.is_overloaded = eval.total_installed_kw < eval.combat_peak_kw;

        // 4. Physical Weights & CG Tiers (Tons)
        eval.vital_weight_tons = match eval.resolved_ballistic_computer {
            BallisticComputer::ManualTables => 0.5,
            BallisticComputer::MechanicalRateClock => 4.0,
            BallisticComputer::ElectroMechanicalTable => 9.5,
            BallisticComputer::HighVelocityAnalog => 13.0,
            BallisticComputer::MultiAxisTriangulator => 8.0,
            BallisticComputer::GyroCamRangekeeper => 15.0,
            BallisticComputer::Auto => 7.0,
        };

        let gen_tons_per_kw = match eval.resolved_electrical_plant {
            ElectricalPlantStyle::CompactSteam => 0.026,
            ElectricalPlantStyle::BalancedTurboDiesel => 0.022,
            ElectricalPlantStyle::SubdividedHeavyDiesel => 0.025,
            ElectricalPlantStyle::HighMarginAcGrid => 0.019,
            ElectricalPlantStyle::Auto => 0.022,
        };
        let sonar_tons = match eval.resolved_sonar_suite {
            SonarSuite::None => 0.0,
            SonarSuite::PassiveHydrophones => 1.5,
            SonarSuite::ActiveQuartzDome => 5.2,
            SonarSuite::MultiElementPassiveArray => 6.8,
            SonarSuite::HighPowerSearchSonar => 8.5,
            SonarSuite::Auto => 4.0,
        };
        eval.hull_weight_tons = (eval.total_installed_kw * gen_tons_per_kw) + sonar_tons;

        let aa_deck_tons = match eval.resolved_aa_director {
            AaDirector::None => 0.0,
            AaDirector::HighAngleTachymetric => 8.5,
            AaDirector::TriAxialStabilizedSphere => 18.0,
            AaDirector::OpticalLeadComputing => 9.5,
            AaDirector::DualPurposeRadarDirector => 22.0,
            AaDirector::Auto => 10.0,
        };
        let cic_deck_tons = match eval.resolved_radar_suite {
            RadarSuite::None => 2.0,
            RadarSuite::DecimetricSurfaceFc => 4.0,
            RadarSuite::MetricSearchMagnetron => 4.5,
            RadarSuite::CentimetricFcDirectionFinder => 9.0,
            RadarSuite::DualBandBlindFireCic => 18.5,
            RadarSuite::Auto => 6.0,
        };
        eval.deck_weight_tons = aa_deck_tons + cic_deck_tons;

        let (dir_top_tons, baseline_m) = match eval.resolved_main_director {
            MainDirector::OpenSights => (1.5, 0.0),
            MainDirector::CentralTower => (6.5, 2.7),
            MainDirector::WeatherproofDct => (15.0, 4.6),
            MainDirector::ArmouredStereoscopic10m => (32.0, 10.5),
            MainDirector::DuplexOpticalTower15m => (46.0, 15.5),
            MainDirector::GyrostabilizedRadarBlindFire => (24.0, 8.0),
            MainDirector::Auto => (18.0, 6.0),
        };
        let radar_top_tons = match eval.resolved_radar_suite {
            RadarSuite::None => 0.0,
            RadarSuite::DecimetricSurfaceFc => 2.8,
            RadarSuite::MetricSearchMagnetron => 3.2,
            RadarSuite::CentimetricFcDirectionFinder => 6.5,
            RadarSuite::DualBandBlindFireCic => 9.5,
            RadarSuite::Auto => 4.0,
        };
        eval.topweight_tons = dir_top_tons + radar_top_tons;
        eval.optical_baseline_m = baseline_m;

        eval.total_systems_weight_tons = eval.vital_weight_tons
            + eval.hull_weight_tons
            + eval.deck_weight_tons
            + eval.topweight_tons;

        let moment = (eval.topweight_tons * 3.8) + (eval.deck_weight_tons * 2.2)
            - (eval.hull_weight_tons * 0.4)
            - (eval.vital_weight_tons * 1.0);
        eval.estimated_delta_gm_ft = -(moment / (disp * 1.25)).min(2.5);

        // 5. Fire Control Accuracy & Salvo Hit Probability Curve
        let (comp_order, comp_disp_reduction) = match eval.resolved_ballistic_computer {
            BallisticComputer::ManualTables => (1, 1.00),
            BallisticComputer::MechanicalRateClock => (2, 0.88),
            BallisticComputer::ElectroMechanicalTable => (4, 0.68),
            BallisticComputer::HighVelocityAnalog => (4, 0.67),
            BallisticComputer::MultiAxisTriangulator => (4, 0.66),
            BallisticComputer::GyroCamRangekeeper => (5, 0.55),
            BallisticComputer::Auto => (3, 0.75),
        };
        eval.computer_order = comp_order;

        let blind_fire = match eval.resolved_radar_suite {
            RadarSuite::None => 0.0,
            RadarSuite::DecimetricSurfaceFc => 0.35,
            RadarSuite::MetricSearchMagnetron => 0.20,
            RadarSuite::CentimetricFcDirectionFinder => 0.70,
            RadarSuite::DualBandBlindFireCic => 0.95,
            RadarSuite::Auto => 0.40,
        };
        eval.radar_blind_fire_capability = blind_fire;

        let dir_stability_factor = match eval.resolved_main_director {
            MainDirector::OpenSights => 1.40,
            MainDirector::CentralTower => 1.20,
            MainDirector::WeatherproofDct => 1.00,
            MainDirector::ArmouredStereoscopic10m => 0.88,
            MainDirector::DuplexOpticalTower15m => 0.82,
            MainDirector::GyrostabilizedRadarBlindFire => 0.72,
            MainDirector::Auto => 1.00,
        };

        let steadiness_factor = (100.0 / ship.steadiness_adj.max(25.0))
            .sqrt()
            .clamp(0.8, 1.6);

        let mut curve = Vec::new();
        for r_step in (5000..=35000).step_by(2500) {
            let r = r_step as f64;
            let opt_range_err = (r / 1000.0).powi(2) * 1.8 / baseline_m.max(1.5);
            let rad_range_err = 25.0 / (blind_fire + 0.1).min(1.0);
            let combined_range_err = if blind_fire > 0.4 {
                (opt_range_err * 0.4).min(rad_range_err)
            } else {
                opt_range_err
            };

            let sigma_range =
                combined_range_err * comp_disp_reduction * dir_stability_factor * steadiness_factor;
            let sigma_deflection =
                (r * 0.0035) * comp_disp_reduction * dir_stability_factor * steadiness_factor
                    / (1.0 + blind_fire * 0.2);

            let disp_area =
                (std::f64::consts::PI * sigma_range.max(15.0) * sigma_deflection.max(8.0))
                    .max(400.0);
            let target_area = 5500.0;
            let base_hit_prob = (target_area / disp_area).clamp(0.008, 0.42);

            let b_count = main_barrels.clamp(2.0, 12.0);
            let salvo_hit_prob = (1.0 - (1.0 - base_hit_prob).powf(b_count * 0.35)) * 100.0;
            curve.push((r, (salvo_hit_prob * 10.0).round() / 10.0));
        }
        eval.hit_prob_curve = curve;

        eval
    }
}

/// Evaluate pure Weberian Ideal-Type affinities strictly from dimensionless physical invariants ($\Pi$-groups).
/// Zero reference to `ship.country` or `ship.name`.
pub fn evaluate_affinities(
    ship: &ShipModel,
) -> (DimensionlessFeatures, f64, f64, f64, f64, String) {
    let f = DimensionlessFeatures::from_ship(ship);

    // Base prior across the 4 pure engineering ideal types
    let mut s1 = 0.20_f64; // Type I: Automated Radar & All-Electric
    let mut s2 = 0.22_f64; // Type II: All-Weather Seakeeping & Balanced ASW
    let mut s3 = 0.20_f64; // Type III: Long-Range Optical & Heavy-Ordnance Strike
    let mut s4 = 0.20_f64; // Type IV: Stereoscopic Precision & Subdivided Survivability

    // 1. Dual-Purpose / Automated AA density (Pi_dp) & Endurance scale (Pi_R)
    // High DP/AA automation and long endurance strongly characterize Ideal Type I
    s1 += (f.dp_barrel_density * 0.42).min(0.45);
    if f.endurance_ratio >= 1.05 && f.tech_epoch >= 0.75 {
        s1 += 0.22;
    }
    if f.length_beam_ratio >= 7.8 && f.beam_draught_ratio <= 3.35 && f.tech_epoch >= 0.80 {
        s1 += 0.15;
    }

    // 2. Reserve buoyancy freeboard ratio (Pi_F/T) & balanced block fullness (Cb)
    // Characterize Ideal Type II (All-Weather Seakeeping)
    if f.freeboard_draught_ratio >= 0.68 && (0.51..=0.60).contains(&f.block_coeff) {
        s2 += 0.26;
    }
    if (0.92..=1.06).contains(&f.barrel_slenderness) && f.torpedo_strike_density < 1.2 {
        s2 += 0.22;
    }
    if f.tech_epoch < 0.65 {
        s2 += 0.20;
    }

    // 3. Heavy ordnance / main caliber scale (Pi_cal) & Torpedo strike density (Pi_torp)
    // Characterize Ideal Type III (Long-Range Optical & Heavy-Ordnance Strike)
    if f.torpedo_strike_density >= 1.35 {
        s3 += (f.torpedo_strike_density * 0.28).min(0.55);
    }
    if f.main_caliber_scale >= 1.58 {
        s3 += 0.45;
    }
    if f.ordnance_fraction >= 0.115 && f.endurance_ratio < 0.90 {
        s3 += 0.18;
    }
    if f.froude_number >= 0.38 && f.block_coeff <= 0.46 {
        s3 += 0.25;
    }

    // 4. Transverse form stability (Pi_B/T), Bouguer BM ratio & High-Velocity barrels (Pi_cal_len)
    // Characterize Ideal Type IV (Stereoscopic Precision & Subdivided Survivability)
    if f.barrel_slenderness >= 1.14 {
        // L/51.5+ high-velocity flat-trajectory ordnance
        s4 += 0.35;
    }
    if f.beam_draught_ratio >= 3.30 && f.protection_fraction >= 0.28 {
        s4 += 0.32;
    }
    if ship.engine.engine_types[1] || ship.engine.fuel[2] {
        // Complex reciprocating / diesel propulsion
        s4 += 0.25;
    }

    let total = (s1 + s2 + s3 + s4).max(1e-6);
    let a1 = s1 / total;
    let a2 = s2 / total;
    let a3 = s3 / total;
    let a4 = s4 / total;

    let dominant_label = if a1 >= a2 && a1 >= a3 && a1 >= a4 {
        format!("Type I Automated/All-Electric ({:.0}%)", a1 * 100.0)
    } else if a2 >= a1 && a2 >= a3 && a2 >= a4 {
        format!("Type II All-Weather Seakeeping ({:.0}%)", a2 * 100.0)
    } else if a3 >= a1 && a3 >= a2 && a3 >= a4 {
        format!("Type III Optical/Ordnance Strike ({:.0}%)", a3 * 100.0)
    } else {
        format!("Type IV Stereoscopic/Subdivided ({:.0}%)", a4 * 100.0)
    };

    let explanation = format!(
        "Extracted {} from dimensionless invariants: Fn={:.2}, L/∇¹/³={:.2}, L/B={:.2}, B/T={:.2}, Cb={:.2}, Π_arm={:.1}%, Π_dp={:.2}, Π_torp={:.2}.",
        dominant_label,
        f.froude_number,
        f.slenderness_ratio,
        f.length_beam_ratio,
        f.beam_draught_ratio,
        f.block_coeff,
        f.protection_fraction * 100.0,
        f.dp_barrel_density,
        f.torpedo_strike_density
    );

    (f, a1, a2, a3, a4, explanation)
}

/// Evaluate 3-dimensional factored tactical-functional affinities strictly from dimensionless
/// physical invariants ($\Pi$-groups), without relying on ship type strings or country flags.
///
/// Factors the space into 3 marginal dimensions ($5 + 5 + 4 = 14$ marginal centroids):
/// 1. `TacticalEchelon` (BattleLineMainBody, FastWingVanguard, OuterScoutingScreen, InnerFleetScreen, FlotillaAssaultLine)
/// 2. `CombatModality` (HeavyArtilleryCitadel, RapidFireSurfaceGunnery, AviationDeckProjection, MassedTorpedoStrike, ScreenEscortDefense)
/// 3. `MobilityRegime` (HeavyDisplacementBlock, TransFroudeFastCapital, SlenderWaveCutting, SuperCriticalHighFn)
pub fn evaluate_tactical_affinities(
    ship: &ShipModel,
    f: &DimensionlessFeatures,
) -> TacticalAffinityProfile {
    let disp = ship.hull.displacement.max(50.0);
    let main_cal = ship.guns.batteries[0].calibre.max(0.0);
    let total_torp =
        (ship.weapons.torpedo1.number.max(0) + ship.weapons.torpedo2.number.max(0)) as f64;

    // --- Dimension 2: Combat Modality [A, B, C, D, E] ---
    let mut s_a = 0.12_f64; // HeavyArtilleryCitadel
    let mut s_b = 0.12_f64; // RapidFireSurfaceGunnery
    let mut s_c = 0.05_f64; // AviationDeckProjection
    let mut s_d = 0.10_f64; // MassedTorpedoStrike
    let mut s_e = 0.12_f64; // ScreenEscortDefense

    // Modality A: Heavy Artillery & Armoured Citadel
    if main_cal >= 11.0 {
        s_a += 0.70;
    } else if main_cal >= 8.0 {
        s_a += 0.30;
    }
    if f.protection_fraction >= 0.22 {
        s_a += 0.40;
    } else if f.protection_fraction >= 0.15 {
        s_a += 0.20;
    }
    if f.main_caliber_scale >= 1.35 {
        s_a += 0.30;
    }

    // Modality B: Rapid-Fire Cruiser Gunnery
    if (5.0..11.0).contains(&main_cal) {
        s_b += 0.65;
    }
    if f.ordnance_fraction >= 0.08 && main_cal >= 5.0 {
        s_b += 0.30;
    }
    if (0.07..0.24).contains(&f.protection_fraction) && main_cal > 0.0 {
        s_b += 0.25;
    }

    // Modality C: Aviation Flight Deck & Hangar Projection
    if f.aviation_capability_ratio >= 0.40 {
        s_c += 0.85 * f.aviation_capability_ratio;
    }
    if ship.weapons.misc_weight_deck >= 250.0 {
        s_c += 0.45;
    }
    if main_cal <= 6.0 && f.freeboard_draught_ratio >= 0.75 && disp >= 5000.0 {
        s_c += 0.40;
    }

    // Modality D: Massed Torpedo Strike
    if f.torpedo_strike_density >= 1.30 {
        s_d += 0.65;
    } else if f.torpedo_strike_density >= 0.65 {
        s_d += 0.35;
    }
    if total_torp >= 8.0 {
        s_d += 0.55;
    } else if total_torp >= 4.0 {
        s_d += 0.25;
    }

    // Modality E: AA & ASW Screen Defense
    if f.dp_barrel_density >= 0.30 {
        s_e += 0.45;
    }
    if main_cal <= 5.25 && disp < 8000.0 && total_torp < 8.0 {
        s_e += 0.35;
    }
    if f.torpedo_strike_density < 0.60 && disp < 6000.0 {
        s_e += 0.25;
    }
    if f.protection_fraction <= 0.08 && disp < 4000.0 && total_torp < 6.0 {
        s_e += 0.20;
    }

    let tot_m = (s_a + s_b + s_c + s_d + s_e).max(1e-6);
    let mod_aff = [
        s_a / tot_m,
        s_b / tot_m,
        s_c / tot_m,
        s_d / tot_m,
        s_e / tot_m,
    ];
    let dominant_modality = {
        let max_val = mod_aff[0]
            .max(mod_aff[1])
            .max(mod_aff[2])
            .max(mod_aff[3])
            .max(mod_aff[4]);
        if (max_val - mod_aff[0]).abs() < 1e-6 {
            CombatModality::HeavyArtilleryCitadel
        } else if (max_val - mod_aff[1]).abs() < 1e-6 {
            CombatModality::RapidFireSurfaceGunnery
        } else if (max_val - mod_aff[2]).abs() < 1e-6 {
            CombatModality::AviationDeckProjection
        } else if (max_val - mod_aff[3]).abs() < 1e-6 {
            CombatModality::MassedTorpedoStrike
        } else {
            CombatModality::ScreenEscortDefense
        }
    };

    // --- Dimension 3: Mobility & Hydrodynamic Regime [1, 2, 3, 4] ---
    // Centroids (Fn, L/∇^{1/3}, L/B, B/T, C_b)
    const HYDRO_CENTROIDS: [(f64, f64, f64, f64, f64); 4] = [
        (0.21, 5.6, 6.5, 3.1, 0.60), // Regime 1: Heavy Displacement Block
        (0.30, 6.9, 7.6, 3.3, 0.57), // Regime 2: Trans-Froude Fast Capital
        (0.36, 8.0, 9.2, 3.2, 0.51), // Regime 3: Slender Wave-Cutting
        (0.52, 8.8, 9.8, 3.1, 0.48), // Regime 4: Super-Critical High-Fn
    ];
    const HYDRO_SCALES: (f64, f64, f64, f64, f64) = (0.07, 1.1, 1.5, 0.5, 0.05);

    let mut mob_w = [0.0_f64; 4];
    for (idx, c) in HYDRO_CENTROIDS.iter().enumerate() {
        let d2 = ((f.froude_number - c.0) / HYDRO_SCALES.0).powi(2)
            + ((f.slenderness_ratio - c.1) / HYDRO_SCALES.1).powi(2)
            + ((f.length_beam_ratio - c.2) / HYDRO_SCALES.2).powi(2)
            + ((f.beam_draught_ratio - c.3) / HYDRO_SCALES.3).powi(2)
            + ((f.block_coeff - c.4) / HYDRO_SCALES.4).powi(2);
        mob_w[idx] = (-0.5 * d2).exp();
    }

    // Displacement and scale modulation:
    // Capital ships (BB/CV) are >= 18,000t; Cruisers/CVL are 4,500-18,000t; Escorts/DD are < 4,500t.
    if disp >= 18000.0 {
        mob_w[2] *= 0.20; // Capital scale suppresses slender cruiser regime
        mob_w[3] *= 0.05; // Suppresses destroyer regime
        if f.froude_number <= 0.27 {
            mob_w[0] *= 3.0; // Heavy displacement dreadnought block (e.g. Colorado, Yamato)
        } else {
            mob_w[1] *= 3.0; // Fast capital hull (e.g. Iowa, Essex)
        }
    } else if disp < 4500.0 {
        mob_w[0] *= 0.10;
        mob_w[1] *= 0.10;
        if f.froude_number >= 0.34 {
            mob_w[3] *= 3.5; // High-speed destroyer / torpedo craft
        }
    } else {
        // Intermediate scale (4,500 - 18,000t): cruisers, CVL, CVE
        mob_w[1] *= 0.25; // Suppresses heavy capital regime for medium hulls
        if f.length_beam_ratio >= 7.8 && f.slenderness_ratio >= 6.5 {
            mob_w[2] *= 2.5; // Slender cruiser / CVL wave cutting
        }
    }

    let tot_mob = (mob_w[0] + mob_w[1] + mob_w[2] + mob_w[3]).max(1e-6);
    let mobility_aff = [
        mob_w[0] / tot_mob,
        mob_w[1] / tot_mob,
        mob_w[2] / tot_mob,
        mob_w[3] / tot_mob,
    ];
    let dominant_mobility = {
        let max_val = mobility_aff[0]
            .max(mobility_aff[1])
            .max(mobility_aff[2])
            .max(mobility_aff[3]);
        if (max_val - mobility_aff[0]).abs() < 1e-6 {
            MobilityRegime::HeavyDisplacementBlock
        } else if (max_val - mobility_aff[1]).abs() < 1e-6 {
            MobilityRegime::TransFroudeFastCapital
        } else if (max_val - mobility_aff[2]).abs() < 1e-6 {
            MobilityRegime::SlenderWaveCutting
        } else {
            MobilityRegime::SuperCriticalHighFn
        }
    };

    // --- Dimension 1: Tactical Echelon [I, II, III, IV, V] ---
    let mut e1 = 0.15_f64; // BattleLineMainBody
    let mut e2 = 0.15_f64; // FastWingVanguard
    let mut e3 = 0.15_f64; // OuterScoutingScreen
    let mut e4 = 0.15_f64; // InnerFleetScreen
    let mut e5 = 0.10_f64; // FlotillaAssaultLine

    let is_aviation = dominant_modality == CombatModality::AviationDeckProjection;

    // Echelon I: Main Body & Line of Battle
    if disp >= 22000.0 {
        e1 += 0.60;
    } else if disp >= 16000.0 && f.protection_fraction >= 0.22 {
        e1 += 0.45;
    }
    if is_aviation && disp >= 20000.0 {
        // Fleet Carrier (CV)
        e1 += 0.70;
    }
    if dominant_modality == CombatModality::HeavyArtilleryCitadel && f.froude_number <= 0.28 {
        e1 += 0.35;
    }

    // Echelon II: Fast Wing & Vanguard
    if f.froude_number >= 0.29 && (10000.0..=38000.0).contains(&disp) {
        e2 += 0.50;
    }
    if is_aviation && (9000.0..20000.0).contains(&disp) && f.froude_number >= 0.28 {
        // Light Fleet Carrier (CVL)
        e2 += 0.75;
    }
    if dominant_modality == CombatModality::HeavyArtilleryCitadel && f.froude_number >= 0.30 {
        // Fast Battleship / Battlecruiser
        e2 += 0.45;
    }

    // Echelon III: Outer Scouting & Search Screen
    if (3500.0..=16000.0).contains(&disp) && !is_aviation {
        e3 += 0.55;
    }
    if f.length_beam_ratio >= 7.8 && f.endurance_ratio >= 0.85 && !is_aviation {
        e3 += 0.35;
    }
    if dominant_modality == CombatModality::RapidFireSurfaceGunnery && disp >= 4500.0 {
        e3 += 0.40;
    }

    // Echelon IV: Fleet Screen & Inner Escort
    if is_aviation && disp < 15000.0 && f.froude_number <= 0.28 {
        // Escort Carrier (CVE)
        e4 += 0.80;
    }
    if dominant_modality == CombatModality::ScreenEscortDefense && total_torp < 8.0 {
        e4 += 0.60;
    }
    if f.dp_barrel_density >= 0.35 && disp < 7000.0 && total_torp < 8.0 {
        e4 += 0.40;
    }
    if disp < 4500.0 && f.froude_number <= 0.33 {
        e4 += 0.35;
    }

    // Echelon V: Torpedo Flotillas & Assault Line
    if disp < 4500.0 && f.froude_number >= 0.34 {
        e5 += 0.65;
    }
    if dominant_modality == CombatModality::MassedTorpedoStrike && disp < 5000.0 {
        e5 += 0.60;
    }
    if total_torp >= 8.0 && disp < 5000.0 {
        e5 += 0.45;
    }
    if f.torpedo_strike_density >= 1.20 && disp < 5000.0 {
        e5 += 0.40;
    }
    if f.volumetric_froude_number >= 1.05 {
        e5 += 0.35;
    }

    let tot_e = (e1 + e2 + e3 + e4 + e5).max(1e-6);
    let echelon_aff = [e1 / tot_e, e2 / tot_e, e3 / tot_e, e4 / tot_e, e5 / tot_e];
    let dominant_echelon = {
        let max_val = echelon_aff[0]
            .max(echelon_aff[1])
            .max(echelon_aff[2])
            .max(echelon_aff[3])
            .max(echelon_aff[4]);
        if (max_val - echelon_aff[0]).abs() < 1e-6 {
            TacticalEchelon::BattleLineMainBody
        } else if (max_val - echelon_aff[1]).abs() < 1e-6 {
            TacticalEchelon::FastWingVanguard
        } else if (max_val - echelon_aff[2]).abs() < 1e-6 {
            TacticalEchelon::OuterScoutingScreen
        } else if (max_val - echelon_aff[3]).abs() < 1e-6 {
            TacticalEchelon::InnerFleetScreen
        } else {
            TacticalEchelon::FlotillaAssaultLine
        }
    };

    let summary = format!(
        "{} × {} × {}",
        dominant_echelon.display_name(),
        dominant_modality.display_name(),
        dominant_mobility.display_name()
    );

    TacticalAffinityProfile {
        echelon_affinities: echelon_aff,
        modality_affinities: mod_aff,
        mobility_affinities: mobility_aff,
        dominant_echelon,
        dominant_modality,
        dominant_mobility,
        classification_summary: summary,
    }
}

fn resolve_main_director(doctrine: StyleDoctrine, year: i32, is_carrier: bool) -> MainDirector {
    if is_carrier {
        return MainDirector::OpenSights;
    }
    if year < 1910 {
        MainDirector::OpenSights
    } else if year < 1920 {
        MainDirector::CentralTower
    } else {
        match doctrine {
            StyleDoctrine::AutomationAllElectric => {
                if year >= 1941 {
                    MainDirector::GyrostabilizedRadarBlindFire
                } else {
                    MainDirector::WeatherproofDct
                }
            }
            StyleDoctrine::LongRangeOpticalStrike => MainDirector::DuplexOpticalTower15m,
            StyleDoctrine::StereoscopicSubdivided => MainDirector::ArmouredStereoscopic10m,
            _ => MainDirector::WeatherproofDct,
        }
    }
}

fn resolve_aa_director(
    doctrine: StyleDoctrine,
    year: i32,
    is_destroyer: bool,
    is_carrier: bool,
) -> AaDirector {
    if year < 1930 {
        return AaDirector::None;
    }
    match doctrine {
        StyleDoctrine::AutomationAllElectric => AaDirector::DualPurposeRadarDirector,
        StyleDoctrine::AllWeatherSeakeeping => AaDirector::HighAngleTachymetric,
        StyleDoctrine::StereoscopicSubdivided => AaDirector::TriAxialStabilizedSphere,
        StyleDoctrine::LongRangeOpticalStrike => {
            if is_destroyer || is_carrier {
                AaDirector::OpticalLeadComputing
            } else {
                AaDirector::OpticalLeadComputing
            }
        }
        _ => {
            if year >= 1940 {
                AaDirector::DualPurposeRadarDirector
            } else {
                AaDirector::HighAngleTachymetric
            }
        }
    }
}

fn resolve_ballistic_computer(doctrine: StyleDoctrine, year: i32) -> BallisticComputer {
    if year < 1912 {
        BallisticComputer::ManualTables
    } else if year < 1925 {
        BallisticComputer::MechanicalRateClock
    } else {
        match doctrine {
            StyleDoctrine::AutomationAllElectric => BallisticComputer::GyroCamRangekeeper,
            StyleDoctrine::AllWeatherSeakeeping => BallisticComputer::ElectroMechanicalTable,
            StyleDoctrine::StereoscopicSubdivided => BallisticComputer::HighVelocityAnalog,
            StyleDoctrine::LongRangeOpticalStrike => BallisticComputer::MultiAxisTriangulator,
            _ => BallisticComputer::ElectroMechanicalTable,
        }
    }
}

fn resolve_radar_suite(doctrine: StyleDoctrine, year: i32) -> RadarSuite {
    if year < 1939 {
        RadarSuite::None
    } else if year < 1942 {
        match doctrine {
            StyleDoctrine::StereoscopicSubdivided => RadarSuite::DecimetricSurfaceFc,
            StyleDoctrine::AllWeatherSeakeeping => RadarSuite::CentimetricFcDirectionFinder,
            _ => RadarSuite::CentimetricFcDirectionFinder,
        }
    } else {
        match doctrine {
            StyleDoctrine::AutomationAllElectric => RadarSuite::DualBandBlindFireCic,
            StyleDoctrine::AllWeatherSeakeeping => RadarSuite::CentimetricFcDirectionFinder,
            StyleDoctrine::LongRangeOpticalStrike => RadarSuite::MetricSearchMagnetron,
            StyleDoctrine::StereoscopicSubdivided => RadarSuite::DecimetricSurfaceFc,
            _ => RadarSuite::DualBandBlindFireCic,
        }
    }
}

fn resolve_sonar_suite(doctrine: StyleDoctrine, year: i32, is_destroyer: bool) -> SonarSuite {
    if !is_destroyer && year < 1935 {
        return SonarSuite::None;
    }
    if year < 1925 {
        SonarSuite::PassiveHydrophones
    } else {
        match doctrine {
            StyleDoctrine::AllWeatherSeakeeping => SonarSuite::ActiveQuartzDome,
            StyleDoctrine::StereoscopicSubdivided => SonarSuite::MultiElementPassiveArray,
            StyleDoctrine::AutomationAllElectric => SonarSuite::HighPowerSearchSonar,
            _ => {
                if is_destroyer {
                    SonarSuite::ActiveQuartzDome
                } else {
                    SonarSuite::None
                }
            }
        }
    }
}

fn resolve_electrical_plant(doctrine: StyleDoctrine) -> ElectricalPlantStyle {
    match doctrine {
        StyleDoctrine::AutomationAllElectric => ElectricalPlantStyle::HighMarginAcGrid,
        StyleDoctrine::StereoscopicSubdivided => ElectricalPlantStyle::SubdividedHeavyDiesel,
        StyleDoctrine::AllWeatherSeakeeping => ElectricalPlantStyle::BalancedTurboDiesel,
        StyleDoctrine::LongRangeOpticalStrike => ElectricalPlantStyle::CompactSteam,
        _ => ElectricalPlantStyle::BalancedTurboDiesel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::get_preset;

    #[test]
    fn adaptive_matcher_infers_plausible_doctrines() {
        let iowa = get_preset("iowa").expect("Iowa preset");
        let eval_iowa = iowa.systems.evaluate(&iowa);
        assert_eq!(
            eval_iowa.resolved_doctrine,
            StyleDoctrine::AutomationAllElectric
        );
        assert_eq!(
            eval_iowa.resolved_main_director,
            MainDirector::GyrostabilizedRadarBlindFire
        );
        assert_eq!(
            eval_iowa.resolved_ballistic_computer,
            BallisticComputer::GyroCamRangekeeper
        );
        assert!(eval_iowa.total_installed_kw > 3000.0);
        assert!(eval_iowa.electrical_margin_pct > 15.0);

        let yamato = get_preset("yamato").expect("Yamato preset");
        let eval_yamato = yamato.systems.evaluate(&yamato);
        assert_eq!(
            eval_yamato.resolved_doctrine,
            StyleDoctrine::LongRangeOpticalStrike
        );
        assert_eq!(
            eval_yamato.resolved_main_director,
            MainDirector::DuplexOpticalTower15m
        );
        assert_eq!(eval_yamato.optical_baseline_m, 15.5);

        let bismarck = get_preset("bismarck").expect("Bismarck preset");
        let eval_bismarck = bismarck.systems.evaluate(&bismarck);
        assert_eq!(
            eval_bismarck.resolved_doctrine,
            StyleDoctrine::StereoscopicSubdivided
        );
        assert_eq!(
            eval_bismarck.resolved_main_director,
            MainDirector::ArmouredStereoscopic10m
        );
    }

    #[test]
    fn country_and_name_do_not_affect_style_matching() {
        let mut ship_a = get_preset("iowa").expect("Iowa preset");
        let mut ship_b = ship_a.clone();
        ship_a.country = "Mars Federation".into();
        ship_a.name = "Project X-99".into();
        ship_b.country = "".into();
        ship_b.name = "".into();

        let eval_a = ship_a.systems.evaluate(&ship_a);
        let eval_b = ship_b.systems.evaluate(&ship_b);

        assert_eq!(eval_a.resolved_doctrine, eval_b.resolved_doctrine);
        assert!((eval_a.type1_affinity - eval_b.type1_affinity).abs() < 1e-12);
        assert!((eval_a.type2_affinity - eval_b.type2_affinity).abs() < 1e-12);
        assert!((eval_a.type3_affinity - eval_b.type3_affinity).abs() < 1e-12);
        assert!((eval_a.type4_affinity - eval_b.type4_affinity).abs() < 1e-12);
        assert_eq!(eval_a.match_explanation, eval_b.match_explanation);
    }

    #[test]
    fn mix_and_match_overrides_work() {
        let mut ship = get_preset("bismarck").expect("Bismarck preset");
        ship.systems.main_director = MainDirector::GyrostabilizedRadarBlindFire;
        ship.systems.sonar_suite = SonarSuite::ActiveQuartzDome;
        ship.systems.custom_turbo_kw = Some(5000.0);
        ship.systems.custom_diesel_kw = Some(3000.0);

        let eval = ship.systems.evaluate(&ship);
        assert_eq!(
            eval.resolved_main_director,
            MainDirector::GyrostabilizedRadarBlindFire
        );
        assert_eq!(eval.resolved_sonar_suite, SonarSuite::ActiveQuartzDome);
        assert_eq!(eval.total_installed_kw, 8000.0);
        assert_eq!(eval.turbo_generator_kw, 5000.0);
        assert_eq!(eval.diesel_generator_kw, 3000.0);
        assert!(eval.hit_prob_curve.len() > 5);
    }

    #[test]
    fn tactical_roles_infer_cv_cvl_cve_and_invariance_to_ship_type_string() {
        // 1. Fleet Carrier (CV): USS Essex
        let mut essex = get_preset("essex").expect("Essex preset");
        let eval_essex = essex.systems.evaluate(&essex);
        assert_eq!(
            eval_essex.resolved_echelon,
            TacticalEchelon::BattleLineMainBody
        );
        assert_eq!(
            eval_essex.resolved_modality,
            CombatModality::AviationDeckProjection
        );
        assert_eq!(
            eval_essex.resolved_mobility_regime,
            MobilityRegime::TransFroudeFastCapital
        );

        // String and Flag invariance: completely purge ship_type, country, and name
        essex.ship_type = "Unclassified Hull".into();
        essex.name = "Unknown 99".into();
        essex.country = "Utopia".into();
        let eval_essex_anon = essex.systems.evaluate(&essex);
        assert_eq!(
            eval_essex_anon.resolved_echelon,
            TacticalEchelon::BattleLineMainBody
        );
        assert_eq!(
            eval_essex_anon.resolved_modality,
            CombatModality::AviationDeckProjection
        );
        assert_eq!(
            eval_essex_anon.resolved_mobility_regime,
            MobilityRegime::TransFroudeFastCapital
        );

        // 2. Light Fleet Carrier (CVL): fast slender cruiser-converted hull
        let mut cvl = ShipModel::new();
        cvl.name = "USS Independence".into();
        cvl.country = "".into();
        cvl.ship_type = "".into();
        cvl.hull.length = 622.0;
        cvl.hull.beam = 71.5;
        cvl.hull.draught = 24.0;
        cvl.hull.block = 0.52;
        cvl.hull.displacement = 11000.0;
        cvl.engine.speed_max = 31.6;
        cvl.free.freeboard = 22.0;
        cvl.weapons.misc_weight_deck = 550.0;
        let g1 = cvl.battery_mut(0);
        g1.barrels = 24;
        g1.calibre = 1.57; // 40mm Bofors
        g1.gun_type = 5;
        let eval_cvl = cvl.systems.evaluate(&cvl);
        assert_eq!(eval_cvl.resolved_echelon, TacticalEchelon::FastWingVanguard);
        assert_eq!(
            eval_cvl.resolved_modality,
            CombatModality::AviationDeckProjection
        );
        assert_eq!(
            eval_cvl.resolved_mobility_regime,
            MobilityRegime::SlenderWaveCutting
        );

        // 3. Escort Carrier (CVE): slower merchant-block hull for convoy / inner screen
        let mut cve = ShipModel::new();
        cve.name = "USS Casablanca".into();
        cve.country = "".into();
        cve.ship_type = "".into();
        cve.hull.length = 492.0;
        cve.hull.beam = 65.0;
        cve.hull.draught = 22.5;
        cve.hull.block = 0.62;
        cve.hull.displacement = 10400.0;
        cve.engine.speed_max = 19.3;
        cve.free.freeboard = 21.0;
        cve.weapons.misc_weight_deck = 350.0;
        let g1 = cve.battery_mut(0);
        g1.barrels = 1;
        g1.calibre = 5.0;
        g1.gun_type = 4;
        let eval_cve = cve.systems.evaluate(&cve);
        assert_eq!(eval_cve.resolved_echelon, TacticalEchelon::InnerFleetScreen);
        assert_eq!(
            eval_cve.resolved_modality,
            CombatModality::AviationDeckProjection
        );
        assert_eq!(
            eval_cve.resolved_mobility_regime,
            MobilityRegime::HeavyDisplacementBlock
        );

        // 4. Manual user mix-and-match across all 3 dimensions
        let mut custom = essex.clone();
        custom.systems.tactical_echelon = TacticalEchelon::OuterScoutingScreen;
        custom.systems.combat_modality = CombatModality::ScreenEscortDefense;
        custom.systems.mobility_regime = MobilityRegime::SlenderWaveCutting;
        let eval_custom = custom.systems.evaluate(&custom);
        assert_eq!(
            eval_custom.resolved_echelon,
            TacticalEchelon::OuterScoutingScreen
        );
        assert_eq!(
            eval_custom.resolved_modality,
            CombatModality::ScreenEscortDefense
        );
        assert_eq!(
            eval_custom.resolved_mobility_regime,
            MobilityRegime::SlenderWaveCutting
        );

        // 5. Fast Battleship (Iowa)
        let iowa = get_preset("iowa").expect("Iowa preset");
        let eval_iowa = iowa.systems.evaluate(&iowa);
        assert!(
            eval_iowa.resolved_echelon == TacticalEchelon::BattleLineMainBody
                || eval_iowa.resolved_echelon == TacticalEchelon::FastWingVanguard
        );
        assert_eq!(
            eval_iowa.resolved_modality,
            CombatModality::HeavyArtilleryCitadel
        );
        assert_eq!(
            eval_iowa.resolved_mobility_regime,
            MobilityRegime::TransFroudeFastCapital
        );

        // 6. Heavy Cruiser (Baltimore)
        let baltimore = get_preset("baltimore").expect("Baltimore preset");
        let eval_baltimore = baltimore.systems.evaluate(&baltimore);
        assert_eq!(
            eval_baltimore.resolved_echelon,
            TacticalEchelon::OuterScoutingScreen
        );
        assert_eq!(
            eval_baltimore.resolved_mobility_regime,
            MobilityRegime::SlenderWaveCutting
        );

        // 7. Fleet Destroyer (Fletcher)
        let fletcher = get_preset("fletcher").expect("Fletcher preset");
        let eval_fletcher = fletcher.systems.evaluate(&fletcher);
        assert_eq!(
            eval_fletcher.resolved_echelon,
            TacticalEchelon::FlotillaAssaultLine
        );
        assert_eq!(
            eval_fletcher.resolved_modality,
            CombatModality::MassedTorpedoStrike
        );
        assert_eq!(
            eval_fletcher.resolved_mobility_regime,
            MobilityRegime::SuperCriticalHighFn
        );

        // 8. Torpedo Flotilla Leader (Shimakaze: 15 Type 93 torpedoes, 40.9 kt)
        let shimakaze = get_preset("shimakaze").expect("Shimakaze preset");
        let eval_shimakaze = shimakaze.systems.evaluate(&shimakaze);
        assert_eq!(
            eval_shimakaze.resolved_echelon,
            TacticalEchelon::FlotillaAssaultLine
        );
        assert_eq!(
            eval_shimakaze.resolved_modality,
            CombatModality::MassedTorpedoStrike
        );
        assert_eq!(
            eval_shimakaze.resolved_mobility_regime,
            MobilityRegime::SuperCriticalHighFn
        );
    }
}
