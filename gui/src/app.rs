//! Native egui/eframe GUI for the SpringSharp Rust engine.
//!
//! Built as the separate `springsharp-gui` workspace crate so the core crate
//! never resolves the eframe dependency tree.  It exposes the most important
//! ship-design inputs, a calibration page and a report/calculate page.

use eframe::egui;

use springsharp::features::demo_ship;
use springsharp::io::{load_from_file, save_to_file};
use springsharp::model::{GunBattery, ResistanceModel, ShipModel, WeaponKind, WeaponMount};
use springsharp::spaces::{
    calculate_internal_spaces, suggest_compartment_placement, CompartmentPlacement,
};
use springsharp::{
    compute_self_immunity_zone, get_preset, get_preset_benchmark, solve_displacement_scale,
    PRESET_LIST,
};

const BOWS: &[&str] = &[
    "Normal bow",
    "Bulbous bow - straight",
    "Bulbous bow - forward",
    "Ram bow",
];

const STERNS: &[&str] = &["Cruiser stern", "Transom stern", "Round stern"];

const GUN_TYPES: &[&str] = &[
    "Muzzle loading gun",
    "Breech loading gun",
    "Quick firing gun",
    "Anti-air gun",
    "Dual purpose gun",
    "Auto rapid fire gun",
    "Machine gun",
];

const MOUNT_TYPES: &[&str] = &[
    "Broadside",
    "Coles/Ericsson turret",
    "Open barbette",
    "Turret on barbette",
    "Deck and hoist",
    "Deck mount",
    "Casemate",
];

const GUNS_IN_MOUNT: &[&str] = &[
    "Single mount (1)",
    "2 row twin mount (2)",
    "4 row quad mount (4)",
    "Twin mount (2)",
    "2-gun mount (2)",
    "2 row quad mount (4)",
    "Triple mount (3)",
    "3-gun mount (3)",
    "2 row sextuple mount (6)",
    "Quad mount (4)",
    "4-gun mount (4)",
    "2 row octuple mount (8)",
    "Quintuple mount (5)",
    "5-gun mount (5)",
    "2 row decuple mount (10)",
];

const LAYOUTS: &[&str] = &[
    "Centreline - distributed",
    "Centreline - ends (fore >= aft)",
    "Centreline - ends (aft >= fore)",
    "Centreline - fore deck forward",
    "Centreline - fore deck",
    "Centreline - fore deck aft",
    "Centreline - aft deck forward",
    "Centreline - aft deck",
    "Centreline - aft deck aft",
    "Sides - distributed",
    "Sides - ends (fore >= aft)",
    "Sides - ends (aft >= fore)",
    "Sides - fore deck forward",
    "Sides - fore deck",
    "Sides - fore deck aft",
    "Sides - aft deck forward",
    "Sides - aft deck",
    "Sides - aft deck aft",
    "None",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Hull,
    Freeboard,
    Armour,
    Guns,
    Weapons,
    Engine,
    Blueprint,
    Spaces,
    Calibration,
    Report,
}

pub struct SpringSharpApp {
    ship: ShipModel,
    page: Page,
    report: String,
    save_path: String,
    load_path: String,
    status: String,
    regime_summary: String,
    selected_preset_index: Option<usize>,
    show_arcs: bool,
    target_cg: f64,
    last_placement: Option<CompartmentPlacement>,
    target_displacement: f64,
    metric_units: bool,
    baseline_ship: Option<ShipModel>,
    compare_preset_idx: Option<usize>,
}

impl SpringSharpApp {
    pub fn new() -> Self {
        let mut ship = demo_ship();
        ship.calculate();
        let report = springsharp::features::generate_report(&ship);
        let regime = springsharp::regime::run_loocv_regime();
        let mut regime_summary = format!(
            "regime (Holtrop + per-type factor): {:.2}%\noriginal + per-type factor:        {:.2}%\nHoltrop uncalibrated:              {:.2}%\n\nper ship type (nested LOOCV):\n",
            regime.overall_mape * 100.0,
            regime.original_baseline_mape * 100.0,
            regime.holtrop_uncalibrated_mape * 100.0
        );
        for (ty, n, m) in &regime.per_type {
            regime_summary.push_str(&format!("  {ty:<12} n={n:<2} {:.2}%\n", m * 100.0));
        }
        Self {
            ship,
            page: Page::Hull,
            report,
            save_path: "example.ssr".to_string(),
            load_path: "example.ssr".to_string(),
            status: "Demo design loaded".to_string(),
            regime_summary,
            selected_preset_index: None,
            show_arcs: false,
            target_cg: 0.45,
            last_placement: None,
            target_displacement: 35000.0,
            metric_units: false,
            baseline_ship: None,
            compare_preset_idx: None,
        }
    }

    fn recalculate(&mut self) {
        self.ship.calculate();
        self.report = springsharp::features::generate_report(&self.ship);
        self.status = "Calculated".to_string();
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        ui.heading("SpringSharp Rust");
        ui.separator();
        for (page, name) in [
            (Page::Hull, "Hull"),
            (Page::Freeboard, "Freeboard"),
            (Page::Armour, "Armour"),
            (Page::Guns, "Guns"),
            (Page::Weapons, "Weapons & Misc"),
            (Page::Engine, "Engine"),
            (Page::Blueprint, "Blueprint & Power"),
            (Page::Spaces, "Spaces & Benchmarks"),
            (Page::Calibration, "Calibration"),
            (Page::Report, "Report"),
        ] {
            if ui.selectable_label(self.page == page, name).clicked() {
                self.page = page;
            }
        }
        ui.separator();
        if ui.button("Calculate").clicked() {
            self.recalculate();
        }
        if ui.button("Load demo").clicked() {
            self.ship = demo_ship();
            self.selected_preset_index = None;
            self.recalculate();
            self.status = "Demo design loaded".to_string();
        }
        ui.separator();
        ui.label("Historical Presets:");
        let current_preset_name = self
            .selected_preset_index
            .and_then(|idx| PRESET_LIST.get(idx))
            .map(|p| p.name)
            .unwrap_or("Select preset...");
        egui::ComboBox::from_id_salt("preset_picker")
            .selected_text(current_preset_name)
            .show_ui(ui, |ui| {
                for (idx, p) in PRESET_LIST.iter().enumerate() {
                    let is_sel = self.selected_preset_index == Some(idx);
                    if ui.selectable_label(is_sel, p.name).clicked() {
                        if let Some(preset_ship) = get_preset(p.id) {
                            self.ship = preset_ship;
                            self.selected_preset_index = Some(idx);
                            self.recalculate();
                            self.status = format!("Loaded preset: {}", p.name);
                        }
                    }
                }
            });
        ui.separator();
        let unit_lbl = if self.metric_units {
            "Units: Metric"
        } else {
            "Units: Imperial"
        };
        if ui.button(unit_lbl).clicked() {
            self.metric_units = !self.metric_units;
        }
        ui.separator();
        ui.label(&self.status);
    }

    fn page_hull(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hull");
        let h = &mut self.ship.hull;
        drag_f64(ui, "Length WL (ft)", &mut h.length, 0.1, 1.0..=2000.0);
        drag_f64(ui, "Beam (ft)", &mut h.beam, 0.1, 1.0..=300.0);
        drag_f64(
            ui,
            "Beam over bulges (ft)",
            &mut h.beam_bulges,
            0.1,
            1.0..=300.0,
        );
        drag_f64(ui, "Draught (ft)", &mut h.draught, 0.1, 0.1..=100.0);
        drag_f64(ui, "Block coefficient", &mut h.block, 0.001, 0.2..=0.95);
        drag_i32(ui, "Hull year", &mut h.date_hull);
        drag_i32(ui, "Engine year", &mut h.date_engine);
        drag_f64(ui, "LCB (% L)", &mut h.lcb_percent, 0.01, -10.0..=10.0);
        drag_f64(
            ui,
            "Midship coeff Cm",
            &mut h.midship_coefficient,
            0.001,
            0.0..=0.99,
        );
        drag_f64(
            ui,
            "Bulb area (sq ft)",
            &mut h.bulb_area_sqft,
            1.0,
            0.0..=10_000.0,
        );
        drag_f64(
            ui,
            "Bulb centre height (ft)",
            &mut h.bulb_centre_height_ft,
            0.1,
            0.0..=100.0,
        );
        drag_f64(
            ui,
            "Transom area (sq ft)",
            &mut h.transom_area_sqft,
            1.0,
            0.0..=10_000.0,
        );
        drag_f64(
            ui,
            "Appendage area (sq ft)",
            &mut h.appendage_area_sqft,
            1.0,
            0.0..=10_000.0,
        );
        drag_f64(
            ui,
            "Stern shape Cstern",
            &mut h.stern_shape,
            0.1,
            -25.0..=25.0,
        );
    }

    fn page_freeboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Freeboard");
        let f = &mut self.ship.free;
        drag_f64(
            ui,
            "Forecastle length fraction",
            &mut f.fc_length,
            0.001,
            0.0..=1.0,
        );
        drag_f64(
            ui,
            "Forward deck length fraction",
            &mut f.fd_length,
            0.001,
            0.0..=1.0,
        );
        drag_f64(
            ui,
            "Quarter deck length fraction",
            &mut f.qd_length,
            0.001,
            0.0..=1.0,
        );
        drag_f64(
            ui,
            "FC freeboard forward (ft)",
            &mut f.free_fc_for,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(
            ui,
            "FC freeboard aft (ft)",
            &mut f.free_fc_aft,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(
            ui,
            "FD freeboard forward (ft)",
            &mut f.free_fd_for,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(
            ui,
            "FD freeboard aft (ft)",
            &mut f.free_fd_aft,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(
            ui,
            "AD freeboard forward (ft)",
            &mut f.free_ad_for,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(
            ui,
            "AD freeboard aft (ft)",
            &mut f.free_ad_aft,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(
            ui,
            "QD freeboard forward (ft)",
            &mut f.free_qd_for,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(
            ui,
            "QD freeboard aft (ft)",
            &mut f.free_qd_aft,
            0.1,
            -50.0..=100.0,
        );
        drag_f64(ui, "Bow overhang (ft)", &mut f.length_bow, 0.1, 0.0..=200.0);
        drag_f64(ui, "Ram length (ft)", &mut f.length_ram, 0.1, 0.0..=200.0);
        drag_f64(
            ui,
            "Stern overhang (ft)",
            &mut f.length_stern,
            0.1,
            0.0..=200.0,
        );
        combo_i32(ui, "Bow shape", &mut f.bow, BOWS);
        combo_i32(ui, "Stern shape", &mut f.stern, STERNS);
        drag_f64(
            ui,
            "Transom width fraction",
            &mut f.transom_width,
            0.001,
            0.0..=1.0,
        );
    }

    fn page_armour(&mut self, ui: &mut egui::Ui) {
        ui.heading("Armour");
        let a = &mut self.ship.armour;
        drag_f64(ui, "Main belt (in)", &mut a.belt_main, 0.1, 0.0..=40.0);
        drag_f64(
            ui,
            "Main belt length (ft)",
            &mut a.belt_main_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            "Main belt height (ft)",
            &mut a.belt_main_height,
            0.1,
            0.0..=100.0,
        );
        drag_f64(ui, "Belt ends (in)", &mut a.belt_ends, 0.1, 0.0..=40.0);
        drag_f64(
            ui,
            "Belt ends length (ft)",
            &mut a.belt_ends_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            "Belt ends height (ft)",
            &mut a.belt_ends_height,
            0.1,
            0.0..=100.0,
        );
        drag_f64(ui, "Upper belt (in)", &mut a.belt_upper, 0.1, 0.0..=40.0);
        drag_f64(
            ui,
            "Upper belt length (ft)",
            &mut a.belt_upper_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            "Upper belt height (ft)",
            &mut a.belt_upper_height,
            0.1,
            0.0..=100.0,
        );
        drag_f64(
            ui,
            "Belt incline (deg)",
            &mut a.belts_incline,
            0.1,
            -45.0..=45.0,
        );
        drag_f64(ui, "Bulkhead (in)", &mut a.bulkhead, 0.1, 0.0..=40.0);
        drag_f64(
            ui,
            "Bulkhead length (ft)",
            &mut a.bulkhead_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            "Bulkhead height (ft)",
            &mut a.bulkhead_height,
            1.0,
            0.0..=100.0,
        );
        drag_f64(
            ui,
            "Bulkhead beam (ft)",
            &mut a.bulkhead_beam,
            1.0,
            0.0..=300.0,
        );
        drag_i32(ui, "Bulkhead type", &mut a.bulkhead_type);
        drag_f64(ui, "Bulge (in)", &mut a.bulge, 0.1, 0.0..=40.0);
        drag_f64(
            ui,
            "Bulge length (ft)",
            &mut a.bulge_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            "Bulge height (ft)",
            &mut a.bulge_height,
            0.1,
            0.0..=100.0,
        );
        drag_f64(ui, "Deck (in)", &mut a.deck, 0.1, 0.0..=40.0);
        drag_f64(ui, "Deck FC (in)", &mut a.deck_fc, 0.1, 0.0..=40.0);
        drag_f64(ui, "Deck QD (in)", &mut a.deck_qd, 0.1, 0.0..=40.0);
        drag_i32(ui, "Deck coverage", &mut a.deck_coverage);
        drag_f64(
            ui,
            "Conning forward (in)",
            &mut a.conning_for,
            0.1,
            0.0..=40.0,
        );
        drag_f64(ui, "Conning aft (in)", &mut a.conning_aft, 0.1, 0.0..=40.0);
    }

    fn page_guns(&mut self, ui: &mut egui::Ui) {
        ui.heading("Gun batteries");
        for (i, g) in self.ship.guns.batteries.iter_mut().enumerate() {
            let title = match i {
                0 => "Main battery",
                1 => "2nd battery",
                2 => "3rd battery",
                3 => "4th battery",
                4 => "5th battery",
                _ => "Battery",
            };
            egui::CollapsingHeader::new(format!("{title} (index {i})")).show(ui, |ui| {
                drag_i32(ui, "Barrels", &mut g.barrels);
                drag_f64(ui, "Calibre (in)", &mut g.calibre, 0.1, 0.0..=40.0);
                drag_f64(ui, "Calibre length", &mut g.cal_length, 0.1, 1.0..=100.0);
                drag_i32(ui, "Gun year", &mut g.date_gun);
                combo_i32(ui, "Gun type", &mut g.gun_type, GUN_TYPES);
                combo_i32(ui, "Mount type", &mut g.mount_type, MOUNT_TYPES);
                drag_i32(ui, "Magazine rounds", &mut g.magazine);
                drag_i32(ui, "Mounts", &mut g.mounts);
                combo_i32(ui, "Guns/mount G1", &mut g.guns_in_mount_g1, GUNS_IN_MOUNT);
                combo_i32(ui, "Guns/mount G2", &mut g.guns_in_mount_g2, GUNS_IN_MOUNT);
                combo_i32(ui, "Layout G1", &mut g.layout_g1, LAYOUTS);
                combo_i32(ui, "Layout G2", &mut g.layout_g2, LAYOUTS);
                drag_i32(ui, "Super mounts G1", &mut g.mount_super_g1);
                drag_i32(ui, "Sub mounts G1", &mut g.mount_sub_g1);
                drag_i32(ui, "Super mounts G2", &mut g.mount_super_g2);
                drag_i32(ui, "Deck mounts G2", &mut g.mount_deck_g2);
                drag_i32(ui, "Sub mounts G2", &mut g.mount_sub_g2);
                drag_f64(ui, "Armour face (in)", &mut g.arm_face, 0.1, 0.0..=40.0);
                drag_f64(ui, "Armour back (in)", &mut g.arm_back, 0.1, 0.0..=40.0);
                drag_f64(ui, "Armour bottom (in)", &mut g.arm_bottom, 0.1, 0.0..=40.0);
                ui.checkbox(&mut g.super_super_chk, "Super-super");
                ui.checkbox(&mut g.sub_sub_chk, "Sub-sub");
            });
        }
    }

    fn page_engine(&mut self, ui: &mut egui::Ui) {
        ui.heading("Machinery");
        let e = &mut self.ship.engine;
        drag_f64(ui, "Max speed (kt)", &mut e.speed_max, 0.1, 0.0..=60.0);
        drag_f64(
            ui,
            "Cruise speed (kt)",
            &mut e.speed_cruise,
            0.1,
            0.0..=60.0,
        );
        drag_f64(ui, "Range (nm)", &mut e.range, 10.0, 0.0..=100_000.0);
        drag_f64(ui, "Percent coal", &mut e.pct_coal, 0.1, 0.0..=100.0);
        drag_i32(ui, "Screws", &mut e.screws);
        drag_f64(
            ui,
            "Power calibration factor",
            &mut e.power_factor,
            0.001,
            0.1..=2.0,
        );
        drag_f64(
            ui,
            "Machinery weight factor",
            &mut e.machinery_weight_factor,
            0.01,
            0.1..=3.0,
        );
        ui.horizontal(|ui| {
            ui.label("Resistance model:");
            egui::ComboBox::from_id_salt("resistance_model")
                .selected_text(match e.resistance_model {
                    ResistanceModel::Original => "Original",
                    ResistanceModel::HoltropMennen => "Holtrop-Mennen",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut e.resistance_model,
                        ResistanceModel::Original,
                        "Original",
                    );
                    ui.selectable_value(
                        &mut e.resistance_model,
                        ResistanceModel::HoltropMennen,
                        "Holtrop-Mennen",
                    );
                });
        });
        drag_f64(
            ui,
            "Holtrop viscous factor",
            &mut e.holtrop_viscous_factor,
            0.001,
            0.0..=5.0,
        );
        drag_f64(
            ui,
            "Holtrop wave factor",
            &mut e.holtrop_wave_factor,
            0.001,
            0.0..=5.0,
        );
        drag_f64(
            ui,
            "Holtrop correlation factor",
            &mut e.holtrop_correlation_factor,
            0.001,
            0.0..=5.0,
        );
        drag_f64(
            ui,
            "Planing calibration factor",
            &mut e.planing_calibration_factor,
            0.001,
            0.0..=2.0,
        );
        drag_f64(
            ui,
            "Planing min speed (kt)",
            &mut e.planing_min_speed_kt,
            0.1,
            0.0..=60.0,
        );
        drag_f64(
            ui,
            "Planing max displacement (tons)",
            &mut e.planing_max_displacement_tons,
            10.0,
            0.0..=100_000.0,
        );
        drag_f64(
            ui,
            "Ensemble reduced weight",
            &mut e.ensemble_reduced_weight,
            0.01,
            0.0..=1.0,
        );
        drag_f64(
            ui,
            "Ensemble structural weight",
            &mut e.ensemble_structural_weight,
            0.01,
            0.0..=1.0,
        );
        ui.separator();
        ui.label("Fuel types");
        for (i, v) in e.fuel.iter_mut().enumerate() {
            let name = match i {
                0 => "Coal fired boilers",
                1 => "Oil fired boilers",
                2 => "Diesel motors",
                3 => "Petrol motors",
                _ => "Batteries",
            };
            ui.checkbox(v, name);
        }
        ui.label("Engine types");
        for (i, v) in e.engine_types.iter_mut().enumerate() {
            let name = match i {
                0 => "Simple reciprocating",
                1 => "Complex reciprocating",
                _ => "Steam turbines",
            };
            ui.checkbox(v, name);
        }
        ui.label("Drive types");
        for (i, v) in e.drive.iter_mut().enumerate() {
            let name = match i {
                0 => "Direct",
                1 => "Geared",
                2 => "Electric",
                _ => "Hydraulic",
            };
            ui.checkbox(v, name);
        }
    }

    fn page_calibration(&mut self, ui: &mut egui::Ui) {
        ui.heading("Calibration (modern regime model)");
        ui.label(
            "Default model: Holtrop-Mennen + per-ship-type calibration factor, \
             fitted to the public historical dataset.",
        );
        ui.separator();
        ui.label(format!("Ship type: {}", self.ship.ship_type));
        match springsharp::regime::regime_for_ship_type(&self.ship.ship_type) {
            Some(r) => {
                ui.label(format!(
                    "Recommended: {}  factor {:.3}  (in-sample MAPE {:.1}%)",
                    r.model_label,
                    r.factor,
                    r.train_mape * 100.0
                ));
                if ui.button("Apply regime model").clicked() {
                    if let Some(applied) =
                        springsharp::regime::apply_regime_calibration(&mut self.ship)
                    {
                        self.recalculate();
                        self.status = format!(
                            "Applied regime: {} factor {:.3}",
                            applied.model_label, applied.factor
                        );
                    } else {
                        self.status = "No regime entry for this ship type".to_string();
                    }
                }
            }
            None => {
                ui.colored_label(
                    egui::Color32::RED,
                    "No regime entry for this ship type (use a known type: battleship, \
                     carrier, cruiser, destroyer).",
                );
            }
        }
        ui.separator();
        ui.label("Nested-LOOCV (out-of-sample) comparison:");
        ui.monospace(self.regime_summary.as_str());
        ui.separator();
        ui.colored_label(
            egui::Color32::YELLOW,
            "Data provenance: public historical references only (not independently \
             verified). Official trial/model-basin data unavailable; treat results as approximate.",
        );

        ui.separator();
        ui.heading("Target Displacement Solver");
        ui.label(
            "Scale hull dimensions (length, beam, draught) proportionally to meet an exact \
             target displacement (converges in 4-6 iterations).",
        );
        ui.horizontal(|ui| {
            ui.label("Target normal displacement:");
            drag_f64(
                ui,
                "tons",
                &mut self.target_displacement,
                500.0,
                100.0..=120000.0,
            );
            if ui.button("Solve & Scale Hull").clicked() {
                let achieved = solve_displacement_scale(&mut self.ship, self.target_displacement);
                self.recalculate();
                self.status = format!(
                    "Displacement scaled to {achieved:.0} t (target: {:.0} t). L={:.1} ft, B={:.1} ft, T={:.1} ft",
                    self.target_displacement,
                    self.ship.hull.length,
                    self.ship.hull.beam,
                    self.ship.hull.draught
                );
            }
        });

        ui.separator();
        ui.heading("Side-by-Side Ship Comparison");
        ui.label(
            "Compare your current design against a pinned baseline or historical warship preset.",
        );
        ui.horizontal(|ui| {
            if ui.button("Pin current as baseline").clicked() {
                self.baseline_ship = Some(self.ship.clone());
                self.compare_preset_idx = None;
                self.status = "Pinned current design as comparison baseline".to_string();
            }
            if ui.button("Clear comparison").clicked() {
                self.baseline_ship = None;
                self.compare_preset_idx = None;
            }
            let sel_text = self
                .compare_preset_idx
                .and_then(|idx| PRESET_LIST.get(idx))
                .map(|p| p.name)
                .unwrap_or("Compare with preset...");
            egui::ComboBox::from_id_salt("compare_preset_picker")
                .selected_text(sel_text)
                .show_ui(ui, |ui| {
                    for (idx, p) in PRESET_LIST.iter().enumerate() {
                        let is_sel = self.compare_preset_idx == Some(idx);
                        if ui.selectable_label(is_sel, p.name).clicked() {
                            if let Some(preset_ship) = get_preset(p.id) {
                                self.baseline_ship = Some(preset_ship);
                                self.compare_preset_idx = Some(idx);
                            }
                        }
                    }
                });
        });

        if let Some(ref base) = self.baseline_ship {
            ui.add_space(6.0);
            egui::Grid::new("comparison_table")
                .striped(true)
                .spacing([24.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("Metric");
                    ui.strong(format!("Current ({})", self.ship.name));
                    ui.strong(format!("Baseline ({})", base.name));
                    ui.strong("Delta (Δ)");
                    ui.strong("Rel. Diff (%)");
                    ui.end_row();

                    let rows = [
                        ("Length WL (ft)", self.ship.hull.length, base.hull.length, 1),
                        ("Beam (ft)", self.ship.hull.beam, base.hull.beam, 1),
                        ("Draught (ft)", self.ship.hull.draught, base.hull.draught, 1),
                        (
                            "Normal disp (t)",
                            self.ship.hull.displacement,
                            base.hull.displacement,
                            0,
                        ),
                        (
                            "Full load disp (t)",
                            self.ship.disp_max_load,
                            base.disp_max_load,
                            0,
                        ),
                        (
                            "Max speed (kt)",
                            self.ship.engine.speed_max,
                            base.engine.speed_max,
                            1,
                        ),
                        (
                            "Max power (hp)",
                            self.ship.engine.hp_max,
                            base.engine.hp_max,
                            0,
                        ),
                        ("Range (nm)", self.ship.engine.range, base.engine.range, 0),
                        (
                            "Main belt (in)",
                            self.ship.armour.belt_main,
                            base.armour.belt_main,
                            1,
                        ),
                        ("Main deck (in)", self.ship.armour.deck, base.armour.deck, 1),
                        (
                            "Main battery cal (in)",
                            self.ship.guns.batteries[0].calibre,
                            base.guns.batteries[0].calibre,
                            1,
                        ),
                        (
                            "Broadside weight (lb)",
                            self.ship.guns.broadside,
                            base.guns.broadside,
                            0,
                        ),
                        (
                            "Metacentre GM (ft)",
                            self.ship.metacentre,
                            base.metacentre,
                            2,
                        ),
                    ];

                    for (name, val1, val2, dec) in rows {
                        ui.label(name);
                        ui.label(format!("{val1:.prec$}", prec = dec));
                        ui.label(format!("{val2:.prec$}", prec = dec));
                        let diff = val1 - val2;
                        let sign = if diff > 0.0 { "+" } else { "" };
                        ui.label(format!("{sign}{diff:.prec$}", prec = dec));
                        if val2.abs() > 1e-6 {
                            let pct = (diff / val2) * 100.0;
                            let p_sign = if pct > 0.0 { "+" } else { "" };
                            ui.label(format!("{p_sign}{pct:.1}%"));
                        } else {
                            ui.label("–");
                        }
                        ui.end_row();
                    }
                });
        }
    }

    fn page_report(&mut self, ui: &mut egui::Ui) {
        ui.heading("Report and files");
        ui.horizontal(|ui| {
            if ui.button("Calculate and regenerate report").clicked() {
                self.recalculate();
            }
            if ui.button("Load demo").clicked() {
                self.ship = demo_ship();
                self.recalculate();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Save path:");
            ui.text_edit_singleline(&mut self.save_path);
            if ui.button("Save .ssr").clicked() {
                match save_to_file(&self.ship, &self.save_path) {
                    Ok(()) => self.status = format!("Saved {}", self.save_path),
                    Err(e) => self.status = format!("Save error: {e}"),
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Load path:");
            ui.text_edit_singleline(&mut self.load_path);
            if ui.button("Load .ssr").clicked() {
                let mut ship = ShipModel::new();
                match load_from_file(&mut ship, &self.load_path) {
                    Ok(()) => {
                        self.ship = ship;
                        self.recalculate();
                        self.status = format!("Loaded {}", self.load_path);
                    }
                    Err(e) => self.status = format!("Load error: {e}"),
                }
            }
        });
        ui.separator();
        ui.label(&self.status);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.report)
                    .desired_width(f32::INFINITY)
                    .desired_rows(30),
            );
        });
    }

    fn page_weapons(&mut self, ui: &mut egui::Ui) {
        ui.heading("Torpedoes, Mines, Depth Charges & Miscellaneous Weights");
        let w = &mut self.ship.weapons;
        weapon_mount_ui(ui, "Torpedo set 1", "weap0", &mut w.torpedo1);
        weapon_mount_ui(ui, "Torpedo set 2", "weap1", &mut w.torpedo2);
        weapon_mount_ui(ui, "Mines", "weap2", &mut w.mine);
        weapon_mount_ui(ui, "Depth charges / AS mortars 1", "weap3", &mut w.dc1);
        weapon_mount_ui(ui, "Depth charges / AS mortars 2", "weap4", &mut w.dc2);

        ui.separator();
        ui.heading("Miscellaneous weights (tons)");
        drag_f64(
            ui,
            "Vital spaces (tons)",
            &mut w.misc_weight_vital,
            1.0,
            0.0..=50_000.0,
        );
        drag_f64(
            ui,
            "Hull spaces (tons)",
            &mut w.misc_weight_hull,
            1.0,
            0.0..=50_000.0,
        );
        drag_f64(
            ui,
            "Deck spaces (tons)",
            &mut w.misc_weight_deck,
            1.0,
            0.0..=50_000.0,
        );
        drag_f64(
            ui,
            "Above-water spaces (tons)",
            &mut w.misc_weight_above,
            1.0,
            0.0..=50_000.0,
        );
        drag_f64(
            ui,
            "Void weight (tons)",
            &mut w.void_weight,
            1.0,
            0.0..=50_000.0,
        );
    }

    fn page_spaces(&mut self, ui: &mut egui::Ui) {
        if let Some(bench) = get_preset_benchmark(&self.ship.name) {
            ui.heading(format!(
                "Historical Subsystem Weight & Stability Benchmark ({})",
                bench.name
            ));
            ui.label(format!("Source: {}", bench.source));
            let sim_hull = self.ship.engine.hull_weight;
            let sim_armour = self.ship.armour.weight
                + self
                    .ship
                    .guns
                    .batteries
                    .iter()
                    .map(|b| b.arm_weight)
                    .sum::<f64>();
            let sim_mach = self.ship.engine.engine_weight;
            let sim_arm =
                self.ship.guns.borne_weight + self.ship.guns.magazine + self.ship.weapons.weight;
            let sim_bunker = self.ship.engine.bunker;
            let sim_gm = self.ship.metacentre;

            egui::Grid::new("bench_grid")
                .striped(true)
                .spacing([24.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("Subsystem / Metric");
                    ui.strong("Simulated");
                    ui.strong("Historical Ref");
                    ui.strong("Rel. Error");
                    ui.end_row();

                    let mut row = |label: &str, sim: f64, r: f64, dec: usize, na: bool| {
                        ui.label(label);
                        ui.label(format!("{sim:.dec$}"));
                        ui.label(format!("{r:.dec$}"));
                        if na || r <= 0.0 {
                            ui.label("n/a (splinter)");
                        } else {
                            let pct = (sim - r) / r * 100.0;
                            let color = if pct.abs() <= 10.0 {
                                egui::Color32::from_rgb(95, 211, 141)
                            } else if pct.abs() <= 20.0 {
                                egui::Color32::from_rgb(255, 181, 71)
                            } else {
                                egui::Color32::from_rgb(255, 107, 107)
                            };
                            ui.colored_label(color, format!("{pct:+.1}%"));
                        }
                        ui.end_row();
                    };

                    row(
                        "Normal displacement (t)",
                        self.ship.hull.displacement,
                        bench.ref_displacement_t,
                        0,
                        false,
                    );
                    row("Hull structure (t)", sim_hull, bench.ref_hull_t, 0, false);
                    row(
                        "Armour + barbettes (t)",
                        sim_armour,
                        bench.ref_armour_t,
                        0,
                        bench.ref_armour_t < 500.0,
                    );
                    row("Machinery (t)", sim_mach, bench.ref_machinery_t, 0, false);
                    row(
                        "Armament + ammo (t)",
                        sim_arm,
                        bench.ref_armament_t,
                        0,
                        false,
                    );
                    row(
                        "Normal bunker (t)",
                        sim_bunker,
                        bench.ref_bunker_t,
                        0,
                        false,
                    );
                    row(
                        "Metacentric height GM (ft)",
                        sim_gm,
                        bench.ref_gm_ft,
                        2,
                        false,
                    );
                });
            ui.separator();
        }

        ui.heading("Internal Space Volume Budget");
        let sp = calculate_internal_spaces(&self.ship);
        egui::Grid::new("space_grid")
            .striped(true)
            .spacing([24.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Space Category");
                ui.strong("Volume (m³)");
                ui.end_row();
                for (name, vol) in [
                    ("Total hull volume", sp.hull_volume_m3),
                    ("Machinery spaces", sp.machinery_volume_m3),
                    ("Magazines & handling", sp.magazine_volume_m3),
                    ("Crew accommodation", sp.accommodation_volume_m3),
                    ("Fuel bunkerage", sp.fuel_volume_m3),
                    ("Stores & provisions", sp.stores_volume_m3),
                    ("Void & access", sp.void_volume_m3),
                    ("Total required volume", sp.required_volume_m3),
                ] {
                    ui.label(name);
                    ui.label(format!("{vol:.0}"));
                    ui.end_row();
                }
                ui.label("Room ratio (Hull / Required)");
                ui.label(format!("{:.3}", sp.room_hull_improved));
                ui.end_row();
            });

        ui.separator();
        ui.heading("Multi-Deck General Arrangement Optimizer (GA)");
        ui.horizontal(|ui| {
            drag_f64(
                ui,
                "Target vertical CG (fraction of depth)",
                &mut self.target_cg,
                0.01,
                0.20..=0.80,
            );
            if ui.button("Run arrangement optimizer").clicked() {
                let placement = suggest_compartment_placement(&self.ship, self.target_cg);
                self.last_placement = Some(placement);
                self.status = "Arrangement optimized".to_string();
            }
        });

        if let Some(p) = &self.last_placement {
            ui.label(format!(
                "Vertical CG: {:.3} (target {:.2}) · Longitudinal CG: {:.3} · Access edges: {} ({}) · Max escape path: {:.1} m · Scantling util: {:.2}",
                p.cg_vertical,
                self.target_cg,
                p.cg_longitudinal,
                p.access_edges,
                if p.access_connected { "connected" } else { "disconnected" },
                p.escape_max_path,
                p.scantling_utilization,
            ));
            let comp_names = [
                ("Machinery", p.layout.machinery),
                ("Magazine", p.layout.magazine),
                ("Accommodation", p.layout.accommodation),
                ("Fuel", p.layout.fuel),
                ("Stores", p.layout.stores),
                ("Void", p.layout.void),
            ];
            egui::Grid::new("ga_grid")
                .striped(true)
                .spacing([18.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("Compartment");
                    ui.strong("Volume (m³)");
                    ui.strong("Deck (0..3)");
                    ui.strong("Long. X (0=stern..1=bow)");
                    ui.strong("Vert. Z");
                    ui.strong("Evac Time (s)");
                    ui.end_row();
                    for (idx, (cname, cvol)) in comp_names.iter().enumerate() {
                        ui.label(*cname);
                        ui.label(format!("{cvol:.0}"));
                        ui.label(format!("{}", p.decks[idx]));
                        ui.label(format!("{:.3}", p.longitudinal[idx]));
                        ui.label(format!("{:.3}", p.centers[idx]));
                        ui.label(format!("{:.1}", p.evacuation_time_s[idx]));
                        ui.end_row();
                    }
                });
        }
    }

    fn dashboard_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(&self.ship.name);
            ui.label(format!("· {} · {}", self.ship.country, self.ship.ship_type));
            if self.ship.converged {
                ui.colored_label(egui::Color32::from_rgb(95, 211, 141), "● Converged");
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 181, 71),
                    "▲ Weights not converged",
                );
            }
        });
        ui.horizontal(|ui| {
            if self.metric_units {
                ui.label(format!(
                    "Disp: {:.0} t normal ({:.0} t full)",
                    self.ship.hull.displacement * 1.01605,
                    self.ship.disp_max_load * 1.01605
                ));
                ui.separator();
                ui.label(format!(
                    "Speed: {:.1} km/h ({:.1} kt)",
                    self.ship.engine.speed_max * 1.852,
                    self.ship.engine.speed_max
                ));
                ui.separator();
                ui.label(format!("Power: {:.0} hp", self.ship.engine.hp_max));
                ui.separator();
                ui.label(format!(
                    "Broadside: {:.0} kg",
                    self.ship.guns.broadside * 0.453592
                ));
                ui.separator();
                ui.label(format!("GM: {:.2} m", self.ship.metacentre * 0.3048));
            } else {
                ui.label(format!(
                    "Disp: {:.0} t normal ({:.0} t full)",
                    self.ship.hull.displacement, self.ship.disp_max_load
                ));
                ui.separator();
                ui.label(format!("Speed: {:.1} kt", self.ship.engine.speed_max));
                ui.separator();
                ui.label(format!("Power: {:.0} hp", self.ship.engine.hp_max));
                ui.separator();
                ui.label(format!("Broadside: {:.0} lb", self.ship.guns.broadside));
                ui.separator();
                ui.label(format!("GM: {:.1} ft", self.ship.metacentre));
            }
        });

        // Color weight distribution bar
        let total = self.ship.hull.displacement.max(1.0);
        let e = &self.ship.engine;
        let g = &self.ship.guns;
        let arm = &self.ship.armour;
        let w = &self.ship.weapons;

        let parts = [
            ("Hull", e.hull_weight, egui::Color32::from_rgb(79, 143, 214)),
            ("Armour", arm.weight, egui::Color32::from_rgb(155, 123, 234)),
            (
                "Machinery",
                e.engine_weight,
                egui::Color32::from_rgb(224, 134, 79),
            ),
            (
                "Armament",
                g.borne_weight + w.weight,
                egui::Color32::from_rgb(224, 93, 111),
            ),
            (
                "Magazine",
                g.magazine,
                egui::Color32::from_rgb(214, 181, 79),
            ),
            ("Bunker", e.bunker, egui::Color32::from_rgb(79, 199, 180)),
        ];

        let bar_height = 14.0;
        let available_w = ui.available_width().max(300.0);
        let (response, painter) =
            ui.allocate_painter(egui::vec2(available_w, bar_height), egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(12, 27, 42));
        let mut cur_x = rect.left();
        for (_, wt, color) in parts {
            if wt > 0.0 {
                let frac = ((wt / total).clamp(0.0, 1.0)) as f32;
                let w_px = (frac * rect.width()).max(1.0);
                let slice = egui::Rect::from_min_max(
                    egui::pos2(cur_x, rect.top()),
                    egui::pos2((cur_x + w_px).min(rect.right()), rect.bottom()),
                );
                painter.rect_filled(slice, 0.0, color);
                cur_x += w_px;
                if cur_x >= rect.right() {
                    break;
                }
            }
        }
        ui.horizontal_wrapped(|ui| {
            for (name, wt, color) in parts {
                let pct = (wt / total * 100.0).clamp(0.0, 100.0);
                ui.colored_label(color, format!("■ {name}: {:.0}t ({:.1}%)", wt, pct));
            }
        });
        ui.separator();
    }

    fn page_blueprint(&mut self, ui: &mut egui::Ui) {
        ui.heading("Vessel Blueprint Schematic");
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_arcs, "Show 290° Turret Firing Arcs");
            ui.separator();
            let dim_lbl = if self.metric_units {
                format!(
                    "Dimensions: {:.1} m × {:.1} m × {:.1} m",
                    self.ship.hull.length * 0.3048,
                    self.ship.hull.beam * 0.3048,
                    self.ship.hull.draught * 0.3048
                )
            } else {
                format!(
                    "Dimensions: {:.0} ft × {:.0} ft × {:.1} ft",
                    self.ship.hull.length, self.ship.hull.beam, self.ship.hull.draught
                )
            };
            ui.label(dim_lbl);
        });
        ui.separator();

        let available_width = ui.available_width().max(600.0);
        let height = 360.0;
        let (response, painter) =
            ui.allocate_painter(egui::vec2(available_width, height), egui::Sense::hover());
        let rect = response.rect;

        // Background canvas
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 19, 30));

        let l = self.ship.hull.length.max(10.0) as f32;
        let b = self.ship.hull.beam.max(5.0) as f32;
        let pad_x = 50.0_f32;
        let draw_w = (rect.width() - pad_x * 2.0).max(100.0);
        let x0 = rect.left() + pad_x;
        let x1 = x0 + draw_w;

        let fc = self.ship.free.fc_length.clamp(0.0, 1.0) as f32;
        let fd = self
            .ship
            .free
            .fd_length
            .clamp(0.0, (1.0 - fc as f64).max(0.0)) as f32;
        let qd = self
            .ship
            .free
            .qd_length
            .clamp(0.0, (1.0 - (fc + fd) as f64).max(0.0)) as f32;
        let ad = (1.0_f32 - fc - fd - qd).max(0.0);

        let x_at = |p: f32| x0 + p * draw_w;

        // Profile view baseline
        let y_wl = rect.top() + 75.0_f32;
        let y_keel = y_wl + 20.0_f32;
        let fb_scale = 1.0_f32;

        // Profile Deck heights
        let y_fc_for = y_wl - ((self.ship.free.free_fc_for as f32) * fb_scale).clamp(8.0, 36.0);
        let y_fc_aft = y_wl - ((self.ship.free.free_fc_aft as f32) * fb_scale).clamp(8.0, 36.0);
        let y_fd_for = y_wl - ((self.ship.free.free_fd_for as f32) * fb_scale).clamp(6.0, 30.0);
        let y_fd_aft = y_wl - ((self.ship.free.free_fd_aft as f32) * fb_scale).clamp(6.0, 30.0);
        let y_ad_for = y_wl - ((self.ship.free.free_ad_for as f32) * fb_scale).clamp(4.0, 26.0);
        let y_ad_aft = y_wl - ((self.ship.free.free_ad_aft as f32) * fb_scale).clamp(4.0, 26.0);
        let y_qd_for = y_wl - ((self.ship.free.free_qd_for as f32) * fb_scale).clamp(4.0, 26.0);
        let y_qd_aft = y_wl - ((self.ship.free.free_qd_aft as f32) * fb_scale).clamp(4.0, 26.0);

        // Waterline
        painter.line_segment(
            [egui::pos2(x0 - 15.0, y_wl), egui::pos2(x1 + 15.0, y_wl)],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(79, 179, 255)),
        );
        painter.text(
            egui::pos2(x1 + 20.0, y_wl),
            egui::Align2::LEFT_CENTER,
            "WL",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(79, 179, 255),
        );

        // Hull profile polygon
        let x_fc = x_at(fc);
        let x_fd = x_at(fc + fd);
        let x_ad = x_at(1.0 - qd);
        let profile_pts = vec![
            egui::pos2(x0, y_fc_for),
            egui::pos2(x_fc, y_fc_aft),
            egui::pos2(x_fc, y_fd_for),
            egui::pos2(x_fd, y_fd_aft),
            egui::pos2(x_fd, y_ad_for),
            egui::pos2(x_ad, y_ad_aft),
            egui::pos2(x_ad, y_qd_for),
            egui::pos2(x1, y_qd_aft),
            egui::pos2(x1 - 4.0, y_keel),
            egui::pos2(x0 + 10.0, y_keel),
        ];
        painter.add(egui::Shape::convex_polygon(
            profile_pts,
            egui::Color32::from_rgba_unmultiplied(79, 179, 255, 20),
            egui::Stroke::new(1.5, egui::Color32::from_rgb(79, 179, 255)),
        ));

        // Carrier flight deck in profile view
        let is_carrier = self.ship.ship_type.to_lowercase().contains("carrier");
        if is_carrier {
            let fd_y = y_fc_for - 12.0;
            painter.line_segment(
                [egui::pos2(x0 + 4.0, fd_y), egui::pos2(x1 - 4.0, fd_y)],
                egui::Stroke::new(2.5, egui::Color32::from_rgb(95, 211, 141)),
            );
            painter.line_segment(
                [egui::pos2(x0 + 15.0, fd_y), egui::pos2(x0 + 15.0, y_fc_for)],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(95, 211, 141)),
            );
            painter.line_segment(
                [egui::pos2(x1 - 15.0, fd_y), egui::pos2(x1 - 15.0, y_qd_aft)],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(95, 211, 141)),
            );
        }

        // Superstructure bridge & funnel
        let x_mid = x0 + draw_w * 0.48;
        let bridge_rect = egui::Rect::from_min_max(
            egui::pos2(x_mid - 24.0, y_fd_for - 16.0),
            egui::pos2(x_mid - 8.0, y_fd_for),
        );
        painter.rect_filled(bridge_rect, 2.0, egui::Color32::from_rgb(138, 164, 191));
        let funnel_rect = egui::Rect::from_min_max(
            egui::pos2(x_mid + 6.0, y_fd_for - 20.0),
            egui::pos2(x_mid + 18.0, y_fd_for),
        );
        painter.rect_filled(funnel_rect, 2.0, egui::Color32::from_rgb(110, 130, 150));

        // Main belt armour
        if self.ship.armour.belt_main > 0.0 && self.ship.armour.belt_main_length > 0.0 {
            let bw =
                (((self.ship.armour.belt_main_length as f32) / l) * draw_w).clamp(10.0, draw_w);
            let bx = x0 + (draw_w - bw) / 2.0;
            let bh = ((self.ship.armour.belt_main_height as f32) * fb_scale).clamp(6.0, 18.0);
            let by = y_wl - bh * 0.6;
            let belt_rect =
                egui::Rect::from_min_max(egui::pos2(bx, by), egui::pos2(bx + bw, by + bh));
            painter.rect_filled(
                belt_rect,
                1.0,
                egui::Color32::from_rgba_unmultiplied(155, 123, 234, 110),
            );
        }

        // Turret positions
        let g1 = &self.ship.guns.batteries[0];
        let positions = get_turret_positions(g1, fc as f64, fd as f64, qd as f64);
        let g1_cal = g1.calibre as f32;
        let t_rad = (g1_cal * 0.3).clamp(3.5, 7.5);

        for &p_f64 in &positions {
            let p = p_f64 as f32;
            let tx = x_at(p);
            let ty = if p <= fc {
                y_fc_for + (y_fc_aft - y_fc_for) * if fc > 0.0 { p / fc } else { 0.0 }
            } else if p <= fc + fd {
                y_fd_for + (y_fd_aft - y_fd_for) * if fd > 0.0 { (p - fc) / fd } else { 0.0 }
            } else if p <= 1.0 - qd {
                y_ad_for + (y_ad_aft - y_ad_for) * if ad > 0.0 { (p - fc - fd) / ad } else { 0.0 }
            } else {
                y_qd_for
                    + (y_qd_aft - y_qd_for) * if qd > 0.0 { (p - (1.0 - qd)) / qd } else { 0.0 }
            };
            let fwd = p < 0.5;
            let t_rect = egui::Rect::from_min_max(
                egui::pos2(tx - t_rad, ty - t_rad * 1.3),
                egui::pos2(tx + t_rad, ty),
            );
            painter.rect_filled(t_rect, 1.0, egui::Color32::from_rgb(224, 93, 111));
            let barrel_x2 = if fwd {
                tx - t_rad - 8.0
            } else {
                tx + t_rad + 8.0
            };
            painter.line_segment(
                [
                    egui::pos2(if fwd { tx - t_rad } else { tx + t_rad }, ty - t_rad * 0.6),
                    egui::pos2(barrel_x2, ty - t_rad * 0.6),
                ],
                egui::Stroke::new(1.5, egui::Color32::WHITE),
            );
        }

        // Plan View
        let y_cl = rect.top() + 245.0;
        let hb = ((b / l) * draw_w * 0.75).clamp(8.0, 32.0);

        // Centerline
        painter.line_segment(
            [egui::pos2(x0 - 15.0, y_cl), egui::pos2(x1 + 15.0, y_cl)],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            ),
        );

        // Plan hull points (smooth approximation)
        let mut plan_pts = Vec::new();
        let steps = 40;
        for i in 0..=steps {
            let u = i as f32 / steps as f32;
            let px = x_at(u);
            let curve = if u < 0.25 {
                (u / 0.25).sin()
            } else if u > 0.75 {
                ((1.0 - u) / 0.25).sin()
            } else {
                1.0
            };
            plan_pts.push(egui::pos2(px, y_cl - hb * curve));
        }
        for i in (0..=steps).rev() {
            let u = i as f32 / steps as f32;
            let px = x_at(u);
            let curve = if u < 0.25 {
                (u / 0.25).sin()
            } else if u > 0.75 {
                ((1.0 - u) / 0.25).sin()
            } else {
                1.0
            };
            plan_pts.push(egui::pos2(px, y_cl + hb * curve));
        }
        painter.add(egui::Shape::convex_polygon(
            plan_pts,
            egui::Color32::from_rgba_unmultiplied(79, 179, 255, 24),
            egui::Stroke::new(1.5, egui::Color32::from_rgb(79, 179, 255)),
        ));

        // Carrier flight deck in plan view
        if is_carrier {
            let f_rect = egui::Rect::from_min_max(
                egui::pos2(x0 + 4.0, y_cl - hb * 1.15),
                egui::pos2(x1 - 4.0, y_cl + hb * 1.15),
            );
            painter.rect_filled(
                f_rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(95, 211, 141, 30),
            );
            painter.rect_stroke(
                f_rect,
                4.0,
                egui::Stroke::new(1.2, egui::Color32::from_rgb(95, 211, 141)),
                egui::StrokeKind::Outside,
            );
            painter.line_segment(
                [egui::pos2(x0 + 12.0, y_cl), egui::pos2(x1 - 12.0, y_cl)],
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 70),
                ),
            );
            let island_rect = egui::Rect::from_min_max(
                egui::pos2(x0 + draw_w * 0.45, y_cl - hb * 1.15 - 2.0),
                egui::pos2(x0 + draw_w * 0.45 + 24.0, y_cl - hb * 1.15 + 7.0),
            );
            painter.rect_filled(island_rect, 2.0, egui::Color32::from_rgb(224, 134, 79));
            painter.rect_stroke(
                island_rect,
                2.0,
                egui::Stroke::new(1.0, egui::Color32::WHITE),
                egui::StrokeKind::Outside,
            );
        }

        // Torpedo tube mounts in plan view
        let torp_count = self.ship.weapons.torpedo1.number + self.ship.weapons.torpedo2.number;
        if torp_count > 0 {
            let t_positions = [x0 + draw_w * 0.58, x0 + draw_w * 0.68];
            for tx in t_positions {
                let center = egui::pos2(tx, y_cl);
                painter.circle_filled(center, 4.5, egui::Color32::from_rgb(56, 239, 125));
                painter.circle_stroke(
                    center,
                    4.5,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(17, 153, 142)),
                );
                painter.line_segment(
                    [
                        egui::pos2(tx - 4.0, y_cl - 3.0),
                        egui::pos2(tx + 4.0, y_cl - 3.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::WHITE),
                );
                painter.line_segment(
                    [egui::pos2(tx - 4.0, y_cl), egui::pos2(tx + 4.0, y_cl)],
                    egui::Stroke::new(1.2, egui::Color32::WHITE),
                );
                painter.line_segment(
                    [
                        egui::pos2(tx - 4.0, y_cl + 3.0),
                        egui::pos2(tx + 4.0, y_cl + 3.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::WHITE),
                );
            }
        }

        // Plan turrets & arcs
        for &p_f64 in &positions {
            let p = p_f64 as f32;
            let tx = x_at(p);
            let fwd = p < 0.5;
            let center = egui::pos2(tx, y_cl);
            painter.circle_filled(center, t_rad, egui::Color32::from_rgb(224, 93, 111));
            painter.circle_stroke(center, t_rad, egui::Stroke::new(1.0, egui::Color32::WHITE));
            let barrel_x2 = if fwd {
                tx - t_rad - 9.0
            } else {
                tx + t_rad + 9.0
            };
            painter.line_segment(
                [center, egui::pos2(barrel_x2, y_cl)],
                egui::Stroke::new(1.5, egui::Color32::WHITE),
            );

            if self.show_arcs {
                let arc_r = (hb * 1.5).clamp(18.0, 40.0);
                let arc_pts = if fwd {
                    vec![
                        center,
                        egui::pos2(tx + arc_r * 0.819_f32, y_cl - arc_r * 0.573_f32),
                        egui::pos2(tx - arc_r * 0.707_f32, y_cl - arc_r * 0.707_f32),
                        egui::pos2(tx - arc_r, y_cl),
                        egui::pos2(tx - arc_r * 0.707_f32, y_cl + arc_r * 0.707_f32),
                        egui::pos2(tx + arc_r * 0.819_f32, y_cl + arc_r * 0.573_f32),
                    ]
                } else {
                    vec![
                        center,
                        egui::pos2(tx - arc_r * 0.819_f32, y_cl - arc_r * 0.573_f32),
                        egui::pos2(tx + arc_r * 0.707_f32, y_cl - arc_r * 0.707_f32),
                        egui::pos2(tx + arc_r, y_cl),
                        egui::pos2(tx + arc_r * 0.707_f32, y_cl + arc_r * 0.707_f32),
                        egui::pos2(tx - arc_r * 0.819_f32, y_cl + arc_r * 0.573_f32),
                    ]
                };
                painter.add(egui::Shape::convex_polygon(
                    arc_pts,
                    egui::Color32::from_rgba_unmultiplied(224, 93, 111, 28),
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(224, 93, 111, 90)),
                ));
            }
        }

        // Section labels
        painter.text(
            egui::pos2(rect.left() + 10.0, rect.top() + 12.0),
            egui::Align2::LEFT_TOP,
            "PROFILE VIEW",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(138, 164, 191),
        );
        painter.text(
            egui::pos2(rect.left() + 10.0, rect.top() + 155.0),
            egui::Align2::LEFT_TOP,
            "PLAN VIEW",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(138, 164, 191),
        );

        ui.separator();
        ui.heading("Speed & Power Curve");
        self.draw_speed_power_chart(ui, available_width);
        ui.separator();
        ui.heading("Armor Penetration & Immunity Zone");
        self.draw_immunity_chart(ui, available_width);
    }

    fn draw_speed_power_chart(&self, ui: &mut egui::Ui, available_width: f32) {
        let chart_h = 220.0_f32;
        let (response, painter) =
            ui.allocate_painter(egui::vec2(available_width, chart_h), egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 19, 30));

        let v_top = (self.ship.engine.speed_max + 2.0).max(20.0).ceil();
        let v_min = 10.0_f64;
        let mut mod_engine = self.ship.engine.clone();
        let mut orig_engine = self.ship.engine.clone();
        orig_engine.resistance_model = ResistanceModel::Original;
        orig_engine.power_factor = 1.0;
        orig_engine.original_power_factor = 1.0;
        orig_engine.blend_weight = 1.0;

        let mut pts = Vec::new();
        let mut max_hp = 1000.0_f64;
        let mut v = v_min;
        while v <= v_top + 1e-6 {
            let hp_mod = mod_engine.speed_hp_calc(v, &self.ship.hull);
            let hp_orig = orig_engine.speed_hp_calc(v, &self.ship.hull);
            if hp_mod > max_hp {
                max_hp = hp_mod;
            }
            if hp_orig > max_hp {
                max_hp = hp_orig;
            }
            pts.push((v as f32, hp_mod as f32, hp_orig as f32));
            v += 2.0;
        }
        max_hp *= 1.12;
        let max_hp_f = max_hp as f32;
        let min_v_f = v_min as f32;
        let max_v_f = (v_top as f32).max(min_v_f + 1.0);

        let pad_l = 65.0_f32;
        let pad_r = 25.0_f32;
        let pad_t = 22.0_f32;
        let pad_b = 30.0_f32;
        let pw = (rect.width() - pad_l - pad_r).max(100.0);
        let ph = (rect.height() - pad_t - pad_b).max(60.0);

        let x_at = |spd: f32| rect.left() + pad_l + ((spd - min_v_f) / (max_v_f - min_v_f)) * pw;
        let y_at = |hp: f32| rect.top() + pad_t + (1.0 - (hp / max_hp_f).clamp(0.0, 1.0)) * ph;

        for i in 0..=4 {
            let val = max_hp_f * (i as f32 / 4.0);
            let y = y_at(val);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + pad_l, y),
                    egui::pos2(rect.left() + pad_l + pw, y),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(28, 45, 66)),
            );
            let label = if val >= 1000.0 {
                format!("{:.0}k hp", val / 1000.0)
            } else {
                format!("{val:.0} hp")
            };
            painter.text(
                egui::pos2(rect.left() + pad_l - 6.0, y),
                egui::Align2::RIGHT_CENTER,
                label,
                egui::FontId::monospace(9.5),
                egui::Color32::from_rgb(138, 164, 191),
            );
        }

        for &(spd, _, _) in &pts {
            let x = x_at(spd);
            painter.line_segment(
                [
                    egui::pos2(x, rect.top() + pad_t),
                    egui::pos2(x, rect.top() + pad_t + ph),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(22, 35, 52)),
            );
            painter.text(
                egui::pos2(x, rect.top() + pad_t + ph + 12.0),
                egui::Align2::CENTER_CENTER,
                format!("{spd:.0} kt"),
                egui::FontId::monospace(9.0),
                egui::Color32::from_rgb(138, 164, 191),
            );
        }

        let orig_line: Vec<egui::Pos2> = pts
            .iter()
            .map(|&(spd, _, hp_o)| egui::pos2(x_at(spd), y_at(hp_o)))
            .collect();
        let mod_line: Vec<egui::Pos2> = pts
            .iter()
            .map(|&(spd, hp_m, _)| egui::pos2(x_at(spd), y_at(hp_m)))
            .collect();

        painter.add(egui::Shape::line(
            orig_line,
            egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 181, 71)),
        ));
        painter.add(egui::Shape::line(
            mod_line.clone(),
            egui::Stroke::new(2.2, egui::Color32::from_rgb(79, 179, 255)),
        ));
        for pt in mod_line {
            painter.circle_filled(pt, 2.8, egui::Color32::from_rgb(79, 179, 255));
        }

        painter.text(
            egui::pos2(rect.left() + pad_l + 10.0, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "■ Current Calibrated Model (cyan)   ■ Original Baseline (amber)",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(219, 231, 243),
        );
    }

    fn draw_immunity_chart(&self, ui: &mut egui::Ui, available_width: f32) {
        let g1 = &self.ship.guns.batteries[0];
        if g1.calibre <= 0.0 || g1.barrels <= 0 {
            ui.label("No active main battery guns for exterior ballistics analysis.");
            return;
        }

        let iz = compute_self_immunity_zone(
            g1,
            self.ship.armour.belt_main,
            self.ship.armour.belts_incline,
            self.ship.armour.deck,
        );

        if iz.curve.is_empty() {
            ui.label("Ballistic trajectory calculation produced no curve points.");
            return;
        }

        ui.horizontal(|ui| {
            ui.label(format!("Gun: {:.1}\" / {} cal", g1.calibre, g1.cal_length));
            ui.separator();
            ui.label(format!(
                "Belt target: {:.1}\" @ {:.0}°",
                iz.target_belt_in, iz.target_belt_incline
            ));
            ui.separator();
            ui.label(format!("Deck target: {:.1}\"", iz.target_deck_in));
            ui.separator();
            if iz.has_zone {
                if let (Some(in_yd), Some(out_yd)) = (iz.inner_limit_yd, iz.outer_limit_yd) {
                    ui.colored_label(
                        egui::Color32::from_rgb(95, 211, 141),
                        format!("● Immunity Zone: {:.0} – {:.0} yd", in_yd, out_yd),
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 181, 71),
                        "▲ No immunity zone against own main armament",
                    );
                }
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 181, 71),
                    "▲ No immunity zone against own main armament",
                );
            }
        });

        let chart_h = 220.0_f32;
        let (response, painter) =
            ui.allocate_painter(egui::vec2(available_width, chart_h), egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 19, 30));

        let min_r = iz.curve.first().map(|p| p.range_yd as f32).unwrap_or(0.0);
        let max_r = iz
            .curve
            .last()
            .map(|p| p.range_yd as f32)
            .unwrap_or(35000.0);
        let mut max_pen = (iz.target_belt_in.max(iz.target_deck_in) as f32).max(12.0);
        for p in &iz.curve {
            if (p.belt_pen_in as f32) > max_pen {
                max_pen = p.belt_pen_in as f32;
            }
            if (p.deck_pen_in as f32) > max_pen {
                max_pen = p.deck_pen_in as f32;
            }
        }
        max_pen *= 1.15;

        let pad_l = 65.0_f32;
        let pad_r = 25.0_f32;
        let pad_t = 22.0_f32;
        let pad_b = 30.0_f32;
        let pw = (rect.width() - pad_l - pad_r).max(100.0);
        let ph = (rect.height() - pad_t - pad_b).max(60.0);

        let x_at = |rng: f32| rect.left() + pad_l + ((rng - min_r) / (max_r - min_r)) * pw;
        let y_at = |pen: f32| rect.top() + pad_t + (1.0 - (pen / max_pen).clamp(0.0, 1.0)) * ph;

        // Immunity zone band
        if iz.has_zone {
            if let (Some(in_yd), Some(out_yd)) = (iz.inner_limit_yd, iz.outer_limit_yd) {
                let x_in = x_at(in_yd as f32);
                let x_out = x_at(out_yd as f32);
                let iz_rect = egui::Rect::from_min_max(
                    egui::pos2(x_in, rect.top() + pad_t),
                    egui::pos2(x_out, rect.top() + pad_t + ph),
                );
                painter.rect_filled(
                    iz_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(95, 211, 141, 35),
                );
                painter.rect_stroke(
                    iz_rect,
                    0.0,
                    egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_unmultiplied(95, 211, 141, 120),
                    ),
                    egui::StrokeKind::Outside,
                );
            }
        }

        // Horizontal grid lines
        for i in 0..=4 {
            let val = max_pen * (i as f32 / 4.0);
            let y = y_at(val);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + pad_l, y),
                    egui::pos2(rect.left() + pad_l + pw, y),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(28, 45, 66)),
            );
            let label = if self.metric_units {
                format!("{:.0} mm", val * 25.4)
            } else {
                format!("{val:.1}\"")
            };
            painter.text(
                egui::pos2(rect.left() + pad_l - 6.0, y),
                egui::Align2::RIGHT_CENTER,
                label,
                egui::FontId::monospace(9.5),
                egui::Color32::from_rgb(138, 164, 191),
            );
        }

        // Vertical range grid
        for p in &iz.curve {
            let x = x_at(p.range_yd as f32);
            painter.line_segment(
                [
                    egui::pos2(x, rect.top() + pad_t),
                    egui::pos2(x, rect.top() + pad_t + ph),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(22, 35, 52)),
            );
            let r_label = if self.metric_units {
                format!("{:.1}k m", p.range_yd * 0.0009144)
            } else {
                format!("{:.0}k yd", p.range_yd / 1000.0)
            };
            painter.text(
                egui::pos2(x, rect.top() + pad_t + ph + 12.0),
                egui::Align2::CENTER_CENTER,
                r_label,
                egui::FontId::monospace(9.0),
                egui::Color32::from_rgb(138, 164, 191),
            );
        }

        // Target belt line (dashed red)
        if iz.target_belt_in > 0.0 {
            let y = y_at(iz.target_belt_in as f32);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + pad_l, y),
                    egui::pos2(rect.left() + pad_l + pw, y),
                ],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 107, 107)),
            );
            painter.text(
                egui::pos2(rect.left() + pad_l + pw - 4.0, y - 4.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("Belt {:.1}\"", iz.target_belt_in),
                egui::FontId::monospace(9.0),
                egui::Color32::from_rgb(255, 107, 107),
            );
        }

        // Target deck line (dashed blue)
        if iz.target_deck_in > 0.0 {
            let y = y_at(iz.target_deck_in as f32);
            painter.line_segment(
                [
                    egui::pos2(rect.left() + pad_l, y),
                    egui::pos2(rect.left() + pad_l + pw, y),
                ],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(79, 179, 255)),
            );
            painter.text(
                egui::pos2(rect.left() + pad_l + pw - 4.0, y - 4.0),
                egui::Align2::RIGHT_BOTTOM,
                format!("Deck {:.1}\"", iz.target_deck_in),
                egui::FontId::monospace(9.0),
                egui::Color32::from_rgb(79, 179, 255),
            );
        }

        // Belt pen line (red)
        let belt_pts: Vec<egui::Pos2> = iz
            .curve
            .iter()
            .map(|p| egui::pos2(x_at(p.range_yd as f32), y_at(p.belt_pen_in as f32)))
            .collect();
        painter.add(egui::Shape::line(
            belt_pts.clone(),
            egui::Stroke::new(2.4, egui::Color32::from_rgb(255, 107, 107)),
        ));
        for pt in belt_pts {
            painter.circle_filled(pt, 2.8, egui::Color32::from_rgb(255, 107, 107));
        }

        // Deck pen line (cyan)
        let deck_pts: Vec<egui::Pos2> = iz
            .curve
            .iter()
            .map(|p| egui::pos2(x_at(p.range_yd as f32), y_at(p.deck_pen_in as f32)))
            .collect();
        painter.add(egui::Shape::line(
            deck_pts.clone(),
            egui::Stroke::new(2.4, egui::Color32::from_rgb(79, 179, 255)),
        ));
        for pt in deck_pts {
            painter.circle_filled(pt, 2.8, egui::Color32::from_rgb(79, 179, 255));
        }

        // Legend
        painter.text(
            egui::pos2(rect.left() + pad_l + 10.0, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "■ Belt Penetration (red)   ■ Deck Penetration (cyan)   ■ Safe Immunity Zone (green band)",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(219, 231, 243),
        );
    }
}

impl Default for SpringSharpApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for SpringSharpApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("nav_panel").show(ui, |ui| self.nav(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            self.dashboard_header(ui);
            egui::ScrollArea::vertical().show(ui, |ui| match self.page {
                Page::Hull => self.page_hull(ui),
                Page::Freeboard => self.page_freeboard(ui),
                Page::Armour => self.page_armour(ui),
                Page::Guns => self.page_guns(ui),
                Page::Weapons => self.page_weapons(ui),
                Page::Engine => self.page_engine(ui),
                Page::Blueprint => self.page_blueprint(ui),
                Page::Spaces => self.page_spaces(ui),
                Page::Calibration => self.page_calibration(ui),
                Page::Report => self.page_report(ui),
            });
        });
    }
}

fn weapon_mount_ui(ui: &mut egui::Ui, title: &str, id_salt: &str, m: &mut WeaponMount) {
    egui::CollapsingHeader::new(title)
        .id_salt(id_salt)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Weapon kind:");
                egui::ComboBox::from_id_salt(format!("{id_salt}_kind"))
                    .selected_text(match m.kind {
                        WeaponKind::Torpedo => "Torpedo",
                        WeaponKind::Mine => "Mine",
                        WeaponKind::DepthCharge => "Depth charge / AS mortar",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut m.kind, WeaponKind::Torpedo, "Torpedo");
                        ui.selectable_value(&mut m.kind, WeaponKind::Mine, "Mine");
                        ui.selectable_value(
                            &mut m.kind,
                            WeaponKind::DepthCharge,
                            "Depth charge / AS mortar",
                        );
                    });
            });
            drag_i32(ui, "Number of tubes / weapons", &mut m.number);
            drag_i32(ui, "Loads / reloads", &mut m.loads);
            drag_f64(
                ui,
                "Size (in diameter / lb charge)",
                &mut m.size,
                0.5,
                0.0..=5000.0,
            );
            drag_f64(ui, "Length (ft, torpedo)", &mut m.length, 0.5, 0.0..=100.0);
            drag_i32(ui, "Location index (0..8)", &mut m.location);
        });
}

fn combo_i32(ui: &mut egui::Ui, label: &str, value: &mut i32, options: &[&str]) {
    ui.horizontal(|ui| {
        ui.label(label);
        let curr_idx = (*value).max(0) as usize;
        let selected_text = options.get(curr_idx).copied().unwrap_or("Other");
        egui::ComboBox::from_id_salt(label)
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (idx, &opt) in options.iter().enumerate() {
                    ui.selectable_value(value, idx as i32, opt);
                }
            });
    });
}

fn get_layout_pos(layout: i32, fc: f64, fd: f64, qd: f64) -> f64 {
    let ad = (1.0 - fc - fd - qd).max(0.0);
    match layout {
        0 | 9 => 0.5,
        1 | 10 => 0.38,
        2 | 11 => 0.62,
        3 | 12 => fc + 0.25 * fd,
        4 | 13 => fc + 0.50 * fd,
        5 | 14 => fc + 0.75 * fd,
        6 | 15 => 1.0 - (qd + 0.75 * ad),
        7 | 16 => 1.0 - (qd + 0.50 * ad),
        8 | 17 => 1.0 - (qd + 0.25 * ad),
        _ => 0.5,
    }
}

fn get_turret_positions(b: &GunBattery, fc: f64, fd: f64, qd: f64) -> Vec<f64> {
    let mut list = Vec::new();
    let mut add_grp = |count: i32, layout: i32| {
        let center = get_layout_pos(layout, fc, fd, qd);
        if count == 1 {
            list.push(center);
        } else if count == 2 {
            list.push(center - 0.04);
            list.push(center + 0.04);
        } else if count == 3 {
            list.push(center - 0.05);
            list.push(center);
            list.push(center + 0.05);
        } else if count >= 4 {
            let step = 0.035;
            let start = center - (count - 1) as f64 * step / 2.0;
            for i in 0..count {
                list.push(start + i as f64 * step);
            }
        }
    };
    if b.mounts > 0 {
        if b.mounts_g1 > 0 {
            add_grp(b.mounts_g1, b.layout_g1);
        }
        if b.mounts_g2 > 0 {
            add_grp(b.mounts_g2, b.layout_g2);
        }
        if b.mounts_g1 == 0 && b.mounts_g2 == 0 {
            add_grp(b.mounts, b.layout_g1);
        }
    }
    list
}

fn drag_f64(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    speed: f64,
    range: std::ops::RangeInclusive<f64>,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).speed(speed).range(range));
    });
}

fn drag_i32(ui: &mut egui::Ui, label: &str, value: &mut i32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).speed(1.0));
    });
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "SpringSharp Rust",
        options,
        Box::new(|_cc| Ok(Box::new(SpringSharpApp::new()))),
    )
}
