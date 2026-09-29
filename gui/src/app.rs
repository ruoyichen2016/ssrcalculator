//! Native egui/eframe GUI for the SpringSharp Rust engine.
//!
//! Built as the separate `springsharp-gui` workspace crate so the core crate
//! never resolves the eframe dependency tree.  It exposes the most important
//! ship-design inputs, a calibration page and a report/calculate page.

use eframe::egui;

use springsharp::features::demo_ship;
use springsharp::io::{load_from_file, save_to_file};
use springsharp::model::{ResistanceModel, ShipModel};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Hull,
    Freeboard,
    Armour,
    Guns,
    Engine,
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
            (Page::Engine, "Engine"),
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
            self.recalculate();
            self.status = "Demo design loaded".to_string();
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
        drag_i32(ui, "Bow shape index", &mut f.bow);
        drag_i32(ui, "Stern shape index", &mut f.stern);
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
                drag_i32(ui, "Gun type", &mut g.gun_type);
                drag_i32(ui, "Mount type", &mut g.mount_type);
                drag_i32(ui, "Magazine rounds", &mut g.magazine);
                drag_i32(ui, "Mounts", &mut g.mounts);
                drag_i32(ui, "Guns/mount G1", &mut g.guns_in_mount_g1);
                drag_i32(ui, "Guns/mount G2", &mut g.guns_in_mount_g2);
                drag_i32(ui, "Layout G1", &mut g.layout_g1);
                drag_i32(ui, "Layout G2", &mut g.layout_g2);
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
            ui.checkbox(v, format!("Fuel {i}"));
        }
        ui.label("Engine types");
        for (i, v) in e.engine_types.iter_mut().enumerate() {
            ui.checkbox(v, format!("Engine type {i}"));
        }
        ui.label("Drive types");
        for (i, v) in e.drive.iter_mut().enumerate() {
            ui.checkbox(v, format!("Drive {i}"));
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
            egui::ScrollArea::vertical().show(ui, |ui| match self.page {
                Page::Hull => self.page_hull(ui),
                Page::Freeboard => self.page_freeboard(ui),
                Page::Armour => self.page_armour(ui),
                Page::Guns => self.page_guns(ui),
                Page::Engine => self.page_engine(ui),
                Page::Calibration => self.page_calibration(ui),
                Page::Report => self.page_report(ui),
            });
        });
    }
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
