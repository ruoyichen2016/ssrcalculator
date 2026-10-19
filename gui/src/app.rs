//! Native egui/eframe GUI for the SpringSharp Rust engine.
//!
//! Built as the separate `springsharp-gui` workspace crate so the core crate
//! never resolves the eframe dependency tree.  It exposes the most important
//! ship-design inputs, a calibration page and a report/calculate page.

use eframe::egui;
use springsharp::design::Evaluation;
use springsharp::presentation::fonts::TextLanguage;
use springsharp::workbench::{Command, Workbench};
use springsharp::DisplayName;
use std::path::PathBuf;
use std::sync::Arc;

use springsharp::model::{ResistanceModel, WeaponKind, WeaponMount};
use springsharp::spaces::calculate_internal_spaces;
use springsharp::systems::{
    AaDirector, BallisticComputer, BeltTdsJunction, CombatModality, ElectricalPlantStyle,
    HorizontalProtectionStyle, MainDirector, MobilityRegime, RadarSuite, SonarSuite, StyleDoctrine,
    TacticalEchelon, TdsArchitecture, TransverseSubdivision,
};
use springsharp::{compute_self_immunity_zone, get_preset_benchmark, PRESET_LIST};

use springsharp::presentation::options::{
    self as opts, BOW_TYPES as BOWS, GUNS_IN_MOUNT, GUN_TYPES, MOUNT_LAYOUTS as LAYOUTS,
    MOUNT_TYPES, STERN_TYPES as STERNS,
};

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
    workbench: Workbench,
    page: Page,
    current_path: Option<PathBuf>,
    text_language: TextLanguage,
    serif_titles: bool,
    status: String,
    regime_summary: String,
    selected_preset_index: Option<usize>,
    show_arcs: bool,
    target_cg: f64,
    target_displacement: f64,
    metric_units: bool,
    compare_preset_idx: Option<usize>,
    report_cache: Option<(Arc<Evaluation>, TextLanguage, String)>,
}

impl SpringSharpApp {
    pub fn new() -> Self {
        let workbench = Workbench::default();
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
            workbench,
            page: Page::Hull,
            current_path: None,
            text_language: TextLanguage::Germanic,
            serif_titles: false,
            status: "Demo design loaded".to_string(),
            regime_summary,
            selected_preset_index: None,
            show_arcs: false,
            target_cg: 0.45,
            target_displacement: 35000.0,
            metric_units: false,
            compare_preset_idx: None,
            report_cache: None,
        }
    }

    fn tr<'a>(&self, en: &'a str, zh: &'a str) -> &'a str {
        crate::i18n::tr(self.text_language, en, zh)
    }

    fn execute(&mut self, command: Command<'_>) {
        let replacing = matches!(
            command,
            Command::Demo | Command::Preset(_) | Command::Import(_)
        );
        self.status = match self.workbench.execute(command) {
            Ok(message) => {
                if replacing {
                    self.current_path = None;
                }
                springsharp::command_message(message)
            }
            Err(error) => format!("Error: {error}"),
        };
    }

    fn recalculate(&mut self) {
        self.execute(Command::Calculate);
    }

    fn open_design(&mut self) {
        let dialog = rfd::FileDialog::new().add_filter("SpringSharp design", &["ssr"]);
        if let Some(path) = dialog.pick_file() {
            match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|text| self.workbench.execute(Command::Import(&text)))
            {
                Ok(event) => {
                    self.status = springsharp::command_message(event);
                    self.current_path = Some(path);
                }
                Err(error) => self.status = format!("Open failed: {error}"),
            }
        }
    }

    fn save_design(&mut self) {
        let mut dialog = rfd::FileDialog::new().add_filter("SpringSharp design", &["ssr"]);
        if let Some(path) = &self.current_path {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(name) = path.file_name() {
                dialog = dialog.set_file_name(name.to_string_lossy());
            }
        } else {
            dialog = dialog.set_file_name("design.ssr");
        }
        if let Some(mut path) = dialog.save_file() {
            if path.extension().is_none() {
                path.set_extension("ssr");
            }
            let result = self.workbench.execute(Command::Calculate).and_then(|_| {
                std::fs::write(&path, self.workbench.export_design()).map_err(|e| e.to_string())
            });
            match result {
                Ok(()) => {
                    self.status = format!("Saved {}", path.display());
                    self.current_path = Some(path);
                }
                Err(error) => self.status = format!("Save failed: {error}"),
            }
        }
    }

    fn nav(&mut self, ui: &mut egui::Ui) {
        use crate::theme;
        let caption = |ui: &mut egui::Ui, t: &str| {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(t)
                    .size(10.5)
                    .strong()
                    .color(theme::BRASS),
            );
            ui.add_space(2.0);
        };
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("⚓").size(24.0).color(theme::BRASS));
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(egui::RichText::new("SpringSharp").size(18.0).strong());
                ui.label(
                    egui::RichText::new(self.tr("Warship design workbench", "战舰总体设计工作台"))
                        .small()
                        .color(theme::MUTED),
                );
            });
        });
        ui.add_space(8.0);
        let full = ui.available_width();
        let calc = egui::Button::new(
            egui::RichText::new(self.tr("⟳  Calculate", "⟳  计算指标"))
                .strong()
                .color(egui::Color32::WHITE),
        )
        .fill(egui::Color32::from_rgb(29, 91, 140))
        .stroke(egui::Stroke::new(1.0, theme::ACCENT))
        .corner_radius(egui::CornerRadius::same(9))
        .min_size(egui::vec2(full, 32.0));
        if ui.add(calc).clicked() {
            self.recalculate();
        }
        ui.columns(2, |cols| {
            let size = egui::vec2(cols[0].available_width(), 26.0);
            if cols[0]
                .add(egui::Button::new(self.tr("📂 Open…", "📂 打开…")).min_size(size))
                .clicked()
            {
                self.open_design();
            }
            if cols[1]
                .add(egui::Button::new(self.tr("💾 Save…", "💾 保存…")).min_size(size))
                .clicked()
            {
                self.save_design();
            }
        });

        caption(ui, self.tr("DESIGN", "设计"));
        for (page, icon, name) in [
            (Page::Hull, "⛴", self.tr("Hull", "船体主尺度")),
            (Page::Freeboard, "〰", self.tr("Freeboard", "干舷与型线")),
            (Page::Armour, "🛡", self.tr("Armour", "装甲与防护")),
            (Page::Guns, "🎯", self.tr("Guns", "舰炮武备")),
            (
                Page::Weapons,
                "✴",
                self.tr("Weapons & Misc", "副武器与杂项"),
            ),
            (Page::Engine, "⚙", self.tr("Engine", "动力推进")),
        ] {
            self.nav_item(ui, page, icon, name);
        }
        caption(ui, self.tr("ANALYSIS", "分析与校核"));
        for (page, icon, name) in [
            (
                Page::Blueprint,
                "📐",
                self.tr("Blueprint & Power", "总布置蓝图与功率"),
            ),
            (
                Page::Spaces,
                "▦",
                self.tr("Spaces & Benchmarks", "舱室与规范校核"),
            ),
            (
                Page::Calibration,
                "📊",
                self.tr("Calibration", "阻力与标定"),
            ),
            (Page::Report, "📄", self.tr("Report", "设计文本报告")),
        ] {
            self.nav_item(ui, page, icon, name);
        }

        caption(ui, self.tr("PRESETS", "历史名舰预设"));
        let current_preset_name = self
            .selected_preset_index
            .and_then(|idx| PRESET_LIST.get(idx))
            .map(|p| p.name)
            .unwrap_or(self.tr("Select preset...", "选择历史名舰预设..."));
        egui::ComboBox::from_id_salt("preset_picker")
            .selected_text(current_preset_name)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (idx, p) in PRESET_LIST.iter().enumerate() {
                    let is_sel = self.selected_preset_index == Some(idx);
                    if ui.selectable_label(is_sel, p.name).clicked() {
                        self.execute(Command::Preset(p.id));
                        self.selected_preset_index = Some(idx);
                    }
                }
            });
        if ui
            .add(
                egui::Button::new(self.tr("Load demo", "加载演示设计"))
                    .min_size(egui::vec2(ui.available_width(), 24.0)),
            )
            .clicked()
        {
            self.execute(Command::Demo);
            self.selected_preset_index = None;
        }

        caption(ui, self.tr("DISPLAY", "显示与语言"));
        let imp_lbl = self.tr("Imperial", "英制");
        let met_lbl = self.tr("Metric", "公制");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.metric_units, false, imp_lbl);
            ui.selectable_value(&mut self.metric_units, true, met_lbl);
        });
        let prev_lang = self.text_language;
        egui::ComboBox::from_id_salt("text_language")
            .selected_text(self.text_language.label())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for language in TextLanguage::ALL {
                    ui.selectable_value(&mut self.text_language, language, language.label());
                }
            });
        if self.text_language != prev_lang {
            crate::fonts::install(ui.ctx(), self.text_language);
            // Labels below the selector must use the new language in this frame.
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new("app_language"), self.text_language));
            ui.ctx().request_repaint();
        }
        let serif_lbl = self.tr("Serif titles", "衬线标题");
        ui.checkbox(&mut self.serif_titles, serif_lbl);
        ui.label(
            egui::RichText::new(self.tr(
                "Font group; interface labels use English.",
                "字体族与界面语言设置（汉字模式启用中文界面）。",
            ))
            .small()
            .color(theme::MUTED),
        );

        ui.add_space(10.0);
        egui::Frame::new()
            .fill(theme::alpha(theme::SURFACE_HI, 120))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let color = if self.status.contains("rror") || self.status.contains("failed") {
                    theme::BAD
                } else {
                    theme::MUTED
                };
                ui.label(
                    egui::RichText::new(
                        springsharp::presentation::latin_ui::status(
                            self.text_language,
                            &self.status,
                        )
                        .as_ref(),
                    )
                    .small()
                    .color(color),
                );
            });
    }

    fn nav_item(&mut self, ui: &mut egui::Ui, page: Page, icon: &str, name: &str) {
        use crate::theme;
        let selected = self.page == page;
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), egui::Sense::click());
        let painter = ui.painter();
        if selected {
            painter.rect_filled(
                rect,
                egui::CornerRadius::same(8),
                theme::alpha(theme::ACCENT, 46),
            );
            let bar =
                egui::Rect::from_min_size(rect.min + egui::vec2(0.0, 6.0), egui::vec2(3.0, 18.0));
            painter.rect_filled(bar, egui::CornerRadius::same(2), theme::ACCENT);
        } else if resp.hovered() {
            painter.rect_filled(
                rect,
                egui::CornerRadius::same(8),
                theme::alpha(theme::SURFACE_HI, 160),
            );
        }
        let color = if selected {
            egui::Color32::WHITE
        } else {
            theme::TEXT
        };
        painter.text(
            rect.left_center() + egui::vec2(14.0, 0.0),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(14.0),
            if selected {
                theme::ACCENT
            } else {
                theme::MUTED
            },
        );
        painter.text(
            rect.left_center() + egui::vec2(38.0, 0.0),
            egui::Align2::LEFT_CENTER,
            name,
            egui::FontId::proportional(14.0),
            color,
        );
        if resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            self.page = page;
        }
    }

    fn page_hull(&mut self, ui: &mut egui::Ui) {
        crate::theme::heading(ui, "Hull");
        let h = &mut self.workbench.design.draft.hull;
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.length"),
            &mut h.length,
            0.1,
            1.0..=2000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.beam"),
            &mut h.beam,
            0.1,
            1.0..=300.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.beam_bulges"),
            &mut h.beam_bulges,
            0.1,
            1.0..=300.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.draught"),
            &mut h.draught,
            0.1,
            0.1..=100.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.block"),
            &mut h.block,
            0.001,
            0.2..=0.95,
        );
        drag_i32(
            ui,
            springsharp::presentation::fields::label("hull.date_hull"),
            &mut h.date_hull,
        );
        drag_i32(
            ui,
            springsharp::presentation::fields::label("hull.date_engine"),
            &mut h.date_engine,
        );
        drag_f64(ui, "LCB (% L)", &mut h.lcb_percent, 0.01, -10.0..=10.0);
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.midship_coefficient"),
            &mut h.midship_coefficient,
            0.001,
            0.0..=0.99,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.bulb_area_sqft"),
            &mut h.bulb_area_sqft,
            1.0,
            0.0..=10_000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.bulb_centre_height_ft"),
            &mut h.bulb_centre_height_ft,
            0.1,
            0.0..=100.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.transom_area_sqft"),
            &mut h.transom_area_sqft,
            1.0,
            0.0..=10_000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.appendage_area_sqft"),
            &mut h.appendage_area_sqft,
            1.0,
            0.0..=10_000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("hull.stern_shape"),
            &mut h.stern_shape,
            0.1,
            -25.0..=25.0,
        );
    }

    fn page_freeboard(&mut self, ui: &mut egui::Ui) {
        crate::theme::heading(ui, "Freeboard");
        let f = &mut self.workbench.design.draft.free;
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
        drag_f64(
            ui,
            springsharp::presentation::fields::label("free.length_ram"),
            &mut f.length_ram,
            0.1,
            0.0..=200.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("free.length_stern"),
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
        crate::theme::heading(ui, "Armour");
        let a = &mut self.workbench.design.draft.armour;
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.belt_main"),
            &mut a.belt_main,
            0.1,
            0.0..=40.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.belt_main_length"),
            &mut a.belt_main_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.belt_main_height"),
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
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.belt_upper"),
            &mut a.belt_upper,
            0.1,
            0.0..=40.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.belt_upper_length"),
            &mut a.belt_upper_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.belt_upper_height"),
            &mut a.belt_upper_height,
            0.1,
            0.0..=100.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.belts_incline"),
            &mut a.belts_incline,
            0.1,
            -45.0..=45.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.bulkhead"),
            &mut a.bulkhead,
            0.1,
            0.0..=40.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.bulkhead_length"),
            &mut a.bulkhead_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.bulkhead_height"),
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
        combo_i32(
            ui,
            "Bulkhead type",
            &mut a.bulkhead_type,
            opts::BULKHEAD_TYPES,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.bulge"),
            &mut a.bulge,
            0.1,
            0.0..=40.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.bulge_length"),
            &mut a.bulge_length,
            1.0,
            0.0..=2000.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("arm.bulge_height"),
            &mut a.bulge_height,
            0.1,
            0.0..=100.0,
        );
        drag_f64(ui, "Deck (in)", &mut a.deck, 0.1, 0.0..=40.0);
        drag_f64(ui, "Deck FC (in)", &mut a.deck_fc, 0.1, 0.0..=40.0);
        drag_f64(ui, "Deck QD (in)", &mut a.deck_qd, 0.1, 0.0..=40.0);
        combo_i32(
            ui,
            "Deck coverage",
            &mut a.deck_coverage,
            opts::DECK_COVERAGE,
        );
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
        crate::theme::heading(ui, "Gun batteries");
        for (i, g) in self
            .workbench
            .design
            .draft
            .guns
            .batteries
            .iter_mut()
            .enumerate()
        {
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
                ui.checkbox(
                    &mut g.super_super_chk,
                    crate::i18n::latin(ui, "Super-super"),
                );
                ui.checkbox(&mut g.sub_sub_chk, crate::i18n::latin(ui, "Sub-sub"));
            });
        }
    }

    fn superstructure_controls(&mut self, ui: &mut egui::Ui) {
        use springsharp::presentation::superstructure_text::{archetype_name, planning_notes};
        use springsharp::superstructure::Archetype;
        ui.collapsing("Superstructure", |ui| {
            if ui.button(crate::i18n::latin(ui, "Reset layout")).clicked() { self.workbench.design.draft.superstructure = Default::default(); }
            let cfg = &mut self.workbench.design.draft.superstructure;
            egui::ComboBox::from_id_salt("superstructure_type").selected_text(archetype_name(cfg.archetype)).show_ui(ui, |ui| {
                for kind in Archetype::ALL { ui.selectable_value(&mut cfg.archetype, kind, archetype_name(kind)); }
            });
            egui::ComboBox::from_id_salt("mast_style").selected_text(springsharp::presentation::superstructure_text::mast_name(cfg.mast)).show_ui(ui, |ui| {
                for style in springsharp::superstructure::MastStyle::ALL {ui.selectable_value(&mut cfg.mast,style,springsharp::presentation::superstructure_text::mast_name(style));}
            });
            ui.small("Positions / dimensions: fractions of hull length from bow. Geometry only; mass is not added.");
            ui.horizontal(|ui| { ui.label(crate::i18n::latin(ui, "Funnels (0 = Auto)")); ui.add(egui::DragValue::new(&mut cfg.funnel_count).range(0..=4)); });
            drag_f64(ui, "Funnel spacing (0 = Auto)", &mut cfg.funnel_spacing, 0.005, 0.0..=0.2);
            for (name, correction) in [("Bridge", &mut cfg.bridge), ("Aft deckhouse", &mut cfg.aft), ("Funnel group", &mut cfg.funnels)] {
                ui.push_id(name, |ui| { ui.collapsing(name, |ui| {
                    ui.checkbox(&mut correction.locked, crate::i18n::latin(ui, "Lock geometry"));
                    if correction.locked {
                        drag_f64(ui, "Position", &mut correction.x, 0.005, 0.0..=1.0);
                        drag_f64(ui, "Length", &mut correction.length, 0.005, 0.005..=0.5);
                        drag_f64(ui, "Height", &mut correction.height, 0.005, 0.005..=0.15);
                    } else {
                        drag_f64(ui, "Relative shift", &mut correction.shift, 0.005, -0.5..=0.5);
                        drag_f64(ui, "Length multiplier", &mut correction.length_scale, 0.1, 0.2..=3.0);
                        drag_f64(ui, "Height multiplier", &mut correction.height_scale, 0.1, 0.2..=3.0);
                    }
                }); });
            }
            ui.small("Aft deckhouse: Low distributed / Fore–aft split only. Funnel lock fixes the group envelope.");
            let result = self.workbench.design.result();
            for note in planning_notes(&result.blueprint.superstructure) { ui.label(note); }
        });
    }

    fn page_engine(&mut self, ui: &mut egui::Ui) {
        self.superstructure_controls(ui);
        crate::theme::heading(ui, "Machinery");
        let e = &mut self.workbench.design.draft.engine;
        drag_f64(
            ui,
            springsharp::presentation::fields::label("engine.speed_max"),
            &mut e.speed_max,
            0.1,
            0.0..=60.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("engine.speed_cruise"),
            &mut e.speed_cruise,
            0.1,
            0.0..=60.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("engine.range"),
            &mut e.range,
            10.0,
            0.0..=100_000.0,
        );
        drag_f64(ui, "Percent coal", &mut e.pct_coal, 0.1, 0.0..=100.0);
        drag_i32(
            ui,
            springsharp::presentation::fields::label("engine.screws"),
            &mut e.screws,
        );
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
            ui.label(crate::i18n::latin(ui, "Resistance model:"));
            egui::ComboBox::from_id_salt("resistance_model")
                .selected_text(match e.resistance_model {
                    ResistanceModel::Original => crate::i18n::latin(ui, "Original"),
                    ResistanceModel::HoltropMennen => "Holtrop-Mennen",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut e.resistance_model,
                        ResistanceModel::Original,
                        crate::i18n::latin(ui, "Original"),
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
            springsharp::presentation::fields::label("engine.holtrop_viscous_factor"),
            &mut e.holtrop_viscous_factor,
            0.001,
            0.0..=5.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("engine.holtrop_wave_factor"),
            &mut e.holtrop_wave_factor,
            0.001,
            0.0..=5.0,
        );
        drag_f64(
            ui,
            springsharp::presentation::fields::label("engine.holtrop_correlation_factor"),
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
        ui.label(crate::i18n::latin(ui, "Fuel types"));
        for (i, v) in e.fuel.iter_mut().enumerate() {
            let name = match i {
                0 => "Coal fired boilers",
                1 => "Oil fired boilers",
                2 => "Diesel motors",
                3 => "Petrol motors",
                _ => "Batteries",
            };
            ui.checkbox(v, crate::i18n::latin(ui, name));
        }
        ui.label(crate::i18n::latin(ui, "Engine types"));
        for (i, v) in e.engine_types.iter_mut().enumerate() {
            let name = match i {
                0 => "Simple reciprocating",
                1 => "Complex reciprocating",
                _ => "Steam turbines",
            };
            ui.checkbox(v, crate::i18n::latin(ui, name));
        }
        ui.label(crate::i18n::latin(ui, "Drive types"));
        for (i, v) in e.drive.iter_mut().enumerate() {
            let name = match i {
                0 => "Direct",
                1 => "Geared",
                2 => "Electric",
                _ => "Hydraulic",
            };
            ui.checkbox(v, crate::i18n::latin(ui, name));
        }
    }

    fn page_calibration(&mut self, ui: &mut egui::Ui) {
        crate::theme::heading(ui, "Calibration (modern regime model)");
        ui.label(
            "Default model: Holtrop-Mennen + per-ship-type calibration factor, \
             fitted to the public historical dataset.",
        );
        ui.separator();
        ui.label(format!(
            "Ship type: {}",
            self.workbench.design.draft.ship_type
        ));
        match springsharp::regime::regime_for_ship_type(&self.workbench.design.draft.ship_type) {
            Some(r) => {
                ui.label(format!(
                    "Recommended: {}  factor {:.3}  (in-sample MAPE {:.1}%)",
                    r.model_label,
                    r.factor,
                    r.train_mape * 100.0
                ));
                if ui
                    .button(crate::i18n::latin(ui, "Apply regime model"))
                    .clicked()
                {
                    self.execute(Command::Regime);
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
        ui.label(crate::i18n::latin(
            ui,
            "Nested-LOOCV (out-of-sample) comparison:",
        ));
        ui.monospace(self.regime_summary.as_str());
        ui.separator();
        ui.colored_label(
            egui::Color32::YELLOW,
            "Data provenance: public historical references only (not independently \
             verified). Official trial/model-basin data unavailable; treat results as approximate.",
        );

        ui.separator();
        crate::theme::heading(ui, "Target Displacement Solver");
        ui.label(
            "Scale hull dimensions (length, beam, draught) proportionally to meet an exact \
             target displacement (converges in 4-6 iterations).",
        );
        ui.horizontal(|ui| {
            ui.label(crate::i18n::latin(ui, "Target normal displacement:"));
            drag_f64(
                ui,
                "tons",
                &mut self.target_displacement,
                500.0,
                100.0..=120000.0,
            );
            if ui
                .button(crate::i18n::latin(ui, "Solve & Scale Hull"))
                .clicked()
            {
                self.execute(Command::Displacement(self.target_displacement));
            }
        });

        ui.separator();
        crate::theme::heading(ui, "Side-by-Side Ship Comparison");
        ui.label(
            "Compare your current design against a pinned baseline or historical warship preset.",
        );
        ui.horizontal(|ui| {
            if ui
                .button(crate::i18n::latin(ui, "Pin current as baseline"))
                .clicked()
            {
                self.execute(Command::PinBaseline);
                self.compare_preset_idx = None;
            }
            if ui
                .button(crate::i18n::latin(ui, "Clear comparison"))
                .clicked()
            {
                self.execute(Command::ClearComparison);
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
                            self.execute(Command::ComparePreset(p.id));
                            self.compare_preset_idx = Some(idx);
                        }
                    }
                });
        });

        let result = self.workbench.design.result();
        if let Some(base) = self.workbench.baseline() {
            ui.add_space(6.0);
            egui::Grid::new("comparison_table")
                .striped(true)
                .spacing([24.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("Metric");
                    ui.strong(format!("Current ({})", result.ship.name));
                    ui.strong(format!("Baseline ({})", base.name));
                    ui.strong("Delta (Δ)");
                    ui.strong("Rel. Diff (%)");
                    ui.end_row();

                    let rows = [
                        (
                            "Length WL (ft)",
                            result.ship.hull.length,
                            base.hull.length,
                            1,
                        ),
                        ("Beam (ft)", result.ship.hull.beam, base.hull.beam, 1),
                        (
                            "Draught (ft)",
                            result.ship.hull.draught,
                            base.hull.draught,
                            1,
                        ),
                        (
                            "Normal disp (t)",
                            result.ship.hull.displacement,
                            base.hull.displacement,
                            0,
                        ),
                        (
                            "Full load disp (t)",
                            result.ship.disp_max_load,
                            base.disp_max_load,
                            0,
                        ),
                        (
                            "Max speed (kt)",
                            result.ship.engine.speed_max,
                            base.engine.speed_max,
                            1,
                        ),
                        (
                            "Max power (hp)",
                            result.ship.engine.hp_max,
                            base.engine.hp_max,
                            0,
                        ),
                        ("Range (nm)", result.ship.engine.range, base.engine.range, 0),
                        (
                            "Main belt (in)",
                            result.ship.armour.belt_main,
                            base.armour.belt_main,
                            1,
                        ),
                        (
                            "Main deck (in)",
                            result.ship.armour.deck,
                            base.armour.deck,
                            1,
                        ),
                        (
                            "Main battery cal (in)",
                            result.ship.guns.batteries[0].calibre,
                            base.guns.batteries[0].calibre,
                            1,
                        ),
                        (
                            "Broadside weight (lb)",
                            result.ship.guns.broadside,
                            base.guns.broadside,
                            0,
                        ),
                        (
                            "Metacentre GM (ft)",
                            result.ship.metacentre,
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
                            ui.label(crate::i18n::latin(ui, "–"));
                        }
                        ui.end_row();
                    }
                });
        }
    }

    fn page_report(&mut self, ui: &mut egui::Ui) {
        let language = self.text_language;
        let tr = |en, zh, la| springsharp::presentation::report::report_text(language, en, zh, la);
        crate::theme::heading(
            ui,
            tr("Report and files", "报告与文件", "Relatio et documenta"),
        );
        ui.horizontal(|ui| {
            if ui
                .button(tr(
                    "Calculate and regenerate report",
                    "计算并重新生成报告",
                    "Computare et relationem renovare",
                ))
                .clicked()
            {
                self.recalculate();
            }
            if ui
                .button(tr("Load demo", "载入示例", "Exemplum aperire"))
                .clicked()
            {
                self.execute(Command::Demo);
            }
        });
        ui.horizontal(|ui| {
            if ui
                .button(tr("Open .ssr…", "打开 .ssr…", "Aperire .ssr…"))
                .clicked()
            {
                self.open_design();
            }
            if ui
                .button(tr("Save .ssr…", "保存 .ssr…", "Servare .ssr…"))
                .clicked()
            {
                self.save_design();
            }
        });
        if let Some(path) = &self.current_path {
            ui.label(path.display().to_string());
        }
        ui.separator();
        ui.label(
            springsharp::presentation::latin_ui::status(self.text_language, &self.status).as_ref(),
        );
        let result = self.workbench.design.result();
        // Recompute the comparison only when the committed design or language changes.
        if !matches!(&self.report_cache, Some((snapshot, language, _))
            if Arc::ptr_eq(snapshot, &result) && *language == self.text_language)
        {
            self.report_cache = Some((
                Arc::clone(&result),
                self.text_language,
                springsharp::generate_report_with_language(&result.ship, self.text_language),
            ));
        }
        let mut report = self.report_cache.as_ref().unwrap().2.as_str();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut report)
                    .desired_width(f32::INFINITY)
                    .desired_rows(30),
            );
        });
    }

    fn page_weapons(&mut self, ui: &mut egui::Ui) {
        crate::theme::heading(
            ui,
            "Torpedoes, Mines, Depth Charges & Miscellaneous Weights",
        );
        let w = &mut self.workbench.design.draft.weapons;
        weapon_mount_ui(ui, "Torpedo set 1", "weap0", &mut w.torpedo1);
        weapon_mount_ui(ui, "Torpedo set 2", "weap1", &mut w.torpedo2);
        weapon_mount_ui(ui, "Mines", "weap2", &mut w.mine);
        weapon_mount_ui(ui, "Depth charges / AS mortars 1", "weap3", &mut w.dc1);
        weapon_mount_ui(ui, "Depth charges / AS mortars 2", "weap4", &mut w.dc2);

        ui.separator();
        crate::theme::heading(ui, "Miscellaneous weights (tons)");
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

        ui.separator();
        crate::theme::heading(
            ui,
            "Electrical Plant & Combat Systems (Dimensionless Ideal-Type & Mix-and-Match)",
        );
        let result = self.workbench.design.result();
        let sys_eval = &result.systems;

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "● Fleet Tactical Role: {}",
                    springsharp::presentation::classification(&sys_eval.tactical_affinities)
                ))
                .color(egui::Color32::from_rgb(88, 166, 255))
                .strong(),
            );
        });
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "● Inferred Engineering Style: {}",
                    sys_eval.resolved_doctrine.display_name()
                ))
                .color(egui::Color32::from_rgb(155, 123, 234))
                .strong(),
            );
            ui.label(format!(
                "(Type I: {:.0}% · Type II: {:.0}% · Type III: {:.0}% · Type IV: {:.0}%)",
                sys_eval.type1_affinity * 100.0,
                sys_eval.type2_affinity * 100.0,
                sys_eval.type3_affinity * 100.0,
                sys_eval.type4_affinity * 100.0
            ));
        });
        ui.label(
            egui::RichText::new(springsharp::presentation::match_explanation(sys_eval))
                .small()
                .color(egui::Color32::from_rgb(201, 209, 217)),
        );

        ui.horizontal(|ui| {
            ui.label(format!(
                "Peak Load: {:.0} kW vs Installed: {:.0} kW ({:.0} Turbo + {:.0} Diesel)",
                sys_eval.combat_peak_kw,
                sys_eval.total_installed_kw,
                sys_eval.turbo_generator_kw,
                sys_eval.diesel_generator_kw
            ));
            if sys_eval.is_overloaded {
                ui.colored_label(
                    egui::Color32::from_rgb(224, 93, 111),
                    "▲ ELECTRICAL OVERLOAD",
                );
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(95, 211, 141),
                    format!("● Margin: {:+.1}%", sys_eval.electrical_margin_pct),
                );
            }
        });
        let load_pct =
            (sys_eval.combat_peak_kw / sys_eval.total_installed_kw.max(1.0)).clamp(0.0, 1.0) as f32;
        ui.add(egui::ProgressBar::new(load_pct).show_percentage());

        ui.label(format!(
            "Load: Turrets {:.0} kW · Hull/Pumps {:.0} kW · Fire-Control {:.0} kW · Sensors {:.0} kW",
            sys_eval.load_turrets_kw,
            sys_eval.load_hull_hotel_kw,
            sys_eval.load_fire_control_kw,
            sys_eval.load_electronics_kw
        ));
        ui.label(format!(
            "Weights: Topweight {:.1} t (ΔGM {:+.2} ft) · Deck {:.1} t · Hull {:.1} t · Vital {:.1} t",
            sys_eval.topweight_tons,
            sys_eval.estimated_delta_gm_ft,
            sys_eval.deck_weight_tons,
            sys_eval.hull_weight_tons,
            sys_eval.vital_weight_tons
        ));

        ui.add_space(4.0);
        egui::Grid::new("systems_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label(crate::i18n::latin(ui, "Style Ideal Type:"));
                egui::ComboBox::from_id_salt("sys_doctrine")
                    .selected_text(self.workbench.design.draft.systems.doctrine.display_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.doctrine,
                            StyleDoctrine::AdaptiveAuto,
                            StyleDoctrine::AdaptiveAuto.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.doctrine,
                            StyleDoctrine::AutomationAllElectric,
                            StyleDoctrine::AutomationAllElectric.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.doctrine,
                            StyleDoctrine::AllWeatherSeakeeping,
                            StyleDoctrine::AllWeatherSeakeeping.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.doctrine,
                            StyleDoctrine::LongRangeOpticalStrike,
                            StyleDoctrine::LongRangeOpticalStrike.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.doctrine,
                            StyleDoctrine::StereoscopicSubdivided,
                            StyleDoctrine::StereoscopicSubdivided.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.doctrine,
                            StyleDoctrine::Custom,
                            StyleDoctrine::Custom.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Tactical Echelon:"));
                egui::ComboBox::from_id_salt("sys_tactical_echelon")
                    .selected_text(
                        if self.workbench.design.draft.systems.tactical_echelon
                            == TacticalEchelon::Auto
                        {
                            format!("Auto ({})", sys_eval.resolved_echelon.display_name())
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .tactical_echelon
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.tactical_echelon,
                            TacticalEchelon::Auto,
                            TacticalEchelon::Auto.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.tactical_echelon,
                            TacticalEchelon::BattleLineMainBody,
                            TacticalEchelon::BattleLineMainBody.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.tactical_echelon,
                            TacticalEchelon::FastWingVanguard,
                            TacticalEchelon::FastWingVanguard.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.tactical_echelon,
                            TacticalEchelon::OuterScoutingScreen,
                            TacticalEchelon::OuterScoutingScreen.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.tactical_echelon,
                            TacticalEchelon::InnerFleetScreen,
                            TacticalEchelon::InnerFleetScreen.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.tactical_echelon,
                            TacticalEchelon::FlotillaAssaultLine,
                            TacticalEchelon::FlotillaAssaultLine.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Combat Modality:"));
                egui::ComboBox::from_id_salt("sys_combat_modality")
                    .selected_text(
                        if self.workbench.design.draft.systems.combat_modality
                            == CombatModality::Auto
                        {
                            format!("Auto ({})", sys_eval.resolved_modality.display_name())
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .combat_modality
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.combat_modality,
                            CombatModality::Auto,
                            CombatModality::Auto.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.combat_modality,
                            CombatModality::HeavyArtilleryCitadel,
                            CombatModality::HeavyArtilleryCitadel.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.combat_modality,
                            CombatModality::RapidFireSurfaceGunnery,
                            CombatModality::RapidFireSurfaceGunnery.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.combat_modality,
                            CombatModality::AviationDeckProjection,
                            CombatModality::AviationDeckProjection.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.combat_modality,
                            CombatModality::MassedTorpedoStrike,
                            CombatModality::MassedTorpedoStrike.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.combat_modality,
                            CombatModality::ScreenEscortDefense,
                            CombatModality::ScreenEscortDefense.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Mobility Regime:"));
                egui::ComboBox::from_id_salt("sys_mobility_regime")
                    .selected_text(
                        if self.workbench.design.draft.systems.mobility_regime
                            == MobilityRegime::Auto
                        {
                            format!(
                                "Auto ({})",
                                sys_eval.resolved_mobility_regime.display_name()
                            )
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .mobility_regime
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.mobility_regime,
                            MobilityRegime::Auto,
                            MobilityRegime::Auto.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.mobility_regime,
                            MobilityRegime::HeavyDisplacementBlock,
                            MobilityRegime::HeavyDisplacementBlock.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.mobility_regime,
                            MobilityRegime::TransFroudeFastCapital,
                            MobilityRegime::TransFroudeFastCapital.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.mobility_regime,
                            MobilityRegime::SlenderWaveCutting,
                            MobilityRegime::SlenderWaveCutting.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.mobility_regime,
                            MobilityRegime::SuperCriticalHighFn,
                            MobilityRegime::SuperCriticalHighFn.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Main Battery Director:"));
                egui::ComboBox::from_id_salt("sys_main_dir")
                    .selected_text(
                        if self.workbench.design.draft.systems.main_director == MainDirector::Auto {
                            format!("Auto ({})", sys_eval.resolved_main_director.display_name())
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .main_director
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.main_director,
                            MainDirector::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.main_director,
                            MainDirector::OpenSights,
                            MainDirector::OpenSights.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.main_director,
                            MainDirector::CentralTower,
                            MainDirector::CentralTower.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.main_director,
                            MainDirector::WeatherproofDct,
                            MainDirector::WeatherproofDct.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.main_director,
                            MainDirector::ArmouredStereoscopic10m,
                            MainDirector::ArmouredStereoscopic10m.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.main_director,
                            MainDirector::DuplexOpticalTower15m,
                            MainDirector::DuplexOpticalTower15m.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.main_director,
                            MainDirector::GyrostabilizedRadarBlindFire,
                            MainDirector::GyrostabilizedRadarBlindFire.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "AA / Secondary Director:"));
                egui::ComboBox::from_id_salt("sys_aa_dir")
                    .selected_text(
                        if self.workbench.design.draft.systems.aa_director == AaDirector::Auto {
                            format!("Auto ({})", sys_eval.resolved_aa_director.display_name())
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .aa_director
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.aa_director,
                            AaDirector::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.aa_director,
                            AaDirector::None,
                            AaDirector::None.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.aa_director,
                            AaDirector::HighAngleTachymetric,
                            AaDirector::HighAngleTachymetric.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.aa_director,
                            AaDirector::TriAxialStabilizedSphere,
                            AaDirector::TriAxialStabilizedSphere.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.aa_director,
                            AaDirector::OpticalLeadComputing,
                            AaDirector::OpticalLeadComputing.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.aa_director,
                            AaDirector::DualPurposeRadarDirector,
                            AaDirector::DualPurposeRadarDirector.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Ballistic Computer:"));
                egui::ComboBox::from_id_salt("sys_comp")
                    .selected_text(
                        if self.workbench.design.draft.systems.ballistic_computer
                            == BallisticComputer::Auto
                        {
                            format!(
                                "Auto ({})",
                                sys_eval.resolved_ballistic_computer.display_name()
                            )
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .ballistic_computer
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.ballistic_computer,
                            BallisticComputer::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.ballistic_computer,
                            BallisticComputer::ManualTables,
                            BallisticComputer::ManualTables.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.ballistic_computer,
                            BallisticComputer::MechanicalRateClock,
                            BallisticComputer::MechanicalRateClock.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.ballistic_computer,
                            BallisticComputer::ElectroMechanicalTable,
                            BallisticComputer::ElectroMechanicalTable.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.ballistic_computer,
                            BallisticComputer::HighVelocityAnalog,
                            BallisticComputer::HighVelocityAnalog.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.ballistic_computer,
                            BallisticComputer::MultiAxisTriangulator,
                            BallisticComputer::MultiAxisTriangulator.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.ballistic_computer,
                            BallisticComputer::GyroCamRangekeeper,
                            BallisticComputer::GyroCamRangekeeper.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Radar & EW Suite:"));
                egui::ComboBox::from_id_salt("sys_radar")
                    .selected_text(
                        if self.workbench.design.draft.systems.radar_suite == RadarSuite::Auto {
                            format!("Auto ({})", sys_eval.resolved_radar_suite.display_name())
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .radar_suite
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.radar_suite,
                            RadarSuite::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.radar_suite,
                            RadarSuite::None,
                            RadarSuite::None.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.radar_suite,
                            RadarSuite::DecimetricSurfaceFc,
                            RadarSuite::DecimetricSurfaceFc.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.radar_suite,
                            RadarSuite::MetricSearchMagnetron,
                            RadarSuite::MetricSearchMagnetron.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.radar_suite,
                            RadarSuite::CentimetricFcDirectionFinder,
                            RadarSuite::CentimetricFcDirectionFinder.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.radar_suite,
                            RadarSuite::DualBandBlindFireCic,
                            RadarSuite::DualBandBlindFireCic.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Subsurface Sonar Suite:"));
                egui::ComboBox::from_id_salt("sys_sonar")
                    .selected_text(
                        if self.workbench.design.draft.systems.sonar_suite == SonarSuite::Auto {
                            format!("Auto ({})", sys_eval.resolved_sonar_suite.display_name())
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .sonar_suite
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.sonar_suite,
                            SonarSuite::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.sonar_suite,
                            SonarSuite::None,
                            SonarSuite::None.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.sonar_suite,
                            SonarSuite::PassiveHydrophones,
                            SonarSuite::PassiveHydrophones.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.sonar_suite,
                            SonarSuite::ActiveQuartzDome,
                            SonarSuite::ActiveQuartzDome.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.sonar_suite,
                            SonarSuite::MultiElementPassiveArray,
                            SonarSuite::MultiElementPassiveArray.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.sonar_suite,
                            SonarSuite::HighPowerSearchSonar,
                            SonarSuite::HighPowerSearchSonar.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Electrical Plant Style:"));
                egui::ComboBox::from_id_salt("sys_plant")
                    .selected_text(
                        if self.workbench.design.draft.systems.electrical_plant
                            == ElectricalPlantStyle::Auto
                        {
                            format!(
                                "Auto ({})",
                                sys_eval.resolved_electrical_plant.display_name()
                            )
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .electrical_plant
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.electrical_plant,
                            ElectricalPlantStyle::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.electrical_plant,
                            ElectricalPlantStyle::CompactSteam,
                            ElectricalPlantStyle::CompactSteam.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.electrical_plant,
                            ElectricalPlantStyle::BalancedTurboDiesel,
                            ElectricalPlantStyle::BalancedTurboDiesel.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.electrical_plant,
                            ElectricalPlantStyle::SubdividedHeavyDiesel,
                            ElectricalPlantStyle::SubdividedHeavyDiesel.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.electrical_plant,
                            ElectricalPlantStyle::HighMarginAcGrid,
                            ElectricalPlantStyle::HighMarginAcGrid.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Belt-TDS Joint"));
                egui::ComboBox::from_id_salt("sys_belt_tds_junction")
                    .selected_text(
                        if self.workbench.design.draft.systems.belt_tds_junction
                            == BeltTdsJunction::Auto
                        {
                            format!(
                                "Auto ({})",
                                self.workbench
                                    .design
                                    .result()
                                    .systems
                                    .resolved_belt_tds_junction
                                    .display_name()
                            )
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .belt_tds_junction
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.belt_tds_junction,
                            BeltTdsJunction::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.belt_tds_junction,
                            BeltTdsJunction::IntegralTaperedHoldingBulkhead,
                            BeltTdsJunction::IntegralTaperedHoldingBulkhead.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.belt_tds_junction,
                            BeltTdsJunction::TurtlebackSlopeKneeJoint,
                            BeltTdsJunction::TurtlebackSlopeKneeJoint.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.belt_tds_junction,
                            BeltTdsJunction::SpacedAbsorberShelfJoint,
                            BeltTdsJunction::SpacedAbsorberShelfJoint.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.belt_tds_junction,
                            BeltTdsJunction::SteppedShelfCitadel,
                            BeltTdsJunction::SteppedShelfCitadel.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.belt_tds_junction,
                            BeltTdsJunction::DirectSkinDoubleBottom,
                            BeltTdsJunction::DirectSkinDoubleBottom.display_name(),
                        );
                    });
                ui.end_row();

                ui.label(crate::i18n::latin(ui, "Horizontal Protection"));
                egui::ComboBox::from_id_salt("sys_horizontal_protection")
                    .selected_text(
                        if self.workbench.design.draft.systems.horizontal_protection
                            == HorizontalProtectionStyle::Auto
                        {
                            format!(
                                "Auto ({})",
                                self.workbench
                                    .design
                                    .result()
                                    .systems
                                    .resolved_horizontal_protection
                                    .display_name()
                            )
                        } else {
                            self.workbench
                                .design
                                .draft
                                .systems
                                .horizontal_protection
                                .display_name()
                                .to_string()
                        },
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.horizontal_protection,
                            HorizontalProtectionStyle::Auto,
                            "Auto (Dimensionless Match)",
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.horizontal_protection,
                            HorizontalProtectionStyle::ProtectedTurtlebackCarapace,
                            HorizontalProtectionStyle::ProtectedTurtlebackCarapace.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.horizontal_protection,
                            HorizontalProtectionStyle::LowTurtlebackSlopedCitadel,
                            HorizontalProtectionStyle::LowTurtlebackSlopedCitadel.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.horizontal_protection,
                            HorizontalProtectionStyle::MultiLayerBombDecappingSandwich,
                            HorizontalProtectionStyle::MultiLayerBombDecappingSandwich
                                .display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.horizontal_protection,
                            HorizontalProtectionStyle::MonolithicCitadelBoxDeck,
                            HorizontalProtectionStyle::MonolithicCitadelBoxDeck.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.horizontal_protection,
                            HorizontalProtectionStyle::ArmouredFlightDeckHangarBox,
                            HorizontalProtectionStyle::ArmouredFlightDeckHangarBox.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.workbench.design.draft.systems.horizontal_protection,
                            HorizontalProtectionStyle::LightSplinterWeatherDeck,
                            HorizontalProtectionStyle::LightSplinterWeatherDeck.display_name(),
                        );
                    });
                ui.end_row();
            });
    }

    fn page_spaces(&mut self, ui: &mut egui::Ui) {
        let result = self.workbench.design.result();
        let ship = &result.ship;
        if let Some(bench) = get_preset_benchmark(&ship.name) {
            ui.heading(format!(
                "Historical Subsystem Weight & Stability Benchmark ({})",
                bench.name
            ));
            ui.label(format!("Source: {}", bench.source));
            let sim_hull = ship.engine.hull_weight;
            let sim_armour = ship.armour.weight
                + ship
                    .guns
                    .batteries
                    .iter()
                    .map(|b| b.arm_weight)
                    .sum::<f64>();
            let sim_mach = ship.engine.engine_weight;
            let sim_arm = ship.guns.borne_weight + ship.guns.magazine + ship.weapons.weight;
            let sim_bunker = ship.engine.bunker;
            let sim_gm = ship.metacentre;

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
                            ui.label(crate::i18n::latin(ui, "n/a (splinter)"));
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
                        ship.hull.displacement,
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

        crate::theme::heading(ui, "Internal Space Volume Budget");
        let sp = calculate_internal_spaces(ship);
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
                ui.label(crate::i18n::latin(ui, "Room ratio (Hull / Required)"));
                ui.label(format!("{:.3}", sp.room_hull_improved));
                ui.end_row();
            });

        ui.separator();
        crate::theme::heading(ui, "Multi-Deck General Arrangement Optimizer (GA)");
        let previous_target = self.target_cg;
        ui.horizontal(|ui| {
            drag_f64(
                ui,
                "Target vertical CG (fraction of depth)",
                &mut self.target_cg,
                0.01,
                0.20..=0.80,
            );
            if self.target_cg != previous_target {
                self.workbench.clear_placement();
            }
            if ui
                .button(crate::i18n::latin(ui, "Run arrangement optimizer"))
                .clicked()
            {
                self.execute(Command::Arrange(self.target_cg));
            }
        });

        if let Some(p) = self.workbench.placement() {
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
        use crate::theme;
        let result = self.workbench.design.result();
        let ship = &result.ship;
        ui.horizontal(|ui| {
            let title = egui::RichText::new(&ship.name).size(24.0).strong();
            ui.label(if self.serif_titles {
                title.family(egui::FontFamily::Name("serif".into()))
            } else {
                title
            });
            ui.label(
                egui::RichText::new(format!("{} · {}", ship.country, ship.ship_type))
                    .color(theme::MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ship.converged {
                    theme::pill(ui, self.tr("● Converged", "● 重量已收敛"), theme::GOOD);
                } else {
                    theme::pill(
                        ui,
                        self.tr("▲ Weights not converged", "▲ 重量未收敛"),
                        theme::WARN,
                    );
                }
            });
        });
        ui.add_space(6.0);

        let m = self.metric_units;
        let disp = if m {
            format!("{:.0} t", ship.hull.displacement * 1.01605)
        } else {
            format!("{:.0} t", ship.hull.displacement)
        };
        let full = if m {
            format!("{:.0} t", ship.disp_max_load * 1.01605)
        } else {
            format!("{:.0} t", ship.disp_max_load)
        };
        let speed = if m {
            format!("{:.1} km/h", ship.engine.speed_max * 1.852)
        } else {
            format!("{:.1} kt", ship.engine.speed_max)
        };
        let broadside = if m {
            format!("{:.0} kg", ship.guns.broadside * 0.453592)
        } else {
            format!("{:.0} lb", ship.guns.broadside)
        };
        let gm = if m {
            format!("{:.2} m", ship.metacentre * 0.3048)
        } else {
            format!("{:.1} ft", ship.metacentre)
        };
        let stab = ship.stability_adj;
        let stab_color = if stab >= 1.0 {
            theme::GOOD
        } else if stab >= 0.9 {
            theme::WARN
        } else {
            theme::BAD
        };
        ui.horizontal_wrapped(|ui| {
            theme::metric(ui, self.tr("Normal disp.", "标准排水量"), disp, theme::TEXT);
            theme::metric(ui, self.tr("Full load", "满载排水量"), full, theme::TEXT);
            theme::metric(ui, self.tr("Max speed", "最大航速"), speed, theme::ACCENT);
            theme::metric(
                ui,
                self.tr("Power", "轴马力"),
                format!("{:.0} hp", ship.engine.hp_max),
                theme::ACCENT,
            );
            theme::metric(
                ui,
                self.tr("Broadside", "侧舷齐射量"),
                broadside,
                theme::W_ARMAMENT,
            );
            theme::metric(ui, self.tr("Metacentre", "初稳心高 (GM)"), gm, stab_color);
            theme::metric(
                ui,
                self.tr("Stability", "稳性指数"),
                format!("{stab:.2}"),
                stab_color,
            );
            theme::metric(
                ui,
                self.tr("Seakeeping", "适航耐波性"),
                format!("{:.2}", ship.seakeeping),
                theme::TEXT,
            );
        });
        ui.add_space(10.0);
        weight_bar(ui, ship, m);
    }

    fn page_blueprint(&mut self, ui: &mut egui::Ui) {
        let result = self.workbench.design.result();
        let ship = &result.ship;
        crate::theme::heading(ui, "Vessel Blueprint Schematic");
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.show_arcs,
                crate::i18n::latin(ui, "Show 290° Turret Firing Arcs"),
            );
            ui.separator();
            let dim_lbl = if self.metric_units {
                format!(
                    "Dimensions: {:.1} m × {:.1} m × {:.1} m",
                    ship.hull.length * 0.3048,
                    ship.hull.beam * 0.3048,
                    ship.hull.draught * 0.3048
                )
            } else {
                format!(
                    "Dimensions: {:.0} ft × {:.0} ft × {:.1} ft",
                    ship.hull.length, ship.hull.beam, ship.hull.draught
                )
            };
            ui.label(dim_lbl);
        });
        ui.separator();

        let available_width = ui.available_width().max(600.0);
        let height = 500.0;
        let (response, painter) =
            ui.allocate_painter(egui::vec2(available_width, height), egui::Sense::hover());
        let rect = response.rect;

        // Background canvas
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 19, 30));

        let l = ship.hull.length.max(10.0) as f32;
        let b = ship.hull.beam.max(5.0) as f32;
        let pad_x = 50.0_f32;
        let draw_w = (rect.width() - pad_x * 2.0).max(100.0);
        let x0 = rect.left() + pad_x;
        let x1 = x0 + draw_w;

        let fc = ship.free.fc_length.clamp(0.0, 1.0) as f32;
        let fd = ship.free.fd_length.clamp(0.0, (1.0 - fc as f64).max(0.0)) as f32;
        let qd = ship
            .free
            .qd_length
            .clamp(0.0, (1.0 - (fc + fd) as f64).max(0.0)) as f32;
        let ad = (1.0_f32 - fc - fd - qd).max(0.0);

        let x_at = |p: f32| x0 + p * draw_w;

        // Profile view baseline
        let y_wl = rect.top() + 70.0_f32;
        let y_keel = y_wl + 20.0_f32;
        let fb_scale = 1.0_f32;

        // Profile Deck heights
        let y_fc_for = y_wl - ((ship.free.free_fc_for as f32) * fb_scale).clamp(8.0, 36.0);
        let y_fc_aft = y_wl - ((ship.free.free_fc_aft as f32) * fb_scale).clamp(8.0, 36.0);
        let y_fd_for = y_wl - ((ship.free.free_fd_for as f32) * fb_scale).clamp(6.0, 30.0);
        let y_fd_aft = y_wl - ((ship.free.free_fd_aft as f32) * fb_scale).clamp(6.0, 30.0);
        let y_ad_for = y_wl - ((ship.free.free_ad_for as f32) * fb_scale).clamp(4.0, 26.0);
        let y_ad_aft = y_wl - ((ship.free.free_ad_aft as f32) * fb_scale).clamp(4.0, 26.0);
        let y_qd_for = y_wl - ((ship.free.free_qd_for as f32) * fb_scale).clamp(4.0, 26.0);
        let y_qd_aft = y_wl - ((ship.free.free_qd_aft as f32) * fb_scale).clamp(4.0, 26.0);

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

        let sys_eval_bp = &result.systems;
        let layout = &result.blueprint;
        let is_carrier = layout.is_carrier;
        let fd_y = y_fc_for - 12.0;
        let g1 = &ship.guns.batteries[0];
        let g2 = &ship.guns.batteries[1];
        let g1_mounts = &layout.batteries[0];
        let g2_mounts = &layout.batteries[1];
        let cit_x0 = layout.citadel[0] as f32;
        let cit_x1 = layout.citadel[1] as f32;
        let cit_center = (cit_x0 + cit_x1) * 0.5;
        let cit_px0 = x_at(cit_x0);
        let cit_px1 = x_at(cit_x1);

        // Citadel enclosure box
        let cit_rect = egui::Rect::from_min_max(
            egui::pos2(cit_px0, y_wl - 16.0),
            egui::pos2(cit_px1, y_keel),
        );
        painter.rect_filled(
            cit_rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(155, 123, 234, 16),
        );
        painter.rect_stroke(
            cit_rect,
            0.0,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(155, 123, 234, 85),
            ),
            egui::StrokeKind::Outside,
        );

        // Shared concept geometry: both renderers project the same parts.
        let super_deck_y = if is_carrier { y_fc_for - 8.0 } else { y_fd_for };
        if is_carrier {
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x0 + 4.0, super_deck_y),
                    egui::pos2(x1 - 4.0, super_deck_y + 8.0),
                ),
                1.0,
                egui::Color32::from_rgb(37, 77, 72),
            );
        }
        for part in &layout.superstructure.parts {
            let height = part.height as f32 * draw_w;
            let color = if part.kind == springsharp::superstructure::PartKind::Funnel {
                egui::Color32::from_rgb(104, 125, 145)
            } else {
                egui::Color32::from_rgb(138, 164, 191)
            };
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x_at(part.x as f32), super_deck_y - height),
                    egui::pos2(x_at((part.x + part.length) as f32), super_deck_y),
                ),
                2.0,
                color,
            );
        }
        let dir_x = x_at(((layout.bridge[0] + layout.bridge[1]) * 0.5) as f32);
        let base_deck_y = super_deck_y - layout.superstructure.parts[0].height as f32 * draw_w;

        for line in &layout.superstructure.mast_lines {
            painter.line_segment(
                [
                    egui::pos2(
                        x_at(line[0][0] as f32),
                        super_deck_y - line[0][1] as f32 * draw_w,
                    ),
                    egui::pos2(
                        x_at(line[1][0] as f32),
                        super_deck_y - line[1][1] as f32 * draw_w,
                    ),
                ],
                egui::Stroke::new(1.2, egui::Color32::LIGHT_GRAY),
            );
        }
        // Fire-control director & radar
        let sys_eval = sys_eval_bp;
        let dir_top_y = super_deck_y - layout.superstructure.mast_tip[1] as f32 * draw_w;
        let rf_arm = (sys_eval.optical_baseline_m as f32 * 0.75).clamp(3.0, 12.0);
        painter.line_segment(
            [egui::pos2(dir_x, base_deck_y), egui::pos2(dir_x, dir_top_y)],
            egui::Stroke::new(1.5, egui::Color32::WHITE),
        );
        painter.line_segment(
            [
                egui::pos2(dir_x - rf_arm, dir_top_y),
                egui::pos2(dir_x + rf_arm, dir_top_y),
            ],
            egui::Stroke::new(2.2, egui::Color32::from_rgb(95, 211, 141)),
        );
        painter.circle_filled(
            egui::pos2(dir_x, dir_top_y),
            2.0,
            egui::Color32::from_rgb(95, 211, 141),
        );

        if sys_eval.radar_blind_fire_capability > 0.0 {
            let radar_x = dir_x + 7.0;
            let radar_top_y = base_deck_y - 12.0;
            painter.line_segment(
                [
                    egui::pos2(radar_x, base_deck_y),
                    egui::pos2(radar_x, radar_top_y),
                ],
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
            painter.line_segment(
                [
                    egui::pos2(radar_x - 3.5, radar_top_y),
                    egui::pos2(radar_x + 3.5, radar_top_y),
                ],
                egui::Stroke::new(1.8, egui::Color32::from_rgb(88, 166, 255)),
            );
        }

        // Main belt armour anchored over citadel center
        if ship.armour.belt_main > 0.0 && ship.armour.belt_main_length > 0.0 {
            let bw = (((ship.armour.belt_main_length as f32) / l) * draw_w).clamp(10.0, draw_w);
            let bx = (x_at(cit_center) - bw * 0.5).clamp(x0 + 4.0, x1 - bw - 4.0);
            let bh = ((ship.armour.belt_main_height as f32) * fb_scale).clamp(6.0, 18.0);
            let by = y_wl - bh * 0.6;
            let belt_rect =
                egui::Rect::from_min_max(egui::pos2(bx, by), egui::pos2(bx + bw, by + bh));
            painter.rect_filled(
                belt_rect,
                1.0,
                egui::Color32::from_rgba_unmultiplied(155, 123, 234, 125),
            );
            painter.rect_stroke(
                belt_rect,
                1.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(155, 123, 234)),
                egui::StrokeKind::Outside,
            );

            // Upper belt
            if ship.armour.belt_upper > 0.0 && ship.armour.belt_upper_length > 0.0 {
                let ub_w =
                    (((ship.armour.belt_upper_length as f32) / l) * draw_w).clamp(10.0, draw_w);
                let ub_x = (x_at(cit_center) - ub_w * 0.5).clamp(x0 + 4.0, x1 - ub_w - 4.0);
                let ub_h = ((ship.armour.belt_upper_height as f32) * fb_scale).clamp(4.0, 12.0);
                let ub_y = by - ub_h;
                let ub_rect =
                    egui::Rect::from_min_max(egui::pos2(ub_x, ub_y), egui::pos2(ub_x + ub_w, by));
                painter.rect_filled(
                    ub_rect,
                    0.5,
                    egui::Color32::from_rgba_unmultiplied(192, 132, 252, 95),
                );
            }

            // Ends belt
            if ship.armour.belt_ends > 0.0 && ship.armour.belt_ends_length > 0.0 {
                let eb_w =
                    (((ship.armour.belt_ends_length as f32) / l) * draw_w * 0.5).clamp(6.0, 60.0);
                let eb_h = bh * 0.75;
                let eb_y = y_wl - eb_h * 0.5;
                let ef_x0 = (bx - eb_w).max(x0 + 2.0);
                let ef_rect =
                    egui::Rect::from_min_max(egui::pos2(ef_x0, eb_y), egui::pos2(bx, eb_y + eb_h));
                painter.rect_filled(
                    ef_rect,
                    0.5,
                    egui::Color32::from_rgba_unmultiplied(147, 129, 255, 60),
                );
                let ea_x1 = (bx + bw + eb_w).min(x1 - 2.0);
                let ea_rect = egui::Rect::from_min_max(
                    egui::pos2(bx + bw, eb_y),
                    egui::pos2(ea_x1, eb_y + eb_h),
                );
                painter.rect_filled(
                    ea_rect,
                    0.5,
                    egui::Color32::from_rgba_unmultiplied(147, 129, 255, 60),
                );
            }
        }

        // Deck armour coverage (Flat, Turtleback sloped, Double deck, Weather deck)
        let d_cov = ship.armour.deck_coverage;
        if ship.armour.deck > 0.0 {
            let d_y = y_wl - 2.0;
            if d_cov == 0 || d_cov == 4 || d_cov == 5 || d_cov == 6 {
                let line_y = if d_cov == 6 { y_wl - 12.0 } else { d_y };
                let (xs, xe) = if d_cov == 4 {
                    (x_at(0.38), x_at(0.62))
                } else {
                    (cit_px0, cit_px1)
                };
                painter.line_segment(
                    [egui::pos2(xs, line_y), egui::pos2(xe, line_y)],
                    egui::Stroke::new(1.8, egui::Color32::from_rgb(56, 239, 125)),
                );
            } else if d_cov == 1 || d_cov == 2 {
                painter.line_segment(
                    [
                        egui::pos2(cit_px0, d_y - 2.0),
                        egui::pos2(cit_px1, d_y - 2.0),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(56, 239, 125)),
                );
                painter.line_segment(
                    [
                        egui::pos2(cit_px0, d_y + 5.0),
                        egui::pos2(cit_px1, d_y + 5.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(230, 162, 60)),
                );
            } else if d_cov == 3 {
                painter.line_segment(
                    [
                        egui::pos2(cit_px0, d_y - 7.0),
                        egui::pos2(cit_px1, d_y - 7.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(56, 239, 125)),
                );
                painter.line_segment(
                    [egui::pos2(cit_px0, d_y), egui::pos2(cit_px1, d_y)],
                    egui::Stroke::new(1.8, egui::Color32::from_rgb(56, 239, 125)),
                );
            }
        }

        let deck_y_at = |p: f32| -> f32 {
            if p <= fc {
                y_fc_for + (y_fc_aft - y_fc_for) * if fc > 0.0 { p / fc } else { 0.0 }
            } else if p <= fc + fd {
                y_fd_for + (y_fd_aft - y_fd_for) * if fd > 0.0 { (p - fc) / fd } else { 0.0 }
            } else if p <= 1.0 - qd {
                y_ad_for + (y_ad_aft - y_ad_for) * if ad > 0.0 { (p - fc - fd) / ad } else { 0.0 }
            } else {
                y_qd_for
                    + (y_qd_aft - y_qd_for) * if qd > 0.0 { (p - (1.0 - qd)) / qd } else { 0.0 }
            }
        };

        // Main Battery #1 (turrets vs casemates)
        let g1_cal = g1.calibre as f32;
        let t_rad = (g1_cal * 0.32).clamp(3.5, 7.2);
        for m in g1_mounts {
            let p = m.x as f32;
            let tx = x_at(p);
            let base_deck = if is_carrier {
                if m.side == -1 {
                    fd_y
                } else {
                    fd_y + 4.5
                }
            } else {
                deck_y_at(p)
            };
            let b_len = (g1_cal * 0.65).clamp(6.0, 12.0);

            if m.is_casemate {
                let case_y = base_deck + 4.5;
                let c_rect = egui::Rect::from_min_max(
                    egui::pos2(tx - 3.5, case_y - 2.5),
                    egui::pos2(tx + 3.5, case_y + 2.5),
                );
                painter.rect_filled(c_rect, 0.5, egui::Color32::from_rgb(20, 34, 48));
                painter.rect_stroke(
                    c_rect,
                    0.5,
                    egui::Stroke::new(0.8, egui::Color32::from_rgb(125, 140, 153)),
                    egui::StrokeKind::Outside,
                );
                painter.circle_filled(
                    egui::pos2(tx, case_y),
                    1.6,
                    egui::Color32::from_rgb(160, 179, 198),
                );
                let bx2 = if m.fwd { tx - b_len } else { tx + b_len };
                painter.line_segment(
                    [egui::pos2(tx, case_y), egui::pos2(bx2, case_y)],
                    egui::Stroke::new(1.3, egui::Color32::WHITE),
                );
            } else {
                let super_step = (m.tier as f32) * (t_rad * 1.35 + 2.0);
                let ty = base_deck - super_step;
                let th = t_rad * 1.25;
                if m.tier > 0 {
                    let barb_rect = egui::Rect::from_min_max(
                        egui::pos2(tx - t_rad * 0.72, ty),
                        egui::pos2(tx + t_rad * 0.72, base_deck),
                    );
                    painter.rect_filled(barb_rect, 0.5, egui::Color32::from_rgb(160, 82, 98));
                }
                let t_rect = egui::Rect::from_min_max(
                    egui::pos2(tx - t_rad, ty - th),
                    egui::pos2(tx + t_rad, ty),
                );
                painter.rect_filled(t_rect, 1.0, egui::Color32::from_rgb(224, 93, 111));
                let barrel_x2 = if m.fwd {
                    tx - t_rad - b_len
                } else {
                    tx + t_rad + b_len
                };
                painter.line_segment(
                    [
                        egui::pos2(if m.fwd { tx - t_rad } else { tx + t_rad }, ty - th * 0.6),
                        egui::pos2(barrel_x2, ty - th * 0.68),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::WHITE),
                );
            }
        }

        // Secondary Battery #2 (turrets vs casemates)
        if !g2_mounts.is_empty() && g2.calibre >= 2.5 {
            let g2_cal = g2.calibre as f32;
            let r2 = (g2_cal * 0.45).clamp(2.5, 4.5);
            for m in g2_mounts {
                if m.side == 1 {
                    continue;
                }
                let p = m.x as f32;
                let tx = x_at(p);
                let base_deck = if is_carrier { fd_y + 4.0 } else { deck_y_at(p) };
                let b_len = (g2_cal * 0.9).clamp(4.5, 8.5);

                if m.is_casemate {
                    let case_y = base_deck + 4.5;
                    let c_rect = egui::Rect::from_min_max(
                        egui::pos2(tx - 3.0, case_y - 2.2),
                        egui::pos2(tx + 3.0, case_y + 2.2),
                    );
                    painter.rect_filled(c_rect, 0.5, egui::Color32::from_rgb(20, 34, 48));
                    painter.rect_stroke(
                        c_rect,
                        0.5,
                        egui::Stroke::new(0.8, egui::Color32::from_rgb(125, 140, 153)),
                        egui::StrokeKind::Outside,
                    );
                    painter.circle_filled(
                        egui::pos2(tx, case_y),
                        1.4,
                        egui::Color32::from_rgb(160, 179, 198),
                    );
                    let bx2 = if m.fwd { tx - b_len } else { tx + b_len };
                    painter.line_segment(
                        [egui::pos2(tx, case_y), egui::pos2(bx2, case_y)],
                        egui::Stroke::new(1.1, egui::Color32::WHITE),
                    );
                } else {
                    let super_step = (m.tier as f32) * (r2 * 1.6 + 2.5);
                    let ty = base_deck - super_step;
                    let th = r2 * 1.2;
                    if m.tier > 0 {
                        let barb_rect = egui::Rect::from_min_max(
                            egui::pos2(tx - r2 * 0.7, ty),
                            egui::pos2(tx + r2 * 0.7, base_deck),
                        );
                        painter.rect_filled(barb_rect, 0.5, egui::Color32::from_rgb(180, 150, 70));
                    }
                    let t_rect = egui::Rect::from_min_max(
                        egui::pos2(tx - r2, ty - th),
                        egui::pos2(tx + r2, ty),
                    );
                    painter.rect_filled(t_rect, 1.0, egui::Color32::from_rgb(214, 181, 79));
                    let barrel_x2 = if m.fwd {
                        tx - r2 - b_len
                    } else {
                        tx + r2 + b_len
                    };
                    painter.line_segment(
                        [
                            egui::pos2(if m.fwd { tx - r2 } else { tx + r2 }, ty - th * 0.6),
                            egui::pos2(barrel_x2, ty - th * 0.75),
                        ],
                        egui::Stroke::new(1.2, egui::Color32::WHITE),
                    );
                }
            }
        }

        // Plan View
        let y_cl = rect.top() + 225.0;
        let hb = ((b / l) * draw_w * 0.75).clamp(8.0, 32.0);

        painter.line_segment(
            [egui::pos2(x0 - 15.0, y_cl), egui::pos2(x1 + 15.0, y_cl)],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            ),
        );

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
                egui::pos2(x0 + 4.0, y_cl - hb * 0.95),
                egui::pos2(x1 - 4.0, y_cl + hb * 0.95),
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
                [
                    egui::pos2(x0 + 12.0, y_cl + hb * 0.15),
                    egui::pos2(x1 - 12.0, y_cl + hb * 0.15),
                ],
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 70),
                ),
            );
        }
        for part in &layout.superstructure.parts {
            let color = if part.kind == springsharp::superstructure::PartKind::Funnel {
                egui::Color32::from_rgb(73, 93, 112)
            } else {
                egui::Color32::from_rgb(95, 116, 137)
            };
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(
                        x_at(part.x as f32),
                        y_cl + (part.y - part.width / 2.0) as f32 * hb * 2.0,
                    ),
                    egui::pos2(
                        x_at((part.x + part.length) as f32),
                        y_cl + (part.y + part.width / 2.0) as f32 * hb * 2.0,
                    ),
                ),
                2.0,
                color,
            );
        }

        // Torpedo tubes in plan view
        let torp_count = ship.weapons.torpedo1.number + ship.weapons.torpedo2.number;
        if torp_count > 0 && !is_carrier {
            let t_positions = [x0 + draw_w * 0.54, x0 + draw_w * 0.63];
            for tx in t_positions {
                let center = egui::pos2(tx, y_cl);
                painter.circle_filled(center, 4.0, egui::Color32::from_rgb(56, 239, 125));
                painter.circle_stroke(
                    center,
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(17, 153, 142)),
                );
                painter.line_segment(
                    [
                        egui::pos2(tx - 3.5, y_cl - 2.5),
                        egui::pos2(tx + 3.5, y_cl - 2.5),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::WHITE),
                );
                painter.line_segment(
                    [egui::pos2(tx - 3.5, y_cl), egui::pos2(tx + 3.5, y_cl)],
                    egui::Stroke::new(1.2, egui::Color32::WHITE),
                );
                painter.line_segment(
                    [
                        egui::pos2(tx - 3.5, y_cl + 2.5),
                        egui::pos2(tx + 3.5, y_cl + 2.5),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::WHITE),
                );
            }
        }

        // Plan turrets & casemates
        for m in g1_mounts {
            let p = m.x as f32;
            let tx = x_at(p);
            let b_len = (g1_cal * 0.65).clamp(6.0, 11.0);
            if m.is_casemate {
                let ty = y_cl + (if m.side == -1 { -hb * 0.94 } else { hb * 0.94 });
                let c_rect = egui::Rect::from_min_max(
                    egui::pos2(tx - 3.0, ty - 2.2),
                    egui::pos2(tx + 3.0, ty + 2.2),
                );
                painter.rect_filled(c_rect, 0.5, egui::Color32::from_rgb(20, 34, 48));
                painter.rect_stroke(
                    c_rect,
                    0.5,
                    egui::Stroke::new(0.8, egui::Color32::from_rgb(125, 140, 153)),
                    egui::StrokeKind::Outside,
                );
                let dy = if m.side == -1 {
                    -b_len * 0.9
                } else {
                    b_len * 0.9
                };
                let dx = if m.fwd { -b_len * 0.4 } else { b_len * 0.4 };
                painter.line_segment(
                    [egui::pos2(tx, ty), egui::pos2(tx + dx, ty + dy)],
                    egui::Stroke::new(1.2, egui::Color32::WHITE),
                );
            } else {
                let ty = if is_carrier {
                    if m.side == -1 {
                        y_cl - hb * 0.78
                    } else {
                        y_cl + hb * 0.82 * (m.side.max(1) as f32)
                    }
                } else if m.side != 0 {
                    y_cl + (m.side as f32) * hb * 0.58
                } else {
                    y_cl
                };
                let center = egui::pos2(tx, ty);
                painter.circle_filled(center, t_rad, egui::Color32::from_rgb(224, 93, 111));
                painter.circle_stroke(center, t_rad, egui::Stroke::new(1.0, egui::Color32::WHITE));
                let n_bar = m.bpm.clamp(1, 4);
                let spread = ((n_bar - 1) as f32 * 1.8).min(t_rad * 1.15);
                for k in 0..n_bar {
                    let dy = if n_bar == 1 {
                        0.0
                    } else {
                        -spread * 0.5 + (k as f32) * (spread / (n_bar - 1) as f32)
                    };
                    let x_start = if m.fwd { tx - t_rad } else { tx + t_rad };
                    let x_end = if m.fwd {
                        tx - t_rad - b_len
                    } else {
                        tx + t_rad + b_len
                    };
                    painter.line_segment(
                        [egui::pos2(x_start, ty + dy), egui::pos2(x_end, ty + dy)],
                        egui::Stroke::new(1.3, egui::Color32::WHITE),
                    );
                }

                if self.show_arcs {
                    let arc_r = (hb * 1.5).clamp(18.0, 40.0);
                    let arc_pts = if m.fwd {
                        vec![
                            center,
                            egui::pos2(tx + arc_r * 0.819_f32, ty - arc_r * 0.573_f32),
                            egui::pos2(tx - arc_r * 0.707_f32, ty - arc_r * 0.707_f32),
                            egui::pos2(tx - arc_r, ty),
                            egui::pos2(tx - arc_r * 0.707_f32, ty + arc_r * 0.707_f32),
                            egui::pos2(tx + arc_r * 0.819_f32, ty + arc_r * 0.573_f32),
                        ]
                    } else {
                        vec![
                            center,
                            egui::pos2(tx - arc_r * 0.819_f32, ty - arc_r * 0.573_f32),
                            egui::pos2(tx + arc_r * 0.707_f32, ty - arc_r * 0.707_f32),
                            egui::pos2(tx + arc_r, ty),
                            egui::pos2(tx + arc_r * 0.707_f32, ty + arc_r * 0.707_f32),
                            egui::pos2(tx - arc_r * 0.819_f32, ty + arc_r * 0.573_f32),
                        ]
                    };
                    painter.add(egui::Shape::convex_polygon(
                        arc_pts,
                        egui::Color32::from_rgba_unmultiplied(224, 93, 111, 28),
                        egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_unmultiplied(224, 93, 111, 90),
                        ),
                    ));
                }
            }
        }

        if !g2_mounts.is_empty() {
            let g2_cal = g2.calibre as f32;
            let r2 = (g2_cal * 0.45).clamp(2.5, 4.2);
            let b_len2 = (g2_cal * 0.85).clamp(4.0, 7.5);
            for m in g2_mounts {
                let p = m.x as f32;
                let tx2 = x_at(p);
                if m.is_casemate {
                    let ty2 = y_cl + (if m.side == -1 { -hb * 0.94 } else { hb * 0.94 });
                    let c_rect = egui::Rect::from_min_max(
                        egui::pos2(tx2 - 2.5, ty2 - 2.0),
                        egui::pos2(tx2 + 2.5, ty2 + 2.0),
                    );
                    painter.rect_filled(c_rect, 0.5, egui::Color32::from_rgb(20, 34, 48));
                    painter.rect_stroke(
                        c_rect,
                        0.5,
                        egui::Stroke::new(0.8, egui::Color32::from_rgb(125, 140, 153)),
                        egui::StrokeKind::Outside,
                    );
                    let dy = if m.side == -1 {
                        -b_len2 * 0.9
                    } else {
                        b_len2 * 0.9
                    };
                    let dx = if m.fwd { -b_len2 * 0.4 } else { b_len2 * 0.4 };
                    painter.line_segment(
                        [egui::pos2(tx2, ty2), egui::pos2(tx2 + dx, ty2 + dy)],
                        egui::Stroke::new(1.1, egui::Color32::WHITE),
                    );
                } else {
                    let ty2 = if is_carrier {
                        y_cl + (if m.side == 0 { -1.0 } else { m.side as f32 }) * hb * 0.96
                    } else if m.side != 0 {
                        y_cl + (m.side as f32) * (hb * 0.62).max(4.0)
                    } else {
                        y_cl
                    };
                    let center2 = egui::pos2(tx2, ty2);
                    painter.circle_filled(center2, r2, egui::Color32::from_rgb(214, 181, 79));
                    painter.circle_stroke(
                        center2,
                        r2,
                        egui::Stroke::new(0.8, egui::Color32::WHITE),
                    );
                    let x_start = if m.fwd { tx2 - r2 } else { tx2 + r2 };
                    let x_end = if m.fwd {
                        tx2 - r2 - b_len2
                    } else {
                        tx2 + r2 + b_len2
                    };
                    painter.line_segment(
                        [egui::pos2(x_start, ty2), egui::pos2(x_end, ty2)],
                        egui::Stroke::new(1.1, egui::Color32::WHITE),
                    );
                }
            }
        }

        // Midship Cross-Section & Internal Topology (y = 310 to 490)
        let sep_y = rect.top() + 305.0;
        painter.line_segment(
            [egui::pos2(x0 - 15.0, sep_y), egui::pos2(x1 + 15.0, sep_y)],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 45),
            ),
        );

        let xc = x0 + 105.0;
        let wb = (b / 80.0 * 50.0).clamp(34.0, 60.0);
        let mid_wl = rect.top() + 415.0;
        let mid_keel = rect.top() + 468.0;
        let mid_deck = rect.top() + 365.0;

        let mid_pts = vec![
            egui::pos2(xc - wb, mid_deck),
            egui::pos2(xc - wb, mid_wl + 15.0),
            egui::pos2(xc - 15.0, mid_keel),
            egui::pos2(xc + 15.0, mid_keel),
            egui::pos2(xc + wb, mid_wl + 15.0),
            egui::pos2(xc + wb, mid_deck),
        ];
        painter.add(egui::Shape::convex_polygon(
            mid_pts,
            egui::Color32::from_rgba_unmultiplied(79, 179, 255, 22),
            egui::Stroke::new(1.5, egui::Color32::from_rgb(79, 179, 255)),
        ));

        // Waterline & Double bottom
        painter.line_segment(
            [
                egui::pos2(xc - wb - 10.0, mid_wl),
                egui::pos2(xc + wb + 10.0, mid_wl),
            ],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(79, 179, 255, 160),
            ),
        );
        painter.line_segment(
            [
                egui::pos2(xc - wb + 12.0, mid_keel - 10.0),
                egui::pos2(xc + wb - 12.0, mid_keel - 10.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(79, 179, 255)),
        );

        // Belt armor on cross-section (with dynamic underwater extension & taper)
        let incline_deg = ship.armour.belts_incline as f32;
        let belt_dx = if incline_deg > 0.0 {
            (incline_deg * 0.45).min(9.0)
        } else {
            0.0
        };
        let pi_sub = sys_eval.features.underwater_belt_submergence_ratio as f32;
        let pi_taper = sys_eval.features.belt_lower_taper_ratio as f32;
        let sub_span = mid_keel - mid_wl;
        let y_sub = mid_wl + (sub_span * pi_sub).clamp(10.0, 42.0);
        let main_w = (ship.armour.belt_main as f32 * 0.35).clamp(2.0, 6.5);
        let lower_w = (main_w * pi_taper).clamp(1.5, main_w);

        if ship.armour.belt_main > 0.0 {
            // Port belt
            painter.line_segment(
                [
                    egui::pos2(xc - wb, mid_deck + 15.0),
                    egui::pos2(xc - wb + belt_dx * 0.5, mid_wl),
                ],
                egui::Stroke::new(main_w, egui::Color32::from_rgb(155, 123, 234)),
            );
            painter.line_segment(
                [
                    egui::pos2(xc - wb + belt_dx * 0.5, mid_wl),
                    egui::pos2(xc - wb + belt_dx, y_sub),
                ],
                egui::Stroke::new(lower_w, egui::Color32::from_rgb(180, 145, 245)),
            );

            // Starboard belt
            painter.line_segment(
                [
                    egui::pos2(xc + wb, mid_deck + 15.0),
                    egui::pos2(xc + wb - belt_dx * 0.5, mid_wl),
                ],
                egui::Stroke::new(main_w, egui::Color32::from_rgb(155, 123, 234)),
            );
            painter.line_segment(
                [
                    egui::pos2(xc + wb - belt_dx * 0.5, mid_wl),
                    egui::pos2(xc + wb - belt_dx, y_sub),
                ],
                egui::Stroke::new(lower_w, egui::Color32::from_rgb(180, 145, 245)),
            );

            // Joint closure to torpedo bulkhead based on resolved_belt_tds_junction
            let junc = sys_eval.resolved_belt_tds_junction;
            match junc {
                BeltTdsJunction::IntegralTaperedHoldingBulkhead => {
                    painter.line_segment(
                        [
                            egui::pos2(xc - wb + belt_dx, y_sub),
                            egui::pos2(xc - wb + 18.0, mid_keel - 10.0),
                        ],
                        egui::Stroke::new(1.8, egui::Color32::from_rgb(155, 123, 234)),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(xc + wb - belt_dx, y_sub),
                            egui::pos2(xc + wb - 18.0, mid_keel - 10.0),
                        ],
                        egui::Stroke::new(1.8, egui::Color32::from_rgb(155, 123, 234)),
                    );
                }
                BeltTdsJunction::TurtlebackSlopeKneeJoint => {
                    painter.line_segment(
                        [
                            egui::pos2(xc - wb + belt_dx, y_sub),
                            egui::pos2(xc - wb + belt_dx, mid_keel - 10.0),
                        ],
                        egui::Stroke::new(1.5, egui::Color32::from_rgb(224, 134, 79)),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(xc + wb - belt_dx, y_sub),
                            egui::pos2(xc + wb - belt_dx, mid_keel - 10.0),
                        ],
                        egui::Stroke::new(1.5, egui::Color32::from_rgb(224, 134, 79)),
                    );
                }
                BeltTdsJunction::SpacedAbsorberShelfJoint => {
                    painter.line_segment(
                        [
                            egui::pos2(xc - wb + belt_dx, y_sub),
                            egui::pos2(xc - wb + 13.0, y_sub),
                        ],
                        egui::Stroke::new(1.6, egui::Color32::from_rgb(224, 134, 79)),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(xc + wb - belt_dx, y_sub),
                            egui::pos2(xc + wb - 13.0, y_sub),
                        ],
                        egui::Stroke::new(1.6, egui::Color32::from_rgb(224, 134, 79)),
                    );
                }
                BeltTdsJunction::SteppedShelfCitadel => {
                    painter.line_segment(
                        [
                            egui::pos2(xc - wb + belt_dx, y_sub),
                            egui::pos2(xc - wb + 16.0, y_sub),
                        ],
                        egui::Stroke::new(1.6, egui::Color32::from_rgb(56, 239, 125)),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(xc + wb - belt_dx, y_sub),
                            egui::pos2(xc + wb - 16.0, y_sub),
                        ],
                        egui::Stroke::new(1.6, egui::Color32::from_rgb(56, 239, 125)),
                    );
                }
                _ => {}
            }
        }

        // Horizontal / Deck Protection Topology on cross-section
        match sys_eval.resolved_horizontal_protection {
            HorizontalProtectionStyle::ProtectedTurtlebackCarapace => {
                painter.line_segment(
                    [egui::pos2(xc - wb, mid_deck), egui::pos2(xc + wb, mid_deck)],
                    egui::Stroke::new(1.1, egui::Color32::from_rgb(56, 239, 125)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 18.0, mid_wl - 8.0),
                        egui::pos2(xc + wb - 18.0, mid_wl - 8.0),
                    ],
                    egui::Stroke::new(2.2, egui::Color32::from_rgb(56, 239, 125)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 18.0, mid_wl - 8.0),
                        egui::pos2(xc - wb, mid_wl + 14.0),
                    ],
                    egui::Stroke::new(2.2, egui::Color32::from_rgb(230, 162, 60)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 18.0, mid_wl - 8.0),
                        egui::pos2(xc + wb, mid_wl + 14.0),
                    ],
                    egui::Stroke::new(2.2, egui::Color32::from_rgb(230, 162, 60)),
                );
            }
            HorizontalProtectionStyle::LowTurtlebackSlopedCitadel => {
                painter.line_segment(
                    [
                        egui::pos2(xc - wb, mid_deck + 12.0),
                        egui::pos2(xc + wb, mid_deck + 12.0),
                    ],
                    egui::Stroke::new(1.3, egui::Color32::from_rgb(124, 227, 163)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 16.0, mid_wl - 10.0),
                        egui::pos2(xc + wb - 16.0, mid_wl - 10.0),
                    ],
                    egui::Stroke::new(2.2, egui::Color32::from_rgb(56, 239, 125)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 16.0, mid_wl - 10.0),
                        egui::pos2(xc - wb + belt_dx, y_sub),
                    ],
                    egui::Stroke::new(2.2, egui::Color32::from_rgb(230, 162, 60)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 16.0, mid_wl - 10.0),
                        egui::pos2(xc + wb - belt_dx, y_sub),
                    ],
                    egui::Stroke::new(2.2, egui::Color32::from_rgb(230, 162, 60)),
                );
            }
            HorizontalProtectionStyle::MultiLayerBombDecappingSandwich => {
                // 1. Bomb / cap-stripping weather deck
                painter.line_segment(
                    [egui::pos2(xc - wb, mid_deck), egui::pos2(xc + wb, mid_deck)],
                    egui::Stroke::new(1.4, egui::Color32::from_rgb(124, 227, 163)),
                );
                // 2. Heavy main ballistic armored deck closing citadel top
                painter.line_segment(
                    [
                        egui::pos2(xc - wb, mid_deck + 15.0),
                        egui::pos2(xc + wb, mid_deck + 15.0),
                    ],
                    egui::Stroke::new(2.8, egui::Color32::from_rgb(56, 239, 125)),
                );
                // 3. Lower splinter catcher deck above vitals
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 16.0, mid_wl - 5.0),
                        egui::pos2(xc + wb - 16.0, mid_wl - 5.0),
                    ],
                    egui::Stroke::new(1.3, egui::Color32::from_rgb(60, 168, 107)),
                );
            }
            HorizontalProtectionStyle::MonolithicCitadelBoxDeck => {
                painter.line_segment(
                    [
                        egui::pos2(xc - wb, mid_deck + 12.0),
                        egui::pos2(xc + wb, mid_deck + 12.0),
                    ],
                    egui::Stroke::new(2.8, egui::Color32::from_rgb(56, 239, 125)),
                );
            }
            HorizontalProtectionStyle::ArmouredFlightDeckHangarBox => {
                painter.line_segment(
                    [
                        egui::pos2(xc - wb - 6.0, mid_deck - 8.0),
                        egui::pos2(xc + wb + 6.0, mid_deck - 8.0),
                    ],
                    egui::Stroke::new(2.6, egui::Color32::from_rgb(56, 239, 125)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb, mid_deck - 8.0),
                        egui::pos2(xc - wb, mid_deck + 15.0),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(79, 179, 255)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb, mid_deck - 8.0),
                        egui::pos2(xc + wb, mid_deck + 15.0),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(79, 179, 255)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb, mid_deck + 15.0),
                        egui::pos2(xc + wb, mid_deck + 15.0),
                    ],
                    egui::Stroke::new(1.8, egui::Color32::from_rgb(56, 239, 125)),
                );
            }
            HorizontalProtectionStyle::LightSplinterWeatherDeck
            | HorizontalProtectionStyle::Auto => {
                painter.line_segment(
                    [egui::pos2(xc - wb, mid_deck), egui::pos2(xc + wb, mid_deck)],
                    egui::Stroke::new(1.3, egui::Color32::from_rgb(56, 239, 125)),
                );
            }
        }

        // TDS internal geometry
        let tds_arch = sys_eval.resolved_tds_architecture;
        match tds_arch {
            TdsArchitecture::PuglieseCylindricalAbsorber => {
                painter.circle_stroke(
                    egui::pos2(xc - wb + 13.0, mid_wl + 14.0),
                    6.5,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(224, 93, 111)),
                );
                painter.circle_stroke(
                    egui::pos2(xc + wb - 13.0, mid_wl + 14.0),
                    6.5,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(224, 93, 111)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 21.0, mid_deck + 10.0),
                        egui::pos2(xc - wb + 21.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(155, 123, 234)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 21.0, mid_deck + 10.0),
                        egui::pos2(xc + wb - 21.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(155, 123, 234)),
                );
            }
            TdsArchitecture::CompoundEboniteDeepTds => {
                let r_p = egui::Rect::from_min_max(
                    egui::pos2(xc - wb + 2.0, mid_deck + 15.0),
                    egui::pos2(xc - wb + 18.0, mid_keel - 10.0),
                );
                let r_s = egui::Rect::from_min_max(
                    egui::pos2(xc + wb - 18.0, mid_deck + 15.0),
                    egui::pos2(xc + wb - 2.0, mid_keel - 10.0),
                );
                painter.rect_filled(
                    r_p,
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(224, 134, 79, 90),
                );
                painter.rect_filled(
                    r_s,
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(224, 134, 79, 90),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 19.0, mid_deck + 10.0),
                        egui::pos2(xc - wb + 19.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.8, egui::Color32::from_rgb(155, 123, 234)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 19.0, mid_deck + 10.0),
                        egui::pos2(xc + wb - 19.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.8, egui::Color32::from_rgb(155, 123, 234)),
                );
            }
            TdsArchitecture::MultiLayerLiquidVoid => {
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 8.0, mid_deck + 15.0),
                        egui::pos2(xc - wb + 8.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(79, 179, 255)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 16.0, mid_deck + 12.0),
                        egui::pos2(xc - wb + 16.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(155, 123, 234)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 8.0, mid_deck + 15.0),
                        egui::pos2(xc + wb - 8.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(79, 179, 255)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 16.0, mid_deck + 12.0),
                        egui::pos2(xc + wb - 16.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(155, 123, 234)),
                );
            }
            TdsArchitecture::ExternalBlisterBulge => {
                let b_pts_p = vec![
                    egui::pos2(xc - wb, mid_wl - 10.0),
                    egui::pos2(xc - wb - 10.0, mid_wl + 15.0),
                    egui::pos2(xc - wb, mid_keel - 12.0),
                ];
                let b_pts_s = vec![
                    egui::pos2(xc + wb, mid_wl - 10.0),
                    egui::pos2(xc + wb + 10.0, mid_wl + 15.0),
                    egui::pos2(xc + wb, mid_keel - 12.0),
                ];
                painter.add(egui::Shape::line(
                    b_pts_p,
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(79, 179, 255)),
                ));
                painter.add(egui::Shape::line(
                    b_pts_s,
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(79, 179, 255)),
                ));
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 12.0, mid_deck + 12.0),
                        egui::pos2(xc - wb + 12.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(155, 123, 234)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 12.0, mid_deck + 12.0),
                        egui::pos2(xc + wb - 12.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(155, 123, 234)),
                );
            }
            TdsArchitecture::CoalBunkerWingBulkhead => {
                painter.line_segment(
                    [
                        egui::pos2(xc - wb + 14.0, mid_deck + 12.0),
                        egui::pos2(xc - wb + 14.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(138, 164, 191)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + wb - 14.0, mid_deck + 12.0),
                        egui::pos2(xc + wb - 14.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(138, 164, 191)),
                );
            }
            TdsArchitecture::UnarmouredDoubleBottom | TdsArchitecture::Auto => {}
        }

        // Transverse subdivision lines
        let sub_arch = sys_eval.resolved_transverse_subdivision;
        match sub_arch {
            TransverseSubdivision::TwinFileCenterline => {
                painter.line_segment(
                    [egui::pos2(xc, mid_deck), egui::pos2(xc, mid_keel)],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(95, 211, 141)),
                );
            }
            TransverseSubdivision::TripleFileCenterAndWings => {
                painter.line_segment(
                    [
                        egui::pos2(xc - 18.0, mid_deck),
                        egui::pos2(xc - 18.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(95, 211, 141)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + 18.0, mid_deck),
                        egui::pos2(xc + 18.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.2, egui::Color32::from_rgb(95, 211, 141)),
                );
            }
            TransverseSubdivision::QuadFileMultiLongitudinal => {
                painter.line_segment(
                    [egui::pos2(xc, mid_deck), egui::pos2(xc, mid_keel)],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(95, 211, 141)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc - 18.0, mid_deck),
                        egui::pos2(xc - 18.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(95, 211, 141)),
                );
                painter.line_segment(
                    [
                        egui::pos2(xc + 18.0, mid_deck),
                        egui::pos2(xc + 18.0, mid_keel - 10.0),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(95, 211, 141)),
                );
            }
            TransverseSubdivision::SingleFileTransverse | TransverseSubdivision::Auto => {}
        }

        // Architecture Summary Card
        let card_rect = egui::Rect::from_min_max(
            egui::pos2(xc + wb + 30.0, mid_deck - 5.0),
            egui::pos2(x1 + 10.0, mid_keel + 5.0),
        );
        painter.rect_filled(
            card_rect,
            3.0,
            egui::Color32::from_rgba_unmultiplied(14, 28, 44, 210),
        );
        painter.rect_stroke(
            card_rect,
            3.0,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 35),
            ),
            egui::StrokeKind::Outside,
        );

        let horiz_str = format!(
            "Deck Protection: {}",
            sys_eval.resolved_horizontal_protection.display_name()
        );
        let sub_str = format!("Subdivision: {}", sub_arch.display_name());
        let tds_str = format!("TDS System: {}", tds_arch.display_name());
        let junc_str = format!(
            "Belt-TDS Joint: {}",
            sys_eval.resolved_belt_tds_junction.display_name()
        );
        let mach_str = format!(
            "Machinery: {}",
            sys_eval.resolved_machinery_arrangement.display_name()
        );
        let arm_str = format!(
            "Belt: {:.1}\" ({:.0}°) · Deck: {:.1}\" (Deck/Belt Π {:.2}) · Sub {:.0}% · TDS {:.0}%",
            ship.armour.belt_main,
            ship.armour.belts_incline,
            ship.armour.deck,
            sys_eval.features.deck_belt_balance_ratio,
            sys_eval.features.underwater_belt_submergence_ratio * 100.0,
            sys_eval.features.tds_depth_ratio * 100.0
        );

        painter.text(
            egui::pos2(card_rect.left() + 10.0, card_rect.top() + 6.0),
            egui::Align2::LEFT_TOP,
            horiz_str,
            egui::FontId::monospace(9.5),
            egui::Color32::from_rgb(56, 239, 125),
        );
        painter.text(
            egui::pos2(card_rect.left() + 10.0, card_rect.top() + 21.0),
            egui::Align2::LEFT_TOP,
            sub_str,
            egui::FontId::monospace(9.5),
            egui::Color32::from_rgb(124, 227, 163),
        );
        painter.text(
            egui::pos2(card_rect.left() + 10.0, card_rect.top() + 36.0),
            egui::Align2::LEFT_TOP,
            tds_str,
            egui::FontId::monospace(9.5),
            egui::Color32::from_rgb(224, 134, 79),
        );
        painter.text(
            egui::pos2(card_rect.left() + 10.0, card_rect.top() + 51.0),
            egui::Align2::LEFT_TOP,
            junc_str,
            egui::FontId::monospace(9.5),
            egui::Color32::from_rgb(180, 145, 245),
        );
        painter.text(
            egui::pos2(card_rect.left() + 10.0, card_rect.top() + 66.0),
            egui::Align2::LEFT_TOP,
            mach_str,
            egui::FontId::monospace(9.5),
            egui::Color32::from_rgb(155, 123, 234),
        );
        painter.text(
            egui::pos2(card_rect.left() + 10.0, card_rect.top() + 81.0),
            egui::Align2::LEFT_TOP,
            arm_str,
            egui::FontId::monospace(9.5),
            egui::Color32::from_rgb(138, 164, 191),
        );

        // Section labels
        painter.text(
            egui::pos2(rect.left() + 10.0, rect.top() + 12.0),
            egui::Align2::LEFT_TOP,
            "PROFILE VIEW",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(138, 164, 191),
        );
        painter.text(
            egui::pos2(rect.left() + 10.0, rect.top() + 145.0),
            egui::Align2::LEFT_TOP,
            "PLAN VIEW",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(138, 164, 191),
        );
        painter.text(
            egui::pos2(rect.left() + 10.0, sep_y + 10.0),
            egui::Align2::LEFT_TOP,
            "MIDSHIP CROSS-SECTION & TDS ARCHITECTURE",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(138, 164, 191),
        );

        ui.separator();
        crate::theme::heading(ui, "Speed & Power Curve");
        self.draw_speed_power_chart(ui, available_width);
        ui.separator();
        crate::theme::heading(ui, "Armor Penetration & Immunity Zone");
        self.draw_immunity_chart(ui, available_width);
    }

    fn draw_speed_power_chart(&self, ui: &mut egui::Ui, available_width: f32) {
        let result = self.workbench.design.result();
        let ship = &result.ship;
        let chart_h = 220.0_f32;
        let (response, painter) =
            ui.allocate_painter(egui::vec2(available_width, chart_h), egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(8, 19, 30));

        let v_top = (ship.engine.speed_max + 2.0).max(20.0).ceil();
        let v_min = 10.0_f64;
        let mut mod_engine = ship.engine.clone();
        let mut orig_engine = ship.engine.clone();
        orig_engine.resistance_model = ResistanceModel::Original;
        orig_engine.power_factor = 1.0;
        orig_engine.original_power_factor = 1.0;
        orig_engine.blend_weight = 1.0;
        orig_engine.cfd_proxy_enabled = false;

        let mut pts = Vec::new();
        let mut max_hp = 1000.0_f64;
        let mut v = v_min;
        while v <= v_top + 1e-6 {
            let hp_mod = mod_engine.speed_hp_calc(v, &ship.hull);
            let hp_orig = orig_engine.speed_hp_calc(v, &ship.hull);
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
        let result = self.workbench.design.result();
        let ship = &result.ship;
        let g1 = &ship.guns.batteries[0];
        if g1.calibre <= 0.0 || g1.barrels <= 0 {
            ui.label(crate::i18n::latin(
                ui,
                "No active main battery guns for exterior ballistics analysis.",
            ));
            return;
        }

        let iz = compute_self_immunity_zone(
            g1,
            ship.armour.belt_main,
            ship.armour.belts_incline,
            ship.armour.deck,
        );

        if iz.curve.is_empty() {
            ui.label(crate::i18n::latin(
                ui,
                "Ballistic trajectory calculation produced no curve points.",
            ));
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

        // Salvo Hit Probability curve (green dashed with right axis ticks)
        let sys_eval = &result.systems;
        let max_prob = 35.0_f32;
        let y_prob = |p: f32| rect.top() + pad_t + (1.0 - (p / max_prob).clamp(0.0, 1.0)) * ph;

        for i in 0..=3 {
            let p_val = (i * 10) as f32;
            let y = y_prob(p_val);
            painter.text(
                egui::pos2(rect.left() + pad_l + pw + 6.0, y),
                egui::Align2::LEFT_CENTER,
                format!("{:.0}%", p_val),
                egui::FontId::monospace(8.5),
                egui::Color32::from_rgb(95, 211, 141),
            );
        }

        let hit_pts: Vec<egui::Pos2> = sys_eval
            .hit_prob_curve
            .iter()
            .map(|(r, p)| egui::pos2(x_at(*r as f32), y_prob(*p as f32)))
            .collect();
        painter.add(egui::Shape::line(
            hit_pts.clone(),
            egui::Stroke::new(2.0, egui::Color32::from_rgb(95, 211, 141)),
        ));
        for pt in hit_pts {
            painter.circle_filled(pt, 2.5, egui::Color32::from_rgb(95, 211, 141));
        }

        // Legend
        painter.text(
            egui::pos2(rect.left() + pad_l + 10.0, rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "■ Belt Pen (red)  ■ Deck Pen (cyan)  ■ Salvo Hit % (green)  ■ Immunity Zone (green band)",
            egui::FontId::monospace(9.5),
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
        use crate::theme;
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new("app_language"), self.text_language));
        self.workbench.sync_edits();
        egui::Panel::left("nav_panel")
            .resizable(true)
            .default_size(236.0)
            .min_size(210.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::same(14))
                    .stroke(egui::Stroke::new(1.0, theme::alpha(theme::LINE, 150))),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.nav(ui));
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ui, |ui| {
                theme::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        if self.workbench.design.is_dirty() {
                            theme::pill(
                                ui,
                                &format!(
                                    "{} v{} {} · {} v{}",
                                    self.tr("● Inputs", "● 参数输入"),
                                    self.workbench.design.revision(),
                                    self.tr("pending", "待计算"),
                                    self.tr("showing", "当前显示"),
                                    self.workbench.design.result().revision,
                                ),
                                theme::WARN,
                            );
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(self.tr("Calculate", "立即计算"))
                                            .strong(),
                                    )
                                    .fill(theme::alpha(theme::WARN, 60)),
                                )
                                .clicked()
                            {
                                self.recalculate();
                            }
                        } else {
                            theme::pill(
                                ui,
                                &format!(
                                    "{} · {} {}",
                                    self.tr("✔ Calculated", "✔ 计算已完成"),
                                    self.tr("revision", "版本"),
                                    self.workbench.design.revision()
                                ),
                                theme::GOOD,
                            );
                        }
                    });
                    ui.add_space(6.0);
                    self.dashboard_header(ui);
                });
                ui.add_space(12.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    theme::card().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        match self.page {
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
                        }
                    });
                });
            });
        if self.workbench.sync_edits() {
            ui.ctx().request_repaint();
        }
    }
}

fn weapon_mount_ui(ui: &mut egui::Ui, title: &str, id_salt: &str, m: &mut WeaponMount) {
    let lang = ui
        .ctx()
        .data(|d| d.get_temp(egui::Id::new("app_language")))
        .unwrap_or(TextLanguage::Germanic);
    let title = match title {
        "Torpedo set 1" => crate::i18n::tr(lang, "Torpedo set 1", "鱼雷发射组 1"),
        "Torpedo set 2" => crate::i18n::tr(lang, "Torpedo set 2", "鱼雷发射组 2"),
        "Mine equipment" => crate::i18n::tr(lang, "Mine equipment", "水雷布设装备"),
        "Depth charges / AS mortar 1" => {
            crate::i18n::tr(lang, "Depth charges / AS mortar 1", "反潜深弹/臼炮组 1")
        }
        "Depth charges / AS mortar 2" => {
            crate::i18n::tr(lang, "Depth charges / AS mortar 2", "反潜深弹/臼炮组 2")
        }
        _ => title,
    };
    egui::CollapsingHeader::new(title)
        .id_salt(id_salt)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                field_label(ui, "Weapon kind:");
                let (torp_txt, mine_txt, dc_txt) = if lang == TextLanguage::Han {
                    ("鱼雷 (Torpedo)", "水雷 (Mine)", "深弹/反潜臼炮 (DC/Mortar)")
                } else if lang == TextLanguage::Italic {
                    (
                        "Torpille",
                        "Mine marine",
                        "Grenade sous-marine / mortier ASM",
                    )
                } else {
                    ("Torpedo", "Mine", "Depth charge / AS mortar")
                };
                let selected_text = match m.kind {
                    WeaponKind::Torpedo => torp_txt,
                    WeaponKind::Mine => mine_txt,
                    WeaponKind::DepthCharge => dc_txt,
                };
                egui::ComboBox::from_id_salt(format!("{id_salt}_kind"))
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut m.kind, WeaponKind::Torpedo, torp_txt);
                        ui.selectable_value(&mut m.kind, WeaponKind::Mine, mine_txt);
                        ui.selectable_value(&mut m.kind, WeaponKind::DepthCharge, dc_txt);
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
            let locations = opts::weapon_locations_lang(m.kind, lang);
            combo_i32(ui, "Location", &mut m.location, locations);
        });
}

/// Fixed-width label column so inputs line up down the page.
fn field_label(ui: &mut egui::Ui, label: &str) {
    let lang = ui
        .ctx()
        .data(|d| d.get_temp(egui::Id::new("app_language")))
        .unwrap_or(TextLanguage::Germanic);
    let display_text = if lang == TextLanguage::Han {
        crate::i18n::translate_field(label)
    } else {
        springsharp::presentation::latin_ui::text(lang, label)
    };
    let width = 230.0_f32.min(ui.available_width() * 0.5);
    ui.add_sized(
        [width, ui.spacing().interact_size.y],
        egui::Label::new(egui::RichText::new(display_text).color(crate::theme::MUTED)).truncate(),
    )
    .on_hover_text(display_text);
}

fn combo_i32(ui: &mut egui::Ui, label: &str, value: &mut i32, options: &[&str]) {
    ui.horizontal(|ui| {
        field_label(ui, label);
        let lang = ui
            .ctx()
            .data(|d| d.get_temp(egui::Id::new("app_language")))
            .unwrap_or(TextLanguage::Germanic);
        let options = localized_options(label, options, lang);
        let curr_idx = (*value).max(0) as usize;
        let selected_text = options
            .get(curr_idx)
            .copied()
            .unwrap_or(crate::i18n::latin(ui, "Other"));
        egui::ComboBox::from_id_salt(ui.id().with(label))
            .selected_text(selected_text)
            .width(220.0)
            .show_ui(ui, |ui| {
                for (idx, &opt) in options.iter().enumerate() {
                    ui.selectable_value(value, idx as i32, opt);
                }
            });
    });
}

fn localized_options<'a>(label: &str, options: &'a [&'a str], lang: TextLanguage) -> &'a [&'a str] {
    match label {
        "Bow shape" => opts::bow_types(lang),
        "Stern shape" => opts::stern_types(lang),
        "Bulkhead type" => opts::bulkhead_types(lang),
        "Deck coverage" => opts::deck_coverage(lang),
        "Gun type" => opts::gun_types(lang),
        "Mount type" => opts::mount_types(lang),
        "Guns/mount G1" | "Guns/mount G2" => opts::guns_in_mount(lang),
        "Layout G1" | "Layout G2" => opts::mount_layouts(lang),
        // Weapon locations already arrive in the selected language from their kind.
        _ => options,
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
        field_label(ui, label);
        ui.add_sized(
            [120.0, ui.spacing().interact_size.y],
            egui::DragValue::new(value).speed(speed).range(range),
        );
    });
}

fn drag_i32(ui: &mut egui::Ui, label: &str, value: &mut i32) {
    ui.horizontal(|ui| {
        field_label(ui, label);
        ui.add_sized(
            [120.0, ui.spacing().interact_size.y],
            egui::DragValue::new(value).speed(1.0),
        );
    });
}

pub fn run(renderer: Option<eframe::Renderer>) -> Result<(), Box<dyn std::error::Error>> {
    let make_options = |renderer| eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 880.0])
            .with_min_inner_size([960.0, 640.0]),
        renderer,
        ..Default::default()
    };

    let make_creator = || {
        let creator: eframe::AppCreator = Box::new(|cc| {
            crate::startup::log("Graphics initialized; installing fonts");
            let missing = crate::fonts::install(&cc.egui_ctx, TextLanguage::Germanic);
            crate::theme::install(&cc.egui_ctx);
            crate::startup::log("Fonts installed; creating workbench");
            let mut app = SpringSharpApp::new();
            crate::startup::log("Workbench created");
            if !missing.is_empty() {
                app.status = format!("Missing bundled fonts: {}", missing.join(", "));
            }
            Ok(Box::new(app))
        });
        creator
    };

    if let Some(renderer) = renderer {
        crate::startup::log(&format!("Starting requested renderer: {renderer:?}"));
        return eframe::run_native("SpringSharp Rust", make_options(renderer), make_creator())
            .map_err(Into::into);
    }
    crate::startup::log("Starting primary renderer: Wgpu");
    match eframe::run_native(
        "SpringSharp Rust",
        make_options(eframe::Renderer::Wgpu),
        make_creator(),
    ) {
        Ok(()) => Ok(()),
        Err(wgpu_err) => {
            crate::startup::log(&format!("Wgpu failed: {wgpu_err}; starting Glow fallback"));
            eframe::run_native(
                "SpringSharp Rust",
                make_options(eframe::Renderer::Glow),
                make_creator(),
            )
            .map_err(|glow_err| {
                format!(
                    "SpringSharp failed to initialize graphics renderer:\n\n- Primary (Wgpu/DirectX): {wgpu_err}\n- Fallback (Glow/OpenGL): {glow_err}"
                )
                .into()
            })
        }
    }
}

/// Segmented, rounded weight-distribution bar with hover details and a legend.
fn weight_bar(ui: &mut egui::Ui, ship: &springsharp::model::ShipModel, metric: bool) {
    use crate::theme;
    let lang = ui
        .ctx()
        .data(|d| d.get_temp(egui::Id::new("app_language")))
        .unwrap_or(TextLanguage::Germanic);
    let total = ship.hull.displacement.max(1.0);
    let (e, g, a, w) = (&ship.engine, &ship.guns, &ship.armour, &ship.weapons);
    let (n_hull, n_arm, n_mach, n_armam, n_mag, n_bunker, n_misc, n_other) =
        if lang == TextLanguage::Han {
            (
                "船体结构",
                "装甲防护",
                "动力机械",
                "武器武备",
                "弹药储备",
                "燃油燃煤",
                "杂项空舱",
                "其余载荷",
            )
        } else if lang == TextLanguage::Italic {
            (
                "Corpus navis",
                "Loricatio",
                "Machinae",
                "Armamenta",
                "Cella munitionum",
                "Combustibile",
                "Cetera",
                "Onus reliquum",
            )
        } else {
            (
                "Hull",
                "Armour",
                "Machinery",
                "Armament",
                "Magazine",
                "Bunker",
                "Miscellaneous",
                "Other load",
            )
        };
    let mut parts = vec![
        (n_hull, e.hull_weight, theme::W_HULL),
        (n_arm, a.weight, theme::W_ARMOUR),
        (n_mach, e.engine_weight, theme::W_MACHINERY),
        (n_armam, g.borne_weight + w.weight, theme::W_ARMAMENT),
        (n_mag, g.magazine, theme::W_MAGAZINE),
        (n_bunker, e.bunker, theme::W_BUNKER),
        (
            n_misc,
            w.misc_weight_vital
                + w.misc_weight_hull
                + w.misc_weight_deck
                + w.misc_weight_above
                + w.void_weight,
            theme::W_MISC,
        ),
    ];
    let used: f64 = parts.iter().map(|p| p.1.max(0.0)).sum();
    if total - used > 0.5 {
        parts.push((n_other, total - used, theme::W_OTHER));
    }
    let scale = if metric { 1.01605 } else { 1.0 };

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(crate::i18n::tr(
                lang,
                "WEIGHT DISTRIBUTION",
                "重量分布与构成",
            ))
            .size(11.0)
            .strong()
            .color(theme::BRASS),
        );
        let subt = if lang == TextLanguage::Han {
            format!("基准: {:.0} 吨标准排水量", total * scale)
        } else if lang == TextLanguage::Italic {
            format!("ex {:.0} t du déplacement normal", total * scale)
        } else {
            format!("of {:.0} t normal", total * scale)
        };
        ui.label(egui::RichText::new(subt).small().color(theme::MUTED));
    });

    let height = 22.0;
    let width = ui.available_width().max(300.0);
    let (resp, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    let rect = resp.rect;
    let radius = egui::CornerRadius::same(11);
    painter.rect_filled(rect, radius, egui::Color32::from_rgb(6, 13, 22));

    let hover = resp.hover_pos();
    let mut hovered = None;
    let mut x = rect.left();
    let visible: Vec<_> = parts.iter().filter(|p| p.1 > 0.0).collect();
    let painter = painter.with_clip_rect(rect);
    for (i, (name, wt, color)) in visible.iter().enumerate() {
        let w_px = ((wt / total).clamp(0.0, 1.0) as f32 * rect.width()).max(1.0);
        let right = (x + w_px).min(rect.right());
        let seg =
            egui::Rect::from_min_max(egui::pos2(x, rect.top()), egui::pos2(right, rect.bottom()));
        let is_hover = hover.is_some_and(|p| seg.contains(p));
        let fill = if is_hover {
            *color
        } else {
            theme::alpha(*color, 215)
        };
        // Round only the outer ends so the bar reads as one pill.
        let mut cr = egui::CornerRadius::ZERO;
        if i == 0 {
            cr.nw = 11;
            cr.sw = 11;
        }
        if i + 1 == visible.len() || right >= rect.right() - 0.5 {
            cr.ne = 11;
            cr.se = 11;
        }
        painter.rect_filled(seg, cr, fill);
        // Glossy top highlight.
        let gloss = egui::Rect::from_min_max(seg.min, egui::pos2(seg.right(), seg.center().y));
        painter.rect_filled(
            gloss,
            cr,
            egui::Color32::from_white_alpha(if is_hover { 46 } else { 22 }),
        );
        if i > 0 {
            painter.line_segment(
                [seg.left_top(), seg.left_bottom()],
                egui::Stroke::new(1.0, egui::Color32::from_black_alpha(110)),
            );
        }
        let pct = wt / total * 100.0;
        if seg.width() > 46.0 {
            painter.text(
                seg.center(),
                egui::Align2::CENTER_CENTER,
                format!("{pct:.0}%"),
                egui::FontId::proportional(11.0),
                egui::Color32::from_rgba_unmultiplied(8, 17, 28, 230),
            );
        }
        if is_hover {
            hovered = Some((*name, *wt, pct, *color));
        }
        x = right;
        if x >= rect.right() {
            break;
        }
    }
    painter.rect_stroke(
        rect,
        radius,
        egui::Stroke::new(1.0, theme::alpha(theme::LINE, 200)),
        egui::StrokeKind::Inside,
    );
    if let Some((name, wt, pct, color)) = hovered {
        resp.on_hover_ui_at_pointer(|ui| {
            ui.label(egui::RichText::new(name).strong().color(color));
            let tip = if lang == TextLanguage::Han {
                format!("{:.0} t  ·  占标排 {pct:.1}%", wt * scale)
            } else if lang == TextLanguage::Italic {
                format!("{:.0} t  ·  {pct:.1}% du déplacement normal", wt * scale)
            } else {
                format!("{:.0} t  ·  {pct:.1}% of normal", wt * scale)
            };
            ui.label(tip);
        });
    }

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        for (name, wt, color) in &parts {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(r, egui::CornerRadius::same(3), *color);
                ui.label(egui::RichText::new(*name).small());
                ui.label(
                    egui::RichText::new(format!(
                        "{:.0} t · {:.1}%",
                        wt * scale,
                        wt / total * 100.0
                    ))
                    .small()
                    .color(theme::MUTED),
                );
            });
        }
    });
}

#[cfg(test)]
mod language_tests {
    use super::*;

    fn rendered_text(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(text) => {
                out.push_str(text.galley.text());
                out.push('\n');
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    rendered_text(shape, out);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn switching_language_renders_fields_and_options_without_changing_design() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, TextLanguage::Germanic);
        let mut app = SpringSharpApp::new();
        let before = springsharp::serialize(&app.workbench.design.draft);
        let snapshot = app.workbench.design.result();
        for (lang, field, option) in [
            (TextLanguage::Germanic, "Length WL (ft)", "Normal bow"),
            (TextLanguage::Han, "水线长 (ft)", "常规型船艏 (Normal)"),
            (
                TextLanguage::Italic,
                "Longitudo ad lineam aquae (ft)",
                "Prora ordinaria",
            ),
            (TextLanguage::Germanic, "Length WL (ft)", "Normal bow"),
        ] {
            app.text_language = lang;
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("app_language"), lang));
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1600.0, 1400.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    app.page_hull(ui);
                    combo_i32(
                        ui,
                        "Bow shape",
                        &mut app.workbench.design.draft.free.bow,
                        opts::BOW_TYPES,
                    );
                },
            );
            let mut text = String::new();
            for clipped in &output.shapes {
                rendered_text(&clipped.shape, &mut text);
            }
            output.textures_delta.clear();
            assert!(text.contains(field), "{lang:?}: missing {field}: {text}");
            assert!(text.contains(option), "{lang:?}: missing {option}: {text}");
            assert_eq!(springsharp::serialize(&app.workbench.design.draft), before);
            assert!(Arc::ptr_eq(&snapshot, &app.workbench.design.result()));
        }
    }

    #[test]
    fn weapon_locations_keep_kind_and_index_in_every_language() {
        for lang in TextLanguage::ALL {
            for kind in [
                WeaponKind::Torpedo,
                WeaponKind::Mine,
                WeaponKind::DepthCharge,
            ] {
                let options = opts::weapon_locations_lang(kind, lang);
                assert_eq!(localized_options("Location", options, lang), options);
            }
        }
    }
}
