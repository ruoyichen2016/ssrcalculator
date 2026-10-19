//! Server-side SVG views. Geometry comes from the same BlueprintLayout as egui.
use crate::presentation::DisplayName;
use crate::systems::{
    BeltTdsJunction, HorizontalProtectionStyle, TdsArchitecture, TransverseSubdivision,
};
use crate::{design::Evaluation, model::ResistanceModel, presentation::html_escape};
use std::fmt::Write;

struct Svg(String);
impl Svg {
    fn new(width: f64, height: f64, title: &str) -> Self {
        Self(format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" role="img" aria-label="{}"><title>{}</title><rect width="100%" height="100%" fill="#08131e"/>"##,
            html_escape(title),
            html_escape(title)
        ))
    }
    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, width: f64) {
        let _ = write!(
            self.0,
            r##"<path d="M{x1:.2},{y1:.2} L{x2:.2},{y2:.2}" fill="none" stroke="{color}" stroke-width="{width}"/>"##
        );
    }
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: &str, title: &str) {
        let _ = write!(
            self.0,
            r##"<rect x="{x:.2}" y="{y:.2}" width="{:.2}" height="{:.2}" rx="2" fill="{color}"><title>{}</title></rect>"##,
            w.max(0.0),
            h.max(0.0),
            html_escape(title)
        );
    }
    fn text(&mut self, x: f64, y: f64, size: f64, color: &str, text: &str) {
        let _ = write!(
            self.0,
            r##"<text x="{x:.2}" y="{y:.2}" font-family="Noto Sans,Noto Sans CJK SC,sans-serif" font-size="{size}" fill="{color}">{}</text>"##,
            html_escape(text)
        );
    }
    fn path(&mut self, d: &str, fill: &str, stroke: &str, width: f64) {
        let _ = write!(
            self.0,
            r##"<path d="{d}" fill="{fill}" stroke="{stroke}" stroke-width="{width}" stroke-linejoin="round"/>"##
        );
    }
    fn finish(mut self) -> String {
        self.0.push_str("</svg>");
        self.0
    }
}

pub fn blueprint(result: &Evaluation, metric: bool, arcs: bool) -> String {
    let ship = &result.ship;
    let layout = &result.blueprint;
    let free = &ship.free;
    let arm = &ship.armour;
    let x = |p: f64| 45.0 + p.clamp(0.0, 1.0) * 910.0;
    let wl = 120.0;
    let keel = wl + ship.hull.draught.clamp(12.0, 52.0);
    let fc = free.fc_length;
    let fd = free.fd_length;
    let qd = free.qd_length;
    let deck_points = [
        (0.0, free.free_fc_for),
        (fc, free.free_fc_aft),
        (fc, free.free_fd_for),
        (fc + fd, free.free_fd_aft),
        (fc + fd, free.free_ad_for),
        (1.0 - qd, free.free_ad_aft),
        (1.0 - qd, free.free_qd_for),
        (1.0, free.free_qd_aft),
    ];
    let deck_y = |p: f64| {
        for segment in deck_points.chunks_exact(2) {
            let [(start, fore), (end, aft)] = [segment[0], segment[1]];
            if p <= end && end > start {
                return wl
                    - (fore + (aft - fore) * ((p - start) / (end - start)).clamp(0.0, 1.0))
                        .max(4.0);
            }
        }
        wl - free.free_qd_aft.max(4.0)
    };
    let mut svg = Svg::new(
        1000.0,
        560.0,
        &format!("{} — profile, plan and midship section", ship.name),
    );
    svg.text(24.0, 26.0, 14.0, "#8aa4bf", "PROFILE · BOW → STERN");
    let dimensions = if metric {
        format!(
            "{:.1} × {:.1} × {:.1} m",
            ship.hull.length * 0.3048,
            ship.hull.beam * 0.3048,
            ship.hull.draught * 0.3048
        )
    } else {
        format!(
            "{:.0} × {:.0} × {:.1} ft",
            ship.hull.length, ship.hull.beam, ship.hull.draught
        )
    };
    svg.text(730.0, 26.0, 13.0, "#8aa4bf", &dimensions);
    let mut profile = String::new();
    for (i, (p, height)) in deck_points.iter().enumerate() {
        let _ = write!(
            profile,
            "{} {:.2},{:.2} ",
            if i == 0 { "M" } else { "L" },
            x(*p),
            wl - height.max(4.0)
        );
    }
    let _ = write!(profile, "L945,{keel} L65,{keel} Z");
    svg.path(&profile, "#10283c", "#4fb3ff", 1.8);
    svg.line(25.0, wl, 975.0, wl, "#315a7b", 1.0);
    svg.text(965.0, wl - 5.0, 10.0, "#4fb3ff", "WL");
    let [cit0, cit1] = layout.citadel.map(x);
    svg.path(
        &format!("M{cit0},{} H{cit1} V{keel} H{cit0} Z", wl - 35.0),
        "none",
        "#6d5d9c",
        1.0,
    );
    let center = (layout.citadel[0] + layout.citadel[1]) * 0.5;
    let belt_width = (arm.belt_main_length / ship.hull.length.max(1.0) * 910.0).clamp(0.0, 890.0);
    let belt_x = (x(center) - belt_width / 2.0).clamp(50.0, 950.0 - belt_width);
    let belt_height = arm.belt_main_height.clamp(3.0, 30.0);
    if arm.belt_main > 0.0 {
        svg.rect(
            belt_x,
            wl - belt_height * 0.6,
            belt_width,
            belt_height,
            "#6f5795",
            &format!(
                "Main belt: {:.1} in, {:.0} ft long",
                arm.belt_main, arm.belt_main_length
            ),
        );
        svg.text(
            belt_x + 6.0,
            wl + 2.0,
            10.0,
            "#ffffff",
            &format!("{:.1} in belt · {:.0}°", arm.belt_main, arm.belts_incline),
        );
    }
    if arm.belt_upper > 0.0 {
        let width = (arm.belt_upper_length / ship.hull.length.max(1.0) * 910.0).clamp(0.0, 890.0);
        svg.rect(
            (x(center) - width / 2.0).clamp(50.0, 950.0 - width),
            wl - belt_height * 0.6 - arm.belt_upper_height.clamp(3.0, 20.0),
            width,
            arm.belt_upper_height.clamp(3.0, 20.0),
            "#72578b",
            &format!("Upper belt: {:.1} in", arm.belt_upper),
        );
    }
    if arm.belt_ends > 0.0 {
        let half = (arm.belt_ends_length / ship.hull.length.max(1.0) * 455.0).max(0.0);
        svg.rect(
            (belt_x - half).max(50.0),
            wl - 4.0,
            half.min((belt_x - 50.0).max(0.0)),
            8.0,
            "#51416f",
            "Forward ends belt",
        );
        svg.rect(
            belt_x + belt_width,
            wl - 4.0,
            half.min((950.0 - belt_x - belt_width).max(0.0)),
            8.0,
            "#51416f",
            "Aft ends belt",
        );
    }
    if arm.deck > 0.0 {
        let y = if arm.deck_coverage == 6 {
            wl - 25.0
        } else {
            wl - 4.0
        };
        svg.line(cit0, y, cit1, y, "#5fd38d", 2.0);
        if matches!(arm.deck_coverage, 1..=3) {
            svg.line(cit0, y - 8.0, cit1, y - 8.0, "#5fd38d", 1.0);
        }
    }
    let flight_y = deck_y(0.0) - 14.0;
    let super_y = if layout.is_carrier {
        flight_y
    } else {
        deck_y(layout.bridge[0])
    };
    if layout.is_carrier {
        svg.rect(
            x(0.025),
            flight_y,
            864.5,
            18.0,
            "#254d48",
            "Enclosed hangar and flight deck",
        );
        svg.line(x(0.02), flight_y, x(0.98), flight_y, "#5fd38d", 3.0);
    }
    for part in &layout.superstructure.parts {
        let height = part.height * 910.0;
        let color = if part.kind == crate::superstructure::PartKind::Funnel {
            "#687d91"
        } else {
            "#8aa4bf"
        };
        svg.rect(
            x(part.x),
            super_y - height,
            part.length * 910.0,
            height,
            color,
            "Superstructure",
        );
    }
    for line in &layout.superstructure.mast_lines {
        svg.line(
            x(line[0][0]),
            super_y - line[0][1] * 910.0,
            x(line[1][0]),
            super_y - line[1][1] * 910.0,
            "#dbe7f3",
            1.5,
        );
    }
    let director_x = x((layout.bridge[0] + layout.bridge[1]) * 0.5);
    svg.line(
        director_x,
        super_y - layout.superstructure.parts[0].height * 910.0,
        director_x,
        super_y - layout.superstructure.mast_tip[1] * 910.0,
        "#dbe7f3",
        2.0,
    );
    let rf = result.systems.optical_baseline_m.clamp(3.0, 16.0);
    svg.line(
        director_x - rf,
        super_y - layout.superstructure.mast_tip[1] * 910.0,
        director_x + rf,
        super_y - layout.superstructure.mast_tip[1] * 910.0,
        "#5fd38d",
        3.0,
    );

    let cy = 285.0;
    let hb = (ship.hull.beam / ship.hull.length.max(1.0) * 455.0).clamp(22.0, 65.0);
    svg.text(24.0, 208.0, 14.0, "#8aa4bf", "PLAN");
    svg.path(&format!("M45,{cy} Q180,{} 310,{} L810,{} Q955,{} 955,{cy} Q955,{} 810,{} L310,{} Q180,{} 45,{cy} Z",cy-hb,cy-hb,cy-hb,cy-hb*0.6,cy+hb*0.6,cy+hb,cy+hb,cy+hb), "#10283c", "#4fb3ff", 1.8);
    svg.line(35.0, cy, 965.0, cy, "#315a7b", 0.8);
    if layout.is_carrier {
        svg.rect(
            x(0.04),
            cy - hb * 0.85,
            837.2,
            hb * 1.7,
            "#193f3b",
            "Flight deck",
        );
        for pos in [0.24, 0.50, 0.75] {
            svg.rect(
                x(pos) - 10.0,
                cy - 10.0,
                20.0,
                20.0,
                "#315c54",
                "Aircraft elevator",
            );
        }
    }
    for part in &layout.superstructure.parts {
        let color = if part.kind == crate::superstructure::PartKind::Funnel {
            "#495d70"
        } else {
            "#5f7489"
        };
        svg.rect(
            x(part.x),
            cy + (part.y - part.width / 2.0) * hb * 2.0,
            part.length * 910.0,
            part.width * hb * 2.0,
            color,
            "Superstructure",
        );
    }
    for (i, mounts) in layout.batteries.iter().enumerate() {
        let battery = &ship.guns.batteries[i];
        let color = if i == 0 { "#ef8595" } else { "#9db6c9" };
        let radius = (battery.calibre * 0.48).clamp(2.2, 8.5);
        for mount in mounts {
            let mx = x(mount.x);
            let side = if mount.side == 0 {
                0.0
            } else {
                mount.side as f64
            };
            let my = cy + side * hb * if mount.is_casemate { 0.92 } else { 0.68 };
            let py = if layout.is_carrier {
                flight_y + if i > 0 { 5.0 } else { 0.0 }
            } else {
                deck_y(mount.x)
            } - mount.tier as f64 * 9.0;
            let direction = if mount.fwd { -1.0 } else { 1.0 };
            let title = format!(
                "Battery {} · {} × {:.2} in · tier {}{}",
                i + 1,
                mount.bpm,
                battery.calibre,
                mount.tier,
                if mount.is_casemate {
                    " · casemate"
                } else {
                    ""
                }
            );
            if mount.is_casemate {
                svg.rect(mx - radius, py + 2.0, radius * 2.0, 5.0, "#203447", &title);
                svg.line(mx, py + 4.0, mx + direction * 12.0, py + 4.0, color, 2.0);
                svg.line(
                    mx,
                    my,
                    mx,
                    my + if side < 0.0 { -10.0 } else { 10.0 },
                    color,
                    2.0,
                );
            } else {
                svg.rect(
                    mx - radius * 0.7,
                    py,
                    radius * 1.4,
                    (deck_y(mount.x) - py).max(4.0),
                    "#6f5795",
                    "Barbette",
                );
                svg.rect(mx - radius, py - 6.0, radius * 2.0, 7.0, color, &title);
                svg.line(
                    mx,
                    py - 3.0,
                    mx + direction * (radius + 15.0),
                    py - 3.0,
                    color,
                    2.0,
                );
                let _ = write!(
                    svg.0,
                    r##"<circle cx="{mx:.2}" cy="{my:.2}" r="{radius:.2}" fill="{color}" stroke="#ffffff" stroke-width="0.8"><title>{}</title></circle>"##,
                    html_escape(&title)
                );
                let barrels = mount.bpm.clamp(1, 4);
                for barrel in 0..barrels {
                    let by = my + (barrel as f64 - (barrels - 1) as f64 * 0.5) * 2.5;
                    svg.line(mx, by, mx + direction * (radius + 15.0), by, "#dbe7f3", 1.1);
                }
                if arcs && i == 0 {
                    let arc_r = 36.0;
                    svg.path(
                        &format!(
                            "M{mx},{my} l{},-25 A{arc_r},{arc_r} 0 1,{} {},{} Z",
                            -direction * 25.0,
                            if mount.fwd { 0 } else { 1 },
                            mx - direction * 25.0,
                            my + 25.0
                        ),
                        "#263046",
                        "#776681",
                        0.7,
                    );
                }
            }
        }
    }
    midship(&mut svg, result);
    svg.finish()
}

fn midship(svg: &mut Svg, result: &Evaluation) {
    let ship = &result.ship;
    let arm = &ship.armour;
    let sys = &result.systems;
    svg.text(
        24.0,
        397.0,
        14.0,
        "#8aa4bf",
        "MIDSHIP SECTION · INTERNAL TOPOLOGY (SCHEMATIC)",
    );
    svg.path(
        "M85,430 L315,430 L300,500 Q290,530 250,530 H150 Q110,530 100,500 Z",
        "#10283c",
        "#4fb3ff",
        1.8,
    );
    svg.line(65.0, 465.0, 335.0, 465.0, "#315a7b", 1.0);
    svg.line(113.0, 516.0, 287.0, 516.0, "#4fb3ff", 1.0);
    let pi_sub = sys
        .features
        .underwater_belt_submergence_ratio
        .clamp(0.0, 1.0);
    let pi_taper = sys.features.belt_lower_taper_ratio.clamp(0.1, 1.0);
    let incline = arm.belts_incline.to_radians().tan().clamp(-0.8, 0.8) * 30.0;
    let y_wl = 465.0;
    let y_keel = 516.0;
    let draught_span = y_keel - y_wl; // 51.0 px
    let y_sub = y_wl + (draught_span * pi_sub).clamp(8.0, 48.0);
    let y_top = if arm.belt_upper > 0.0 {
        448.0
    } else if arm.deck_coverage == 6 {
        432.0
    } else {
        442.0
    };

    let main_belt_w = (arm.belt_main * 0.30).clamp(1.5, 6.5);
    let lower_belt_w = (main_belt_w * pi_taper).clamp(1.2, main_belt_w);

    for (sx, sign) in [(98.0, 1.0), (302.0, -1.0)] {
        if arm.belt_main > 0.0 {
            let x_top = sx - sign * incline * ((y_wl - y_top) / 25.0);
            let x_wl = sx;
            let x_sub = sx + sign * incline * ((y_sub - y_wl) / 25.0);

            // Upper portion (deck to waterline)
            svg.line(x_top, y_top, x_wl, y_wl, "#b590e0", main_belt_w);
            // Underwater extension (waterline to sub-depth, with taper thinning)
            svg.line(x_wl, y_wl, x_sub, y_sub, "#9b7bea", lower_belt_w);

            // Joint closure to torpedo bulkhead according to resolved topology
            match sys.resolved_belt_tds_junction {
                BeltTdsJunction::IntegralTaperedHoldingBulkhead => {
                    // Direct tapered continuous holding bulkhead closure down into double bottom
                    let x_hold = sx + sign * 28.0;
                    svg.line(x_sub, y_sub, x_hold, y_keel, "#9b7bea", 1.8);
                }
                BeltTdsJunction::TurtlebackSlopeKneeJoint => {
                    // Holding bulkhead dropping vertically from turtleback knee joint to double bottom
                    svg.line(x_sub, y_sub, x_sub + sign * 6.0, y_keel, "#e0864f", 1.6);
                }
                BeltTdsJunction::SpacedAbsorberShelfJoint => {
                    // Horizontal shelf connecting lower belt edge inward to cylinder absorber
                    let x_cyl = if sign > 0.0 { 125.0 } else { 275.0 };
                    svg.line(x_sub, y_sub, x_cyl, y_sub, "#e0864f", 1.8);
                }
                BeltTdsJunction::SteppedShelfCitadel => {
                    // Stepped horizontal deck shelf to vertical holding bulkhead
                    let x_bulk = sx + sign * 22.0;
                    svg.line(x_sub, y_sub, x_bulk, y_sub, "#5fd38d", 1.8);
                    svg.line(x_bulk, y_sub, x_bulk, y_keel, "#e0864f", 1.6);
                }
                BeltTdsJunction::DirectSkinDoubleBottom => {
                    // Smooth skin closure
                    svg.line(x_sub, y_sub, sx + sign * 14.0, y_keel, "#4fb3ff", 1.2);
                }
                BeltTdsJunction::Auto => {}
            }
        }
    }
    match sys.resolved_horizontal_protection {
        HorizontalProtectionStyle::ProtectedTurtlebackCarapace => {
            svg.path(
                "M98,488 Q145,446 200,446 Q255,446 302,488",
                "none",
                "#5fd38d",
                3.0,
            );
            svg.line(95.0, 434.0, 305.0, 434.0, "#5fd38d", 1.2);
        }
        HorizontalProtectionStyle::LowTurtlebackSlopedCitadel => {
            svg.line(95.0, 434.0, 305.0, 434.0, "#7ce3a3", 1.5);
            svg.path(
                &format!("M100,{y_sub:.1} L145,452 H255 L300,{y_sub:.1}"),
                "none",
                "#5fd38d",
                3.0,
            );
        }
        HorizontalProtectionStyle::MultiLayerBombDecappingSandwich => {
            // 1. Bomb / cap-stripping weather deck
            svg.line(95.0, 432.0, 305.0, 432.0, "#7ce3a3", 1.6);
            // 2. Heavy main ballistic armored deck closing citadel top
            svg.line(98.0, 448.0, 302.0, 448.0, "#5fd38d", 3.4);
            // 3. Lower splinter catcher deck above vitals
            svg.line(112.0, 460.0, 288.0, 460.0, "#3ca86b", 1.5);
        }
        HorizontalProtectionStyle::MonolithicCitadelBoxDeck => {
            svg.line(96.0, 438.0, 304.0, 438.0, "#5fd38d", 3.4);
        }
        HorizontalProtectionStyle::ArmouredFlightDeckHangarBox => {
            svg.line(88.0, 418.0, 312.0, 418.0, "#5fd38d", 3.0);
            svg.line(94.0, 418.0, 94.0, 448.0, "#4fb3ff", 1.6);
            svg.line(306.0, 418.0, 306.0, 448.0, "#4fb3ff", 1.6);
            svg.line(95.0, 448.0, 305.0, 448.0, "#5fd38d", 2.2);
        }
        HorizontalProtectionStyle::LightSplinterWeatherDeck | HorizontalProtectionStyle::Auto => {
            svg.line(95.0, 442.0, 305.0, 442.0, "#5fd38d", 1.4);
        }
    }
    let divisions: &[f64] = match sys.resolved_transverse_subdivision {
        TransverseSubdivision::TwinFileCenterline => &[200.0],
        TransverseSubdivision::TripleFileCenterAndWings => &[155.0, 245.0],
        TransverseSubdivision::QuadFileMultiLongitudinal => &[145.0, 200.0, 255.0],
        _ => &[],
    };
    for sx in divisions {
        svg.line(*sx, 451.0, *sx, 516.0, "#8aa4bf", 1.5);
    }
    match sys.resolved_tds_architecture {
        TdsArchitecture::PuglieseCylindricalAbsorber => {
            for cx in [125, 275] {
                let _ = write!(
                    svg.0,
                    r##"<ellipse cx="{cx}" cy="487" rx="14" ry="23" fill="none" stroke="#e0864f" stroke-width="2"><title>Pugliese cylinder</title></ellipse>"##
                );
            }
        }
        TdsArchitecture::CompoundEboniteDeepTds => {
            for sx in [110.0, 265.0] {
                for row in 0..5 {
                    for col in 0..3 {
                        svg.rect(
                            sx + col as f64 * 7.0,
                            461.0 + row as f64 * 9.0,
                            5.0,
                            7.0,
                            "#886348",
                            "Ébonite foam cells",
                        );
                    }
                }
            }
        }
        TdsArchitecture::MultiLayerLiquidVoid
        | TdsArchitecture::ExternalBlisterBulge
        | TdsArchitecture::CoalBunkerWingBulkhead => {
            for sx in [115.0, 130.0, 270.0, 285.0] {
                svg.line(sx, 454.0, sx, 508.0, "#e0864f", 1.4);
            }
        }
        _ => {}
    }
    svg.text(
        365.0,
        420.0,
        11.2,
        "#5fd38d",
        sys.resolved_horizontal_protection.display_name(),
    );
    svg.text(
        365.0,
        439.0,
        11.2,
        "#7ce3a3",
        sys.resolved_transverse_subdivision.display_name(),
    );
    svg.text(
        365.0,
        458.0,
        11.2,
        "#e0864f",
        sys.resolved_tds_architecture.display_name(),
    );
    svg.text(
        365.0,
        477.0,
        11.2,
        "#9b7bea",
        sys.resolved_belt_tds_junction.display_name(),
    );
    svg.text(
        365.0,
        496.0,
        11.2,
        "#b590e0",
        sys.resolved_machinery_arrangement.display_name(),
    );
    svg.text(
        365.0,
        515.0,
        10.8,
        "#8aa4bf",
        &format!(
            "Belt {:.1}\" ({:.0}° incl) · Deck {:.1}\" (Deck/Belt Π {:.2}) · Sub {:.0}% · TDS {:.0}%",
            arm.belt_main,
            arm.belts_incline,
            arm.deck,
            sys.features.deck_belt_balance_ratio,
            sys.features.underwater_belt_submergence_ratio * 100.0,
            sys.features.tds_depth_ratio * 100.0
        ),
    );
}

type ChartSeries<'a> = (&'a str, &'a str, Vec<(f64, f64)>);

fn chart(
    title: &str,
    x_label: &str,
    y_label: &str,
    series: &[ChartSeries],
    xmin: f64,
    xmax: f64,
) -> String {
    let ymax = series
        .iter()
        .flat_map(|(_, _, pts)| pts.iter().map(|p| p.1))
        .filter(|v| v.is_finite())
        .fold(1.0_f64, f64::max)
        * 1.12;
    let xx = |v: f64| 70.0 + (v - xmin) / (xmax - xmin).max(1.0) * 820.0;
    let yy = |v: f64| 300.0 - v / ymax * 230.0;
    let mut svg = Svg::new(960.0, 370.0, title);
    svg.text(20.0, 24.0, 15.0, "#dbe7f3", title);
    svg.text(750.0, 355.0, 12.0, "#8aa4bf", x_label);
    svg.text(16.0, 53.0, 11.0, "#8aa4bf", y_label);
    for i in 0..=5 {
        let yv = ymax * i as f64 / 5.0;
        svg.line(70.0, yy(yv), 890.0, yy(yv), "#203447", 1.0);
        svg.text(5.0, yy(yv) + 4.0, 10.0, "#8aa4bf", &format!("{yv:.0}"));
        let xv = xmin + (xmax - xmin) * i as f64 / 5.0;
        svg.text(xx(xv) - 8.0, 320.0, 10.0, "#8aa4bf", &format!("{xv:.0}"));
    }
    for (i, (name, color, pts)) in series.iter().enumerate() {
        svg.text(75.0 + i as f64 * 265.0, 52.0, 11.0, color, name);
        let mut path = String::new();
        for (j, (px, py)) in pts
            .iter()
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .enumerate()
        {
            let _ = write!(
                path,
                "{} {:.2},{:.2} ",
                if j == 0 { "M" } else { "L" },
                xx(*px),
                yy(*py)
            );
        }
        svg.path(&path, "none", color, 2.0);
    }
    svg.finish()
}

pub fn speed_power(
    result: &Evaluation,
    comparison: Option<&crate::model::ShipModel>,
    metric: bool,
) -> String {
    let ship = &result.ship;
    let top = (ship.engine.speed_max + 2.0)
        .max(comparison.map(|s| s.engine.speed_max + 2.0).unwrap_or(20.0))
        .max(20.0);
    let factor = if metric { 1.852 } else { 1.0 };
    let points = |ship: &crate::model::ShipModel, original: bool| {
        let mut engine = ship.engine.clone();
        if original {
            engine.resistance_model = ResistanceModel::Original;
            engine.blend_weight = 1.0;
            engine.power_factor = 1.0;
            engine.original_power_factor = 1.0;
            engine.cfd_proxy_enabled = false;
        }
        (0..=40)
            .map(|i| {
                let v = 10.0 + (top - 10.0) * i as f64 / 40.0;
                (v * factor, engine.speed_hp_calc(v, &ship.hull))
            })
            .collect()
    };
    let mut series = vec![
        ("Current model", "#4fb3ff", points(ship, false)),
        ("Original model", "#ffb547", points(ship, true)),
    ];
    if let Some(base) = comparison {
        series.push((base.name.as_str(), "#b590e0", points(base, false)));
    }
    chart(
        "Speed & power",
        if metric { "Speed (km/h)" } else { "Speed (kt)" },
        "Power (hp)",
        &series,
        10.0 * factor,
        top * factor,
    )
}

pub fn immunity(result: &Evaluation, metric: bool) -> String {
    let ship = &result.ship;
    let iz = crate::compute_self_immunity_zone(
        &ship.guns.batteries[0],
        ship.armour.belt_main,
        ship.armour.belts_incline,
        ship.armour.deck,
    );
    if iz.curve.is_empty() {
        return "<p class=hint>No active main battery for a penetration curve.</p>".into();
    }
    let xf = if metric { 0.9144 } else { 1.0 };
    let yf = if metric { 25.4 } else { 1.0 };
    let belt = iz
        .curve
        .iter()
        .map(|p| (p.range_yd * xf, p.belt_pen_in * yf))
        .collect();
    let deck = iz
        .curve
        .iter()
        .map(|p| (p.range_yd * xf, p.deck_pen_in * yf))
        .collect();
    let label = if iz.has_zone {
        format!(
            "Immunity: {:.0}–{:.0} {}",
            iz.inner_limit_yd.unwrap_or(0.0) * xf,
            iz.outer_limit_yd.unwrap_or(0.0) * xf,
            if metric { "m" } else { "yd" }
        )
    } else {
        "No immunity zone".into()
    };
    let series = vec![
        ("Belt penetration", "#ef8595", belt),
        ("Deck penetration", "#4fb3ff", deck),
        (
            "Own belt",
            "#b590e0",
            vec![
                (5000.0 * xf, ship.armour.belt_main * yf),
                (35000.0 * xf, ship.armour.belt_main * yf),
            ],
        ),
    ];
    let mut out = chart(
        &label,
        if metric { "Range (m)" } else { "Range (yd)" },
        if metric { "Armour (mm)" } else { "Armour (in)" },
        &series,
        5000.0 * xf,
        35000.0 * xf,
    );
    let hit = vec![(
        "Salvo hit probability",
        "#5fd38d",
        result
            .systems
            .hit_prob_curve
            .iter()
            .map(|(r, p)| (*r * xf, *p))
            .collect(),
    )];
    out.push_str(&chart(
        "Fire-control estimate",
        if metric { "Range (m)" } else { "Range (yd)" },
        "Probability (%)",
        &hit,
        5000.0 * xf,
        35000.0 * xf,
    ));
    out
}
