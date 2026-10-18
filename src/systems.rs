//! Ship electrical plant, fire-control directors, ballistic computers,
//! radar/sensors, and adaptive doctrine style matching.
//!
//! Provides both an intelligent adaptive feature matching engine (which infers
//! historical technological doctrine from hull, era, scale, armament, and
//! stability margin) and complete mix-and-match freedom for individual subsystem slots.

use crate::model::ShipModel;

/// Naval engineering doctrine / style archetype.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StyleDoctrine {
    /// Inferred automatically from ship features, era, dimensions, armament, and country.
    #[default]
    AdaptiveAuto,
    /// United States Navy: High electrical power, heavy radar, Mk 38/37 directors, Mk 1A computer, CIC.
    UsnRadarCentric,
    /// Royal Navy: Balanced steam/diesel, DCT/HACS, AFCT table, centimetric radar, standard ASDIC sonar.
    RnAllWeather,
    /// Imperial Japanese Navy: Pagoda mast, massive 10m–15.5m optical rangefinders, Type 98 table, compact dynamos.
    IjnOpticalLongRange,
    /// Kriegsmarine: Triple stereoscopic armored turrets, C/38 analog computer, heavy multi-room diesel plant.
    KmStereoscopicDiesel,
    /// User custom mix-and-match combination.
    Custom,
}

impl StyleDoctrine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AdaptiveAuto => "adaptive_auto",
            Self::UsnRadarCentric => "usn_radar_centric",
            Self::RnAllWeather => "rn_all_weather",
            Self::IjnOpticalLongRange => "ijn_optical_long_range",
            Self::KmStereoscopicDiesel => "km_stereoscopic_diesel",
            Self::Custom => "custom",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AdaptiveAuto => "Adaptive Auto (AI Feature Match)",
            Self::UsnRadarCentric => "USN Radar-Centric & All-Electric",
            Self::RnAllWeather => "RN All-Weather & ASDIC Balanced",
            Self::IjnOpticalLongRange => "IJN Long-Range Optical & Night Fighter",
            Self::KmStereoscopicDiesel => "KM Stereoscopic & Heavy Diesel Plant",
            Self::Custom => "Custom Mix-and-Match",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "usn" | "usn_radar_centric" | "usn_radar" => Self::UsnRadarCentric,
            "rn" | "rn_all_weather" | "royal_navy" => Self::RnAllWeather,
            "ijn" | "ijn_optical_long_range" | "ijn_optical" => Self::IjnOpticalLongRange,
            "km" | "km_stereoscopic_diesel" | "km_stereoscopic" => Self::KmStereoscopicDiesel,
            "custom" | "manual" => Self::Custom,
            _ => Self::AdaptiveAuto,
        }
    }
}

/// Main Battery Fire-Control Director.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MainDirector {
    #[default]
    Auto,
    /// Open sights / individual turret rangefinder hoods (pre-dreadnought / early 1900s).
    OpenSights,
    /// Early central sighting tripod tower + 2.7m coincidence rangefinder (1906–1914).
    CentralTower,
    /// Royal Navy Director Control Tower (DCT) + 4.6m/9m coincidence rangefinder (1915–1938).
    RnDctOptical,
    /// Kriegsmarine 10.5m stereoscopic armored rotating dome (Drehhaube) (1934–1942).
    KmStereoscopic10m,
    /// IJN Pagoda mast tower + 10m–15.5m stereoscopic/coincidence rangefinder (1930–1941).
    IjnPagoda15m,
    /// USN Mk 34 / Mk 38 stabilized director + Mk 8/13 centimetric fire-control radar (1941–1945).
    UsnMk38Radar,
}

impl MainDirector {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::OpenSights => "open_sights",
            Self::CentralTower => "central_tower",
            Self::RnDctOptical => "rn_dct_optical",
            Self::KmStereoscopic10m => "km_stereoscopic_10m",
            Self::IjnPagoda15m => "ijn_pagoda_15m",
            Self::UsnMk38Radar => "usn_mk38_radar",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Adaptive)",
            Self::OpenSights => "Open Sights & Local Hoods (1890–1905)",
            Self::CentralTower => "Central Tower + 2.7m Rangefinder (1906–1914)",
            Self::RnDctOptical => "RN DCT + 4.6m/9m Coincidence Director (1915–1938)",
            Self::KmStereoscopic10m => "KM 10.5m Stereoscopic Armored Dome (1934–1942)",
            Self::IjnPagoda15m => "IJN 15m Giant Pagoda Optical Tower (1930–1941)",
            Self::UsnMk38Radar => "USN Mk 38 Stabilized Radar Blind-Fire Director (1941–1945)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "open_sights" | "open" => Self::OpenSights,
            "central_tower" | "central" => Self::CentralTower,
            "rn_dct_optical" | "rn_dct" | "dct" => Self::RnDctOptical,
            "km_stereoscopic_10m" | "km_10m" | "drehhaube" => Self::KmStereoscopic10m,
            "ijn_pagoda_15m" | "ijn_15m" | "pagoda" => Self::IjnPagoda15m,
            "usn_mk38_radar" | "usn_mk38" | "mk38" => Self::UsnMk38Radar,
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
    /// RN High Angle Control System (HACS) (1930–1942).
    RnHacs,
    /// KM Wackeltopf tri-axially stabilized spherical director (1935–1943).
    KmWackeltopf,
    /// IJN Type 94 Kosha Sochi high-angle director (1937–1944).
    IjnType94,
    /// USN Mk 37 Dual-Purpose Director with Mk 12/22 radar (1939–1945).
    UsnMk37,
}

impl AaDirector {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::RnHacs => "rn_hacs",
            Self::KmWackeltopf => "km_wackeltopf",
            Self::IjnType94 => "ijn_type94",
            Self::UsnMk37 => "usn_mk37",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Adaptive)",
            Self::None => "None / Local Ring Sights",
            Self::RnHacs => "RN HACS High-Angle Control System (1930–1942)",
            Self::KmWackeltopf => "KM Wackeltopf Tri-Axial Stabilized Sphere (1935–1943)",
            Self::IjnType94 => "IJN Type 94 High-Angle Director (1937–1944)",
            Self::UsnMk37 => "USN Mk 37 DP Director with Radar (1939–1945)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "none" | "no" => Self::None,
            "rn_hacs" | "hacs" => Self::RnHacs,
            "km_wackeltopf" | "wackeltopf" => Self::KmWackeltopf,
            "ijn_type94" | "type94" => Self::IjnType94,
            "usn_mk37" | "mk37" => Self::UsnMk37,
            _ => Self::Auto,
        }
    }
}

/// Ballistic Computer / Transmitting Station / Plotting Room.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BallisticComputer {
    #[default]
    Auto,
    /// Manual firing tables & Dumaresq slide rules (pre-1910).
    ManualTables,
    /// Vickers range clock / Dreyer mechanical integrator table (1911–1924).
    VickersDreyer,
    /// RN Admiralty Fire Control Table (AFCT Mk I–X) electro-mechanical (1925–1943).
    RnAfct,
    /// KM C/38 Artillerierechenstelle analog fire-control table (1935–1942).
    KmC38Analog,
    /// IJN Type 92 / Type 98 Hoiban fire-control table (1932–1941).
    IjnType98Hoiban,
    /// USN Ford Mk 1 / Mk 1A Rangekeeper 3D-cam mechanical computer (1938–1945).
    UsnFordMk1a,
}

impl BallisticComputer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ManualTables => "manual_tables",
            Self::VickersDreyer => "vickers_dreyer",
            Self::RnAfct => "rn_afct",
            Self::KmC38Analog => "km_c38_analog",
            Self::IjnType98Hoiban => "ijn_type98_hoiban",
            Self::UsnFordMk1a => "usn_ford_mk1a",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Adaptive)",
            Self::ManualTables => "Manual Tables & Slide Rules (1890–1909)",
            Self::VickersDreyer => "Vickers Clock & Dreyer Table (1910–1924)",
            Self::RnAfct => "RN Admiralty AFCT Electro-Mechanical Table (1925–1943)",
            Self::KmC38Analog => "KM C/38 Analog Plotting Table (1935–1942)",
            Self::IjnType98Hoiban => "IJN Type 98 Hoiban & Sokutekiban (1932–1941)",
            Self::UsnFordMk1a => "USN Ford Mk 1A Gyro-Coupled Computer (1938–1945)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "manual_tables" | "manual" => Self::ManualTables,
            "vickers_dreyer" | "dreyer" | "vickers" => Self::VickersDreyer,
            "rn_afct" | "afct" => Self::RnAfct,
            "km_c38_analog" | "km_c38" | "c38" => Self::KmC38Analog,
            "ijn_type98_hoiban" | "ijn_type98" | "hoiban" => Self::IjnType98Hoiban,
            "usn_ford_mk1a" | "ford_mk1a" | "mk1a" => Self::UsnFordMk1a,
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
    /// KM FuMO decimetric fire-control + Metox passive radar warning (1939–1943).
    KmFumoDecimetric,
    /// IJN Type 21/22 surface/air search + E-27 radar detector (1942–1944).
    IjnType21_22,
    /// RN Type 279/281 air search + Type 274/284 centimetric fire-control + HF/DF (1940–1944).
    RnCentimetricHfdf,
    /// USN SK/SG air/surface search + Mk 8/13 centimetric blind-fire + Full CIC (1942–1945).
    UsnCentimetricCic,
}

impl RadarSuite {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::KmFumoDecimetric => "km_fumo_decimetric",
            Self::IjnType21_22 => "ijn_type21_22",
            Self::RnCentimetricHfdf => "rn_centimetric_hfdf",
            Self::UsnCentimetricCic => "usn_centimetric_cic",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Adaptive)",
            Self::None => "None (Optical & Telegraph Only)",
            Self::KmFumoDecimetric => "KM FuMO Decimetric Radar + Metox (1939–1943)",
            Self::IjnType21_22 => "IJN Type 21/22 Air/Surface Radar (1942–1944)",
            Self::RnCentimetricHfdf => "RN Type 274/284 Centimetric Radar + HF/DF (1940–1944)",
            Self::UsnCentimetricCic => "USN SK/SG Search + Mk 8/13 Radar + Full CIC (1942–1945)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "none" | "no" => Self::None,
            "km_fumo_decimetric" | "km_fumo" | "fumo" => Self::KmFumoDecimetric,
            "ijn_type21_22" | "ijn_type21" | "type21" => Self::IjnType21_22,
            "rn_centimetric_hfdf" | "rn_radar" | "hfdf" => Self::RnCentimetricHfdf,
            "usn_centimetric_cic" | "usn_radar" | "cic" => Self::UsnCentimetricCic,
            _ => Self::Auto,
        }
    }
}

/// Sonar / ASDIC Subsurface Detection Suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SonarSuite {
    #[default]
    Auto,
    /// None (no subsurface acoustic sensors).
    None,
    /// Passive hydrophone listening array (1915–1930).
    PassiveHydrophones,
    /// RN Standard ASDIC active quartz transducer dome (1934–1945).
    RnAsdic,
    /// KM Gruppenhorchgerät (GHG) multi-element passive array + S-Gerät (1935–1944).
    KmGhgArray,
    /// USN QC / QGA high-power searchlight active sonar (1939–1945).
    UsnQcSonar,
}

impl SonarSuite {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::PassiveHydrophones => "passive_hydrophones",
            Self::RnAsdic => "rn_asdic",
            Self::KmGhgArray => "km_ghg_array",
            Self::UsnQcSonar => "usn_qc_sonar",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Adaptive)",
            Self::None => "None",
            Self::PassiveHydrophones => "Passive Hydrophone Array (1915–1930)",
            Self::RnAsdic => "RN Standard ASDIC Active Quartz Dome (1934–1945)",
            Self::KmGhgArray => "KM GHG Passive Listening Array + S-Gerät (1935–1944)",
            Self::UsnQcSonar => "USN QC/QGA High-Power Search Sonar (1939–1945)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "none" | "no" => Self::None,
            "passive_hydrophones" | "hydrophones" => Self::PassiveHydrophones,
            "rn_asdic" | "asdic" => Self::RnAsdic,
            "km_ghg_array" | "ghg" => Self::KmGhgArray,
            "usn_qc_sonar" | "usn_qc" | "qc" => Self::UsnQcSonar,
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
    /// RN balanced steam turbo-generators + emergency diesel generators (25–30% reserve).
    RnBalanced,
    /// KM heavy survivability multi-compartment diesel/turbo plant (30–35% reserve).
    KmHeavyDiesel,
    /// USN high-margin 450V AC turbo-generator plant + robust diesel backup (40–50% reserve).
    UsnHighMarginAc,
}

impl ElectricalPlantStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::CompactSteam => "compact_steam",
            Self::RnBalanced => "rn_balanced",
            Self::KmHeavyDiesel => "km_heavy_diesel",
            Self::UsnHighMarginAc => "usn_high_margin_ac",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Adaptive)",
            Self::CompactSteam => "Compact Steam Dynamos (10–15% Reserve)",
            Self::RnBalanced => "RN Balanced Turbo & Diesel Plant (25–30% Reserve)",
            Self::KmHeavyDiesel => "KM Multi-Room Heavy Diesel Plant (30–35% Reserve)",
            Self::UsnHighMarginAc => "USN High-Margin All-Electric AC Plant (40–50% Reserve)",
        }
    }

    pub fn from_str_name(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "compact_steam" | "compact" => Self::CompactSteam,
            "rn_balanced" | "balanced" => Self::RnBalanced,
            "km_heavy_diesel" | "diesel" => Self::KmHeavyDiesel,
            "usn_high_margin_ac" | "usn_ac" | "ac" => Self::UsnHighMarginAc,
            _ => Self::Auto,
        }
    }
}

/// Full configuration for a ship's systems suite.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ShipSystemsConfig {
    pub doctrine: StyleDoctrine,
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
    // Inferred doctrine and resolved equipment
    pub resolved_doctrine: StyleDoctrine,
    pub resolved_main_director: MainDirector,
    pub resolved_aa_director: AaDirector,
    pub resolved_ballistic_computer: BallisticComputer,
    pub resolved_radar_suite: RadarSuite,
    pub resolved_sonar_suite: SonarSuite,
    pub resolved_electrical_plant: ElectricalPlantStyle,

    // Style match affinities (0.0 to 1.0)
    pub usn_affinity: f64,
    pub rn_affinity: f64,
    pub ijn_affinity: f64,
    pub km_affinity: f64,
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

        // 1. Run adaptive feature matcher to score doctrine affinities
        let (aff_usn, aff_rn, aff_ijn, aff_km, explanation) = evaluate_affinities(ship);
        eval.usn_affinity = aff_usn;
        eval.rn_affinity = aff_rn;
        eval.ijn_affinity = aff_ijn;
        eval.km_affinity = aff_km;
        eval.match_explanation = explanation;

        let dominant_doctrine = if self.doctrine != StyleDoctrine::AdaptiveAuto
            && self.doctrine != StyleDoctrine::Custom
        {
            self.doctrine
        } else {
            let max_aff = aff_usn.max(aff_rn).max(aff_ijn).max(aff_km);
            if (max_aff - aff_usn).abs() < 1e-4 {
                StyleDoctrine::UsnRadarCentric
            } else if (max_aff - aff_rn).abs() < 1e-4 {
                StyleDoctrine::RnAllWeather
            } else if (max_aff - aff_ijn).abs() < 1e-4 {
                StyleDoctrine::IjnOpticalLongRange
            } else {
                StyleDoctrine::KmStereoscopicDiesel
            }
        };
        eval.resolved_doctrine = dominant_doctrine;

        // 2. Resolve equipment slots (mix-and-match: keep explicit choices, resolve Auto slots)
        let year = ship.hull.date_hull;
        let is_carrier = ship.ship_type.to_lowercase().contains("carrier");
        let is_destroyer = ship.ship_type.to_lowercase().contains("destroyer");
        let _is_cruiser = ship.ship_type.to_lowercase().contains("cruiser");

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
        // Turrets: depends on calibre and barrels
        let main_cal = ship.guns.batteries[0].calibre;
        let main_barrels = ship.guns.batteries[0].barrels.max(0) as f64;
        let sec_barrels =
            (ship.guns.batteries[1].barrels.max(0) + ship.guns.batteries[2].barrels.max(0)) as f64;
        let turret_kw = (main_barrels * (main_cal / 12.0).powf(2.4) * 22.0)
            + (sec_barrels * 8.0)
            + (ship.weapons.torpedo1.number.max(0) as f64 * 3.5);
        eval.load_turrets_kw = turret_kw.max(10.0);

        // Hull / hotel / pumps / steering: scales with displacement and crew
        let disp = ship.hull.displacement.max(100.0);
        let crew = ship.hull.crew_min.max(50) as f64;
        let hotel_kw = (disp.powf(0.60) * 3.8) + (crew * 0.22);
        let steering_pumps_kw = disp.powf(0.52) * 5.2;
        eval.load_hull_hotel_kw = hotel_kw + steering_pumps_kw;

        // Fire-control director & computer load
        let fc_dir_kw = match eval.resolved_main_director {
            MainDirector::OpenSights => 0.0,
            MainDirector::CentralTower => 8.0,
            MainDirector::RnDctOptical => 24.0,
            MainDirector::KmStereoscopic10m => 48.0,
            MainDirector::IjnPagoda15m => 35.0,
            MainDirector::UsnMk38Radar => 80.0,
            MainDirector::Auto => 20.0,
        };
        let aa_dir_kw = match eval.resolved_aa_director {
            AaDirector::None => 0.0,
            AaDirector::RnHacs => 20.0,
            AaDirector::KmWackeltopf => 40.0,
            AaDirector::IjnType94 => 22.0,
            AaDirector::UsnMk37 => 55.0,
            AaDirector::Auto => 15.0,
        };
        let comp_kw = match eval.resolved_ballistic_computer {
            BallisticComputer::ManualTables => 0.0,
            BallisticComputer::VickersDreyer => 8.0,
            BallisticComputer::RnAfct => 28.0,
            BallisticComputer::KmC38Analog => 36.0,
            BallisticComputer::IjnType98Hoiban => 24.0,
            BallisticComputer::UsnFordMk1a => 50.0,
            BallisticComputer::Auto => 20.0,
        };
        eval.load_fire_control_kw = fc_dir_kw + aa_dir_kw + comp_kw;

        // Radar & Sonar & CIC electronics load
        let radar_kw = match eval.resolved_radar_suite {
            RadarSuite::None => 0.0,
            RadarSuite::KmFumoDecimetric => 45.0,
            RadarSuite::IjnType21_22 => 38.0,
            RadarSuite::RnCentimetricHfdf => 95.0,
            RadarSuite::UsnCentimetricCic => 180.0,
            RadarSuite::Auto => 40.0,
        };
        let sonar_kw = match eval.resolved_sonar_suite {
            SonarSuite::None => 0.0,
            SonarSuite::PassiveHydrophones => 3.0,
            SonarSuite::RnAsdic => 16.0,
            SonarSuite::KmGhgArray => 12.0,
            SonarSuite::UsnQcSonar => 32.0,
            SonarSuite::Auto => 10.0,
        };
        eval.load_electronics_kw = radar_kw + sonar_kw;

        eval.combat_peak_kw = eval.load_turrets_kw
            + eval.load_hull_hotel_kw
            + eval.load_fire_control_kw
            + eval.load_electronics_kw;

        // Generator Installed Capacity
        let reserve_factor = match eval.resolved_electrical_plant {
            ElectricalPlantStyle::CompactSteam => 1.15,
            ElectricalPlantStyle::RnBalanced => 1.30,
            ElectricalPlantStyle::KmHeavyDiesel => 1.38,
            ElectricalPlantStyle::UsnHighMarginAc => 1.48,
            ElectricalPlantStyle::Auto => 1.30,
        };
        let required_kw = eval.combat_peak_kw * reserve_factor;

        let (turbo_ratio, diesel_ratio) = match eval.resolved_electrical_plant {
            ElectricalPlantStyle::CompactSteam => (0.90, 0.10),
            ElectricalPlantStyle::RnBalanced => (0.75, 0.25),
            ElectricalPlantStyle::KmHeavyDiesel => (0.50, 0.50),
            ElectricalPlantStyle::UsnHighMarginAc => (0.75, 0.25),
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
        // Vital: Ballistic computers, transmitting room, master gyros
        eval.vital_weight_tons = match eval.resolved_ballistic_computer {
            BallisticComputer::ManualTables => 0.5,
            BallisticComputer::VickersDreyer => 4.0,
            BallisticComputer::RnAfct => 9.5,
            BallisticComputer::KmC38Analog => 13.0,
            BallisticComputer::IjnType98Hoiban => 8.0,
            BallisticComputer::UsnFordMk1a => 15.0,
            BallisticComputer::Auto => 7.0,
        };

        // Hull: Generators, main switchboards, sonar dome
        let gen_tons_per_kw = match eval.resolved_electrical_plant {
            ElectricalPlantStyle::CompactSteam => 0.026,
            ElectricalPlantStyle::RnBalanced => 0.022,
            ElectricalPlantStyle::KmHeavyDiesel => 0.025,
            ElectricalPlantStyle::UsnHighMarginAc => 0.019,
            ElectricalPlantStyle::Auto => 0.022,
        };
        let sonar_tons = match eval.resolved_sonar_suite {
            SonarSuite::None => 0.0,
            SonarSuite::PassiveHydrophones => 1.5,
            SonarSuite::RnAsdic => 5.2,
            SonarSuite::KmGhgArray => 6.8,
            SonarSuite::UsnQcSonar => 8.5,
            SonarSuite::Auto => 4.0,
        };
        eval.hull_weight_tons = (eval.total_installed_kw * gen_tons_per_kw) + sonar_tons;

        // Deck: Secondary/AA directors, CIC, wireless telegraph room
        let aa_deck_tons = match eval.resolved_aa_director {
            AaDirector::None => 0.0,
            AaDirector::RnHacs => 8.5,
            AaDirector::KmWackeltopf => 18.0,
            AaDirector::IjnType94 => 9.5,
            AaDirector::UsnMk37 => 22.0,
            AaDirector::Auto => 10.0,
        };
        let cic_deck_tons = match eval.resolved_radar_suite {
            RadarSuite::None => 2.0,
            RadarSuite::KmFumoDecimetric => 4.0,
            RadarSuite::IjnType21_22 => 4.5,
            RadarSuite::RnCentimetricHfdf => 9.0,
            RadarSuite::UsnCentimetricCic => 18.5,
            RadarSuite::Auto => 6.0,
        };
        eval.deck_weight_tons = aa_deck_tons + cic_deck_tons;

        // Topweight (Masthead): Main director, optical rangefinders, radar antennas
        let (dir_top_tons, baseline_m) = match eval.resolved_main_director {
            MainDirector::OpenSights => (1.5, 0.0),
            MainDirector::CentralTower => (6.5, 2.7),
            MainDirector::RnDctOptical => (15.0, 4.6),
            MainDirector::KmStereoscopic10m => (32.0, 10.5),
            MainDirector::IjnPagoda15m => (46.0, 15.5),
            MainDirector::UsnMk38Radar => (24.0, 8.0),
            MainDirector::Auto => (18.0, 6.0),
        };
        let radar_top_tons = match eval.resolved_radar_suite {
            RadarSuite::None => 0.0,
            RadarSuite::KmFumoDecimetric => 2.8,
            RadarSuite::IjnType21_22 => 3.2,
            RadarSuite::RnCentimetricHfdf => 6.5,
            RadarSuite::UsnCentimetricCic => 9.5,
            RadarSuite::Auto => 4.0,
        };
        eval.topweight_tons = dir_top_tons + radar_top_tons;
        eval.optical_baseline_m = baseline_m;

        eval.total_systems_weight_tons = eval.vital_weight_tons
            + eval.hull_weight_tons
            + eval.deck_weight_tons
            + eval.topweight_tons;

        // Estimated delta GM from topweight leverage
        // Topweight has ~4x vertical arm compared to keel reference, Deck 2.5x, Hull -0.5x, Vital -1.0x
        let moment = (eval.topweight_tons * 3.8) + (eval.deck_weight_tons * 2.2)
            - (eval.hull_weight_tons * 0.4)
            - (eval.vital_weight_tons * 1.0);
        eval.estimated_delta_gm_ft = -(moment / (disp * 1.25)).min(2.5);

        // 5. Fire Control Accuracy & Salvo Hit Probability Curve
        let (comp_order, comp_disp_reduction) = match eval.resolved_ballistic_computer {
            BallisticComputer::ManualTables => (1, 1.00),
            BallisticComputer::VickersDreyer => (2, 0.88),
            BallisticComputer::RnAfct => (4, 0.68),
            BallisticComputer::KmC38Analog => (4, 0.67),
            BallisticComputer::IjnType98Hoiban => (4, 0.66),
            BallisticComputer::UsnFordMk1a => (5, 0.55),
            BallisticComputer::Auto => (3, 0.75),
        };
        eval.computer_order = comp_order;

        let blind_fire = match eval.resolved_radar_suite {
            RadarSuite::None => 0.0,
            RadarSuite::KmFumoDecimetric => 0.35,
            RadarSuite::IjnType21_22 => 0.20,
            RadarSuite::RnCentimetricHfdf => 0.70,
            RadarSuite::UsnCentimetricCic => 0.95,
            RadarSuite::Auto => 0.40,
        };
        eval.radar_blind_fire_capability = blind_fire;

        let dir_stability_factor = match eval.resolved_main_director {
            MainDirector::OpenSights => 1.40,
            MainDirector::CentralTower => 1.20,
            MainDirector::RnDctOptical => 1.00,
            MainDirector::KmStereoscopic10m => 0.88,
            MainDirector::IjnPagoda15m => 0.82,
            MainDirector::UsnMk38Radar => 0.72,
            MainDirector::Auto => 1.00,
        };

        // Ship steadiness adjustment factor
        let steadiness_factor = (100.0 / ship.steadiness_adj.max(25.0))
            .sqrt()
            .clamp(0.8, 1.6);

        // Compute salvo hit probability (%) at ranges 5,000 to 35,000 yd
        // Standard naval target: ~240m (787ft) length, ~32m (105ft) beam
        let mut curve = Vec::new();
        for r_step in (5000..=35000).step_by(2500) {
            let r = r_step as f64;
            // Optical ranging error grows with range squared, inversely with baseline
            let opt_range_err = (r / 1000.0).powi(2) * 1.8 / baseline_m.max(1.5);
            // Radar ranging error is roughly constant (~25yd for good centimetric radar)
            let rad_range_err = 25.0 / (blind_fire + 0.1).min(1.0);
            let combined_range_err = if blind_fire > 0.4 {
                (opt_range_err * 0.4).min(rad_range_err)
            } else {
                opt_range_err
            };

            // Dispersion length and width (yards)
            let sigma_range =
                combined_range_err * comp_disp_reduction * dir_stability_factor * steadiness_factor;
            let sigma_deflection =
                (r * 0.0035) * comp_disp_reduction * dir_stability_factor * steadiness_factor
                    / (1.0 + blind_fire * 0.2);

            // Ellipse area (yd^2) vs standard battleship target presented area (~7000 yd^2)
            let disp_area =
                (std::f64::consts::PI * sigma_range.max(15.0) * sigma_deflection.max(8.0))
                    .max(400.0);
            let target_area = 5500.0;
            let base_hit_prob = (target_area / disp_area).clamp(0.008, 0.42);

            // Salvo barrel concentration bonus
            let b_count = main_barrels.clamp(2.0, 12.0);
            let salvo_hit_prob = (1.0 - (1.0 - base_hit_prob).powf(b_count * 0.35)) * 100.0;
            curve.push((r, (salvo_hit_prob * 10.0).round() / 10.0));
        }
        eval.hit_prob_curve = curve;

        eval
    }
}

/// Helper to evaluate style affinities and generate explanation text.
fn evaluate_affinities(ship: &ShipModel) -> (f64, f64, f64, f64, String) {
    let mut usn = 0.25_f64;
    let mut rn = 0.25_f64;
    let mut ijn = 0.25_f64;
    let mut km = 0.25_f64;

    let country = ship.country.to_lowercase();
    let name = ship.name.to_lowercase();
    let year = ship.hull.date_hull;
    let disp = ship.hull.displacement;
    let main_cal = ship.guns.batteries[0].calibre;
    let sec_count = ship.guns.batteries[1].barrels + ship.guns.batteries[2].barrels;
    let torp_count = ship.weapons.torpedo1.number + ship.weapons.torpedo2.number;

    // 1. Country & Name heuristics
    if country.contains("united states")
        || country.contains("us")
        || country.contains("usa")
        || name.starts_with("uss")
    {
        usn += 0.45;
    } else if country.contains("brit")
        || country.contains("uk")
        || country.contains("royal")
        || name.starts_with("hms")
    {
        rn += 0.45;
    } else if country.contains("japan") || country.contains("ijn") {
        ijn += 0.45;
    } else if country.contains("german")
        || country.contains("kriegsmarine")
        || country.contains("dkm")
    {
        km += 0.45;
    }

    // 2. Technical features
    // Heavy dual-purpose battery + high modern year favors USN
    if year >= 1938 && sec_count >= 10 {
        usn += 0.20;
    }
    // High torpedo density favors IJN
    if torp_count >= 8 {
        ijn += 0.25;
    }
    // Extremely heavy caliber + long hull favors IJN / USN
    if main_cal >= 16.0 {
        usn += 0.15;
        ijn += 0.15;
    }
    // Triple/double turret German style or high speed with diesel
    if ship.engine.engine_types[1] || (disp > 30000.0 && main_cal <= 12.0) {
        km += 0.25;
    }
    // Moderate balanced cruiser / carrier favors RN / USN
    if ship.ship_type.to_lowercase().contains("carrier") {
        usn += 0.15;
        rn += 0.15;
    }

    // Normalize affinities
    let total = (usn + rn + ijn + km).max(0.01);
    let u = usn / total;
    let r = rn / total;
    let j = ijn / total;
    let k = km / total;

    let dominant = if u >= r && u >= j && u >= k {
        format!("USN Radar-Centric ({:.0}%)", u * 100.0)
    } else if r >= u && r >= j && r >= k {
        format!("RN All-Weather ({:.0}%)", r * 100.0)
    } else if j >= u && j >= r && j >= k {
        format!("IJN Long-Range Optical ({:.0}%)", j * 100.0)
    } else {
        format!("KM Stereoscopic & Diesel ({:.0}%)", k * 100.0)
    };

    let explanation = format!(
        "Matched {} from {} hull date ({}), {:.0}t normal displacement, and armament configuration.",
        dominant, ship.country, year, disp
    );

    (u, r, j, k, explanation)
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
            StyleDoctrine::UsnRadarCentric => {
                if year >= 1941 {
                    MainDirector::UsnMk38Radar
                } else {
                    MainDirector::RnDctOptical
                }
            }
            StyleDoctrine::IjnOpticalLongRange => MainDirector::IjnPagoda15m,
            StyleDoctrine::KmStereoscopicDiesel => MainDirector::KmStereoscopic10m,
            _ => MainDirector::RnDctOptical,
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
        StyleDoctrine::UsnRadarCentric => AaDirector::UsnMk37,
        StyleDoctrine::RnAllWeather => AaDirector::RnHacs,
        StyleDoctrine::KmStereoscopicDiesel => AaDirector::KmWackeltopf,
        StyleDoctrine::IjnOpticalLongRange => {
            if is_destroyer || is_carrier {
                AaDirector::IjnType94
            } else {
                AaDirector::IjnType94
            }
        }
        _ => {
            if year >= 1940 {
                AaDirector::UsnMk37
            } else {
                AaDirector::RnHacs
            }
        }
    }
}

fn resolve_ballistic_computer(doctrine: StyleDoctrine, year: i32) -> BallisticComputer {
    if year < 1912 {
        BallisticComputer::ManualTables
    } else if year < 1925 {
        BallisticComputer::VickersDreyer
    } else {
        match doctrine {
            StyleDoctrine::UsnRadarCentric => BallisticComputer::UsnFordMk1a,
            StyleDoctrine::RnAllWeather => BallisticComputer::RnAfct,
            StyleDoctrine::KmStereoscopicDiesel => BallisticComputer::KmC38Analog,
            StyleDoctrine::IjnOpticalLongRange => BallisticComputer::IjnType98Hoiban,
            _ => BallisticComputer::RnAfct,
        }
    }
}

fn resolve_radar_suite(doctrine: StyleDoctrine, year: i32) -> RadarSuite {
    if year < 1939 {
        RadarSuite::None
    } else if year < 1942 {
        match doctrine {
            StyleDoctrine::KmStereoscopicDiesel => RadarSuite::KmFumoDecimetric,
            StyleDoctrine::RnAllWeather => RadarSuite::RnCentimetricHfdf,
            _ => RadarSuite::RnCentimetricHfdf,
        }
    } else {
        match doctrine {
            StyleDoctrine::UsnRadarCentric => RadarSuite::UsnCentimetricCic,
            StyleDoctrine::RnAllWeather => RadarSuite::RnCentimetricHfdf,
            StyleDoctrine::IjnOpticalLongRange => RadarSuite::IjnType21_22,
            StyleDoctrine::KmStereoscopicDiesel => RadarSuite::KmFumoDecimetric,
            _ => RadarSuite::UsnCentimetricCic,
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
            StyleDoctrine::RnAllWeather => SonarSuite::RnAsdic,
            StyleDoctrine::KmStereoscopicDiesel => SonarSuite::KmGhgArray,
            StyleDoctrine::UsnRadarCentric => SonarSuite::UsnQcSonar,
            _ => {
                if is_destroyer {
                    SonarSuite::RnAsdic
                } else {
                    SonarSuite::None
                }
            }
        }
    }
}

fn resolve_electrical_plant(doctrine: StyleDoctrine) -> ElectricalPlantStyle {
    match doctrine {
        StyleDoctrine::UsnRadarCentric => ElectricalPlantStyle::UsnHighMarginAc,
        StyleDoctrine::KmStereoscopicDiesel => ElectricalPlantStyle::KmHeavyDiesel,
        StyleDoctrine::RnAllWeather => ElectricalPlantStyle::RnBalanced,
        StyleDoctrine::IjnOpticalLongRange => ElectricalPlantStyle::CompactSteam,
        _ => ElectricalPlantStyle::RnBalanced,
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
        assert_eq!(eval_iowa.resolved_doctrine, StyleDoctrine::UsnRadarCentric);
        assert_eq!(eval_iowa.resolved_main_director, MainDirector::UsnMk38Radar);
        assert_eq!(
            eval_iowa.resolved_ballistic_computer,
            BallisticComputer::UsnFordMk1a
        );
        assert!(eval_iowa.total_installed_kw > 3000.0);
        assert!(eval_iowa.electrical_margin_pct > 15.0);

        let yamato = get_preset("yamato").expect("Yamato preset");
        let eval_yamato = yamato.systems.evaluate(&yamato);
        assert_eq!(
            eval_yamato.resolved_doctrine,
            StyleDoctrine::IjnOpticalLongRange
        );
        assert_eq!(
            eval_yamato.resolved_main_director,
            MainDirector::IjnPagoda15m
        );
        assert_eq!(eval_yamato.optical_baseline_m, 15.5);
    }

    #[test]
    fn mix_and_match_overrides_work() {
        let mut ship = get_preset("bismarck").expect("Bismarck preset");
        // Mix: KM hull with USN Mk 38 radar director and RN ASDIC sonar
        ship.systems.main_director = MainDirector::UsnMk38Radar;
        ship.systems.sonar_suite = SonarSuite::RnAsdic;
        ship.systems.custom_turbo_kw = Some(5000.0);
        ship.systems.custom_diesel_kw = Some(3000.0);

        let eval = ship.systems.evaluate(&ship);
        assert_eq!(eval.resolved_main_director, MainDirector::UsnMk38Radar);
        assert_eq!(eval.resolved_sonar_suite, SonarSuite::RnAsdic);
        assert_eq!(eval.total_installed_kw, 8000.0);
        assert_eq!(eval.turbo_generator_kw, 5000.0);
        assert_eq!(eval.diesel_generator_kw, 3000.0);
        assert!(eval.hit_prob_curve.len() > 5);
    }
}
