//! Historical ship presets with complete full-model configurations.
//!
//! While `accuracy.rs` stores historical speed/power cases for empirical
//! calibration, this module provides fully configured `.ssr` models including
//! hull dimensions, freeboards, complete gun batteries, armour schemes, machinery
//! and secondary armaments.

use crate::model::{ResistanceModel, ShipModel, WeaponAggregate, WeaponKind, WeaponMount};

/// Descriptor for a historical ship preset.
pub struct PresetInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub country: &'static str,
    pub ship_type: &'static str,
    pub year: i32,
    pub displacement_tons: f64,
}

pub const PRESET_LIST: &[PresetInfo] = &[
    PresetInfo {
        id: "dreadnought",
        name: "HMS Dreadnought",
        country: "United Kingdom",
        ship_type: "Battleship",
        year: 1906,
        displacement_tons: 18_110.0,
    },
    PresetInfo {
        id: "bismarck",
        name: "Bismarck",
        country: "Germany",
        ship_type: "Battleship",
        year: 1940,
        displacement_tons: 45_000.0,
    },
    PresetInfo {
        id: "iowa",
        name: "USS Iowa",
        country: "United States",
        ship_type: "Battleship",
        year: 1943,
        displacement_tons: 48_000.0,
    },
    PresetInfo {
        id: "yamato",
        name: "IJN Yamato",
        country: "Japan",
        ship_type: "Battleship",
        year: 1941,
        displacement_tons: 65_000.0,
    },
    PresetInfo {
        id: "baltimore",
        name: "USS Baltimore",
        country: "United States",
        ship_type: "Heavy cruiser",
        year: 1943,
        displacement_tons: 14_500.0,
    },
    PresetInfo {
        id: "belfast",
        name: "HMS Belfast",
        country: "United Kingdom",
        ship_type: "Light cruiser",
        year: 1939,
        displacement_tons: 11_500.0,
    },
    PresetInfo {
        id: "fletcher",
        name: "USS Fletcher",
        country: "United States",
        ship_type: "Destroyer",
        year: 1942,
        displacement_tons: 2_100.0,
    },
    PresetInfo {
        id: "shimakaze",
        name: "IJN Shimakaze",
        country: "Japan",
        ship_type: "Destroyer",
        year: 1943,
        displacement_tons: 2_560.0,
    },
    PresetInfo {
        id: "hood",
        name: "HMS Hood",
        country: "United Kingdom",
        ship_type: "Battlecruiser",
        year: 1920,
        displacement_tons: 41_785.0,
    },
    PresetInfo {
        id: "essex",
        name: "USS Essex",
        country: "United States",
        ship_type: "Aircraft carrier",
        year: 1943,
        displacement_tons: 31_500.0,
    },
    PresetInfo {
        id: "kgv",
        name: "HMS King George V",
        country: "United Kingdom",
        ship_type: "Battleship",
        year: 1940,
        displacement_tons: 38_000.0,
    },
    PresetInfo {
        id: "scharnhorst",
        name: "Scharnhorst",
        country: "Germany",
        ship_type: "Battleship",
        year: 1939,
        displacement_tons: 34_850.0,
    },
];

/// Get preset ship model by identifier.
pub fn get_preset(id: &str) -> Option<ShipModel> {
    let mut ship = match id {
        "dreadnought" => preset_dreadnought(),
        "bismarck" => preset_bismarck(),
        "iowa" => preset_iowa(),
        "yamato" => preset_yamato(),
        "baltimore" => preset_baltimore(),
        "belfast" => preset_belfast(),
        "fletcher" => preset_fletcher(),
        "shimakaze" => preset_shimakaze(),
        "hood" => preset_hood(),
        "essex" => preset_essex(),
        "kgv" => preset_kgv(),
        "scharnhorst" => preset_scharnhorst(),
        _ => return None,
    };
    ship.calculate();
    Some(ship)
}

fn preset_dreadnought() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "HMS Dreadnought".into();
    ship.country = "United Kingdom".into();
    ship.ship_type = "Battleship".into();

    ship.hull.length = 527.0;
    ship.hull.beam = 82.0;
    ship.hull.beam_bulges = 82.0;
    ship.hull.draught = 26.5;
    ship.hull.block = 0.56;
    ship.hull.date_hull = 1906;
    ship.hull.date_engine = 1906;

    ship.free.fc_length = 0.30;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.20;
    ship.free.free_fc_for = 28.0;
    ship.free.free_fc_aft = 26.0;
    ship.free.free_fd_for = 20.0;
    ship.free.free_fd_aft = 18.0;
    ship.free.free_ad_for = 17.0;
    ship.free.free_ad_aft = 17.0;
    ship.free.free_qd_for = 17.0;
    ship.free.free_qd_aft = 17.0;

    // 10 x 12"/45 Mk X guns in 5 twin turrets (1 fore, 2 wing, 2 aft)
    let g1 = ship.battery_mut(0);
    g1.barrels = 10;
    g1.calibre = 12.0;
    g1.cal_length = 45.0;
    g1.date_gun = 1906;
    g1.gun_type = 1; // Breech loading
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 80;
    g1.mounts = 5;
    g1.guns_in_mount_g1 = 3; // Twin mount
    g1.layout_g1 = 2; // Centreline ends (aft >= fore: 1 fore, 2 aft)
    g1.mount_super_g1 = 0;
    g1.guns_in_mount_g2 = 3; // Twin mount
    g1.layout_g2 = 9; // Sides - distributed (wing turrets)
    g1.mount_deck_g2 = 2;
    g1.mount_super_g2 = 0;
    g1.arm_face = 11.0;
    g1.arm_back = 8.0;
    g1.arm_bottom = 8.0;

    // 27 x 12-pdr (3") anti-torpedo boat guns
    let g2 = ship.battery_mut(1);
    g2.barrels = 27;
    g2.calibre = 3.0;
    g2.cal_length = 50.0;
    g2.date_gun = 1906;
    g2.gun_type = 2; // Quick firing
    g2.mount_type = 5;
    g2.magazine = 300;
    g2.mounts = 27;
    g2.guns_in_mount_g1 = 0; // Single
    g2.layout_g1 = 9;
    g2.mount_super_g1 = 27;

    ship.armour.belt_main = 11.0;
    ship.armour.belt_main_length = 300.0;
    ship.armour.belt_main_height = 12.5;
    ship.armour.belt_ends = 6.0;
    ship.armour.belt_ends_length = 120.0;
    ship.armour.belt_ends_height = 10.0;
    ship.armour.belt_upper = 8.0;
    ship.armour.belt_upper_length = 260.0;
    ship.armour.belt_upper_height = 8.0;
    ship.armour.deck = 1.75;
    ship.armour.deck_fc = 0.75;
    ship.armour.deck_qd = 1.0;
    ship.armour.bulkhead = 8.0;
    ship.armour.bulkhead_length = 260.0;
    ship.armour.bulkhead_height = 14.0;
    ship.armour.bulkhead_beam = 60.0;
    ship.armour.conning_for = 11.0;

    ship.engine.fuel[0] = true; // Coal
    ship.engine.fuel[1] = true; // Oil spray
    ship.engine.engine_types[2] = true; // Steam turbines
    ship.engine.drive[0] = true; // Direct
    ship.engine.speed_max = 21.0;
    ship.engine.speed_cruise = 10.0;
    ship.engine.range = 6620.0;
    ship.engine.pct_coal = 75.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.767;
    // Early direct-drive Parsons turbine + Babcock & Wilcox coal/oil boiler plant (~23,000 shp in 2,050t)
    ship.engine.machinery_weight_factor = 1.23;

    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 5,
            loads: 2,
            size: 18.0,
            length: 17.0,
            location: 4,
            ..Default::default()
        },
        ..Default::default()
    };
    ship
}

fn preset_bismarck() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "Bismarck".into();
    ship.country = "Germany".into();
    ship.ship_type = "Battleship".into();

    ship.hull.length = 823.0;
    ship.hull.beam = 118.0;
    ship.hull.beam_bulges = 118.0;
    ship.hull.draught = 30.5;
    ship.hull.block = 0.56;
    ship.hull.date_hull = 1940;
    ship.hull.date_engine = 1940;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 29.5;
    ship.free.free_fc_aft = 22.0;
    ship.free.free_fd_for = 20.0;
    ship.free.free_fd_aft = 17.0;
    ship.free.free_ad_for = 16.0;
    ship.free.free_ad_aft = 15.0;
    ship.free.free_qd_for = 15.0;
    ship.free.free_qd_aft = 14.5;
    ship.free.bow = 1; // Bulbous bow
    ship.free.stern = 0; // Cruiser stern

    // 8 x 38cm SK C/34 guns in 4 twin turrets
    let g1 = ship.battery_mut(0);
    g1.barrels = 8;
    g1.calibre = 15.0;
    g1.cal_length = 52.0;
    g1.date_gun = 1940;
    g1.gun_type = 1;
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 120;
    g1.mounts = 4;
    g1.guns_in_mount_g1 = 3; // Twin
    g1.layout_g1 = 1; // Ends (fore >= aft)
    g1.mount_super_g1 = 2; // Superfiring fore & aft
    g1.arm_face = 14.2;
    g1.arm_back = 8.7;
    g1.arm_bottom = 13.4;

    // 12 x 15cm SK C/28 secondary in 6 twin turrets
    let g2 = ship.battery_mut(1);
    g2.barrels = 12;
    g2.calibre = 5.9;
    g2.cal_length = 55.0;
    g2.date_gun = 1940;
    g2.gun_type = 2;
    g2.mount_type = 3;
    g2.magazine = 150;
    g2.mounts = 6;
    g2.guns_in_mount_g1 = 3; // Twin
    g2.layout_g1 = 9; // Sides
    g2.mount_super_g1 = 6;
    g2.arm_face = 3.9;

    // 16 x 10.5cm FlaK
    let g3 = ship.battery_mut(2);
    g3.barrels = 16;
    g3.calibre = 4.1;
    g3.cal_length = 65.0;
    g3.date_gun = 1940;
    g3.gun_type = 3; // AA
    g3.mount_type = 5;
    g3.magazine = 400;
    g3.mounts = 8;
    g3.guns_in_mount_g1 = 3; // Twin
    g3.layout_g1 = 9;

    ship.armour.belt_main = 12.6;
    ship.armour.belt_main_length = 560.0;
    ship.armour.belt_main_height = 15.8;
    ship.armour.belt_ends = 2.4;
    ship.armour.belt_ends_length = 160.0;
    ship.armour.belt_ends_height = 12.0;
    ship.armour.belt_upper = 5.7;
    ship.armour.belt_upper_length = 560.0;
    ship.armour.belt_upper_height = 10.0;
    ship.armour.deck = 3.9;
    ship.armour.deck_fc = 2.0;
    ship.armour.bulkhead = 8.7;
    ship.armour.bulkhead_length = 500.0;
    ship.armour.bulkhead_height = 18.0;
    ship.armour.bulkhead_beam = 85.0;
    ship.armour.bulge = 1.8;
    ship.armour.conning_for = 13.8;

    ship.engine.fuel[1] = true; // Oil
    ship.engine.engine_types[2] = true; // Turbines
    ship.engine.drive[1] = true; // Geared
    ship.engine.speed_max = 30.0;
    ship.engine.speed_cruise = 19.0;
    ship.engine.range = 8500.0;
    ship.engine.screws = 3;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.767;
    ship.engine.machinery_weight_factor = 1.13;
    ship
}

fn preset_iowa() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "USS Iowa".into();
    ship.country = "United States".into();
    ship.ship_type = "Battleship".into();

    ship.hull.length = 860.0;
    ship.hull.beam = 108.0;
    ship.hull.beam_bulges = 108.0;
    ship.hull.draught = 36.0;
    ship.hull.block = 0.60;
    ship.hull.date_hull = 1943;
    ship.hull.date_engine = 1943;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 37.0;
    ship.free.free_fc_aft = 24.0;
    ship.free.free_fd_for = 24.0;
    ship.free.free_fd_aft = 18.0;
    ship.free.free_ad_for = 17.5;
    ship.free.free_ad_aft = 17.0;
    ship.free.free_qd_for = 17.0;
    ship.free.free_qd_aft = 16.5;
    ship.free.bow = 2; // Bulbous forward
    ship.free.stern = 1; // Transom stern

    // 9 x 16"/50 Mark 7 guns in 3 triple turrets (2 fore, 1 aft)
    let g1 = ship.battery_mut(0);
    g1.barrels = 9;
    g1.calibre = 16.0;
    g1.cal_length = 50.0;
    g1.date_gun = 1943;
    g1.gun_type = 1;
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 130;
    g1.mounts = 3;
    g1.guns_in_mount_g1 = 6; // Triple mount
    g1.layout_g1 = 1; // Centreline ends
    g1.mount_super_g1 = 1; // Fore superfiring
    g1.arm_face = 17.0;
    g1.arm_back = 12.0;
    g1.arm_bottom = 7.25;

    // 20 x 5"/38 Mark 12 DP guns in 10 twin mounts
    let g2 = ship.battery_mut(1);
    g2.barrels = 20;
    g2.calibre = 5.0;
    g2.cal_length = 38.0;
    g2.date_gun = 1943;
    g2.gun_type = 4; // Dual purpose
    g2.mount_type = 5;
    g2.magazine = 450;
    g2.mounts = 10;
    g2.guns_in_mount_g1 = 3; // Twin
    g2.layout_g1 = 9; // Sides
    g2.mount_super_g1 = 6;
    g2.arm_face = 2.5;

    ship.armour.belt_main = 12.1;
    ship.armour.belt_main_length = 540.0;
    ship.armour.belt_main_height = 18.0;
    ship.armour.belts_incline = 19.0;
    ship.armour.deck = 6.0;
    ship.armour.deck_fc = 1.5;
    ship.armour.bulkhead = 11.3;
    ship.armour.bulkhead_length = 480.0;
    ship.armour.bulkhead_height = 20.0;
    ship.armour.bulkhead_beam = 80.0;
    ship.armour.conning_for = 17.3;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 33.0;
    ship.engine.speed_cruise = 15.0;
    ship.engine.range = 15000.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.767;
    // High-pressure 600 psi B&W boiler + GE geared turbine plant (212,000 shp in 4,900t)
    ship.engine.machinery_weight_factor = 0.87;
    ship
}

fn preset_yamato() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "IJN Yamato".into();
    ship.country = "Japan".into();
    ship.ship_type = "Battleship".into();

    ship.hull.length = 862.0;
    ship.hull.beam = 121.0;
    ship.hull.beam_bulges = 127.0;
    ship.hull.draught = 34.0;
    ship.hull.block = 0.61;
    ship.hull.date_hull = 1941;
    ship.hull.date_engine = 1941;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 33.0;
    ship.free.free_fc_aft = 25.0;
    ship.free.free_fd_for = 22.0;
    ship.free.free_fd_aft = 18.0;
    ship.free.free_ad_for = 17.0;
    ship.free.free_ad_aft = 16.5;
    ship.free.free_qd_for = 16.5;
    ship.free.free_qd_aft = 16.0;
    ship.free.bow = 2; // Bulbous forward
    ship.free.stern = 1; // Transom

    // 9 x 46cm (18.1") / 45 Type 94 in 3 triple turrets
    let g1 = ship.battery_mut(0);
    g1.barrels = 9;
    g1.calibre = 18.1;
    g1.cal_length = 45.0;
    g1.date_gun = 1941;
    g1.gun_type = 1;
    g1.mount_type = 3;
    g1.magazine = 100;
    g1.mounts = 3;
    g1.guns_in_mount_g1 = 6; // Triple
    g1.layout_g1 = 1;
    g1.mount_super_g1 = 1;
    g1.arm_face = 25.6;
    g1.arm_back = 10.0;
    g1.arm_bottom = 8.0;

    // 12 x 15.5cm (6.1") / 60 Type 3 in 4 triple turrets
    let g2 = ship.battery_mut(1);
    g2.barrels = 12;
    g2.calibre = 6.1;
    g2.cal_length = 60.0;
    g2.date_gun = 1941;
    g2.gun_type = 2;
    g2.mount_type = 3;
    g2.magazine = 150;
    g2.mounts = 4;
    g2.guns_in_mount_g1 = 6; // Triple
    g2.layout_g1 = 1;
    g2.mount_super_g1 = 2;
    g2.arm_face = 1.0;

    // 12 x 12.7cm (5") Type 89 DP in 6 twin mounts
    let g3 = ship.battery_mut(2);
    g3.barrels = 12;
    g3.calibre = 5.0;
    g3.cal_length = 40.0;
    g3.date_gun = 1941;
    g3.gun_type = 4;
    g3.mount_type = 5;
    g3.magazine = 300;
    g3.mounts = 6;
    g3.guns_in_mount_g1 = 3;
    g3.layout_g1 = 9;

    ship.armour.belt_main = 16.1;
    ship.armour.belt_main_length = 450.0;
    ship.armour.belt_main_height = 19.5;
    ship.armour.belts_incline = 20.0;
    ship.armour.deck = 9.1;
    ship.armour.deck_fc = 2.0;
    ship.armour.bulkhead = 13.8;
    ship.armour.bulkhead_length = 420.0;
    ship.armour.bulkhead_height = 22.0;
    ship.armour.bulkhead_beam = 95.0;
    ship.armour.bulge = 4.0;
    ship.armour.conning_for = 19.7;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 27.0;
    ship.engine.speed_cruise = 16.0;
    ship.engine.range = 7200.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.767;
    // 12 Kampon boilers + 4 geared turbine sets (150,000 shp in 5,215t => ~28.8 shp/ton)
    ship.engine.machinery_weight_factor = 1.60;
    ship
}

fn preset_baltimore() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "USS Baltimore".into();
    ship.country = "United States".into();
    ship.ship_type = "Heavy cruiser".into();

    ship.hull.length = 673.0;
    ship.hull.beam = 71.0;
    ship.hull.beam_bulges = 71.0;
    ship.hull.draught = 22.0;
    ship.hull.block = 0.53;
    ship.hull.date_hull = 1943;
    ship.hull.date_engine = 1943;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 28.0;
    ship.free.free_fc_aft = 20.0;
    ship.free.free_fd_for = 18.0;
    ship.free.free_fd_aft = 15.0;
    ship.free.free_ad_for = 14.0;
    ship.free.free_ad_aft = 13.0;
    ship.free.free_qd_for = 13.0;
    ship.free.free_qd_aft = 12.0;

    // 9 x 8"/55 Mark 12/15 in 3 triple turrets
    let g1 = ship.battery_mut(0);
    g1.barrels = 9;
    g1.calibre = 8.0;
    g1.cal_length = 55.0;
    g1.date_gun = 1943;
    g1.gun_type = 1;
    g1.mount_type = 3;
    g1.magazine = 150;
    g1.mounts = 3;
    g1.guns_in_mount_g1 = 6; // Triple
    g1.layout_g1 = 1;
    g1.mount_super_g1 = 1;
    g1.arm_face = 8.0;
    g1.arm_back = 3.75;
    g1.arm_bottom = 2.5;

    // 12 x 5"/38 DP in 6 twin mounts
    let g2 = ship.battery_mut(1);
    g2.barrels = 12;
    g2.calibre = 5.0;
    g2.cal_length = 38.0;
    g2.date_gun = 1943;
    g2.gun_type = 4;
    g2.mount_type = 5;
    g2.magazine = 400;
    g2.mounts = 6;
    g2.guns_in_mount_g1 = 3;
    g2.layout_g1 = 9;
    g2.mount_super_g1 = 4;
    g2.arm_face = 1.0;

    ship.armour.belt_main = 6.0;
    ship.armour.belt_main_length = 380.0;
    ship.armour.belt_main_height = 12.0;
    ship.armour.deck = 3.0;
    ship.armour.bulkhead = 5.0;
    ship.armour.bulkhead_length = 360.0;
    ship.armour.bulkhead_height = 14.0;
    ship.armour.bulkhead_beam = 55.0;
    ship.armour.conning_for = 6.5;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 33.0;
    ship.engine.speed_cruise = 15.0;
    ship.engine.range = 10000.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.919;
    // Compact 600 psi B&W cruiser machinery (120,000 shp in 2,100t => ~57 shp/ton)
    ship.engine.machinery_weight_factor = 0.65;
    ship.weapons.misc_weight_vital = 650.0;
    ship
}

fn preset_belfast() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "HMS Belfast".into();
    ship.country = "United Kingdom".into();
    ship.ship_type = "Light cruiser".into();

    ship.hull.length = 613.0;
    ship.hull.beam = 66.0;
    ship.hull.beam_bulges = 66.0;
    ship.hull.draught = 19.5;
    ship.hull.block = 0.515;
    ship.hull.date_hull = 1939;
    ship.hull.date_engine = 1939;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 27.0;
    ship.free.free_fc_aft = 19.0;
    ship.free.free_fd_for = 17.0;
    ship.free.free_fd_aft = 14.0;
    ship.free.free_ad_for = 13.5;
    ship.free.free_ad_aft = 13.0;
    ship.free.free_qd_for = 13.0;
    ship.free.free_qd_aft = 12.0;

    // 12 x 6"/50 Mk XXIII in 4 triple turrets (2 fore, 2 aft)
    let g1 = ship.battery_mut(0);
    g1.barrels = 12;
    g1.calibre = 6.0;
    g1.cal_length = 50.0;
    g1.date_gun = 1939;
    g1.gun_type = 2;
    g1.mount_type = 3;
    g1.magazine = 200;
    g1.mounts = 4;
    g1.guns_in_mount_g1 = 6; // Triple
    g1.layout_g1 = 1;
    g1.mount_super_g1 = 2;
    g1.arm_face = 4.0;
    g1.arm_back = 2.0;
    g1.arm_bottom = 2.0;

    // 12 x 4"/45 QF Mk XVI DP in 6 twin mounts
    let g2 = ship.battery_mut(1);
    g2.barrels = 12;
    g2.calibre = 4.0;
    g2.cal_length = 45.0;
    g2.date_gun = 1939;
    g2.gun_type = 4;
    g2.mount_type = 5;
    g2.magazine = 350;
    g2.mounts = 6;
    g2.guns_in_mount_g1 = 3;
    g2.layout_g1 = 9;

    ship.armour.belt_main = 4.5;
    ship.armour.belt_main_length = 340.0;
    ship.armour.belt_main_height = 11.0;
    ship.armour.deck = 3.0;
    ship.armour.bulkhead = 2.5;
    ship.armour.bulkhead_length = 300.0;
    ship.armour.bulkhead_height = 12.0;
    ship.armour.bulkhead_beam = 50.0;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 32.5;
    ship.engine.speed_cruise = 13.0;
    ship.engine.range = 8600.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.919;
    // Admiralty 3-drum compact unit-machinery cruiser plant (80,000 shp in 1,550t => ~51.6 shp/ton)
    ship.engine.machinery_weight_factor = 0.56;

    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 6,
            loads: 2,
            size: 21.0,
            length: 22.0,
            location: 9,
            ..Default::default()
        },
        misc_weight_vital: 850.0,
        ..Default::default()
    };
    ship
}

fn preset_fletcher() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "USS Fletcher".into();
    ship.country = "United States".into();
    ship.ship_type = "Destroyer".into();

    ship.hull.length = 376.0;
    ship.hull.beam = 40.0;
    ship.hull.beam_bulges = 40.0;
    ship.hull.draught = 13.0;
    ship.hull.block = 0.48;
    ship.hull.date_hull = 1942;
    ship.hull.date_engine = 1942;

    ship.free.fc_length = 0.40;
    ship.free.fd_length = 0.20;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 22.0;
    ship.free.free_fc_aft = 16.0;
    ship.free.free_fd_for = 13.0;
    ship.free.free_fd_aft = 10.0;
    ship.free.free_ad_for = 9.5;
    ship.free.free_ad_aft = 9.0;
    ship.free.free_qd_for = 9.0;
    ship.free.free_qd_aft = 8.5;

    // 5 x 5"/38 Mark 12 in 5 single enclosed mounts
    let g1 = ship.battery_mut(0);
    g1.barrels = 5;
    g1.calibre = 5.0;
    g1.cal_length = 38.0;
    g1.date_gun = 1942;
    g1.gun_type = 4; // DP
    g1.mount_type = 5;
    g1.magazine = 350;
    g1.mounts = 5;
    g1.guns_in_mount_g1 = 0; // Single
    g1.layout_g1 = 1;
    g1.mount_super_g1 = 2; // Superfiring fore & aft
    g1.arm_face = 0.75;

    // Splinter protection only (destroyer)
    ship.armour.belt_main = 0.75;
    ship.armour.belt_main_length = 150.0;
    ship.armour.belt_main_height = 8.0;
    ship.armour.deck = 0.5;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 36.5;
    ship.engine.speed_cruise = 15.0;
    ship.engine.range = 5500.0;
    ship.engine.screws = 2;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 1.017;
    // WWII compact lightweight high-pressure destroyer machinery (60,000 shp in ~700t => ~85 shp/ton)
    ship.engine.machinery_weight_factor = 0.50;
    ship.steadiness = 10;

    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 10,
            loads: 1,
            size: 21.0,
            length: 22.0,
            location: 4,
            ..Default::default()
        },
        dc1: WeaponMount {
            kind: WeaponKind::DepthCharge,
            number: 6,
            loads: 5,
            size: 300.0,
            location: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    ship
}

fn preset_shimakaze() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "IJN Shimakaze".into();
    ship.country = "Japan".into();
    ship.ship_type = "Destroyer".into();

    ship.hull.length = 413.0;
    ship.hull.beam = 37.0;
    ship.hull.beam_bulges = 37.0;
    ship.hull.draught = 16.0;
    ship.hull.block = 0.44;
    ship.hull.date_hull = 1943;
    ship.hull.date_engine = 1943;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.20;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 21.0;
    ship.free.free_fc_aft = 15.0;
    ship.free.free_fd_for = 12.0;
    ship.free.free_fd_aft = 9.5;
    ship.free.free_ad_for = 9.0;
    ship.free.free_ad_aft = 8.5;
    ship.free.free_qd_for = 8.5;
    ship.free.free_qd_aft = 8.0;

    // 6 x 12.7cm (5") / 50 Type 3 in 3 twin mounts (1 fore, 2 aft superfiring)
    let g1 = ship.battery_mut(0);
    g1.barrels = 6;
    g1.calibre = 5.0;
    g1.cal_length = 50.0;
    g1.date_gun = 1943;
    g1.gun_type = 4;
    g1.mount_type = 5;
    g1.magazine = 200;
    g1.mounts = 3;
    g1.guns_in_mount_g1 = 3; // Twin
    g1.layout_g1 = 1;
    g1.mount_super_g1 = 1;
    g1.arm_face = 0.5;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 40.9;
    ship.engine.speed_cruise = 18.0;
    ship.engine.range = 6000.0;
    ship.engine.screws = 2;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 1.017;
    // Experimental high-temperature/high-pressure Kampon machinery (75,000+ shp in ~820t => ~92 shp/ton)
    ship.engine.machinery_weight_factor = 0.42;
    ship.steadiness = 5;

    // 15 x 61cm (24") Type 93 Long Lance torpedoes in 3 quintuple mounts
    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 15,
            loads: 1,
            size: 24.0,
            length: 29.5,
            location: 4,
            ..Default::default()
        },
        dc1: WeaponMount {
            kind: WeaponKind::DepthCharge,
            number: 6,
            loads: 3,
            size: 350.0,
            location: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    ship
}

fn preset_hood() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "HMS Hood".into();
    ship.country = "United Kingdom".into();
    ship.ship_type = "Battlecruiser".into();

    ship.hull.length = 850.0;
    ship.hull.beam = 104.0;
    ship.hull.beam_bulges = 104.0;
    ship.hull.draught = 28.5;
    ship.hull.block = 0.578;
    ship.hull.date_hull = 1920;
    ship.hull.date_engine = 1920;

    ship.free.fc_length = 0.45;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.20;
    ship.free.free_fc_for = 29.0;
    ship.free.free_fc_aft = 25.0;
    ship.free.free_fd_for = 21.0;
    ship.free.free_fd_aft = 19.0;
    ship.free.free_ad_for = 17.5;
    ship.free.free_ad_aft = 17.0;
    ship.free.free_qd_for = 17.0;
    ship.free.free_qd_aft = 16.5;

    // 8 x 15"/42 BL Mk I in 4 twin turrets (2 fore, 2 aft superfiring)
    let g1 = ship.battery_mut(0);
    g1.barrels = 8;
    g1.calibre = 15.0;
    g1.cal_length = 42.0;
    g1.date_gun = 1920;
    g1.gun_type = 1;
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 120;
    g1.mounts = 4;
    g1.guns_in_mount_g1 = 3; // Twin
    g1.layout_g1 = 1; // Centreline ends
    g1.mount_super_g1 = 2; // Superfiring fore & aft
    g1.arm_face = 15.0;
    g1.arm_back = 11.0;
    g1.arm_bottom = 12.0;

    // 12 x 5.5"/50 BL Mk I secondary guns
    let g2 = ship.battery_mut(1);
    g2.barrels = 12;
    g2.calibre = 5.5;
    g2.cal_length = 50.0;
    g2.date_gun = 1920;
    g2.gun_type = 2; // QF/BL
    g2.mount_type = 5;
    g2.magazine = 150;
    g2.mounts = 12;
    g2.guns_in_mount_g1 = 0; // Single
    g2.layout_g1 = 9; // Sides
    g2.mount_super_g1 = 10;
    g2.arm_face = 1.0;

    // 4 x 4"/45 QF Mk V AA
    let g3 = ship.battery_mut(2);
    g3.barrels = 4;
    g3.calibre = 4.0;
    g3.cal_length = 45.0;
    g3.date_gun = 1920;
    g3.gun_type = 3; // AA
    g3.mount_type = 5;
    g3.magazine = 200;
    g3.mounts = 4;
    g3.guns_in_mount_g1 = 0;
    g3.layout_g1 = 9;

    ship.armour.belt_main = 12.0;
    ship.armour.belt_main_length = 562.0;
    ship.armour.belt_main_height = 12.5;
    ship.armour.belts_incline = 12.0;
    ship.armour.belt_upper = 7.0;
    ship.armour.belt_upper_length = 500.0;
    ship.armour.belt_upper_height = 10.5;
    ship.armour.belt_ends = 5.0;
    ship.armour.belt_ends_length = 160.0;
    ship.armour.belt_ends_height = 9.0;
    ship.armour.deck = 3.0;
    ship.armour.deck_fc = 1.5;
    ship.armour.deck_qd = 2.0;
    ship.armour.bulkhead = 5.0;
    ship.armour.bulkhead_length = 500.0;
    ship.armour.bulkhead_height = 16.0;
    ship.armour.bulkhead_beam = 80.0;
    ship.armour.bulge = 1.5;
    ship.armour.conning_for = 11.0;

    ship.engine.fuel[1] = true; // Oil
    ship.engine.engine_types[2] = true; // Turbines
    ship.engine.drive[1] = true; // Geared
    ship.engine.speed_max = 31.0;
    ship.engine.speed_cruise = 14.0;
    ship.engine.range = 7500.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.767;
    // 24 Yarrow small-tube boilers + 4 Brown-Curtis geared turbines (144,000 shp in 5,300t)
    ship.engine.machinery_weight_factor = 1.21;

    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 6,
            loads: 2,
            size: 21.0,
            length: 22.0,
            location: 4,
            ..Default::default()
        },
        misc_weight_vital: 2200.0,
        ..Default::default()
    };
    ship
}

fn preset_essex() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "USS Essex".into();
    ship.country = "United States".into();
    ship.ship_type = "Aircraft carrier".into();

    ship.hull.length = 820.0;
    ship.hull.beam = 93.0;
    ship.hull.beam_bulges = 93.0;
    ship.hull.draught = 27.5;
    ship.hull.block = 0.525;
    ship.hull.date_hull = 1943;
    ship.hull.date_engine = 1943;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.35;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 28.0;
    ship.free.free_fc_aft = 26.0;
    ship.free.free_fd_for = 25.0;
    ship.free.free_fd_aft = 24.0;
    ship.free.free_ad_for = 24.0;
    ship.free.free_ad_aft = 23.0;
    ship.free.free_qd_for = 23.0;
    ship.free.free_qd_aft = 22.0;
    ship.free.bow = 2; // Bulbous
    ship.free.stern = 1; // Transom

    // 12 x 5"/38 Mark 12 DP guns (4 twin turrets on starboard island + 4 single gallery mounts)
    let g1 = ship.battery_mut(0);
    g1.barrels = 12;
    g1.calibre = 5.0;
    g1.cal_length = 38.0;
    g1.date_gun = 1943;
    g1.gun_type = 4; // DP
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 400;
    g1.mounts = 8;
    g1.guns_in_mount_g1 = 3; // Twin (4 mounts = 8 guns)
    g1.layout_g1 = 9; // Sides
    g1.mount_super_g1 = 4;
    g1.guns_in_mount_g2 = 0; // Single (4 mounts = 4 guns)
    g1.layout_g2 = 9;
    g1.mount_deck_g2 = 4;
    g1.arm_face = 1.5;
    g1.arm_back = 1.0;
    g1.arm_bottom = 1.0;

    // 32 x 40mm/56 Bofors AA in 8 quad mounts
    let g2 = ship.battery_mut(1);
    g2.barrels = 32;
    g2.calibre = 1.57;
    g2.cal_length = 56.0;
    g2.date_gun = 1943;
    g2.gun_type = 5; // Auto rapid fire
    g2.mount_type = 5;
    g2.magazine = 600;
    g2.mounts = 8;
    g2.guns_in_mount_g1 = 9; // Quad mount
    g2.layout_g1 = 9;
    g2.mount_super_g1 = 8;

    ship.armour.belt_main = 4.0;
    ship.armour.belt_main_length = 508.0;
    ship.armour.belt_main_height = 8.5;
    ship.armour.deck = 1.5;
    ship.armour.deck_fc = 0.75;
    ship.armour.deck_qd = 0.75;
    ship.armour.bulkhead = 4.0;
    ship.armour.bulkhead_length = 508.0;
    ship.armour.bulkhead_height = 10.0;
    ship.armour.bulkhead_beam = 70.0;
    ship.armour.conning_for = 1.5;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 33.0;
    ship.engine.speed_cruise = 15.0;
    ship.engine.range = 15000.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.878;
    // 8 B&W 600 psi boilers + 4 Westinghouse geared turbines (150,000 shp in 3,150t => ~47.6 shp/ton)
    ship.engine.machinery_weight_factor = 0.83;

    // Aviation ordnance (aerial torpedoes & bombs), flight deck arresting/catapult gear & air group allowance
    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 36,
            loads: 2,
            size: 22.4,
            length: 18.0,
            location: 4,
            ..Default::default()
        },
        misc_weight_deck: 1150.0,
        ..Default::default()
    };
    ship
}

fn preset_kgv() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "HMS King George V".into();
    ship.country = "United Kingdom".into();
    ship.ship_type = "Battleship".into();

    ship.hull.length = 740.0;
    ship.hull.beam = 103.0;
    ship.hull.beam_bulges = 103.0;
    ship.hull.draught = 29.0;
    ship.hull.block = 0.59;
    ship.hull.date_hull = 1940;
    ship.hull.date_engine = 1940;

    ship.free.fc_length = 0.35;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.20;
    ship.free.free_fc_for = 28.0;
    ship.free.free_fc_aft = 25.0;
    ship.free.free_fd_for = 22.0;
    ship.free.free_fd_aft = 20.0;
    ship.free.free_ad_for = 18.0;
    ship.free.free_ad_aft = 17.0;
    ship.free.free_qd_for = 17.0;
    ship.free.free_qd_aft = 16.5;

    // 10 x 14"/45 BL Mk VII in 2 quadruple turrets (A, Y) and 1 superfiring twin turret (B)
    let g1 = ship.battery_mut(0);
    g1.barrels = 10;
    g1.calibre = 14.0;
    g1.cal_length = 45.0;
    g1.date_gun = 1940;
    g1.gun_type = 1; // BL
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 100;
    g1.mounts = 3;
    g1.guns_in_mount_g1 = 9; // Quad mount (2 mounts = 8 guns)
    g1.layout_g1 = 1; // Ends
    g1.mounts_g1 = 2;
    g1.per_mount_g1 = 4;
    g1.guns_in_mount_g2 = 3; // Twin mount (1 mount = 2 guns)
    g1.layout_g2 = 3; // Fore deck forward
    g1.mounts_g2 = 1;
    g1.per_mount_g2 = 2;
    g1.mount_super_g2 = 1;
    g1.arm_face = 12.8;
    g1.arm_back = 7.0;
    g1.arm_bottom = 12.8;

    // 16 x 5.25"/50 QF Mk I DP in 8 twin mounts
    let g2 = ship.battery_mut(1);
    g2.barrels = 16;
    g2.calibre = 5.25;
    g2.cal_length = 50.0;
    g2.date_gun = 1940;
    g2.gun_type = 4; // DP
    g2.mount_type = 5;
    g2.magazine = 300;
    g2.mounts = 8;
    g2.guns_in_mount_g1 = 3; // Twin
    g2.layout_g1 = 9; // Sides
    g2.mount_super_g1 = 4;
    g2.arm_face = 1.5;
    g2.arm_back = 1.0;
    g2.arm_bottom = 1.0;

    // Heavy vertical belt (14.7" main belt + 5.5" upper strake)
    ship.armour.belt_main = 14.7;
    ship.armour.belt_main_length = 396.0;
    ship.armour.belt_main_height = 15.0;
    ship.armour.belt_upper = 5.5;
    ship.armour.belt_upper_length = 396.0;
    ship.armour.belt_upper_height = 8.5;
    ship.armour.belts_incline = 0.0; // Vertical cemented belt
    ship.armour.deck = 5.9;
    ship.armour.deck_fc = 4.9;
    ship.armour.deck_qd = 4.9;
    ship.armour.deck_coverage = 6;
    ship.armour.bulkhead = 11.8;
    ship.armour.bulkhead_length = 396.0;
    ship.armour.bulkhead_height = 15.0;
    ship.armour.bulkhead_beam = 90.0;
    ship.armour.bulge = 1.5;
    ship.armour.conning_for = 4.0;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 28.0;
    ship.engine.speed_cruise = 14.0;
    ship.engine.range = 14000.0;
    ship.engine.screws = 4;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.767;
    // Admiralty 3-drum boilers + Parsons geared turbines (110,000 shp in 2,720t => ~40.4 shp/ton)
    ship.engine.machinery_weight_factor = 1.016;

    ship.weapons = WeaponAggregate {
        misc_weight_vital: 2200.0,
        ..Default::default()
    };
    ship
}

fn preset_scharnhorst() -> ShipModel {
    let mut ship = ShipModel::new();
    ship.name = "Scharnhorst".into();
    ship.country = "Germany".into();
    ship.ship_type = "Battleship".into();

    ship.hull.length = 741.5;
    ship.hull.beam = 98.4;
    ship.hull.beam_bulges = 98.4;
    ship.hull.draught = 29.5;
    ship.hull.block = 0.565;
    ship.hull.date_hull = 1939;
    ship.hull.date_engine = 1939;

    ship.free.fc_length = 0.40;
    ship.free.fd_length = 0.25;
    ship.free.qd_length = 0.15;
    ship.free.free_fc_for = 27.5;
    ship.free.free_fc_aft = 23.0;
    ship.free.free_fd_for = 19.5;
    ship.free.free_fd_aft = 18.0;
    ship.free.free_ad_for = 16.5;
    ship.free.free_ad_aft = 16.0;
    ship.free.free_qd_for = 16.0;
    ship.free.free_qd_aft = 15.5;

    // 9 x 11.1"/54.5 (28.3 cm) SK C/34 in 3 triple turrets
    let g1 = ship.battery_mut(0);
    g1.barrels = 9;
    g1.calibre = 11.1;
    g1.cal_length = 54.5;
    g1.date_gun = 1934;
    g1.gun_type = 1; // BL
    g1.mount_type = 3; // Turret on barbette
    g1.magazine = 150;
    g1.mounts = 3;
    g1.guns_in_mount_g1 = 6; // Triple mount
    g1.layout_g1 = 1; // Ends
    g1.mounts_g1 = 2;
    g1.per_mount_g1 = 3;
    g1.guns_in_mount_g2 = 6; // Triple mount
    g1.layout_g2 = 3; // Fore deck forward
    g1.mounts_g2 = 1;
    g1.per_mount_g2 = 3;
    g1.mount_super_g2 = 1;
    g1.arm_face = 14.2;
    g1.arm_back = 7.1;
    g1.arm_bottom = 13.8;

    // 12 x 5.9"/55 (15 cm) SK C/28 in 4 twin + 4 single mounts
    let g2 = ship.battery_mut(1);
    g2.barrels = 12;
    g2.calibre = 5.9;
    g2.cal_length = 55.0;
    g2.date_gun = 1934;
    g2.gun_type = 2; // QF
    g2.mount_type = 3;
    g2.magazine = 150;
    g2.mounts = 8;
    g2.guns_in_mount_g1 = 3; // Twin
    g2.layout_g1 = 9;
    g2.mount_super_g1 = 4;
    g2.arm_face = 1.5;
    g2.arm_back = 1.0;
    g2.arm_bottom = 1.0;

    // 14 x 4.1"/65 (10.5 cm) SK C/33 heavy AA in 7 twin mounts
    let g3 = ship.battery_mut(2);
    g3.barrels = 14;
    g3.calibre = 4.1;
    g3.cal_length = 65.0;
    g3.date_gun = 1935;
    g3.gun_type = 4; // DP
    g3.mount_type = 5;
    g3.magazine = 400;
    g3.mounts = 7;
    g3.guns_in_mount_g1 = 3; // Twin
    g3.layout_g1 = 9;
    g3.mount_super_g1 = 4;

    // Thick main belt (13.8" / 350 mm) + 1.8" upper belt + 3.1" ends belt
    ship.armour.belt_main = 13.8;
    ship.armour.belt_main_length = 492.0;
    ship.armour.belt_main_height = 14.8;
    ship.armour.belt_upper = 1.8;
    ship.armour.belt_upper_length = 492.0;
    ship.armour.belt_upper_height = 12.0;
    ship.armour.belt_ends = 3.1;
    ship.armour.belt_ends_length = 120.0;
    ship.armour.belt_ends_height = 10.0;
    ship.armour.deck = 4.5; // 95-105 mm effective over vital spaces
    ship.armour.deck_fc = 2.0;
    ship.armour.deck_qd = 2.0;
    ship.armour.deck_coverage = 6;
    ship.armour.bulkhead = 5.9;
    ship.armour.bulkhead_length = 492.0;
    ship.armour.bulkhead_height = 15.0;
    ship.armour.bulkhead_beam = 85.0;
    ship.armour.bulge = 1.5;
    ship.armour.conning_for = 13.8;

    ship.engine.fuel[1] = true;
    ship.engine.engine_types[2] = true;
    ship.engine.drive[1] = true;
    ship.engine.speed_max = 31.5;
    ship.engine.speed_cruise = 19.0;
    ship.engine.range = 8800.0;
    ship.engine.screws = 3;
    ship.engine.resistance_model = ResistanceModel::HoltropMennen;
    ship.engine.power_factor = 0.767;
    // 12 Wagner high-pressure boilers + 3 Brown-Boveri geared turbines (160,000 shp in 2,580t => ~62 shp/ton)
    ship.engine.machinery_weight_factor = 0.708;

    // 6 x 21" (53.3 cm) torpedo tubes in 2 triple deck mounts
    ship.weapons = WeaponAggregate {
        torpedo1: WeaponMount {
            kind: WeaponKind::Torpedo,
            number: 6,
            loads: 2,
            size: 21.0,
            length: 23.0,
            location: 4,
            ..Default::default()
        },
        misc_weight_vital: 1400.0,
        ..Default::default()
    };
    ship
}

/// Published historical subsystem weight & stability reference for a preset ship.
///
/// Accounting convention (matching SpringSharp's output groups):
/// - `ref_hull_t`: hull structure, fittings and general outfit (`engine.hull_weight`).
/// - `ref_armour_t`: total armour protection including belt, deck, bulkheads,
///   conning tower and turret/barbette armour (`armour.weight`).
/// - `ref_machinery_t`: propulsion plant weight (`engine.engine_weight`).
/// - `ref_armament_t`: guns, mountings, ammunition and torpedo/ASW weights,
///   excluding heavy turret/barbette armour (`guns.guns_weight + guns.magazine + weapons.weight`).
/// - `ref_bunker_t`: normal fuel load (`engine.bunker`).
/// - `ref_gm_ft`: transverse metacentric height GM in feet (`metacentre`).
#[derive(Clone, Copy, Debug)]
pub struct PresetWeightBenchmark {
    pub id: &'static str,
    pub name: &'static str,
    pub ship_type: &'static str,
    pub year: i32,
    pub ref_displacement_t: f64,
    pub ref_hull_t: f64,
    pub ref_armour_t: f64,
    pub ref_machinery_t: f64,
    pub ref_armament_t: f64,
    pub ref_bunker_t: f64,
    pub ref_gm_ft: f64,
    pub source: &'static str,
}

pub const PRESET_WEIGHT_BENCHMARKS: &[PresetWeightBenchmark] = &[
    PresetWeightBenchmark {
        id: "dreadnought",
        name: "HMS Dreadnought",
        ship_type: "Battleship",
        year: 1906,
        ref_displacement_t: 18_110.0,
        ref_hull_t: 6_660.0,
        ref_armour_t: 6_050.0,
        ref_machinery_t: 2_050.0,
        ref_armament_t: 2_050.0,
        ref_bunker_t: 900.0,
        ref_gm_ft: 5.0,
        source: "Roberts (1992) Anatomy of the Ship: HMS Dreadnought; Conway's 1906-1921",
    },
    PresetWeightBenchmark {
        id: "bismarck",
        name: "Bismarck",
        ship_type: "Battleship",
        year: 1940,
        ref_displacement_t: 45_950.0,
        ref_hull_t: 16_800.0,
        ref_armour_t: 18_700.0,
        ref_machinery_t: 4_050.0,
        ref_armament_t: 4_550.0,
        ref_bunker_t: 3_200.0,
        ref_gm_ft: 13.1,
        source: "Garzke & Dulin (1985) Battleships: Axis and Neutral; Gröner",
    },
    PresetWeightBenchmark {
        id: "iowa",
        name: "USS Iowa",
        ship_type: "Battleship",
        year: 1943,
        ref_displacement_t: 55_000.0,
        ref_hull_t: 21_500.0,
        ref_armour_t: 18_400.0,
        ref_machinery_t: 4_900.0,
        ref_armament_t: 5_200.0,
        ref_bunker_t: 4_000.0,
        ref_gm_ft: 7.5,
        source: "Friedman (1985) U.S. Battleships; Garzke & Dulin (1995)",
    },
    PresetWeightBenchmark {
        id: "yamato",
        name: "IJN Yamato",
        ship_type: "Battleship",
        year: 1941,
        ref_displacement_t: 65_000.0,
        ref_hull_t: 23_500.0,
        ref_armour_t: 22_895.0,
        ref_machinery_t: 5_215.0,
        ref_armament_t: 7_800.0,
        ref_bunker_t: 4_200.0,
        ref_gm_ft: 11.0,
        source: "Garzke & Dulin (1985); Lengerer",
    },
    PresetWeightBenchmark {
        id: "baltimore",
        name: "USS Baltimore",
        ship_type: "Heavy cruiser",
        year: 1943,
        ref_displacement_t: 15_500.0,
        ref_hull_t: 6_900.0,
        ref_armour_t: 3_700.0,
        ref_machinery_t: 2_100.0,
        ref_armament_t: 1_550.0,
        ref_bunker_t: 1_100.0,
        ref_gm_ft: 5.5,
        source: "Friedman (1984) U.S. Cruisers",
    },
    PresetWeightBenchmark {
        id: "belfast",
        name: "HMS Belfast",
        ship_type: "Light cruiser",
        year: 1939,
        ref_displacement_t: 11_550.0,
        ref_hull_t: 5_350.0,
        ref_armour_t: 2_550.0,
        ref_machinery_t: 1_550.0,
        ref_armament_t: 1_050.0,
        ref_bunker_t: 950.0,
        ref_gm_ft: 5.1,
        source: "Raven & Roberts (1980) British Cruisers; Watton",
    },
    PresetWeightBenchmark {
        id: "fletcher",
        name: "USS Fletcher",
        ship_type: "Destroyer",
        year: 1942,
        ref_displacement_t: 2_500.0,
        ref_hull_t: 980.0,
        ref_armour_t: 140.0,
        ref_machinery_t: 690.0,
        ref_armament_t: 260.0,
        ref_bunker_t: 350.0,
        ref_gm_ft: 3.6,
        source: "Friedman (1982) U.S. Destroyers",
    },
    PresetWeightBenchmark {
        id: "shimakaze",
        name: "IJN Shimakaze",
        ship_type: "Destroyer",
        year: 1943,
        ref_displacement_t: 2_900.0,
        ref_hull_t: 1_150.0,
        ref_armour_t: 80.0,
        ref_machinery_t: 820.0,
        ref_armament_t: 360.0,
        ref_bunker_t: 420.0,
        ref_gm_ft: 3.4,
        source: "Conway's All the World's Fighting Ships 1922-1946; Stille",
    },
    PresetWeightBenchmark {
        id: "hood",
        name: "HMS Hood",
        ship_type: "Battlecruiser",
        year: 1920,
        ref_displacement_t: 41_785.0,
        ref_hull_t: 15_200.0,
        ref_armour_t: 13_550.0,
        ref_machinery_t: 5_300.0,
        ref_armament_t: 3_850.0,
        ref_bunker_t: 1_200.0,
        ref_gm_ft: 6.2,
        source: "Roberts (2001) Anatomy of the Ship: The Battlecruiser Hood; Raven & Roberts",
    },
    PresetWeightBenchmark {
        id: "essex",
        name: "USS Essex",
        ship_type: "Aircraft carrier",
        year: 1943,
        ref_displacement_t: 31_500.0,
        ref_hull_t: 19_800.0,
        ref_armour_t: 2_900.0,
        ref_machinery_t: 3_150.0,
        ref_armament_t: 650.0,
        ref_bunker_t: 3_000.0,
        ref_gm_ft: 6.5,
        source: "Friedman (1983) U.S. Aircraft Carriers; Raven (1998) Essex-Class Carriers",
    },
    PresetWeightBenchmark {
        id: "kgv",
        name: "HMS King George V",
        ship_type: "Battleship",
        year: 1940,
        ref_displacement_t: 38_000.0,
        ref_hull_t: 13_830.0,
        ref_armour_t: 12_500.0,
        ref_machinery_t: 2_720.0,
        ref_armament_t: 7_400.0,
        ref_bunker_t: 1_500.0,
        ref_gm_ft: 6.1,
        source: "Raven & Roberts (1976) British Battleships of World War Two; Friedman",
    },
    PresetWeightBenchmark {
        id: "scharnhorst",
        name: "Scharnhorst",
        ship_type: "Battleship",
        year: 1939,
        ref_displacement_t: 34_850.0,
        ref_hull_t: 12_400.0,
        ref_armour_t: 14_245.0,
        ref_machinery_t: 2_580.0,
        ref_armament_t: 4_990.0,
        ref_bunker_t: 1_500.0,
        ref_gm_ft: 7.2,
        source: "Garzke & Dulin (1985) Battleships: Axis and Neutral; Gröner (1990)",
    },
];

/// Look up the historical weight/stability benchmark for a preset by ID or ship name.
pub fn get_preset_benchmark(id_or_name: &str) -> Option<&'static PresetWeightBenchmark> {
    let q = id_or_name.trim();
    PRESET_WEIGHT_BENCHMARKS
        .iter()
        .find(|b| b.id.eq_ignore_ascii_case(q) || b.name.eq_ignore_ascii_case(q))
}

/// Per-ship comparison between SpringSharp's simulated subsystem weights/GM and
/// published naval-architecture references.
#[derive(Clone, Debug)]
pub struct PresetWeightValidationRow {
    pub benchmark: PresetWeightBenchmark,
    pub sim_displacement_t: f64,
    pub sim_hull_t: f64,
    pub sim_armour_t: f64,
    pub sim_hull_armour_t: f64,
    pub sim_gun_armour_t: f64,
    pub sim_machinery_t: f64,
    pub sim_armament_t: f64,
    pub sim_bunker_t: f64,
    pub sim_gm_ft: f64,
    pub sim_strength_comp: f64,
    pub sim_magazine_vol_m3: f64,
    pub sim_room_ratio: f64,
    pub converged: bool,
    pub disp_err_pct: f64,
    pub hull_err_pct: f64,
    /// `Some(pct)` for armoured ships (`ref_armour_t >= 500.0`), `None` for
    /// unarmoured destroyers where splinter plating is negligible.
    pub armour_err_pct: Option<f64>,
    pub machinery_err_pct: f64,
    pub armament_err_pct: f64,
    pub gm_err_pct: f64,
}

/// Summary of subsystem weight, stability and space validation across all built-in presets.
#[derive(Clone, Debug)]
pub struct WeightValidationReport {
    pub rows: Vec<PresetWeightValidationRow>,
    pub disp_mape: f64,
    pub hull_mape: f64,
    pub hull_armoured_mape: f64,
    pub armour_mape: f64,
    pub machinery_mape: f64,
    pub machinery_armoured_mape: f64,
    pub armament_mape: f64,
    pub armament_armoured_mape: f64,
    pub gm_mape: f64,
    pub physical_hull_count: usize,
    pub positive_magazine_count: usize,
    pub converged_count: usize,
}

fn rel_err_pct(sim: f64, reference: f64) -> f64 {
    if reference.abs() > f64::EPSILON {
        (sim - reference) / reference * 100.0
    } else {
        0.0
    }
}

fn mean_abs_pct(values: impl Iterator<Item = f64>) -> f64 {
    let mut sum = 0.0;
    let mut n = 0_usize;
    for v in values {
        sum += v.abs() / 100.0;
        n += 1;
    }
    if n > 0 {
        sum / n as f64
    } else {
        0.0
    }
}

/// Run subsystem weight, stability ($GM$) and internal-space validation across
/// all 8 complete historical ship presets.
pub fn validate_preset_weights() -> WeightValidationReport {
    let mut rows = Vec::with_capacity(PRESET_WEIGHT_BENCHMARKS.len());
    let mut physical_hull_count = 0_usize;
    let mut positive_magazine_count = 0_usize;
    let mut converged_count = 0_usize;

    for &b in PRESET_WEIGHT_BENCHMARKS {
        let ship = get_preset(b.id).expect("benchmark id must match a built-in preset");
        let spaces = crate::spaces::calculate_internal_spaces(&ship);

        let sim_displacement_t = ship.hull.displacement;
        let sim_hull_t = ship.engine.hull_weight;
        let sim_armour_t = ship.armour.weight;
        let sim_gun_armour_t = ship.guns.arm_weight;
        let sim_hull_armour_t = (sim_armour_t - sim_gun_armour_t).max(0.0);
        let sim_machinery_t = ship.engine.engine_weight;
        let sim_armament_t = ship.guns.guns_weight + ship.guns.magazine + ship.weapons.weight;
        let sim_bunker_t = ship.engine.bunker;
        let sim_gm_ft = ship.metacentre;

        if sim_hull_t.is_finite() && sim_hull_t > 0.0 && sim_hull_t < sim_displacement_t {
            physical_hull_count += 1;
        }
        if spaces.magazine_volume_m3 > 0.0 {
            positive_magazine_count += 1;
        }
        if ship.converged {
            converged_count += 1;
        }

        let armour_err_pct = if b.ref_armour_t >= 500.0 {
            Some(rel_err_pct(sim_armour_t, b.ref_armour_t))
        } else {
            None
        };

        rows.push(PresetWeightValidationRow {
            benchmark: b,
            sim_displacement_t,
            sim_hull_t,
            sim_armour_t,
            sim_hull_armour_t,
            sim_gun_armour_t,
            sim_machinery_t,
            sim_armament_t,
            sim_bunker_t,
            sim_gm_ft,
            sim_strength_comp: ship.strength_comp,
            sim_magazine_vol_m3: spaces.magazine_volume_m3,
            sim_room_ratio: spaces.room_hull_improved,
            converged: ship.converged,
            disp_err_pct: rel_err_pct(sim_displacement_t, b.ref_displacement_t),
            hull_err_pct: rel_err_pct(sim_hull_t, b.ref_hull_t),
            armour_err_pct,
            machinery_err_pct: rel_err_pct(sim_machinery_t, b.ref_machinery_t),
            armament_err_pct: rel_err_pct(sim_armament_t, b.ref_armament_t),
            gm_err_pct: rel_err_pct(sim_gm_ft, b.ref_gm_ft),
        });
    }

    let disp_mape = mean_abs_pct(rows.iter().map(|r| r.disp_err_pct));
    let hull_mape = mean_abs_pct(rows.iter().map(|r| r.hull_err_pct));
    let hull_armoured_mape = mean_abs_pct(
        rows.iter()
            .filter(|r| r.armour_err_pct.is_some())
            .map(|r| r.hull_err_pct),
    );
    let armour_mape = mean_abs_pct(rows.iter().filter_map(|r| r.armour_err_pct));
    let machinery_mape = mean_abs_pct(rows.iter().map(|r| r.machinery_err_pct));
    let machinery_armoured_mape = mean_abs_pct(
        rows.iter()
            .filter(|r| r.armour_err_pct.is_some())
            .map(|r| r.machinery_err_pct),
    );
    let armament_mape = mean_abs_pct(rows.iter().map(|r| r.armament_err_pct));
    let armament_armoured_mape = mean_abs_pct(
        rows.iter()
            .filter(|r| r.armour_err_pct.is_some())
            .map(|r| r.armament_err_pct),
    );
    let gm_mape = mean_abs_pct(rows.iter().map(|r| r.gm_err_pct));

    WeightValidationReport {
        rows,
        disp_mape,
        hull_mape,
        hull_armoured_mape,
        armour_mape,
        machinery_mape,
        machinery_armoured_mape,
        armament_mape,
        armament_armoured_mape,
        gm_mape,
        physical_hull_count,
        positive_magazine_count,
        converged_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_all_calculate_and_converge() {
        for info in PRESET_LIST {
            let ship = get_preset(info.id).expect("preset must exist");
            assert_eq!(ship.name, info.name);
            assert!(
                ship.hull.displacement > 0.0,
                "displacement must be positive for {}",
                info.id
            );
            assert!(
                ship.engine.hp_max > 0.0,
                "power must be positive for {}",
                info.id
            );
            assert!(
                ship.converged,
                "preset {} failed to converge weights",
                info.id
            );
        }
    }

    #[test]
    fn preset_weight_validation_runs_and_is_physical() {
        let report = validate_preset_weights();
        assert_eq!(report.rows.len(), PRESET_LIST.len());
        assert_eq!(report.physical_hull_count, PRESET_LIST.len());
        assert_eq!(report.positive_magazine_count, PRESET_LIST.len());
        assert_eq!(report.converged_count, PRESET_LIST.len());
        assert!(
            report.armour_mape < 0.15,
            "armour MAPE {:.2}% exceeded 15%",
            report.armour_mape * 100.0
        );
        assert!(
            report.gm_mape < 0.20,
            "GM MAPE {:.2}% exceeded 20%",
            report.gm_mape * 100.0
        );
        println!(
            "disp_mape={:.2}% hull_arm={:.2}% hull_all={:.2}% arm={:.2}% mach_arm={:.2}% mach_all={:.2}% wpn_arm={:.2}% wpn_all={:.2}% gm={:.2}%",
            report.disp_mape * 100.0,
            report.hull_armoured_mape * 100.0,
            report.hull_mape * 100.0,
            report.armour_mape * 100.0,
            report.machinery_armoured_mape * 100.0,
            report.machinery_mape * 100.0,
            report.armament_armoured_mape * 100.0,
            report.armament_mape * 100.0,
            report.gm_mape * 100.0,
        );
    }
}
