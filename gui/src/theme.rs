//! "Night chart-room" theme: deep navy surfaces, translucent glass cards,
//! soft shadows and rounded geometry, with brass/cyan naval accents.

use eframe::egui::{self, Color32, CornerRadius, Margin, Shadow, Stroke, Visuals};

pub const BG: Color32 = Color32::from_rgb(8, 17, 28);
pub const PANEL: Color32 = Color32::from_rgb(12, 24, 38);
pub const SURFACE: Color32 = Color32::from_rgb(17, 33, 51);
pub const SURFACE_HI: Color32 = Color32::from_rgb(25, 46, 70);
pub const LINE: Color32 = Color32::from_rgb(36, 64, 92);
pub const TEXT: Color32 = Color32::from_rgb(219, 231, 243);
pub const MUTED: Color32 = Color32::from_rgb(138, 164, 191);
pub const ACCENT: Color32 = Color32::from_rgb(79, 179, 255);
pub const BRASS: Color32 = Color32::from_rgb(214, 181, 110);
pub const GOOD: Color32 = Color32::from_rgb(95, 211, 141);
pub const WARN: Color32 = Color32::from_rgb(255, 181, 71);
pub const BAD: Color32 = Color32::from_rgb(255, 107, 107);

/// Weight-group colours, shared by the dashboard bar and its legend.
pub const W_HULL: Color32 = Color32::from_rgb(79, 143, 214);
pub const W_ARMOUR: Color32 = Color32::from_rgb(155, 123, 234);
pub const W_MACHINERY: Color32 = Color32::from_rgb(224, 134, 79);
pub const W_ARMAMENT: Color32 = Color32::from_rgb(224, 93, 111);
pub const W_MAGAZINE: Color32 = Color32::from_rgb(214, 181, 79);
pub const W_BUNKER: Color32 = Color32::from_rgb(79, 199, 180);
pub const W_MISC: Color32 = Color32::from_rgb(122, 140, 160);
pub const W_OTHER: Color32 = Color32::from_rgb(70, 88, 108);

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    let mut v = Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = alpha(SURFACE, 245);
    v.window_stroke = Stroke::new(1.0, LINE);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(10);
    v.window_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };
    v.popup_shadow = Shadow {
        offset: [0, 6],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    v.extreme_bg_color = Color32::from_rgb(6, 13, 22);
    v.faint_bg_color = alpha(SURFACE_HI, 70);
    v.code_bg_color = Color32::from_rgb(10, 20, 32);
    v.hyperlink_color = ACCENT;
    v.warn_fg_color = WARN;
    v.error_fg_color = BAD;
    v.selection.bg_fill = alpha(ACCENT, 70);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.striped = true;
    v.slider_trailing_fill = true;
    v.override_text_color = Some(TEXT);

    let r = CornerRadius::same(7);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = SURFACE;
    w.noninteractive.weak_bg_fill = SURFACE;
    w.noninteractive.bg_stroke = Stroke::new(1.0, alpha(LINE, 160));
    w.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
    w.noninteractive.corner_radius = r;

    w.inactive.bg_fill = Color32::from_rgb(14, 29, 45);
    w.inactive.weak_bg_fill = alpha(SURFACE_HI, 200);
    w.inactive.bg_stroke = Stroke::new(1.0, alpha(LINE, 200));
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    w.inactive.corner_radius = r;

    w.hovered.bg_fill = Color32::from_rgb(22, 44, 68);
    w.hovered.weak_bg_fill = Color32::from_rgb(31, 58, 88);
    w.hovered.bg_stroke = Stroke::new(1.0, alpha(ACCENT, 180));
    w.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    w.hovered.corner_radius = r;
    w.hovered.expansion = 1.0;

    w.active.bg_fill = Color32::from_rgb(29, 91, 140);
    w.active.weak_bg_fill = Color32::from_rgb(29, 91, 140);
    w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    w.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
    w.active.corner_radius = r;
    w.active.expansion = 1.0;

    w.open.bg_fill = SURFACE_HI;
    w.open.weak_bg_fill = SURFACE_HI;
    w.open.bg_stroke = Stroke::new(1.0, ACCENT);
    w.open.fg_stroke = Stroke::new(1.0, TEXT);
    w.open.corner_radius = r;
    ctx.set_visuals_of(egui::Theme::Dark, v);

    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.interact_size.y = 24.0;
        s.spacing.indent = 16.0;
    });
}

/// Translucent rounded card used to group content.
pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(alpha(SURFACE, 215))
        .stroke(Stroke::new(1.0, alpha(LINE, 170)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(14))
        .shadow(Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(90),
        })
}

/// Small rounded status pill.
pub fn pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(alpha(color, 38))
        .stroke(Stroke::new(1.0, alpha(color, 150)))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(9, 2))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).color(color).small().strong());
        });
}

/// Compact metric tile: muted caption above a large tabular value.
pub fn metric(ui: &mut egui::Ui, caption: &str, value: String, accent: Color32) {
    egui::Frame::new()
        .fill(alpha(SURFACE_HI, 150))
        .stroke(Stroke::new(1.0, alpha(LINE, 140)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 7))
        .show(ui, |ui| {
            ui.set_min_width(112.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(
                    egui::RichText::new(caption.to_uppercase())
                        .size(10.0)
                        .color(MUTED),
                );
                ui.label(egui::RichText::new(value).size(17.0).strong().color(accent));
            });
        });
}

/// Section heading with a brass accent rule.
pub fn heading(ui: &mut egui::Ui, text: &str) {
    let lang = ui
        .ctx()
        .data(|d| d.get_temp(egui::Id::new("app_language")))
        .unwrap_or(springsharp::presentation::fonts::TextLanguage::Germanic);
    let display_text = if lang == springsharp::presentation::fonts::TextLanguage::Han {
        translate_heading(text)
    } else {
        springsharp::presentation::latin_ui::text(lang, text)
    };
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 20.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::same(2), BRASS);
        ui.label(
            egui::RichText::new(display_text)
                .size(19.0)
                .strong()
                .color(TEXT),
        );
    });
    ui.add_space(4.0);
}

fn translate_heading(text: &str) -> &str {
    match text {
        "Hull" => "船体与主尺度 (Hull)",
        "Freeboard" => "干舷与型线 (Freeboard)",
        "Armour" => "装甲与防护 (Armour)",
        "Gun batteries" => "舰炮武备配置 (Gun batteries)",
        "Machinery" => "动力推进系统 (Machinery)",
        "Calibration (modern regime model)" => "阻力标定与回归模型 (Calibration)",
        "Target Displacement Solver" => "目标排水量反推求解器 (Target Disp Solver)",
        "Side-by-Side Ship Comparison" => "舰船指标并排对比 (Comparison)",
        "Report and files" => "设计报告与数据文件 (Report & Files)",
        "Weapons and systems" => "副武备与舱室系统 (Weapons & Systems)",
        "Miscellaneous weights (tons)" => "杂项重量构成 (Misc Weights)",
        "Subdivision, internal arrangements and machinery layout" => {
            "分舱与动力布置架构 (Arrangements)"
        }
        "Internal Space Volume Budget" => "舱室容积预算 (Space Budget)",
        "Multi-Deck General Arrangement Optimizer (GA)" => "多甲板总布置与规范校核 (GA Optimizer)",
        "Vessel Blueprint Schematic" => "总布置蓝图设计图纸 (Blueprint)",
        "Speed & Power Curve" => "航速与轴功率曲线 (Speed & Power Curve)",
        "Armor Penetration & Immunity Zone" => "装甲穿深与安全免疫区 (Immunity Zone)",
        _ => text,
    }
}
