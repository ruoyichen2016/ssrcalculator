//! Accuracy/validation helpers and literature pointers.
//!
//! The original SpringSharp resistance model is a compact empirical method.  To
//! reason about its accuracy we provide a small set of historical validation
//! cases and a calibration factor.  The listed papers are starting points for
//! replacing the resistance model with a more modern method.

use crate::model::{ResistanceModel, ShipModel};

/// How a historical reference case was obtained.  This is the primary
/// provenance signal; the per-case `confidence` is only a coarse, declared
/// estimate and must not be conflated with independent verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceType {
    /// Official trial report / measured full-scale data.
    OfficialTrial,
    /// Official or institutional model-basin test data.
    ModelTest,
    /// Public historical reference (books, registries, secondary sources).
    PublicReference,
    /// Synthetic or self-generated data.
    Synthetic,
}

impl SourceType {
    pub const fn label(self) -> &'static str {
        match self {
            SourceType::OfficialTrial => "official trial",
            SourceType::ModelTest => "model test",
            SourceType::PublicReference => "public reference",
            SourceType::Synthetic => "synthetic",
        }
    }

    /// Only official-trial and model-test data count as independently
    /// verified.  This is deliberately separated from `confidence`, so that
    /// self-declared confidence can never fake a 100% verification score.
    pub const fn is_verified(self) -> bool {
        matches!(self, SourceType::OfficialTrial | SourceType::ModelTest)
    }

    /// Coarse evidence weight used when aggregating data quality.
    pub const fn evidence_weight(self) -> f64 {
        match self {
            SourceType::OfficialTrial => 1.0,
            SourceType::ModelTest => 0.9,
            SourceType::PublicReference => 0.5,
            SourceType::Synthetic => 0.2,
        }
    }
}

/// Citation shared by all current built-in cases.
pub const PUBLIC_REFERENCE_SOURCE: &str = "Public historical references (Conway's/Jane's/class trial summaries); displacement and power are approximate";

/// Which displacement convention a reference power figure refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplacementType {
    Normal,
    Standard,
    FullLoad,
    Trial,
    Deep,
    Unspecified,
}

impl DisplacementType {
    pub const fn label(self) -> &'static str {
        match self {
            DisplacementType::Normal => "normal",
            DisplacementType::Standard => "standard",
            DisplacementType::FullLoad => "full load",
            DisplacementType::Trial => "trial",
            DisplacementType::Deep => "deep",
            DisplacementType::Unspecified => "unspecified",
        }
    }
}

/// Provenance attached to every built-in historical case.
///
/// A case is **traceable** only when it has a citation, a URL and a named
/// verifier.  This is deliberately stricter than having a `source` string, so
/// that the quality gate reflects real evidence rather than labels.
#[derive(Clone, Copy, Debug)]
pub struct Provenance {
    pub source_type: SourceType,
    pub displacement_type: DisplacementType,
    pub confidence: f64,
    pub source: &'static str,
    pub source_url: &'static str,
    pub verifier: &'static str,
    pub verification_date: &'static str,
}

impl Provenance {
    /// Default provenance for the current built-in database: public
    /// references, *not* independently verified and not yet traceable to a
    /// specific URL/verifier.
    pub const fn public_reference() -> Self {
        Provenance {
            source_type: SourceType::PublicReference,
            displacement_type: DisplacementType::Normal,
            confidence: 0.6,
            source: PUBLIC_REFERENCE_SOURCE,
            source_url: "",
            verifier: "",
            verification_date: "2026-09-27",
        }
    }

    pub const fn is_verified(self) -> bool {
        self.source_type.is_verified()
    }

    /// A case is traceable only with citation + URL + named verifier.
    pub const fn is_traceable(self) -> bool {
        !self.source.is_empty() && !self.source_url.is_empty() && !self.verifier.is_empty()
    }
}

/// One historical speed/power validation point.
#[derive(Clone, Copy, Debug)]
pub struct HistoricalCase {
    pub name: &'static str,
    pub ship_type: &'static str,
    pub length_ft: f64,
    pub beam_ft: f64,
    pub draught_ft: f64,
    pub displacement_tons: f64,
    pub block_coefficient: f64,
    pub speed_kt: f64,
    pub reference_hp: f64,
    pub provenance: Provenance,
}

pub fn historical_cases() -> Vec<HistoricalCase> {
    vec![
        HistoricalCase {
            name: "HMS Dreadnought (1906)",
            ship_type: "battleship",
            length_ft: 527.0,
            beam_ft: 82.0,
            draught_ft: 26.5,
            displacement_tons: 18_200.0,
            block_coefficient: 0.61,
            speed_kt: 21.0,
            reference_hp: 23_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Bismarck (1940)",
            ship_type: "battleship",
            length_ft: 823.0,
            beam_ft: 118.0,
            draught_ft: 30.5,
            displacement_tons: 45_000.0,
            block_coefficient: 0.56,
            speed_kt: 30.0,
            reference_hp: 138_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Iowa (1943)",
            ship_type: "battleship",
            length_ft: 860.0,
            beam_ft: 108.0,
            draught_ft: 36.0,
            displacement_tons: 48_000.0,
            block_coefficient: 0.60,
            speed_kt: 33.0,
            reference_hp: 212_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Fletcher class DD (1942)",
            ship_type: "destroyer",
            length_ft: 376.0,
            beam_ft: 40.0,
            draught_ft: 13.0,
            displacement_tons: 2_100.0,
            block_coefficient: 0.48,
            speed_kt: 36.5,
            reference_hp: 60_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Yamato (1941)",
            ship_type: "battleship",
            length_ft: 862.0,
            beam_ft: 121.0,
            draught_ft: 34.0,
            displacement_tons: 65_000.0,
            block_coefficient: 0.61,
            speed_kt: 27.0,
            reference_hp: 150_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Kongo (refit 1936)",
            ship_type: "battleship",
            length_ft: 728.0,
            beam_ft: 101.0,
            draught_ft: 32.0,
            displacement_tons: 36_600.0,
            block_coefficient: 0.57,
            speed_kt: 30.0,
            reference_hp: 136_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Essex (1943)",
            ship_type: "carrier",
            length_ft: 820.0,
            beam_ft: 93.0,
            draught_ft: 23.0,
            displacement_tons: 27_100.0,
            block_coefficient: 0.59,
            speed_kt: 33.0,
            reference_hp: 150_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Atlanta class CL (1942)",
            ship_type: "cruiser",
            length_ft: 541.0,
            beam_ft: 53.0,
            draught_ft: 17.0,
            displacement_tons: 6_000.0,
            block_coefficient: 0.50,
            speed_kt: 32.5,
            reference_hp: 75_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Tribal class DD (1938)",
            ship_type: "destroyer",
            length_ft: 377.0,
            beam_ft: 36.0,
            draught_ft: 11.0,
            displacement_tons: 1_900.0,
            block_coefficient: 0.47,
            speed_kt: 36.0,
            reference_hp: 44_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "South Dakota (1942)",
            ship_type: "battleship",
            length_ft: 680.0,
            beam_ft: 108.0,
            draught_ft: 34.0,
            displacement_tons: 35_000.0,
            block_coefficient: 0.60,
            speed_kt: 27.5,
            reference_hp: 130_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "King George V (1940)",
            ship_type: "battleship",
            length_ft: 745.0,
            beam_ft: 103.0,
            draught_ft: 34.0,
            displacement_tons: 36_750.0,
            block_coefficient: 0.58,
            speed_kt: 28.0,
            reference_hp: 110_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Baltimore CA (1943)",
            ship_type: "cruiser",
            length_ft: 673.0,
            beam_ft: 71.0,
            draught_ft: 22.0,
            displacement_tons: 13_600.0,
            block_coefficient: 0.52,
            speed_kt: 33.0,
            reference_hp: 120_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Cleveland CL (1942)",
            ship_type: "cruiser",
            length_ft: 608.0,
            beam_ft: 66.0,
            draught_ft: 20.0,
            displacement_tons: 10_000.0,
            block_coefficient: 0.51,
            speed_kt: 32.5,
            reference_hp: 100_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Yorktown (1937)",
            ship_type: "carrier",
            length_ft: 770.0,
            beam_ft: 83.0,
            draught_ft: 21.0,
            displacement_tons: 19_800.0,
            block_coefficient: 0.55,
            speed_kt: 32.5,
            reference_hp: 120_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Illustrious (1940)",
            ship_type: "carrier",
            length_ft: 740.0,
            beam_ft: 95.0,
            draught_ft: 24.0,
            displacement_tons: 23_000.0,
            block_coefficient: 0.56,
            speed_kt: 30.5,
            reference_hp: 111_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "North Carolina (1941)",
            ship_type: "battleship",
            length_ft: 729.0,
            beam_ft: 108.0,
            draught_ft: 33.0,
            displacement_tons: 35_000.0,
            block_coefficient: 0.61,
            speed_kt: 28.0,
            reference_hp: 121_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Vanguard (1946)",
            ship_type: "battleship",
            length_ft: 814.0,
            beam_ft: 108.0,
            draught_ft: 30.0,
            displacement_tons: 44_500.0,
            block_coefficient: 0.60,
            speed_kt: 30.0,
            reference_hp: 130_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Shokaku (1941)",
            ship_type: "carrier",
            length_ft: 845.0,
            beam_ft: 95.0,
            draught_ft: 27.0,
            displacement_tons: 25_675.0,
            block_coefficient: 0.58,
            speed_kt: 34.2,
            reference_hp: 160_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Hood (1920)",
            ship_type: "battleship",
            length_ft: 860.0,
            beam_ft: 104.0,
            draught_ft: 30.0,
            displacement_tons: 42_000.0,
            block_coefficient: 0.56,
            speed_kt: 32.0,
            reference_hp: 144_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Zara class CA (1931)",
            ship_type: "cruiser",
            length_ft: 600.0,
            beam_ft: 68.0,
            draught_ft: 20.0,
            displacement_tons: 11_500.0,
            block_coefficient: 0.52,
            speed_kt: 32.0,
            reference_hp: 95_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Mogami (1935)",
            ship_type: "cruiser",
            length_ft: 650.0,
            beam_ft: 66.0,
            draught_ft: 18.0,
            displacement_tons: 11_200.0,
            block_coefficient: 0.51,
            speed_kt: 35.0,
            reference_hp: 152_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Kagero class DD (1939)",
            ship_type: "destroyer",
            length_ft: 388.0,
            beam_ft: 38.0,
            draught_ft: 12.0,
            displacement_tons: 2_500.0,
            block_coefficient: 0.48,
            speed_kt: 35.0,
            reference_hp: 52_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Brooklyn CL (1937)",
            ship_type: "cruiser",
            length_ft: 608.0,
            beam_ft: 61.0,
            draught_ft: 19.0,
            displacement_tons: 9_700.0,
            block_coefficient: 0.53,
            speed_kt: 32.5,
            reference_hp: 100_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Dido CL (1940)",
            ship_type: "cruiser",
            length_ft: 512.0,
            beam_ft: 50.0,
            draught_ft: 15.0,
            displacement_tons: 5_600.0,
            block_coefficient: 0.51,
            speed_kt: 32.25,
            reference_hp: 62_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Somers DD (1937)",
            ship_type: "destroyer",
            length_ft: 381.0,
            beam_ft: 36.2,
            draught_ft: 10.3,
            displacement_tons: 1_840.0,
            block_coefficient: 0.47,
            speed_kt: 36.0,
            reference_hp: 52_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Benson DD (1942)",
            ship_type: "destroyer",
            length_ft: 341.0,
            beam_ft: 36.1,
            draught_ft: 11.75,
            displacement_tons: 1_620.0,
            block_coefficient: 0.47,
            speed_kt: 37.5,
            reference_hp: 50_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Independence CVL (1943)",
            ship_type: "carrier",
            length_ft: 622.0,
            beam_ft: 71.0,
            draught_ft: 24.0,
            displacement_tons: 10_662.0,
            block_coefficient: 0.56,
            speed_kt: 31.6,
            reference_hp: 100_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Bogue CVE (1943)",
            ship_type: "carrier",
            length_ft: 496.0,
            beam_ft: 69.0,
            draught_ft: 24.0,
            displacement_tons: 9_393.0,
            block_coefficient: 0.59,
            speed_kt: 18.0,
            reference_hp: 8_500.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Alaska CB (1944)",
            ship_type: "cruiser",
            length_ft: 808.0,
            beam_ft: 91.0,
            draught_ft: 31.0,
            displacement_tons: 29_779.0,
            block_coefficient: 0.55,
            speed_kt: 33.0,
            reference_hp: 150_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Gneisenau (1938)",
            ship_type: "battleship",
            length_ft: 754.0,
            beam_ft: 98.0,
            draught_ft: 32.0,
            displacement_tons: 31_500.0,
            block_coefficient: 0.56,
            speed_kt: 32.0,
            reference_hp: 160_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Porter class DD (1936)",
            ship_type: "destroyer",
            length_ft: 381.0,
            beam_ft: 36.2,
            draught_ft: 10.4,
            displacement_tons: 1_850.0,
            block_coefficient: 0.47,
            speed_kt: 37.0,
            reference_hp: 50_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Sims class DD (1939)",
            ship_type: "destroyer",
            length_ft: 348.25,
            beam_ft: 36.1,
            draught_ft: 13.33,
            displacement_tons: 1_570.0,
            block_coefficient: 0.47,
            speed_kt: 37.0,
            reference_hp: 51_138.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Mahan class DD (1936)",
            ship_type: "destroyer",
            length_ft: 341.25,
            beam_ft: 35.5,
            draught_ft: 10.6,
            displacement_tons: 1_500.0,
            block_coefficient: 0.47,
            speed_kt: 37.0,
            reference_hp: 46_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "New Orleans class CA (1934)",
            ship_type: "cruiser",
            length_ft: 588.2,
            beam_ft: 61.75,
            draught_ft: 19.4,
            displacement_tons: 9_950.0,
            block_coefficient: 0.52,
            speed_kt: 32.7,
            reference_hp: 107_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Kuma class CL (1920)",
            ship_type: "cruiser",
            length_ft: 531.8,
            beam_ft: 46.6,
            draught_ft: 15.7,
            displacement_tons: 5_500.0,
            block_coefficient: 0.50,
            speed_kt: 36.0,
            reference_hp: 90_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Sendai class CL (1924)",
            ship_type: "cruiser",
            length_ft: 532.0,
            beam_ft: 46.5,
            draught_ft: 15.7,
            displacement_tons: 5_195.0,
            block_coefficient: 0.50,
            speed_kt: 35.25,
            reference_hp: 90_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Admiral Hipper class CA (1939)",
            ship_type: "cruiser",
            length_ft: 665.4,
            beam_ft: 69.9,
            draught_ft: 23.6,
            displacement_tons: 14_000.0,
            block_coefficient: 0.52,
            speed_kt: 32.0,
            reference_hp: 132_000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Lexington CV (1927)",
            ship_type: "carrier",
            length_ft: 888.0,
            beam_ft: 106.0,
            draught_ft: 32.0,
            displacement_tons: 36000.0,
            block_coefficient: 0.585,
            speed_kt: 33.25,
            reference_hp: 180000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Saratoga CV (1927)",
            ship_type: "carrier",
            length_ft: 888.0,
            beam_ft: 106.0,
            draught_ft: 32.0,
            displacement_tons: 36000.0,
            block_coefficient: 0.585,
            speed_kt: 33.25,
            reference_hp: 180000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Ranger CV (1934)",
            ship_type: "carrier",
            length_ft: 769.0,
            beam_ft: 80.0,
            draught_ft: 22.0,
            displacement_tons: 14500.0,
            block_coefficient: 0.545,
            speed_kt: 29.25,
            reference_hp: 53500.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Wasp CV (1940)",
            ship_type: "carrier",
            length_ft: 741.0,
            beam_ft: 93.0,
            draught_ft: 25.0,
            displacement_tons: 14700.0,
            block_coefficient: 0.56,
            speed_kt: 29.5,
            reference_hp: 70000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Akagi CV (1927)",
            ship_type: "carrier",
            length_ft: 855.0,
            beam_ft: 102.0,
            draught_ft: 31.0,
            displacement_tons: 36500.0,
            block_coefficient: 0.585,
            speed_kt: 31.5,
            reference_hp: 133000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Kaga CV (1928)",
            ship_type: "carrier",
            length_ft: 812.0,
            beam_ft: 106.0,
            draught_ft: 31.0,
            displacement_tons: 38200.0,
            block_coefficient: 0.6,
            speed_kt: 28.3,
            reference_hp: 127000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Soryu CV (1937)",
            ship_type: "carrier",
            length_ft: 746.0,
            beam_ft: 69.0,
            draught_ft: 25.0,
            displacement_tons: 15900.0,
            block_coefficient: 0.55,
            speed_kt: 34.5,
            reference_hp: 152000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Hiryu CV (1939)",
            ship_type: "carrier",
            length_ft: 746.0,
            beam_ft: 73.0,
            draught_ft: 25.0,
            displacement_tons: 17300.0,
            block_coefficient: 0.555,
            speed_kt: 34.3,
            reference_hp: 153000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Ark Royal CV (1938)",
            ship_type: "carrier",
            length_ft: 800.0,
            beam_ft: 94.0,
            draught_ft: 27.0,
            displacement_tons: 22000.0,
            block_coefficient: 0.57,
            speed_kt: 30.0,
            reference_hp: 102000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Victorious CV (1941)",
            ship_type: "carrier",
            length_ft: 781.0,
            beam_ft: 95.0,
            draught_ft: 28.0,
            displacement_tons: 23200.0,
            block_coefficient: 0.57,
            speed_kt: 30.5,
            reference_hp: 111000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Graf Zeppelin CV (1938)",
            ship_type: "carrier",
            length_ft: 861.0,
            beam_ft: 103.0,
            draught_ft: 28.0,
            displacement_tons: 33500.0,
            block_coefficient: 0.58,
            speed_kt: 33.8,
            reference_hp: 200000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Taiho CV (1944)",
            ship_type: "carrier",
            length_ft: 855.0,
            beam_ft: 91.0,
            draught_ft: 27.0,
            displacement_tons: 29300.0,
            block_coefficient: 0.56,
            speed_kt: 33.3,
            reference_hp: 180000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Unryu CV (1944)",
            ship_type: "carrier",
            length_ft: 745.0,
            beam_ft: 71.0,
            draught_ft: 26.0,
            displacement_tons: 17100.0,
            block_coefficient: 0.55,
            speed_kt: 34.0,
            reference_hp: 152000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Casablanca CVE (1943)",
            ship_type: "carrier",
            length_ft: 512.0,
            beam_ft: 65.0,
            draught_ft: 20.0,
            displacement_tons: 7800.0,
            block_coefficient: 0.55,
            speed_kt: 19.0,
            reference_hp: 9000.0,
            provenance: Provenance::public_reference(),
        },
        HistoricalCase {
            name: "Commencement Bay CVE (1944)",
            ship_type: "carrier",
            length_ft: 557.0,
            beam_ft: 75.0,
            draught_ft: 26.0,
            displacement_tons: 10900.0,
            block_coefficient: 0.57,
            speed_kt: 19.0,
            reference_hp: 16000.0,
            provenance: Provenance::public_reference(),
        },
    ]
}

#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub name: &'static str,
    pub speed_kt: f64,
    pub simulated_hp: f64,
    pub holtrop_hp: f64,
    pub reference_hp: f64,
    pub error_percent: f64,
}

/// Build a minimal ship from a historical case and compute the SpringSharp
/// resistance estimate.
pub fn simulate_case(case: &HistoricalCase) -> ShipModel {
    simulate_case_with_model(case, ResistanceModel::Original)
}

pub fn simulate_case_with_model(case: &HistoricalCase, model: ResistanceModel) -> ShipModel {
    let mut ship = ShipModel::new();
    ship.engine.resistance_model = model;
    ship.name = case.name.into();
    ship.country = "Validation".into();
    ship.ship_type = "Historical".into();

    ship.hull.length = case.length_ft;
    ship.hull.beam = case.beam_ft;
    ship.hull.beam_bulges = case.beam_ft;
    ship.hull.draught = case.draught_ft;
    ship.hull.block = case.block_coefficient;
    ship.hull.date_hull = 1940;
    ship.hull.date_engine = 1940;

    ship.free.fc_length = 0.2;
    ship.free.fd_length = 0.3;
    ship.free.qd_length = 0.15;
    ship.free.free_fd_for = 18.0;
    ship.free.free_fd_aft = 14.0;
    ship.free.free_ad_for = 14.0;
    ship.free.free_ad_aft = 12.0;

    ship.engine.fuel[0] = true;
    ship.engine.engine_types[1] = true;
    ship.engine.drive[0] = true;
    ship.engine.screws = 4;
    ship.engine.speed_max = case.speed_kt;
    ship.engine.speed_cruise = case.speed_kt * 0.5;
    ship.engine.range = 7000.0;
    ship.engine.pct_coal = 0.0;

    ship.calculate();
    ship
}

/// Validation error summary grouped by ship type.
#[derive(Clone, Debug)]
pub struct TypeValidation {
    pub ship_type: &'static str,
    pub count: usize,
    pub mean_orig_error: f64,
    pub mean_holtrop_error: f64,
}

pub fn validation_by_type() -> Vec<TypeValidation> {
    let cases = historical_cases();
    let mut types: Vec<&'static str> = cases.iter().map(|c| c.ship_type).collect();
    types.sort_unstable();
    types.dedup();
    let mut out = Vec::new();
    for ty in types {
        let subset: Vec<_> = cases.iter().filter(|c| c.ship_type == ty).collect();
        let n = subset.len() as f64;
        if n == 0.0 {
            continue;
        }
        let mut orig = 0.0;
        let mut holtrop = 0.0;
        for c in &subset {
            let ship = simulate_case(c);
            let sim_h = simulate_case_with_model(c, ResistanceModel::HoltropMennen);
            if c.reference_hp > 0.0 {
                orig += ((ship.engine.hp_max - c.reference_hp) / c.reference_hp).abs();
                holtrop += ((sim_h.engine.hp_max - c.reference_hp) / c.reference_hp).abs();
            }
        }
        out.push(TypeValidation {
            ship_type: ty,
            count: subset.len(),
            mean_orig_error: orig / n,
            mean_holtrop_error: holtrop / n,
        });
    }
    out
}

/// Per-ship-type calibration factors.
#[derive(Clone, Debug)]
pub struct TypeCalibrationFactor {
    pub ship_type: &'static str,
    pub count: usize,
    pub factor_original: f64,
    pub factor_holtrop: f64,
}

pub fn type_calibration_factors() -> Vec<TypeCalibrationFactor> {
    let cases = historical_cases();
    let mut types: Vec<&'static str> = cases.iter().map(|c| c.ship_type).collect();
    types.sort_unstable();
    types.dedup();
    let mut out = Vec::new();
    for ty in types {
        let subset: Vec<_> = cases.iter().filter(|c| c.ship_type == ty).collect();
        let mut sum_orig = 0.0;
        let mut sum_holtrop = 0.0;
        let mut n = 0_u32;
        for c in &subset {
            let ship = simulate_case(c);
            let sim_h = simulate_case_with_model(c, ResistanceModel::HoltropMennen);
            if c.reference_hp > 0.0 && ship.engine.hp_max > 0.0 && sim_h.engine.hp_max > 0.0 {
                sum_orig += c.reference_hp / ship.engine.hp_max;
                sum_holtrop += c.reference_hp / sim_h.engine.hp_max;
                n += 1;
            }
        }
        if n > 0 {
            out.push(TypeCalibrationFactor {
                ship_type: ty,
                count: subset.len(),
                factor_original: sum_orig / n as f64,
                factor_holtrop: sum_holtrop / n as f64,
            });
        }
    }
    out
}

/// Data sourcing metadata for the historical validation set.
#[derive(Clone, Debug)]
pub struct DataQualityEntry {
    pub name: &'static str,
    pub confidence: f64,
    pub source: &'static str,
    pub source_url: &'static str,
    pub verifier: &'static str,
    pub verification_date: &'static str,
    pub source_type: SourceType,
    pub displacement_type: DisplacementType,
    pub traceable: bool,
}

/// Return data-quality entries sorted by replacement priority (lowest
/// confidence first, then weakest provenance).
pub fn data_replacement_priority() -> Vec<DataQualityEntry> {
    let mut entries = data_quality_summary();
    entries.sort_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.source_type
                    .evidence_weight()
                    .partial_cmp(&b.source_type.evidence_weight())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    entries
}

/// Summarize built-in historical cases grouped by source citation.
pub fn data_source_summary() -> Vec<(String, usize, f64)> {
    let mut groups: Vec<(String, usize, f64)> = Vec::new();
    for entry in data_quality_summary() {
        if let Some(g) = groups.iter_mut().find(|g| g.0 == entry.source) {
            g.1 += 1;
            let total = g.2 * (g.1 - 1) as f64 + entry.confidence;
            g.2 = total / g.1 as f64;
        } else {
            groups.push((entry.source.to_string(), 1, entry.confidence));
        }
    }
    groups
}

/// Overall built-in data-quality score.
///
/// `verified_fraction` is derived from [`SourceType`] (official trial or
/// model-basin test), *not* from self-declared confidence, so it cannot be
/// inflated by relabelling cases as "upgraded".
#[derive(Clone, Copy, Debug)]
pub struct DataQualityScore {
    pub mean_confidence: f64,
    pub mean_evidence: f64,
    pub verified_fraction: f64,
    pub total_cases: usize,
    pub verified_cases: usize,
}

pub fn data_quality_score() -> DataQualityScore {
    let entries = data_quality_summary();
    let n = entries.len();
    if n == 0 {
        return DataQualityScore {
            mean_confidence: 0.0,
            mean_evidence: 0.0,
            verified_fraction: 0.0,
            total_cases: 0,
            verified_cases: 0,
        };
    }
    let mean_confidence = entries.iter().map(|e| e.confidence).sum::<f64>() / n as f64;
    let mean_evidence = entries
        .iter()
        .map(|e| e.source_type.evidence_weight())
        .sum::<f64>()
        / n as f64;
    let verified = entries
        .iter()
        .filter(|e| e.source_type.is_verified())
        .count();
    DataQualityScore {
        mean_confidence,
        mean_evidence,
        verified_fraction: verified as f64 / n as f64,
        total_cases: n,
        verified_cases: verified,
    }
}

/// Confidence tier used to prioritise data replacement.
///
/// This is a *declared* confidence bucket, not a verification level.  Use
/// [`SourceType::is_verified`] for provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTier {
    High,
    Medium,
    Low,
}

pub fn data_tier(confidence: f64) -> DataTier {
    if confidence >= 0.8 {
        DataTier::High
    } else if confidence >= 0.6 {
        DataTier::Medium
    } else {
        DataTier::Low
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DataTierSummary {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

pub fn data_tier_summary() -> DataTierSummary {
    let mut summary = DataTierSummary {
        high: 0,
        medium: 0,
        low: 0,
    };
    for entry in data_quality_summary() {
        match data_tier(entry.confidence) {
            DataTier::High => summary.high += 1,
            DataTier::Medium => summary.medium += 1,
            DataTier::Low => summary.low += 1,
        }
    }
    summary
}

/// Data replacement progress for the built-in database.
#[derive(Clone, Copy, Debug)]
pub struct DataReplacementProgress {
    /// Cases backed by an official trial or model-basin test.
    pub verified: usize,
    /// Cases that are only public-reference / approximate.
    pub approximate: usize,
    pub total: usize,
    /// Cases with citation + URL + named verifier.
    pub traceable: usize,
    /// Fraction independently verified (`verified / total`).
    pub progress: f64,
    /// Fraction of cases carrying an explicit source citation.
    pub provenance_coverage: f64,
    /// Fraction of cases that are fully traceable (`traceable / total`).
    pub traceability: f64,
}

pub fn data_replacement_progress() -> DataReplacementProgress {
    let entries = data_quality_summary();
    let total = entries.len();
    let verified = entries
        .iter()
        .filter(|e| e.source_type.is_verified())
        .count();
    let approximate = total - verified;
    let with_source = entries.iter().filter(|e| !e.source.is_empty()).count();
    let traceable = entries.iter().filter(|e| e.traceable).count();
    let frac = |n: usize| {
        if total > 0 {
            n as f64 / total as f64
        } else {
            0.0
        }
    };
    DataReplacementProgress {
        verified,
        approximate,
        total,
        traceable,
        progress: frac(verified),
        provenance_coverage: frac(with_source),
        traceability: frac(traceable),
    }
}

/// Composite quality gate.
///
/// `composite = 0.4 * official_model_fraction
///            + 0.2 * traceability_fraction
///            + 0.4 * loocv_accuracy_score`
///
/// This replaces the old "verified fraction from self-declared confidence"
/// gate: it cannot be satisfied without real official/model-test data, fully
/// traceable sources and a competitive out-of-sample error.
pub const QUALITY_OFFICIAL_WEIGHT: f64 = 0.4;
pub const QUALITY_TRACEABILITY_WEIGHT: f64 = 0.2;
pub const QUALITY_ACCURACY_WEIGHT: f64 = 0.4;
/// LOOCV MAPE at which the accuracy component saturates at 1.0.
pub const LOOCV_MAPE_BEST: f64 = 0.10;
/// LOOCV MAPE at which the accuracy component reaches 0.0.
pub const LOOCV_MAPE_WORST: f64 = 0.30;

/// Map an out-of-sample MAPE to a 0..1 accuracy score (linear between the
/// best/worst thresholds).
pub fn loocv_accuracy_score(loocv_mape: f64) -> f64 {
    if !loocv_mape.is_finite() {
        return 0.0;
    }
    if loocv_mape <= LOOCV_MAPE_BEST {
        return 1.0;
    }
    if loocv_mape >= LOOCV_MAPE_WORST {
        return 0.0;
    }
    (LOOCV_MAPE_WORST - loocv_mape) / (LOOCV_MAPE_WORST - LOOCV_MAPE_BEST)
}

#[derive(Clone, Copy, Debug)]
pub struct QualityGate {
    pub official_model_fraction: f64,
    pub traceability_fraction: f64,
    pub loocv_mape: f64,
    pub loocv_accuracy_score: f64,
    pub composite: f64,
    pub total_cases: usize,
}

pub fn quality_gate_with_loocv(loocv_mape: f64) -> QualityGate {
    let p = data_replacement_progress();
    let accuracy = loocv_accuracy_score(loocv_mape);
    let composite = QUALITY_OFFICIAL_WEIGHT * p.progress
        + QUALITY_TRACEABILITY_WEIGHT * p.traceability
        + QUALITY_ACCURACY_WEIGHT * accuracy;
    QualityGate {
        official_model_fraction: p.progress,
        traceability_fraction: p.traceability,
        loocv_mape,
        loocv_accuracy_score: accuracy,
        composite,
        total_cases: p.total,
    }
}

#[derive(Clone, Debug)]
pub struct TypeQualityGate {
    pub ship_type: &'static str,
    pub count: usize,
    pub official_model_fraction: f64,
    pub traceability_fraction: f64,
    pub loocv_mape: f64,
    pub composite: f64,
}

/// Per-ship-type quality gate.  `loocv_by_type` is
/// `(ship_type, count, loocv_mape)` as returned by the calibration module.
pub fn quality_gate_by_type(loocv_by_type: &[(String, usize, f64)]) -> Vec<TypeQualityGate> {
    let entries = data_quality_summary();
    let cases = historical_cases();
    let mut types: Vec<&'static str> = cases.iter().map(|c| c.ship_type).collect();
    types.sort_unstable();
    types.dedup();
    let mut out = Vec::new();
    for ty in types {
        let idx: Vec<usize> = cases
            .iter()
            .enumerate()
            .filter(|(_, c)| c.ship_type == ty)
            .map(|(i, _)| i)
            .collect();
        let n = idx.len();
        if n == 0 {
            continue;
        }
        let verified = idx
            .iter()
            .filter(|&&i| entries[i].source_type.is_verified())
            .count();
        let traceable = idx.iter().filter(|&&i| entries[i].traceable).count();
        let official = verified as f64 / n as f64;
        let traceability = traceable as f64 / n as f64;
        let mape = loocv_by_type
            .iter()
            .find(|(t, _, _)| t == ty)
            .map(|(_, _, m)| *m)
            .unwrap_or(f64::NAN);
        let accuracy = loocv_accuracy_score(mape);
        let composite = QUALITY_OFFICIAL_WEIGHT * official
            + QUALITY_TRACEABILITY_WEIGHT * traceability
            + QUALITY_ACCURACY_WEIGHT * accuracy;
        out.push(TypeQualityGate {
            ship_type: ty,
            count: n,
            official_model_fraction: official,
            traceability_fraction: traceability,
            loocv_mape: mape,
            composite,
        });
    }
    out.sort_by(|a, b| {
        a.composite
            .partial_cmp(&b.composite)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Check several composite-quality targets at once.
pub fn data_target_matrix_for(composite: f64) -> Vec<(f64, bool)> {
    [0.6, 0.7, 0.8, 0.9]
        .iter()
        .map(|t| (*t, composite >= *t))
        .collect()
}

/// Suggested replacement actions for the built-in database.
pub fn data_replacement_log() -> Vec<(String, f64, &'static str)> {
    let mut entries = data_quality_summary();
    entries.sort_by(|a, b| {
        a.confidence
            .partial_cmp(&b.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries
        .into_iter()
        .map(|e| {
            let action = match data_tier(e.confidence) {
                DataTier::High => "keep / verified",
                DataTier::Medium => "replace with official trial or model-test data",
                DataTier::Low => "priority replacement",
            };
            (e.name.to_string(), e.confidence, action)
        })
        .collect()
}

/// Human-readable version tag for the built-in historical database.
pub fn data_version() -> String {
    let p = data_replacement_progress();
    format!(
        "built-in-v1.6 ({} model-test verified, {} public-reference; traceable {}/{}; provenance coverage {:.0}%)",
        p.verified,
        p.approximate,
        p.traceable,
        p.total,
        p.provenance_coverage * 100.0
    )
}

/// Generate an HTML report for the built-in historical database quality.
pub fn data_quality_report_html() -> String {
    let cases = historical_cases();
    let quality = data_quality_summary();
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Data Quality Report</title></head><body>",
    );
    html.push_str("<h1>Built-in Data Quality Report</h1>");
    html.push_str("<p>Verified = official trial or model test. Traceable = citation + URL + named verifier. Public reference is <b>not</b> independently verified.</p>");
    html.push_str("<table border=\"1\" cellpadding=\"4\" cellspacing=\"0\">");
    html.push_str(
        "<tr><th>Ship</th><th>Type</th><th>Confidence</th><th>Source type</th><th>Displacement</th><th>Traceable</th><th>Verifier</th><th>Source</th></tr>",
    );
    for (case, q) in cases.iter().zip(quality.iter()) {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{:.2}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            case.name,
            case.ship_type,
            q.confidence,
            q.source_type.label(),
            q.displacement_type.label(),
            if q.traceable { "yes" } else { "no" },
            q.verifier,
            q.source
        ));
    }
    html.push_str("</table>");
    html.push_str("</body></html>");
    html
}

/// Declared confidence for a built-in case, read from its provenance record.
pub fn case_confidence(name: &str) -> f64 {
    historical_cases()
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.provenance.confidence)
        .unwrap_or(0.5)
}

/// Return the provenance metadata for every historical case.
pub fn data_quality_summary() -> Vec<DataQualityEntry> {
    historical_cases()
        .iter()
        .map(|c| DataQualityEntry {
            name: c.name,
            confidence: c.provenance.confidence,
            source: c.provenance.source,
            source_url: c.provenance.source_url,
            verifier: c.provenance.verifier,
            verification_date: c.provenance.verification_date,
            source_type: c.provenance.source_type,
            displacement_type: c.provenance.displacement_type,
            traceable: c.provenance.is_traceable(),
        })
        .collect()
}

/// A user-supplied historical/model-test case.
#[derive(Clone, Debug)]
pub struct ExternalCase {
    pub name: String,
    pub ship_type: String,
    pub source: String,
    pub length_ft: f64,
    pub beam_ft: f64,
    pub draught_ft: f64,
    pub displacement_tons: f64,
    pub block_coefficient: f64,
    pub speed_kt: f64,
    pub reference_hp: f64,
    pub confidence: f64,
}

/// Parse a simple CSV of external ship cases.
///
/// Expected columns:
/// `name,type,length_ft,beam_ft,draught_ft,displacement_tons,block_coefficient,speed_kt,reference_hp,confidence`
pub fn parse_external_cases(text: &str) -> Result<Vec<ExternalCase>, String> {
    let mut out = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("name,") {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(str::trim).collect();
        if cols.len() < 9 || cols.len() > 11 {
            return Err(format!(
                "line {}: expected 9 to 11 columns, got {}",
                line_no + 1,
                cols.len()
            ));
        }
        let parse = |i: usize, label: &str| -> Result<f64, String> {
            cols[i]
                .parse::<f64>()
                .map_err(|e| format!("line {}: {label} invalid: {}", line_no + 1, e))
        };
        out.push(ExternalCase {
            name: cols[0].to_string(),
            ship_type: cols[1].to_string(),
            source: if cols.len() == 11 {
                cols[10].to_string()
            } else {
                "external user data".to_string()
            },
            length_ft: parse(2, "length_ft")?,
            beam_ft: parse(3, "beam_ft")?,
            draught_ft: parse(4, "draught_ft")?,
            displacement_tons: parse(5, "displacement_tons")?,
            block_coefficient: parse(6, "block_coefficient")?,
            speed_kt: parse(7, "speed_kt")?,
            reference_hp: parse(8, "reference_hp")?,
            confidence: if cols.len() >= 10 {
                parse(9, "confidence")?
            } else {
                0.70
            },
        });
    }
    Ok(out)
}

/// Summarize external cases grouped by `source`.
pub fn summarize_external_by_source(cases: &[ExternalCase]) -> Vec<(String, usize, f64)> {
    let mut groups: Vec<(String, usize, f64)> = Vec::new();
    for case in cases {
        if let Some(g) = groups.iter_mut().find(|g| g.0 == case.source) {
            g.1 += 1;
            let total = g.2 * (g.1 - 1) as f64 + case.confidence;
            g.2 = total / g.1 as f64;
        } else {
            groups.push((case.source.clone(), 1, case.confidence));
        }
    }
    groups
}

/// Detect duplicate ship names in an external dataset.
pub fn detect_duplicate_external_cases(cases: &[ExternalCase]) -> Vec<String> {
    let mut seen = std::collections::HashMap::new();
    for case in cases {
        *seen.entry(case.name.clone()).or_insert(0usize) += 1;
    }
    let mut out: Vec<String> = seen
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name)
        .collect();
    out.sort();
    out
}

/// Simple diagnostic for small/fast external cases: returns average absolute
/// percentage error for cases above a speed threshold.
pub fn small_fast_error_diagnostic(cases: &[ExternalCase], _min_speed_kt: f64) -> (usize, f64) {
    let mut count = 0_usize;
    let mut total = 0.0_f64;
    for (_, sim, reference, _) in validate_external_cases(cases) {
        if reference > 0.0 && sim > 0.0 {
            count += 1;
            total += ((sim - reference) / reference).abs();
        }
    }
    if count > 0 {
        (count, total / count as f64)
    } else {
        (0, f64::NAN)
    }
    // The min_speed_kt parameter is accepted for API stability; in this simple
    // diagnostic we currently aggregate all provided cases.
}

/// Summarize external cases grouped by ship type.
pub fn summarize_external_by_type(cases: &[ExternalCase]) -> Vec<(String, usize, f64)> {
    let mut groups: Vec<(String, usize, f64)> = Vec::new();
    for case in cases {
        if let Some(g) = groups.iter_mut().find(|g| g.0 == case.ship_type) {
            g.1 += 1;
            let total = g.2 * (g.1 - 1) as f64 + case.confidence;
            g.2 = total / g.1 as f64;
        } else {
            groups.push((case.ship_type.clone(), 1, case.confidence));
        }
    }
    groups
}

/// Simulate an external case with the Holtrop-Mennen model.
pub fn simulate_external_case(case: &ExternalCase) -> ShipModel {
    let mut ship = ShipModel::new();
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.name = case.name.clone();
    ship.ship_type = case.ship_type.clone();
    ship.country = "External".into();
    ship.hull.length = case.length_ft;
    ship.hull.beam = case.beam_ft;
    ship.hull.beam_bulges = case.beam_ft;
    ship.hull.draught = case.draught_ft;
    ship.hull.block = case.block_coefficient;
    ship.hull.date_hull = 1940;
    ship.hull.date_engine = 1940;
    ship.free.fc_length = 0.2;
    ship.free.fd_length = 0.3;
    ship.free.qd_length = 0.15;
    ship.free.free_fd_for = 18.0;
    ship.free.free_fd_aft = 14.0;
    ship.free.free_ad_for = 14.0;
    ship.free.free_ad_aft = 12.0;
    ship.engine.fuel[0] = true;
    ship.engine.engine_types[1] = true;
    ship.engine.drive[0] = true;
    ship.engine.screws = 4;
    ship.engine.speed_max = case.speed_kt;
    ship.engine.speed_cruise = case.speed_kt * 0.5;
    ship.engine.range = 7000.0;
    ship.engine.pct_coal = 0.0;
    ship.calculate();
    ship
}

/// Generate an HTML report for external cases, including validation table,
/// source grouping and ship-type grouping.
pub fn external_report_html(cases: &[ExternalCase]) -> String {
    let results = validate_external_cases(cases);
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>External Data Report</title></head><body>",
    );
    html.push_str("<h1>External Data Validation Report</h1>");
    html.push_str("<table border=\"1\" cellpadding=\"4\" cellspacing=\"0\">");
    html.push_str("<tr><th>Ship</th><th>Sim HP</th><th>Ref HP</th><th>Confidence</th></tr>");
    for (name, sim, reference, confidence) in results {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{:.0}</td><td>{:.0}</td><td>{:.2}</td></tr>",
            name, sim, reference, confidence
        ));
    }
    html.push_str("</table>");

    html.push_str("<h2>Sources</h2><ul>");
    for (source, count, conf) in summarize_external_by_source(cases) {
        html.push_str(&format!(
            "<li>{source}: count={count}, mean confidence={conf:.2}</li>"
        ));
    }
    html.push_str("</ul>");

    html.push_str("<h2>Ship types</h2><ul>");
    for (typ, count, conf) in summarize_external_by_type(cases) {
        html.push_str(&format!(
            "<li>{typ}: count={count}, mean confidence={conf:.2}</li>"
        ));
    }
    html.push_str("</ul></body></html>");
    html
}

/// Validate a set of external cases and return simulated vs reference power.
pub fn validate_external_cases(cases: &[ExternalCase]) -> Vec<(String, f64, f64, f64)> {
    cases
        .iter()
        .map(|c| {
            let ship = simulate_external_case(c);
            (
                c.name.clone(),
                ship.engine.hp_max,
                c.reference_hp,
                c.confidence,
            )
        })
        .collect()
}

/// Run all built-in historical validation cases.
pub fn run_validation() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    for case in historical_cases() {
        let ship = simulate_case(&case);
        let sim = ship.engine.hp_max;
        let ship_h = simulate_case_with_model(&case, ResistanceModel::HoltropMennen);
        let holtrop = ship_h.engine.hp_max;
        let ref_hp = case.reference_hp;
        let error = if ref_hp > 0.0 {
            (sim - ref_hp) / ref_hp * 100.0
        } else {
            0.0
        };
        results.push(ValidationResult {
            name: case.name,
            speed_kt: case.speed_kt,
            simulated_hp: sim,
            holtrop_hp: holtrop,
            reference_hp: ref_hp,
            error_percent: error,
        });
    }
    results
}

/// Deterministic holdout split: every fourth case (indices 3, 7, 11, ...) is
/// held out.  The split is index-based so it is stable across runs.
///
/// Calibration must be fitted on [`training_cases`] only when evaluating on
/// [`holdout_cases`]; otherwise the estimate leaks.
pub fn holdout_indices() -> Vec<usize> {
    let n = historical_cases().len();
    (0..n).filter(|i| i % 4 == 3).collect()
}

/// Cases used to fit calibration (complement of the holdout split).
pub fn training_cases() -> Vec<HistoricalCase> {
    historical_cases()
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i % 4 != 3)
        .map(|(_, c)| c)
        .collect()
}

/// Cases reserved for out-of-sample evaluation.
pub fn holdout_cases() -> Vec<HistoricalCase> {
    historical_cases()
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i % 4 == 3)
        .map(|(_, c)| c)
        .collect()
}

/// Out-of-sample validation: fit the power calibration on the training split,
/// then evaluate the corrected model on the held-out cases.
#[derive(Clone, Copy, Debug)]
pub struct HoldoutValidation {
    pub train_cases: usize,
    pub holdout_cases: usize,
    pub original_factor: f64,
    pub holtrop_factor: f64,
    pub original_mape: f64,
    pub holtrop_mape: f64,
}

pub fn holdout_validation() -> HoldoutValidation {
    let train = training_cases();
    let test = holdout_cases();

    let mut orig_sum = 0.0_f64;
    let mut holt_sum = 0.0_f64;
    let mut n = 0.0_f64;
    for case in &train {
        let orig = simulate_case(case).engine.hp_max;
        let holt = simulate_case_with_model(case, ResistanceModel::HoltropMennen)
            .engine
            .hp_max;
        if case.reference_hp > 0.0 && orig > 0.0 && holt > 0.0 {
            orig_sum += case.reference_hp / orig;
            holt_sum += case.reference_hp / holt;
            n += 1.0;
        }
    }
    let original_factor = if n > 0.0 { orig_sum / n } else { 1.0 };
    let holtrop_factor = if n > 0.0 { holt_sum / n } else { 1.0 };

    let mut orig_err = 0.0_f64;
    let mut holt_err = 0.0_f64;
    let m = test.len() as f64;
    for case in &test {
        let orig = simulate_case(case).engine.hp_max * original_factor;
        let holt = simulate_case_with_model(case, ResistanceModel::HoltropMennen)
            .engine
            .hp_max
            * holtrop_factor;
        if case.reference_hp > 0.0 {
            orig_err += ((orig - case.reference_hp) / case.reference_hp).abs();
            holt_err += ((holt - case.reference_hp) / case.reference_hp).abs();
        }
    }
    HoldoutValidation {
        train_cases: train.len(),
        holdout_cases: test.len(),
        original_factor,
        holtrop_factor,
        original_mape: if m > 0.0 { orig_err / m } else { f64::NAN },
        holtrop_mape: if m > 0.0 { holt_err / m } else { f64::NAN },
    }
}

/// Mean simulated-power scaling factor that would match the historical set:
/// `factor = mean(reference_power / simulated_power)`.
pub fn calibration_factor(results: &[ValidationResult]) -> f64 {
    calibration_factor_for(results, |r| r.simulated_hp)
}

pub fn calibration_factor_holtrop(results: &[ValidationResult]) -> f64 {
    calibration_factor_for(results, |r| r.holtrop_hp)
}

fn calibration_factor_for(
    results: &[ValidationResult],
    power: impl Fn(&ValidationResult) -> f64,
) -> f64 {
    if results.is_empty() {
        return 1.0;
    }
    let mut sum = 0.0_f64;
    let mut n = 0.0_f64;
    for r in results {
        let p = power(r);
        if p > 0.0 {
            sum += r.reference_hp / p;
            n += 1.0;
        }
    }
    if n > 0.0 {
        sum / n
    } else {
        1.0
    }
}

#[derive(Clone, Debug)]
pub struct CalibrationSegment {
    pub name: &'static str,
    pub min_froude: f64,
    pub max_froude: f64,
    pub factor_original: f64,
    pub factor_holtrop: f64,
}

/// Froude number for a ship speed/length pair: `Fn = V / sqrt(g L)`.
pub fn froude_number(speed_kt: f64, length_ft: f64) -> f64 {
    let v = speed_kt * 0.514444;
    let l = length_ft * 0.3048;
    v / (9.80665 * l).sqrt()
}

fn case_length(name: &str) -> f64 {
    historical_cases()
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.length_ft)
        .unwrap_or(600.0)
}

/// Build Froude-number-banded calibration factors.  This is more physical than
/// a single global factor or a simple knot band because slow full-form ships and
/// fast destroyers operate in very different wave-resistance regimes.
pub fn segmented_calibration(results: &[ValidationResult]) -> Vec<CalibrationSegment> {
    let bands: [(&'static str, f64, f64); 3] = [
        ("low Fn (<=0.18)", f64::NEG_INFINITY, 0.18),
        ("medium Fn (0.18-0.28)", 0.18, 0.28),
        ("high Fn (>0.28)", 0.28, f64::INFINITY),
    ];
    bands
        .iter()
        .map(|(name, lo, hi)| {
            let subset: Vec<_> = results
                .iter()
                .filter(|r| {
                    let fn_ = froude_number(r.speed_kt, case_length(r.name));
                    fn_ > *lo && fn_ <= *hi
                })
                .collect();
            let factor_original = if subset.is_empty() {
                1.0
            } else {
                subset
                    .iter()
                    .map(|r| r.reference_hp / r.simulated_hp)
                    .sum::<f64>()
                    / subset.len() as f64
            };
            let factor_holtrop = if subset.is_empty() {
                1.0
            } else {
                subset
                    .iter()
                    .map(|r| r.reference_hp / r.holtrop_hp)
                    .sum::<f64>()
                    / subset.len() as f64
            };
            CalibrationSegment {
                name,
                min_froude: *lo,
                max_froude: *hi,
                factor_original,
                factor_holtrop,
            }
        })
        .collect()
}

/// Literature pointers for improving the empirical resistance/power model.
pub const REFERENCES: &[&str] = &[
    "Holtrop, J. & Mennen, G.J. (1978). A statistical power prediction method. International Shipbuilding Progress, 25(252).",
    "Holtrop, J. & Mennen, G.J. (1982). An approximate power prediction method. International Shipbuilding Progress, 29(335).",
    "Bertram, V. (2012). Practical Ship Hydrodynamics, 2nd edition. Butterworth-Heinemann.",
    "ITTC (1978). ITTC-78 Performance Prediction Method.",
    "Liu, S. et al. (2019). A study on the statistical calibration of the Holtrop and Mennen approximate power prediction method for full hull form, low Froude number vessels. Journal of Ship Production and Design.",
    "Kim, S. et al. (2014). On the prediction of wave-added resistance with empirical methods. Journal of Ship Production and Design, 30(4).",
    "OMAE2021-62395 (2021). Benchmark study of supervised machine learning methods for a ship speed-power prediction at sea.",
];

#[test]
fn external_csv_rejects_malformed_line() {
    let csv = "name,type,length,beam,draught,displacement,block,speed,hp\nTest,battleship,600,90\n";
    assert!(parse_external_cases(csv).is_err());
}

#[test]
fn example_csv_files_all_parse() {
    for file in [
        "data/examples/sample_ships.csv",
        "data/examples/more_ships.csv",
        "data/examples/fast_ships.csv",
        "data/examples/escort_ships.csv",
        "data/examples/small_ships.csv",
        "data/examples/planing_ships.csv",
        "data/examples/more_planing.csv",
    ] {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        let cases = parse_external_cases(&text);
        assert!(cases.is_ok(), "failed to parse {file}");
        let cases = cases.unwrap();
        assert!(!cases.is_empty(), "no cases in {file}");
        let dup = detect_duplicate_external_cases(&cases);
        assert!(dup.is_empty(), "{file} has duplicates: {:?}", dup);
    }
}

#[test]
fn data_target_matrix_is_consistent() {
    let matrix = data_target_matrix_for(0.75);
    assert_eq!(matrix.len(), 4);
    assert!(matrix[0].1 && matrix[1].1 && !matrix[2].1 && !matrix[3].1);
}

#[test]
fn data_version_is_non_empty() {
    assert!(!data_version().is_empty());
}

#[test]
fn data_replacement_log_has_actions() {
    let log = data_replacement_log();
    assert_eq!(log.len(), historical_cases().len());
    for (_, conf, action) in log {
        assert!(conf > 0.0 && conf <= 1.0);
        assert!(!action.is_empty());
    }
}

#[test]
fn built_in_cases_carry_honest_provenance() {
    // The current database is public-reference only; it must NOT be able to
    // report a 100% "verified" score by self-declared confidence.
    let entries = data_quality_summary();
    assert_eq!(entries.len(), historical_cases().len());
    for e in &entries {
        assert!(!e.source.is_empty(), "{} has no source citation", e.name);
    }
    let verified = entries
        .iter()
        .filter(|e| e.source_type.is_verified())
        .count();
    assert_eq!(
        verified, 0,
        "current built-in data is public-reference only, verified={verified}"
    );
    let p = data_replacement_progress();
    assert_eq!(p.verified, 0);
    assert_eq!(p.traceable, 0);
    assert_eq!(p.provenance_coverage, 1.0);
    assert_eq!(p.traceability, 0.0);
    // The composite gate must be well below the default 0.8 target.
    let gate = quality_gate_with_loocv(0.105);
    assert!(gate.composite < 0.8);
    assert!((gate.official_model_fraction - 0.0).abs() < 1e-12);
    assert!((gate.traceability_fraction - 0.0).abs() < 1e-12);
    assert!((gate.loocv_accuracy_score - 0.975).abs() < 1e-9);
    assert!((gate.composite - 0.4 * 0.975).abs() < 1e-9);
    assert!(data_target_matrix_for(gate.composite)
        .iter()
        .all(|(_, met)| !met));
}

#[test]
fn data_tier_summary_is_consistent() {
    let s = data_tier_summary();
    assert_eq!(s.high + s.medium + s.low, historical_cases().len());
}

#[test]
fn type_calibration_factors_are_positive() {
    for f in type_calibration_factors() {
        assert!(f.factor_original > 0.0);
        assert!(f.factor_holtrop > 0.0);
        assert!(f.count > 0);
    }
}

#[test]
fn validation_by_type_totals_cases() {
    let groups = validation_by_type();
    let total: usize = groups.iter().map(|g| g.count).sum();
    assert_eq!(total, historical_cases().len());
    for g in groups {
        assert!(g.mean_orig_error.is_finite());
        assert!(g.mean_holtrop_error.is_finite());
    }
}

#[test]
fn holdout_split_is_disjoint_and_nonempty() {
    let train = training_cases();
    let test = holdout_cases();
    assert!(!test.is_empty());
    assert_eq!(train.len() + test.len(), historical_cases().len());
    for c in &train {
        assert!(!test.iter().any(|t| t.name == c.name));
    }
    for i in holdout_indices() {
        assert_eq!(i % 4, 3);
    }
    let report = holdout_validation();
    assert_eq!(report.train_cases, train.len());
    assert_eq!(report.holdout_cases, test.len());
    assert!(report.original_mape.is_finite() && report.original_mape >= 0.0);
    assert!(report.holtrop_mape.is_finite() && report.holtrop_mape >= 0.0);
}

#[test]
fn data_replacement_progress_is_consistent() {
    let p = data_replacement_progress();
    assert_eq!(p.verified + p.approximate, p.total);
    assert!(p.progress >= 0.0 && p.progress <= 1.0);
}

#[test]
fn data_quality_score_is_valid() {
    let score = data_quality_score();
    assert_eq!(score.total_cases, historical_cases().len());
    assert_eq!(score.verified_cases, 0);
    assert!(score.mean_confidence > 0.0 && score.mean_confidence <= 1.0);
    assert!(score.mean_evidence > 0.0 && score.mean_evidence <= 1.0);
    assert!(score.verified_fraction >= 0.0 && score.verified_fraction <= 1.0);
}

#[test]
fn data_source_summary_is_consistent() {
    let groups = data_source_summary();
    let total: usize = groups.iter().map(|g| g.1).sum();
    assert_eq!(total, historical_cases().len());
    for (_, count, conf) in groups {
        assert!(count > 0);
        assert!(conf > 0.0 && conf <= 1.0);
    }
}

#[test]
fn data_replacement_priority_is_sorted() {
    let entries = data_replacement_priority();
    for pair in entries.windows(2) {
        assert!(pair[0].confidence <= pair[1].confidence);
    }
}

#[test]
fn quality_report_html_contains_ship_names() {
    let html = data_quality_report_html();
    assert!(html.contains("Data Quality Report"));
    assert!(html.contains("HMS Dreadnought"));
    assert!(html.contains("battleship"));
}

#[test]
fn external_csv_detects_duplicates() {
    let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nDup,battleship,600,90,28,23760,0.55,24,50000,0.9,a\nDup,cruiser,500,70,20,10000,0.5,30,80000,0.8,b\n";
    let cases = parse_external_cases(csv).unwrap();
    let dup = detect_duplicate_external_cases(&cases);
    assert_eq!(dup, vec!["Dup"]);
}

#[test]
fn external_csv_rejects_non_numeric_field() {
    let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,abc,90,28,23760,0.55,24,50000,0.9,example\n";
    assert!(parse_external_cases(csv).is_err());
}

#[test]
fn external_csv_skips_comments_and_header() {
    let csv = "# comment\nname,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,600,90,28,23760,0.55,24,50000,0.9,example\n";
    let cases = parse_external_cases(csv).unwrap();
    assert_eq!(cases.len(), 1);
}

#[test]
fn small_fast_error_diagnostic_returns_finite_mape() {
    let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\nCorvette,patrol,205,33,11,925,0.55,16,2750,0.70,approx\n";
    let cases = parse_external_cases(csv).unwrap();
    let (count, mape) = small_fast_error_diagnostic(&cases, 0.0);
    assert_eq!(count, 2);
    assert!(mape.is_finite());
}

#[test]
fn small_fast_external_validation_is_finite() {
    let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nPT Boat,motor,80,20,4,50,0.42,40,1500,0.65,approx\nCorvette,patrol,205,33,11,925,0.55,16,2750,0.70,approx\n";
    let cases = parse_external_cases(csv).unwrap();
    let results = validate_external_cases(&cases);
    for (_, sim, _, _) in results {
        assert!(sim.is_finite());
    }
}

#[test]
fn external_csv_parse_and_validate() {
    let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\nTest,battleship,600,90,28,23760,0.55,24,50000,0.9,example\n";
    let cases = parse_external_cases(csv).unwrap();
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].source, "example");
    assert_eq!(cases[0].confidence, 0.9);
    let results = validate_external_cases(&cases);
    assert!(results[0].1.is_finite());
    assert!(results[0].1 > 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_runs_and_produces_finite_power() {
        let results = run_validation();
        assert_eq!(results.len(), historical_cases().len());
        for r in &results {
            assert!(r.simulated_hp.is_finite() && r.simulated_hp > 0.0);
        }
    }

    #[test]
    fn calibration_factor_is_positive() {
        let results = run_validation();
        let f = calibration_factor(&results);
        assert!(f.is_finite() && f > 0.0);
    }
}
