use crate::*;
pub trait DisplayName {
    fn display_name(&self) -> &'static str;
}
impl DisplayName for MachineryArrangement {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Machinery Match)",
            Self::GroupedBoilerForward => {
                "Arrangement 1 · Grouped Boilers Forward (集中式前炉后机 B-E)"
            }
            Self::GroupedEngineForward => {
                "Arrangement 2 · Grouped Engines Forward (逆向集中式前机后炉 E-B)"
            }
            Self::SandwichEngineBoilerEngine => {
                "Arrangement 3 · Sandwich Engine-Boiler-Engine (夹心式 E-B-E)"
            }
            Self::UnitAlternatingMachinery => {
                "Arrangement 4 · Unit Alternating System (交替单元式 B-E-B-E)"
            }
            Self::CompactEchelonSural => {
                "Arrangement 5 · Compact Echelon / Sural (梯队穿插增压紧凑型)"
            }
            Self::TurboElectricDistributed => {
                "Arrangement 6 · Turbo-Electric Distributed (涡轮电力分布式)"
            }
        }
    }
}
impl DisplayName for TdsArchitecture {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless TDS Match)",
            Self::UnarmouredDoubleBottom => "Type 0 · Unarmoured Double Bottom (无装甲双层底)",
            Self::CoalBunkerWingBulkhead => "Type 1 · Coal Bunker Wing Bulkhead (储煤翼舱吸能)",
            Self::ExternalBlisterBulge => "Type 2 · External Blister Bulge (外挂防雷突出部)",
            Self::MultiLayerLiquidVoid => "Type 3 · Multi-Layer Liquid / Void (多层空液交替)",
            Self::PuglieseCylindricalAbsorber => {
                "Type 4 · Pugliese Collapsible Cylinder (普列塞吸能圆筒)"
            }
            Self::CompoundEboniteDeepTds => {
                "Type 5 · Deep Ébonite Foam Compound (大纵深闭孔泡沫复合)"
            }
        }
    }
}
impl DisplayName for BeltTdsJunction {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Belt-TDS Match)",
            Self::IntegralTaperedHoldingBulkhead => {
                "Joint 1 · Integral Tapered Holding Bulkhead (连续削薄下延闭合)"
            }
            Self::TurtlebackSlopeKneeJoint => {
                "Joint 2 · Turtleback Slope Knee Joint (穹甲坡脚汇合节点)"
            }
            Self::SpacedAbsorberShelfJoint => {
                "Joint 3 · Spaced Strake & Absorber Shelf (防雷圆筒内倾搁板过渡)"
            }
            Self::SteppedShelfCitadel => {
                "Joint 4 · Stepped Horizontal Shelf Citadel (阶梯装甲盒平台转接)"
            }
            Self::DirectSkinDoubleBottom => {
                "Joint 5 · Direct Hull Skin to Double Bottom (平滑闭合双层底)"
            }
        }
    }
}
impl DisplayName for HorizontalProtectionStyle {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Deck Protection Match)",
            Self::ProtectedTurtlebackCarapace => {
                "Deck 1 · Protected Turtleback Carapace (弧形穹甲防护壳 · 无主带)"
            }
            Self::LowTurtlebackSlopedCitadel => {
                "Deck 2 · Low Turtleback Sloped Citadel (低置内倾穹甲 + 双层侧带)"
            }
            Self::MultiLayerBombDecappingSandwich => {
                "Deck 3 · Multi-Layer Bomb-Decapping Sandwich (被帽剥离 + 主甲板 + 防破片三层)"
            }
            Self::MonolithicCitadelBoxDeck => {
                "Deck 4 · Monolithic Citadel Box Deck (高置单层承力厚甲板装甲盒)"
            }
            Self::ArmouredFlightDeckHangarBox => {
                "Deck 5 · Armoured Flight Deck & Hangar Box (航母装甲飞行甲板/封闭机库盒)"
            }
            Self::LightSplinterWeatherDeck => {
                "Deck 6 · Light Splinter Weather Deck (轻型单层防破片甲板)"
            }
        }
    }
}
impl DisplayName for TransverseSubdivision {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Subdivision Match)",
            Self::SingleFileTransverse => "1-Row · Single-File Transverse (单列横通式)",
            Self::TwinFileCenterline => "2-Row · Twin-File Centerline (双列中轴式)",
            Self::TripleFileCenterAndWings => "3-Row · Triple-File Center & Wings (三列中轴两翼式)",
            Self::QuadFileMultiLongitudinal => "4-Row · Quad-File Multi-Longitudinal (四列蜂窝式)",
        }
    }
}
impl DisplayName for ElectricalPlantStyle {
    fn display_name(&self) -> &'static str {
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
}
impl DisplayName for SonarSuite {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::None => "None",
            Self::PassiveHydrophones => "Hull Passive Hydrophone Listening Array",
            Self::ActiveQuartzDome => "Retractable Active Piezoelectric Quartz Dome",
            Self::MultiElementPassiveArray => "Multi-Element Balcony Passive Array + Active Pulse",
            Self::HighPowerSearchSonar => "High-Power Magnetostrictive Searchlight Sonar",
        }
    }
}
impl DisplayName for RadarSuite {
    fn display_name(&self) -> &'static str {
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
}
impl DisplayName for BallisticComputer {
    fn display_name(&self) -> &'static str {
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
}
impl DisplayName for AaDirector {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto (Dimensionless Match)",
            Self::None => "None / Local Optical Ring Sights",
            Self::HighAngleTachymetric => "High-Angle Tachymetric Control Director",
            Self::TriAxialStabilizedSphere => "Tri-Axially Stabilized Spherical Director",
            Self::OpticalLeadComputing => "Optical High-Angle Lead-Computing Director",
            Self::DualPurposeRadarDirector => "Dual-Purpose Gyrostabilized Radar Director",
        }
    }
}
impl DisplayName for MainDirector {
    fn display_name(&self) -> &'static str {
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
}
impl DisplayName for MobilityRegime {
    fn display_name(&self) -> &'static str {
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
}
impl DisplayName for CombatModality {
    fn display_name(&self) -> &'static str {
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
}
impl DisplayName for TacticalEchelon {
    fn display_name(&self) -> &'static str {
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
}
impl DisplayName for StyleDoctrine {
    fn display_name(&self) -> &'static str {
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
}
pub fn classification(profile: &TacticalAffinityProfile) -> String {
    format!(
        "{} × {} × {}",
        profile.dominant_echelon.display_name(),
        profile.dominant_modality.display_name(),
        profile.dominant_mobility.display_name()
    )
}
pub fn match_explanation(eval: &SystemsEvaluation) -> String {
    let f = &eval.features;
    let (a1, a2, a3, a4) = (
        eval.type1_affinity,
        eval.type2_affinity,
        eval.type3_affinity,
        eval.type4_affinity,
    );
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

    explanation
}
