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
    ship.hull.block = 0.61;
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
    g1.mount_type = 5; // Deck mount
    g1.magazine = 80;
    g1.mounts = 5;
    g1.guns_in_mount_g1 = 3; // Twin mount
    g1.layout_g1 = 1; // Centreline ends
    g1.mount_super_g1 = 3;
    g1.guns_in_mount_g2 = 3; // Twin mount
    g1.layout_g2 = 9; // Sides - distributed (wing turrets)
    g1.mount_super_g2 = 2;
    g1.arm_face = 11.0;
    g1.arm_back = 8.0;
    g1.arm_bottom = 4.0;

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
    g1.arm_bottom = 4.0;

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
    ship.hull.draught = 20.0;
    ship.hull.block = 0.52;
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
    ship.armour.belt_main_length = 320.0;
    ship.armour.belt_main_height = 10.0;
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
}
