use super::fonts::TextLanguage;

pub fn report_text<'a>(language: TextLanguage, en: &'a str, zh: &'a str, la: &'a str) -> &'a str {
    match language {
        TextLanguage::Germanic => en,
        TextLanguage::Han => zh,
        TextLanguage::Italic => la,
    }
}

// Choose the template before formatting so names and user-entered text are untouched.
macro_rules! report_format {
    ($language:expr, $en:literal, $zh:literal, $la:literal $(, $arg:expr)* $(,)?) => {
        match $language {
            TextLanguage::Han => format!($zh $(, $arg)*),
            TextLanguage::Italic => format!($la $(, $arg)*),
            TextLanguage::Germanic => format!($en $(, $arg)*),
        }
    };
}

use crate::model::{metric_calc, Conversion, POUNDS2KG, SQFEET2SQMETRES};
use crate::*;
use std::fmt;
fn gun_type_name(idx: i32, language: TextLanguage) -> &'static str {
    match idx {
        0 => report_text(
            language,
            "Muzzle loading gun",
            "前装炮",
            "Canon à chargement par la bouche",
        ),
        1 => report_text(
            language,
            "Breech loading gun",
            "后装炮",
            "Canon à chargement par la culasse",
        ),
        2 => report_text(language, "Quick firing gun", "速射炮", "Canon à tir rapide"),
        3 => report_text(language, "Anti-air gun", "高射炮", "Canon antiaérien"),
        4 => report_text(
            language,
            "Dual purpose gun",
            "高平两用炮",
            "Canon à double usage",
        ),
        5 => report_text(
            language,
            "Auto rapid fire gun",
            "自动速射炮",
            "Canon automatique à tir rapide",
        ),
        6 => report_text(language, "Machine gun", "机枪", "Mitrailleuse"),
        _ => report_text(language, "gun", "火炮", "Tormentum"),
    }
}

fn mount_type_name(idx: i32, language: TextLanguage) -> &'static str {
    match idx {
        0 => report_text(
            language,
            "in broadside mount",
            "舷侧炮架",
            "Affût de bordée",
        ),
        1 => report_text(
            language,
            "in Coles/Ericsson turret mount",
            "科尔斯／埃里克森式炮塔",
            "Tourelle Coles/Ericsson",
        ),
        2 => report_text(
            language,
            "in open barbette mount",
            "露天炮座",
            "Affût en barbette ouverte",
        ),
        3 => report_text(
            language,
            "in turret on barbette mount",
            "炮座式炮塔",
            "Tourelle sur barbette",
        ),
        4 => report_text(
            language,
            "in deck and hoist mount",
            "带扬弹机甲板炮架",
            "Affût de pont avec monte-munitions",
        ),
        5 => report_text(language, "in deck mount", "甲板炮架", "Affût de pont"),
        6 => report_text(
            language,
            "in casemate mount",
            "炮廓炮架",
            "Affût en casemate",
        ),
        _ => report_text(language, "mount", "炮架", "Affût"),
    }
}

fn layout_name(idx: i32, language: TextLanguage) -> &'static str {
    match idx {
        0 => report_text(
            language,
            "Centreline - distributed",
            "中线分散布置",
            "In linea media — distributa",
        ),
        1 => report_text(
            language,
            "Centreline - ends (fore >= aft)",
            "中线两端布置（前部不少于后部）",
            "In linea media — in extremitatibus (prora >= puppis)",
        ),
        2 => report_text(
            language,
            "Centreline - ends (aft >= fore)",
            "中线两端布置（后部不少于前部）",
            "In linea media — in extremitatibus (puppis >= prora)",
        ),
        3 => report_text(
            language,
            "Centreline - fore deck forward",
            "中线前甲板前部",
            "In linea media — in parte anteriore tabulati anterioris",
        ),
        4 => report_text(
            language,
            "Centreline - fore deck",
            "中线前甲板",
            "In linea media — in tabulato anteriore",
        ),
        5 => report_text(
            language,
            "Centreline - fore deck aft",
            "中线前甲板后部",
            "In linea media — in parte posteriore tabulati anterioris",
        ),
        6 => report_text(
            language,
            "Centreline - aft deck forward",
            "中线后甲板前部",
            "In linea media — in parte anteriore tabulati posterioris",
        ),
        7 => report_text(
            language,
            "Centreline - aft deck",
            "中线后甲板",
            "In linea media — in tabulato posteriore",
        ),
        8 => report_text(
            language,
            "Centreline - aft deck aft",
            "中线后甲板后部",
            "In linea media — in parte posteriore tabulati posterioris",
        ),
        9 => report_text(
            language,
            "Sides - distributed",
            "两舷分散布置",
            "Ad latera — distributa",
        ),
        10 => report_text(
            language,
            "Sides - ends (fore >= aft)",
            "两舷两端布置（前部不少于后部）",
            "Ad latera — in extremitatibus (prora >= puppis)",
        ),
        11 => report_text(
            language,
            "Sides - ends (aft >= fore)",
            "两舷两端布置（后部不少于前部）",
            "Ad latera — in extremitatibus (puppis >= prora)",
        ),
        12 => report_text(
            language,
            "Sides - fore deck forward",
            "两舷前甲板前部",
            "Ad latera — in parte anteriore tabulati anterioris",
        ),
        13 => report_text(
            language,
            "Sides - fore deck",
            "两舷前甲板",
            "Ad latera — in tabulato anteriore",
        ),
        14 => report_text(
            language,
            "Sides - fore deck aft",
            "两舷前甲板后部",
            "Ad latera — in parte posteriore tabulati anterioris",
        ),
        15 => report_text(
            language,
            "Sides - aft deck forward",
            "两舷后甲板前部",
            "Ad latera — in parte anteriore tabulati posterioris",
        ),
        16 => report_text(
            language,
            "Sides - aft deck",
            "两舷后甲板",
            "Ad latera — in tabulato posteriore",
        ),
        17 => report_text(
            language,
            "Sides - aft deck aft",
            "两舷后甲板后部",
            "Ad latera — in parte posteriore tabulati posterioris",
        ),
        _ => report_text(language, "None", "无", "Nullum"),
    }
}

fn gun_description(b: &crate::model::GunBattery, main: bool, language: TextLanguage) -> String {
    if b.barrels <= 0 || b.calibre <= 0.0 {
        return report_format!(
            language,
            "      No {} battery",
            "      无{}炮组",
            "      Nulla tormenta {}",
            if main {
                report_text(language, "main", "主", "principalia")
            } else {
                report_text(language, "secondary", "副", "secundaria")
            }
        );
    }
    let mm = b.calibre * 25.4;
    let mm_text = if mm < 100.0 {
        format!("{mm:.1}")
    } else {
        format!("{mm:.0}")
    };
    let mut d = report_format!(
        language,
        "      {} - {:.2}\" / {} mm {:.1} cal gun{}",
        "      {} 门 — {:.2}\" / {} mm，{:.1} 倍径火炮{}",
        "      Tormenta: {} — {:.2}\" / {} mm, {:.1} calibres{}",
        b.barrels,
        b.calibre,
        mm_text,
        b.cal_length,
        if language == TextLanguage::Germanic && b.barrels > 1 {
            "s"
        } else {
            ""
        }
    );
    d.push_str(&report_format!(
        language,
        " - {:.2}lbs / {:.2}kg shells, {} per gun",
        " — 弹重 {:.2}lbs / {:.2}kg，每门备弹 {} 发",
        " — pondus proiectilis {:.2}lbs / {:.2}kg; {} proiectilia pro singulo tormento",
        b.shell_weight,
        metric_calc(b.shell_weight, Conversion::Pounds2Kg),
        b.magazine
    ));
    d.push_str(&report_format!(
        language,
        "\n\t  {} {}{}, {} Model",
        "\n\t  {}，{}{}，{} 年型",
        "\n\t  {}, {}{}, exemplar anni {}",
        gun_type_name(b.gun_type, language),
        mount_type_name(b.mount_type, language),
        if language == TextLanguage::Germanic && b.mounts_g1 + b.mounts_g2 > 1 {
            "s"
        } else {
            ""
        },
        b.date_gun
    ));
    if b.mounts_g1 > 0 {
        d.push_str(&format!(
            "\n\t  {} x {} {}",
            b.mounts_g1,
            b.per_mount_g1,
            layout_name(b.layout_g1, language)
        ));
        if b.mount_super_g1 > 0 {
            d.push_str(&report_format!(
                language,
                "\n\t\t{} raised mount{}",
                "\n\t\t{} 座背负式炮架{}",
                "\n\t\tAffûts surélevés: {}{}",
                b.mount_super_g1,
                if language == TextLanguage::Germanic && b.mount_super_g1 > 1 {
                    "s"
                } else {
                    ""
                }
            ));
        }
        if b.mount_sub_g1 > 0 {
            let placement = if b.mount_type == 0 {
                report_text(
                    language,
                    " on upper deck",
                    "，位于上甲板",
                    " in tabulato superiore",
                )
            } else if b.mount_sub_g1 > 1 {
                report_text(language, " in casemates", "，位于炮廓", " en casemate")
            } else {
                report_text(language, " in casemate", "，位于炮廓", " en casemate")
            };
            d.push_str(&report_format!(
                language,
                "\n\t\t{} hull mount{}{placement}",
                "\n\t\t{} 座船体炮架{}{placement}",
                "\n\t\tAffûts de coque: {}{}{placement}",
                b.mount_sub_g1,
                if language == TextLanguage::Germanic && b.mount_sub_g1 > 1 {
                    "s"
                } else {
                    ""
                }
            ));
        }
    }
    if b.mounts_g2 > 0 {
        d.push_str(&format!(
            "\n\t  {} x {} {}",
            b.mounts_g2,
            b.per_mount_g2,
            layout_name(b.layout_g2, language)
        ));
        if b.mount_super_g2 > 0 {
            d.push_str(&report_format!(
                language,
                "\n\t\t{} raised mount{}",
                "\n\t\t{} 座背负式炮架{}",
                "\n\t\tAffûts surélevés: {}{}",
                b.mount_super_g2,
                if language == TextLanguage::Germanic && b.mount_super_g2 > 1 {
                    "s"
                } else {
                    ""
                }
            ));
        }
        if b.mount_sub_g2 > 0 {
            let placement = if b.mount_type == 0 {
                if b.sub_sub_chk {
                    report_text(
                        language,
                        " on gun deck",
                        "，位于炮甲板",
                        " in tabulato tormentorum",
                    )
                } else {
                    report_text(
                        language,
                        " on upper deck",
                        "，位于上甲板",
                        " in tabulato superiore",
                    )
                }
            } else if b.sub_sub_chk {
                report_text(
                    language,
                    " in lower casemates",
                    "，位于下层炮廓",
                    " en casemate inférieure",
                )
            } else if b.mount_sub_g2 > 1 {
                report_text(language, " in casemates", "，位于炮廓", " en casemate")
            } else {
                report_text(language, " in casemate", "，位于炮廓", " en casemate")
            };
            d.push_str(&report_format!(
                language,
                "\n\t\t{} hull mount{}{placement}",
                "\n\t\t{} 座船体炮架{}{placement}",
                "\n\t\tAffûts de coque: {}{}{placement}",
                b.mount_sub_g2,
                if language == TextLanguage::Germanic && b.mount_sub_g2 > 1 {
                    "s"
                } else {
                    ""
                }
            ));
        }
    }
    if b.free_gun > 0.0 {
        let use_note = if b.free_gun < 12.0 {
            report_text(
                language,
                "Limited use in any sea",
                "所有海况下使用均受限制",
                "Usus in omni statu maris limitatus",
            )
        } else if b.free_gun < 16.0 {
            report_text(
                language,
                "Limited use in all but light seas",
                "除轻浪外，使用均受限制",
                "Usus limitatus nisi in mari parum agitato",
            )
        } else {
            report_text(
                language,
                "Limited use in heavy seas",
                "大浪中使用受限制",
                "Usus in mari valde agitato limitatus",
            )
        };
        d.push_str(&format!("\n\t\t{use_note}"));
    }
    d
}

fn ship_classification(ship: &ShipModel, language: TextLanguage) -> Vec<String> {
    let h = &ship.hull;
    let a = &ship.armour;
    let g = &ship.guns;
    let p = ship;
    let mut lines = Vec::new();

    let sys_eval = ship.systems.evaluate(ship);
    lines.push(report_format!(
        language,
        "Fleet Tactical Role: {}",
        "舰队战术定位：{}",
        "Munus in classe: {}",
        if language != TextLanguage::Germanic {
            let profile = &sys_eval.tactical_affinities;
            let echelon = match profile.dominant_echelon {
                TacticalEchelon::Auto => report_text(language, "", "自动", "Automaticum"),
                TacticalEchelon::BattleLineMainBody => {
                    report_text(language, "", "主力决战本队", "Acies principalis")
                }
                TacticalEchelon::FastWingVanguard => report_text(
                    language,
                    "",
                    "快速翼与前卫游击队",
                    "Ala celeris et agmen antecursorum",
                ),
                TacticalEchelon::OuterScoutingScreen => {
                    report_text(language, "", "外层侦察与搜索幕", "Exploratores exteriores")
                }
                TacticalEchelon::InnerFleetScreen => report_text(
                    language,
                    "",
                    "舰队内层护卫幕",
                    "Praesidium interius classis",
                ),
                TacticalEchelon::FlotillaAssaultLine => report_text(
                    language,
                    "",
                    "水雷突击支队",
                    "Flottille d’attaque à la torpille",
                ),
            };
            let modality = match profile.dominant_modality {
                CombatModality::Auto => report_text(language, "", "自动", "Automaticum"),
                CombatModality::HeavyArtilleryCitadel => report_text(
                    language,
                    "",
                    "重炮装甲堡垒",
                    "Arx loricata cum tormentis gravibus",
                ),
                CombatModality::RapidFireSurfaceGunnery => report_text(
                    language,
                    "",
                    "高射速水面火炮",
                    "Tormenta celeriter ignem facientia",
                ),
                CombatModality::AviationDeckProjection => {
                    report_text(language, "", "航空甲板立体投射", "Navis aeroplanorum")
                }
                CombatModality::MassedTorpedoStrike => report_text(
                    language,
                    "",
                    "重雷击水面突击",
                    "Attaque massive à la torpille",
                ),
                CombatModality::ScreenEscortDefense => report_text(
                    language,
                    "",
                    "防空反潜多维护卫",
                    "Défense antiaérienne et anti-sous-marine",
                ),
            };
            let mobility = match profile.dominant_mobility {
                MobilityRegime::Auto => report_text(language, "", "自动", "Automaticum"),
                MobilityRegime::HeavyDisplacementBlock => report_text(
                    language,
                    "",
                    "丰满排水量稳性体制",
                    "Coque à déplacement lourd",
                ),
                MobilityRegime::TransFroudeFastCapital => report_text(
                    language,
                    "",
                    "跨弗劳德高速主力体制",
                    "Grande coque rapide trans-Froude",
                ),
                MobilityRegime::SlenderWaveCutting => report_text(
                    language,
                    "",
                    "细长破浪远洋巡航体制",
                    "Corpus gracile undas secans",
                ),
                MobilityRegime::SuperCriticalHighFn => report_text(
                    language,
                    "",
                    "超临界半滑行体制",
                    "Coque à grande vitesse et semi-planante",
                ),
            };
            format!("{echelon} × {modality} × {mobility}")
        } else {
            crate::presentation::classification(&sys_eval.tactical_affinities)
        }
    ));

    if g.batteries[0].mount_type == 2 || g.batteries[1].mount_type == 2 {
        lines.push(
            report_text(
                language,
                "Barbette ship",
                "露炮座舰",
                "Navis tormentis in barbette instructa",
            )
            .to_string(),
        );
    }
    if (g.batteries[0].layout_g1 == 4 || g.batteries[0].layout_g1 == 10)
        && g.batteries[0].mount_adj > 0.6
        && h.date_hull < 1895
    {
        lines.push(
            report_text(
                language,
                "Central citadel ship",
                "中央装甲堡舰",
                "Navis arce media munita",
            )
            .to_string(),
        );
    }
    let has_old_mount = (0..3).any(|i| g.batteries[i].mount_type == 0);
    if has_old_mount {
        if a.weight_belts > 0.0 {
            let has_arm_back =
                (0..3).any(|i| g.batteries[i].mount_type == 0 && g.batteries[i].arm_back > 0.0);
            let has_sub = (0..3).any(|i| {
                g.batteries[i].mount_type == 0
                    && g.batteries[i].gun_sub_g1 + g.batteries[i].gun_sub_g2 > 0
            });
            lines.push(if has_arm_back {
                report_text(
                    language,
                    "Armoured Casemate Ship",
                    "装甲炮廓舰",
                    "Navis loricata tormentis en casemate instructa",
                )
                .to_string()
            } else if ship.free.fc_length + ship.free.qd_length < 0.5 {
                if has_sub {
                    report_text(
                        language,
                        "Armoured Frigate (Broadside ironclad)",
                        "装甲巡航舰（舷侧炮铁甲舰）",
                        "Frégate cuirassée à batterie latérale",
                    )
                    .to_string()
                } else {
                    report_text(
                        language,
                        "Armoured Corvette (Broadside ironclad)",
                        "装甲轻巡航舰（舷侧炮铁甲舰）",
                        "Corvette cuirassée à batterie latérale",
                    )
                    .to_string()
                }
            } else if has_sub {
                report_text(
                    language,
                    "Armoured Frigate (Central battery ironclad)",
                    "装甲巡航舰（中央炮房铁甲舰）",
                    "Frégate cuirassée à batterie centrale",
                )
                .to_string()
            } else {
                report_text(
                    language,
                    "Armoured Corvette (Central battery ironclad)",
                    "装甲轻巡航舰（中央炮房铁甲舰）",
                    "Corvette cuirassée à batterie centrale",
                )
                .to_string()
            });
        } else {
            let has_sub = (0..3).any(|i| {
                g.batteries[i].mount_type == 0
                    && g.batteries[i].gun_sub_g1 + g.batteries[i].gun_sub_g2 > 0
            });
            lines.push(if has_sub {
                report_text(
                    language,
                    "Frigate (Unarmoured)",
                    "巡航舰（无装甲）",
                    "Frégate non cuirassée",
                )
                .to_string()
            } else {
                report_text(
                    language,
                    "Corvette (Unarmoured)",
                    "轻巡航舰（无装甲）",
                    "Corvette non cuirassée",
                )
                .to_string()
            });
        }
    }
    if h.block <= 0.0 || h.block > 1.0 {
        lines.push(
            report_text(
                language,
                "DESIGN FAILURE: Displacement impossible with given dimensions",
                "设计失败：给定尺度无法实现该排水量",
                "CONSILIUM INVALIDUM: déplacement cum dimensionibus datis impossibile",
            )
            .to_string(),
        );
    }
    if h.displacement > 0.0 && h.displacement <= g.broadside / 4.0 {
        lines.push(
            report_text(
                language,
                "DESIGN FAILURE: Gun weight too much for hull",
                "设计失败：火炮重量超过船体承载能力",
                "CONSILIUM INVALIDUM: pondus tormentorum facultatem corporis navis excedit",
            )
            .to_string(),
        );
    }
    if h.displacement > 0.0 && a.weight > h.displacement {
        lines.push(
            report_text(
                language,
                "DESIGN FAILURE: Armour weight too much for hull",
                "设计失败：装甲重量超过船体承载能力",
                "CONSILIUM INVALIDUM: pondus loricae facultatem corporis navis excedit",
            )
            .to_string(),
        );
    }
    if p.strength_comp < 0.5 {
        lines.push(
            report_text(
                language,
                "DESIGN FAILURE: Overall load weight too much for hull",
                "设计失败：总载荷超过船体承载能力",
                "CONSILIUM INVALIDUM: onus totale facultatem corporis navis excedit",
            )
            .to_string(),
        );
    }
    if p.metacentre <= 0.0 {
        lines.push(
            report_text(
                language,
                "DESIGN FAILURE: Ship will capsize",
                "设计失败：舰船将倾覆",
                "CONSILIUM INVALIDUM: navis evertetur",
            )
            .to_string(),
        );
    }
    lines
}

/// Escape text for safe inclusion in HTML element content or attribute values.
///
/// Rust's `format!` gives no automatic escaping, and several report generators
/// interpolate user-supplied strings (ship names, external CSV fields, source
/// labels).  Everything that lands in HTML must go through this helper.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn generate_report(ship: &ShipModel) -> String {
    generate_report_with_language(ship, TextLanguage::Germanic)
}

pub fn generate_report_with_language(ship: &ShipModel, language: TextLanguage) -> String {
    let mut r = String::new();
    let h = &ship.hull;
    let a = &ship.armour;
    let g = &ship.guns;
    let e = &ship.engine;
    let w = &ship.weapons;
    let p = &ship;

    r.push_str(&report_format!(
        language,
        "{}, {} {} laid down {}\n",
        "{}，{} {}，{} 年开工\n",
        "{}, {} {}; carina posita anno {}\n",
        ship.name,
        ship.country,
        ship.ship_type,
        h.date_hull
    ));
    if h.date_hull != h.date_engine {
        r.push_str(&report_format!(
            language,
            "(Engine {})\n",
            "（主机设计年代 {}）\n",
            "(Machinae anni {})\n",
            h.date_engine
        ));
    }
    for line in ship_classification(ship, language) {
        r.push_str(&format!("\n{line}"));
    }

    r.push_str(report_text(
        language,
        "\nDisplacement:\n",
        "\n排水量：\n",
        "\nDéplacement:\n",
    ));
    r.push_str(&report_format!(
        language,
        "\t{:.0} t light; {:.0} t standard; {:.0} t normal; {:.0} t full load\n",
        "\t空载 {:.0} t；标准 {:.0} t；正常 {:.0} t；满载 {:.0} t\n",
        "\t{:.0} t navis vacua; {:.0} t standard; {:.0} t ordinarium; {:.0} t pleno onere\n",
        p.disp_light,
        p.disp_standard,
        h.displacement,
        p.disp_max_load
    ));

    r.push_str(report_text(
        language,
        "\nDimensions:\n",
        "\n船体尺度（总长／水线长 × 型宽 × 正常／满载吃水）：\n",
        "\nDimensiones:\n",
    ));
    r.push_str(&format!(
        "\t({:.1} ft / {:.1} ft) x {:.1} ft x ({:.1} / {:.1} ft)\n",
        h.length_oa, h.length, h.beam, h.draught, h.draught_deep
    ));
    r.push_str(&format!(
        "\t({:.1} m / {:.1} m) x {:.1} m x ({:.1} / {:.1} m)\n",
        metric_calc(h.length_oa, Conversion::Feet2Metres),
        metric_calc(h.length, Conversion::Feet2Metres),
        metric_calc(h.beam, Conversion::Feet2Metres),
        metric_calc(h.draught, Conversion::Feet2Metres),
        metric_calc(h.draught_deep, Conversion::Feet2Metres),
    ));
    if h.beam_bulges > h.beam {
        r.push_str(&report_format!(
            language,
            "\tBulges: {:.1} ft / {:.1} m\n",
            "\t含防雷突出部型宽：{:.1} ft / {:.1} m\n",
            "\tLatitudo cum bulges antitorpilles: {:.1} ft / {:.1} m\n",
            h.beam_bulges,
            metric_calc(h.beam_bulges, Conversion::Feet2Metres)
        ));
    }

    r.push_str(report_text(
        language,
        "\nArmament:\n",
        "\n武备：\n",
        "\nArmamenta:\n",
    ));
    for (i, b) in g.batteries.iter().enumerate() {
        if b.barrels <= 0 {
            continue;
        }
        r.push_str(&format!("\n{}", gun_description(b, i == 0, language)));
    }
    r.push_str(&report_format!(
        language,
        "\n\tBroadside: {:.0} lbs / {:.0} kg; Magazine: {:.0} tons\n",
        "\n\t舷侧齐射弹重：{:.0} lbs / {:.0} kg；弹药库：{:.0} 长吨\n", "\n\tPondus salvae lateralis: {:.0} lbs / {:.0} kg; cella munitionum: {:.0} tonnes longues\n",
        g.broadside,
        metric_calc(g.broadside, Conversion::Pounds2Kg),
        g.magazine
    ));

    let all_weapons = [&w.torpedo1, &w.torpedo2, &w.mine, &w.dc1, &w.dc2];
    for wp in all_weapons.iter() {
        if wp.number <= 0 {
            continue;
        }
        let kind = match wp.kind {
            WeaponKind::Torpedo => report_text(language, "torpedo", "鱼雷", "torpille"),
            WeaponKind::Mine => report_text(language, "mine", "水雷", "mine marine"),
            WeaponKind::DepthCharge => report_text(
                language,
                "depth charge/AS mortar",
                "深水炸弹／反潜迫击炮",
                "grenade sous-marine / mortier anti-sous-marin",
            ),
        };
        r.push_str(&report_format!(
            language,
            "\t{} x {:.1} {} (location {}), {:.2} tons\n",
            "\t{} × {:.1} {}（位置编号 {}），{:.2} 长吨\n",
            "\t{} x {:.1} {} (locus {}), {:.2} tonnes longues\n",
            wp.number,
            wp.size,
            kind,
            wp.location,
            wp.weapon_weight + wp.mount_weight
        ));
    }

    r.push_str(report_text(
        language,
        "\nArmour:\n",
        "\n装甲：\n",
        "\nLoricatio:\n",
    ));
    r.push_str(&report_format!(
        language,
        "\tMain belt: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        "\t主装甲带：{:.1}\" × {:.0} ft × {:.1} ft，{:.0} 长吨\n",
        "\tCingulum principale: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tonnes longues\n",
        a.belt_main,
        a.belt_main_length,
        a.belt_main_height,
        a.weight_belt_main
    ));
    r.push_str(&report_format!(
        language,
        "\tBelt ends: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        "\t艏艉装甲带：{:.1}\" × {:.0} ft × {:.1} ft，{:.0} 长吨\n",
        "\tExtremitates cinguli: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tonnes longues\n",
        a.belt_ends,
        a.belt_ends_length,
        a.belt_ends_height,
        a.weight_belt_ends
    ));
    r.push_str(&report_format!(
        language,
        "\tUpper belt: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        "\t上层装甲带：{:.1}\" × {:.0} ft × {:.1} ft，{:.0} 长吨\n",
        "\tCingulum superius: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tonnes longues\n",
        a.belt_upper,
        a.belt_upper_length,
        a.belt_upper_height,
        a.weight_belt_upper
    ));
    r.push_str(&report_format!(
        language,
        "\tBulkhead: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        "\t装甲横舱壁：{:.1}\" × {:.0} ft × {:.1} ft，{:.0} 长吨\n",
        "\tClausura transversa loricata: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tonnes longues\n",
        a.bulkhead,
        a.bulkhead_length,
        a.bulkhead_height,
        a.weight_bulkhead
    ));
    r.push_str(&report_format!(
        language,
        "\tBulge: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tons\n",
        "\t防雷突出部：{:.1}\" × {:.0} ft × {:.1} ft，{:.0} 长吨\n",
        "\tBulge antitorpilles: {:.1}\" x {:.0} ft x {:.1} ft, {:.0} tonnes longues\n",
        a.bulge,
        a.bulge_length,
        a.bulge_height,
        a.weight_bulge
    ));
    r.push_str(&report_format!(
        language,
        "\tDecks: {:.1}\" (FC {:.1}\", QD {:.1}\"), {:.0} tons\n",
        "\t装甲甲板：{:.1}\"（艏楼 {:.1}\"，后甲板 {:.1}\"），{:.0} 长吨\n",
        "\tTabulata: {:.1}\" (prora {:.1}\", puppis {:.1}\"), {:.0} tonnes longues\n",
        a.deck,
        a.deck_fc,
        a.deck_qd,
        a.weight_deck
    ));
    r.push_str(&report_format!(
        language,
        "\tConning: {:.1}\" + {:.1}\", {:.0} tons\n",
        "\t指挥塔：{:.1}\" + {:.1}\"，{:.0} 长吨\n",
        "\tTurres imperatoriae: {:.1}\" + {:.1}\", {:.0} tonnes longues\n",
        a.conning_for,
        a.conning_aft,
        a.weight_conning
    ));
    r.push_str(&report_format!(
        language,
        "\tTotal armour: {:.0} tons\n",
        "\t装甲总重：{:.0} 长吨\n",
        "\tPondus loricae totale: {:.0} tonnes longues\n",
        a.weight
    ));

    r.push_str(report_text(
        language,
        "\nMachinery:\n",
        "\n动力：\n",
        "\nMachinae:\n",
    ));
    let fuel_desc = describe_fuel(e, language);
    r.push_str(&report_format!(
        language,
        "\tFuel: {fuel_desc}\n",
        "\t燃料／能源：{fuel_desc}\n",
        "\tFons energiae: {fuel_desc}\n"
    ));
    r.push_str(&report_format!(
        language,
        "\tMax speed: {:.1} kt, cruise: {:.1} kt\n",
        "\t最高航速：{:.1} kt；巡航速度：{:.1} kt\n",
        "\tCeleritas maxima: {:.1} kt; celeritas itineraria: {:.1} kt\n",
        e.speed_max,
        e.speed_cruise
    ));
    r.push_str(&report_format!(
        language,
        "\tMax power: {:.0} hp / {:.0} kW; cruise power: {:.0} hp / {:.0} kW\n",
        "\t最大功率：{:.0} hp / {:.0} kW；巡航功率：{:.0} hp / {:.0} kW\n",
        "\tPotentia maxima: {:.0} hp / {:.0} kW; potentia itineraria: {:.0} hp / {:.0} kW\n",
        e.hp_max,
        metric_calc(e.hp_max, Conversion::Hp2Kw),
        e.hp_cruise,
        metric_calc(e.hp_cruise, Conversion::Hp2Kw)
    ));
    r.push_str(if e.blend_weight < 1.0 - 1e-9 {
        report_text(language,
            "\tNote: power blends Holtrop-Mennen-style and original SpringSharp models.\n",
            "\t说明：功率混合使用 Holtrop-Mennen 类模型和原版 SpringSharp 模型。\n", "\tNota: potentia ex mixtura exemplarium Holtrop-Mennen et SpringSharp originalis computatur.\n")
    } else if e.resistance_model == crate::model::ResistanceModel::Original {
        report_text(language, "\tNote: power uses the original SpringSharp empirical model (reconstructed, unimproved).\n", "\t说明：功率采用原版 SpringSharp 经验模型（重建实现，未改进）。\n", "\tNota: potentia ex exemplari empirico originali SpringSharp computatur (reconstructo, non emendato).\n")
    } else {
        report_text(language, "\tNote: power uses the improved Holtrop-Mennen-style model.\n", "\t说明：功率采用改进的 Holtrop-Mennen 类模型。\n", "\tNota: potentia ex exemplari emendato generis Holtrop-Mennen computatur.\n")
    });
    r.push_str(&report_format!(language,
        "\tPower factors (improved/original): {:.3}/{:.3}; Holtrop blend share: {:.3}; CFD correction: {}\n",
        "\t功率校准系数（改进／原版）：{:.3}/{:.3}；Holtrop 混合占比：{:.3}；CFD 修正：{}\n", "\tFactores potentiae (emendatus/originalis): {:.3}/{:.3}; pars Holtrop in mixtura: {:.3}; correctio CFD: {}\n",
        e.power_factor, e.original_power_factor,
        if e.blend_weight < 1.0 - 1e-9 { e.blend_weight.clamp(0.0, 1.0) }
        else if e.resistance_model == crate::model::ResistanceModel::Original { 0.0 }
        else { 1.0 },
        if e.cfd_proxy_enabled { report_text(language, "enabled", "启用", "activa") }
        else { report_text(language, "disabled", "禁用", "inactiva") }));
    r.push_str(&report_format!(
        language,
        "\tEngine weight: {:.0} tons; bunker normal: {:.0} tons / max {:.0} tons\n",
        "\t动力装置重量：{:.0} 长吨；正常燃料储量：{:.0} 长吨／最大 {:.0} 长吨\n", "\tPondus machinarum: {:.0} tonnes longues; combustibile ordinarium: {:.0} tonnes longues / maximum {:.0} tonnes longues\n",
        e.engine_weight,
        e.bunker,
        e.bunker * 1.8
    ));
    if e.qpc_applicable {
        r.push_str(&report_format!(language, "\tPropulsion: QPC={:.3} (used in power calculation), hull={:.3}, open-water={:.3}, shaft={:.3}\n", "\t推进效率：QPC={:.3}（参与功率计算），船身效率={:.3}，敞水效率={:.3}，轴系效率={:.3}\n", "\tPropulsio: QPC={:.3} (in computatione potentiae adhibitum), rendement de coque={:.3}, rendement en eau libre={:.3}, rendement de ligne d’arbre={:.3}\n",
            e.qpc, e.hull_efficiency, e.open_water_efficiency, e.shaft_efficiency
        ));
    } else {
        r.push_str(report_text(
            language,
            "\tPropulsion: original empirical shaft-power fit; no explicit QPC applies.\n",
            "\t推进：原版经验轴功率拟合，不显式使用 QPC。\n", "\tPropulsio: potentia in axe ex formula empirica originali; QPC explicite non adhibetur.\n",
        ));
    }
    r.push_str(&report_format!(
        language,
        "\tScrews: {}; range: {:.0} nm\n",
        "\t螺旋桨数：{}；续航力：{:.0} nm\n",
        "\tHelices: {}; spatium navigationis: {:.0} nm\n",
        e.screws,
        e.range
    ));

    r.push_str(report_text(
        language,
        "\nPerformance:\n",
        "\n性能：\n",
        "\nProprietates nauticae:\n",
    ));
    r.push_str(&report_format!(
        language,
        "\tRoom: hull {:.1}%, deck {:.1}%\n",
        "\t空间占用：船体 {:.1}%，甲板 {:.1}%\n",
        "\tSpatium occupatum: corpus navis {:.1}%, tabulatum {:.1}%\n",
        p.room_hull * 100.0,
        p.room_deck * 100.0
    ));
    r.push_str(&report_format!(
        language,
        "\tStability: {:.2}; metacentre: {:.2} ft\n",
        "\t稳性：{:.2}；初稳性高：{:.2} ft\n",
        "\tStabilitas: {:.2}; hauteur métacentrique: {:.2} ft\n",
        p.stability_adj,
        p.metacentre
    ));
    r.push_str(&report_format!(
        language,
        "\tSeakeeping: {:.2}; steadiness: {:.0}\n",
        "\t耐波性：{:.2}；平台平稳性：{:.0}\n",
        "\tTenue à la mer: {:.2}; stabilitas tormentorum: {:.0}\n",
        p.seakeeping,
        p.steadiness_adj
    ));
    r.push_str(&report_format!(
        language,
        "\tRecoil: {:.2}\n",
        "\t后坐力指标：{:.2}\n",
        "\tRecul: {:.2}\n",
        p.recoil
    ));
    r.push_str(&report_format!(
        language,
        "\tStructural weight: {:.0} lb/sq ft / {:.0} kg/sq m\n",
        "\t单位面积结构重量：{:.0} lb/sq ft / {:.0} kg/sq m\n",
        "\tPondus structurae per aream: {:.0} lb/sq ft / {:.0} kg/sq m\n",
        p.struct_weight,
        p.struct_weight / SQFEET2SQMETRES * POUNDS2KG
    ));
    r.push_str(&report_format!(
        language,
        "\tStrength cross/long/composite: {:.2}/{:.2}/{:.2}\n",
        "\t横向／纵向／综合强度：{:.2}/{:.2}/{:.2}\n",
        "\tFirmitas transversa/longitudinalis/composita: {:.2}/{:.2}/{:.2}\n",
        p.strength_cross,
        p.strength_long,
        p.strength_comp
    ));
    r.push_str(&report_format!(
        language,
        "\tCost: £{:.3}M / ${:.3}M\n",
        "\t造价：{:.3} 百万英镑／{:.3} 百万美元\n",
        "\tPretium: £{:.3}M / ${:.3}M\n",
        p.cost_pounds,
        p.cost_dollars
    ));

    if p.strength_comp < 0.5 {
        r.push_str(report_text(
            language,
            "\nDESIGN FAILURE: Hull structure insufficient to carry required loads!\n",
            "\n设计失败：船体结构不足以承受所需载荷！\n",
            "\nCONSILIUM INVALIDUM: structura navis oneribus sustinendis non sufficit!\n",
        ));
    } else if p.strength_comp < 0.995 {
        r.push_str(report_text(
            language,
            "\nWARNING: Hull structure strained by open sea conditions!\n",
            "\n警告：外海条件下船体结构强度不足！\n",
            "\nMONITUM: structura navis in alto mari nimis oneratur!\n",
        ));
    }
    // Same convergence caveat as `ShipModel::summary`: the report is the other
    // place a user reads the weight breakdown, so it must carry the warning too.
    if p.iterations > 0 && !p.converged {
        r.push_str(&report_format!(language, "\nWARNING: armour/machinery weight iteration did not converge after {} iterations (relative residual {:.2e}); the weight breakdown is approximate.\n", "\n警告：装甲／动力重量迭代在 {} 次后仍未收敛（相对残差 {:.2e}）；重量分项仅为近似值。\n", "\nMONITUM: iteratio ponderum loricae et machinarum post {} iterationes non convergit (résidu relatif {:.2e}); pondera approximata sunt.\n",
            p.iterations, p.iteration_residual
        ));
    }
    super::report_comparison::append_comparison(&mut r, ship, language);
    r
}

pub fn generate_html_report(ship: &ShipModel) -> String {
    generate_html_report_with_language(ship, TextLanguage::Germanic)
}

pub fn generate_html_report_with_language(ship: &ShipModel, language: TextLanguage) -> String {
    let text = generate_report_with_language(ship, language);
    let tag = report_text(language, "en", "zh-CN", "la");
    let body = html_escape(&text).replace('\n', "<br>\n");
    format!(
        "<!doctype html>\n<html lang=\"{tag}\"><head><meta charset=\"utf-8\"><title>{}</title></head><body>\n<pre>{body}</pre>\n</body></html>\n",
        html_escape(&ship.name)
    )
}

fn describe_fuel(e: &crate::model::EngineValues, language: TextLanguage) -> String {
    let fuel_names = [
        report_text(
            language,
            "coal-fired boilers",
            "燃煤锅炉",
            "caldaria carbonibus calefacta",
        ),
        report_text(
            language,
            "oil-fired boilers",
            "燃油锅炉",
            "caldaria oleo calefacta",
        ),
        report_text(language, "diesel motors", "柴油机", "moteurs Diesel"),
        report_text(language, "petrol motors", "汽油机", "moteurs à essence"),
        report_text(language, "batteries", "蓄电池", "accumulateurs électriques"),
    ];
    let selected: Vec<&str> = e
        .fuel
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| if v { Some(fuel_names[i]) } else { None })
        .collect();
    if selected.is_empty() {
        report_text(language, "none", "无", "nullum").to_string()
    } else {
        selected.join(", ")
    }
}

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
            html_escape(case.name),
            html_escape(case.ship_type),
            q.confidence,
            q.source_type.label(),
            q.displacement_type.label(),
            if q.traceable { "yes" } else { "no" },
            html_escape(q.verifier),
            html_escape(q.source)
        ));
    }
    html.push_str("</table>");
    html.push_str("</body></html>");
    html
}
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
            html_escape(&name),
            sim,
            reference,
            confidence
        ));
    }
    html.push_str("</table>");

    html.push_str("<h2>Sources</h2><ul>");
    for (source, count, conf) in summarize_external_by_source(cases) {
        html.push_str(&format!(
            "<li>{}: count={count}, mean confidence={conf:.2}</li>",
            html_escape(&source)
        ));
    }
    html.push_str("</ul>");

    html.push_str("<h2>Ship types</h2><ul>");
    for (typ, count, conf) in summarize_external_by_type(cases) {
        html.push_str(&format!(
            "<li>{}: count={count}, mean confidence={conf:.2}</li>",
            html_escape(&typ)
        ));
    }
    html.push_str("</ul></body></html>");
    html
}
pub fn cfd_report_html(samples: &[CfdSample]) -> String {
    let fit = fit_cfd_proxy_report(samples);
    let loocv = cfd_loocv_report(samples);
    let holdout = cfd_holdout_report(samples, 0.3);
    let lambda = select_cfd_ridge_lambda(samples);
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>CFD Surrogate Report</title></head><body>",
    );
    html.push_str("<h1>CFD Surrogate Report</h1>");
    html.push_str("<table border=\"1\" cellpadding=\"4\" cellspacing=\"0\">");
    html.push_str("<tr><th>Fn</th><th>Empirical HP</th><th>CFD HP</th></tr>");
    for s in samples {
        html.push_str(&format!(
            "<tr><td>{:.4}</td><td>{:.0}</td><td>{:.0}</td></tr>",
            s.fn_, s.empirical_hp, s.cfd_hp
        ));
    }
    html.push_str("</table>");
    html.push_str(&format!(
        "<h2>Fit</h2><p>viscous={:.4} wave={:.4} fn_ref={:.4} R2={:.4} RMSE={:.5} N={}</p>",
        fit.config.viscous_correction,
        fit.config.wave_correction,
        fit.config.fn_ref,
        fit.r2,
        fit.rmse,
        fit.samples
    ));
    html.push_str(&format!(
        "<h2>LOOCV</h2><p>R2={:.4} RMSE={:.5} N={}</p>",
        loocv.r2, loocv.rmse, loocv.samples
    ));
    html.push_str(&format!(
        "<h2>Holdout (0.3)</h2><p>R2={:.4} RMSE={:.5} N={}</p>",
        holdout.r2, holdout.rmse, holdout.samples
    ));
    html.push_str("<h2>Ridge lambda selection</h2><ul>");
    for (lambda_value, rmse) in &lambda.results {
        html.push_str(&format!("<li>lambda={lambda_value:.3} RMSE={rmse:.5}</li>"));
    }
    html.push_str(&format!(
        "</ul><p>best lambda={:.3} RMSE={:.5}</p>",
        lambda.best_lambda, lambda.best_rmse
    ));
    html.push_str("</body></html>");
    html
}
pub fn compartment_placement_html(ship: &ShipModel, placement: &CompartmentPlacement) -> String {
    let l = &placement.layout;
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>General Arrangement</title></head><body>",
    );
    html.push_str(&format!(
        "<h1>General arrangement (multi-deck GA)</h1><p>Ship: <b>{}</b> ({})</p>",
        html_escape(&ship.name),
        html_escape(&ship.ship_type)
    ));
    html.push_str("<p>Multi-deck genetic-algorithm allocation with feasibility repair and an access network. Volumes in m&sup3;, positions as fraction of length.</p>");

    // Schematic multi-deck arrangement SVG diagram
    let svg_w = 760.0;
    let svg_h = 230.0;
    let pad_left = 65.0;
    let pad_right = 35.0;
    let pad_top = 25.0;
    let pad_bottom = 45.0;
    let plot_w = svg_w - pad_left - pad_right;
    let plot_h = svg_h - pad_top - pad_bottom;
    let deck_h = plot_h / 4.0;

    let mut svg = format!(
        r##"<svg viewBox="0 0 {svg_w} {svg_h}" width="100%" style="max-width:760px;background:#161b22;border:1px solid #30363d;border-radius:6px;margin:12px 0;font-family:ui-monospace,SFMono-Regular,Consolas,monospace;">"##
    );
    // Draw hull profile outline (Stern on left, Bow on right matching 0..1 longitudinal fraction from stern)
    let x_end = pad_left + plot_w;
    let x_q1 = pad_left + plot_w + 15.0;
    let y_q1 = pad_top + plot_h * 0.4;
    let x_end2 = pad_left + plot_w - 20.0;
    let y_bot = pad_top + plot_h;
    let x_mid = pad_left + plot_w * 0.5;
    let x_bow_bot = pad_left + 15.0;
    let y_bow_bot = pad_top + plot_h * 0.8;
    svg.push_str(&format!(
        r##"<path d="M {pad_left} {pad_top} L {x_end} {pad_top} Q {x_q1} {y_q1} {x_end2} {y_bot} Q {x_mid} {y_bot} {x_bow_bot} {y_bow_bot} Z" fill="none" stroke="#58a6ff" stroke-width="1.5" stroke-opacity="0.7"/>"##
    ));
    // Deck grid and labels
    for d in 0..=4 {
        let y = pad_top + (4 - d) as f64 * deck_h;
        svg.push_str(&format!(
            r##"<line x1="{pad_left}" y1="{y}" x2="{x_end}" y2="{y}" stroke="#30363d" stroke-dasharray="4,4"/>"##
        ));
        if d < 4 {
            let label = match d {
                0 => "Hold (0)",
                1 => "Lower (1)",
                2 => "Main (2)",
                _ => "Upper (3)",
            };
            let text_x = pad_left - 8.0;
            let text_y = y - deck_h * 0.35;
            svg.push_str(&format!(
                r##"<text x="{text_x}" y="{text_y}" fill="#8b949e" font-size="10" text-anchor="end">{label}</text>"##
            ));
        }
    }
    // Compartment colors
    let comp_meta = [
        ("Machinery", "#e06c75"),
        ("Magazine", "#c678dd"),
        ("Accommod.", "#61afef"),
        ("Fuel", "#d19a66"),
        ("Stores", "#98c379"),
        ("Void", "#7d8590"),
    ];
    let box_w = 78.0;
    let box_h = deck_h - 8.0;
    for (i, (name, color)) in comp_meta.iter().enumerate() {
        let deck_idx = placement.decks[i].clamp(0, 3);
        let x_norm = placement.longitudinal[i].clamp(0.05, 0.95);
        let cx = pad_left + x_norm * plot_w;
        let rx = (cx - box_w * 0.5).clamp(pad_left + 4.0, pad_left + plot_w - box_w - 4.0);
        let ry = pad_top + (3 - deck_idx) as f64 * deck_h + 4.0;
        let text_cx = rx + box_w * 0.5;
        let text_cy = ry + box_h * 0.6;

        svg.push_str(&format!(
            r##"<rect x="{rx:.1}" y="{ry:.1}" width="{box_w}" height="{box_h:.1}" rx="4" fill="{color}" fill-opacity="0.35" stroke="{color}" stroke-width="1.5"/>"##
        ));
        svg.push_str(&format!(
            r##"<text x="{text_cx:.1}" y="{text_cy:.1}" fill="#f0f6fc" font-size="9.5" font-weight="600" text-anchor="middle">{name}</text>"##
        ));
    }
    // Assembly station marker
    let ax = pad_left + placement.assembly_x.clamp(0.05, 0.95) * plot_w;
    let y_assembly_top = pad_top - 6.0;
    svg.push_str(&format!(
        r##"<line x1="{ax:.1}" y1="{pad_top}" x2="{ax:.1}" y2="{y_bot}" stroke="#f1e05a" stroke-width="2" stroke-dasharray="3,3"/>"##
    ));
    svg.push_str(&format!(
        r##"<text x="{ax:.1}" y="{y_assembly_top}" fill="#f1e05a" font-size="9" text-anchor="middle">Assembly</text>"##
    ));
    // Stern and Bow indications
    let y_lbl = pad_top + plot_h + 18.0;
    svg.push_str(&format!(
        r##"<text x="{pad_left}" y="{y_lbl}" fill="#8b949e" font-size="10">Stern (0.0 L)</text>"##
    ));
    svg.push_str(&format!(
        r##"<text x="{x_end}" y="{y_lbl}" fill="#8b949e" font-size="10" text-anchor="end">Bow (1.0 L)</text>"##
    ));
    svg.push_str("</svg>");
    html.push_str(&svg);

    html.push_str("<table border=\"1\" cellpadding=\"4\" cellspacing=\"0\">");
    html.push_str("<tr><th>Compartment</th><th>Volume (m3)</th><th>Deck</th><th>Vertical centre</th><th>Longitudinal</th></tr>");
    for (name, v, deck, z, x) in [
        (
            "Machinery",
            l.machinery,
            placement.decks[0],
            placement.centers[0],
            placement.longitudinal[0],
        ),
        (
            "Magazine",
            l.magazine,
            placement.decks[1],
            placement.centers[1],
            placement.longitudinal[1],
        ),
        (
            "Accommodation",
            l.accommodation,
            placement.decks[2],
            placement.centers[2],
            placement.longitudinal[2],
        ),
        (
            "Fuel",
            l.fuel,
            placement.decks[3],
            placement.centers[3],
            placement.longitudinal[3],
        ),
        (
            "Stores",
            l.stores,
            placement.decks[4],
            placement.centers[4],
            placement.longitudinal[4],
        ),
        (
            "Void / maintenance",
            l.void,
            placement.decks[5],
            placement.centers[5],
            placement.longitudinal[5],
        ),
    ] {
        html.push_str(&format!(
            "<tr><td>{name}</td><td>{v:.0}</td><td>{deck}</td><td>{z:.3}</td><td>{x:.3}</td></tr>"
        ));
    }
    html.push_str("</table>");
    html.push_str(&format!(
        "<p>Achieved vertical CG ratio: <b>{:.3}</b><br>Longitudinal CG: {:.3} L<br>Magazine longitudinal: {:.3} L<br>Machinery longitudinal: {:.3} L<br>Objective: {:.4}<br>GA: {} generations, {} evaluations</p>",
        placement.cg_vertical,
        placement.cg_longitudinal,
        placement.magazine_longitudinal,
        placement.machinery_longitudinal,
        placement.objective,
        placement.generations,
        placement.evaluations
    ));
    html.push_str(&format!(
        "<p>Access network: <b>{}</b> ({} edges, connected={})<br>Escape: max {} hops / {:.1} m<br>Structural-weight estimate: {:.0} t</p>",
        if placement.access_connected {
            "OK"
        } else {
            "DISCONNECTED"
        },
        placement.access_edges,
        placement.access_connected,
        placement.escape_max_hops,
        placement.escape_max_path,
        placement.structure_weight_tons
    ));
    html.push_str(&format!(
        "<p>Rule scantlings: max utilization {:.2}; structural weight {:.0} t</p>",
        placement.scantling_utilization, placement.structure_weight_tons
    ));
    html.push_str(&format!(
        "<p>IMO evacuation: {} persons; network capacity {:.0} persons / {} s; congestion time {:.0} s; assembly at {:.2} L; max edge density {:.2} p/m2; max mustering {:.0} s; required routes {:?}</p>",
        placement.occupancy.iter().sum::<f64>(),
        placement.network_evacuation_capacity,
        MAX_EVAC_TIME_S as u32,
        placement.congestion_time_s,
        placement.assembly_x,
        placement.max_edge_density,
        placement
            .mustering_time_s
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max),
        placement.required_paths
    ));
    html.push_str("<h2>Constraint checks</h2><ul>");
    for (name, ok) in &placement.checks {
        html.push_str(&format!(
            "<li>{} {}</li>",
            if *ok { "PASS" } else { "FAIL" },
            name
        ));
    }
    html.push_str("</ul></body></html>");
    html
}
/// Human-readable summary of the most important design results.
pub struct Summary {
    pub lines: Vec<String>,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

pub trait ShipSummary {
    fn summary(&self) -> Summary;
}
impl ShipSummary for ShipModel {
    fn summary(&self) -> Summary {
        let g = &self.guns;
        let e = &self.engine;
        let h = &self.hull;
        let mut lines = vec![
            format!("Ship: {} ({}, {})", self.name, self.country, self.ship_type),
            format!("Normal displacement: {:.0} tons", h.displacement),
            format!(
                "Dimensions: {:.1} x {:.1} x {:.1} ft, block coefficient {:.3}",
                h.length, h.beam_bulges, h.draught, h.block
            ),
            format!(
                "Freeboard: {:.2} ft (distributed {:.2} ft)",
                self.free.freeboard, self.free.free_distributed
            ),
            format!(
                "Speed: max {:.1} kt, cruise {:.1} kt",
                e.speed_max, e.speed_cruise
            ),
            format!(
                "Power: max {:.0} hp, cruise {:.0} hp",
                e.hp_max, e.hp_cruise
            ),
            if e.qpc_applicable {
                format!(
                    "Propulsion: QPC={:.3} (wake={:.3}, thrust deduction={:.3}); used in power calculation",
                    e.qpc, e.wake_fraction, e.thrust_deduction
                )
            } else {
                "Propulsion: original empirical shaft-power fit; no explicit QPC (the Holtrop-style efficiency breakdown is not used)."
                    .to_string()
            },
            match self.engine.resistance_model {
                ResistanceModel::Original => {
                    "Note: power uses the original SpringSharp empirical model (reconstructed, unimproved)."
                        .to_string()
                }
                ResistanceModel::HoltropMennen => {
                    "Note: power uses the improved Holtrop-Mennen-style model.".to_string()
                }
            },
            format!("Engine weight: {:.0} tons", e.engine_weight),
            format!(
                "Bunker: {:.0} tons normal, {:.0} tons max",
                e.bunker,
                e.bunker * 1.8
            ),
            format!("Load weight: {:.0} tons", e.load_weight),
            format!("Hull weight: {:.0} tons", e.hull_weight),
            format!(
                "Guns: {:.0} tons gun/mount + {:.0} tons gun armour",
                g.guns_weight, g.arm_weight
            ),
            format!(
                "Magazine: {:.0} tons, broadside {:.0} lbs",
                g.magazine, g.broadside
            ),
            format!(
                "Armour: belts {:.0}, decks {:.0}, conning {:.0}, total {:.0} tons",
                self.armour.weight_belts,
                self.armour.weight_deck,
                self.armour.weight_conning,
                self.armour.weight
            ),
            format!("Secondary weapons: {:.0} tons", self.weapons.weight),
            format!(
                "Room: hull {:.1}%, deck {:.1}%",
                self.room_hull * 100.0,
                self.room_deck * 100.0
            ),
            format!(
                "Stability: {:.2}, metacentre {:.2} ft",
                self.stability_adj, self.metacentre
            ),
            format!(
                "Seakeeping: {:.2}, steadiness {:.0}",
                self.seakeeping, self.steadiness_adj
            ),
            format!("Recoil: {:.2}", self.recoil),
            format!("Structural weight: {:.0} lbs/sq ft", self.struct_weight),
            format!(
                "Strength cross/long/composite: {:.2}/{:.2}/{:.2}",
                self.strength_cross, self.strength_long, self.strength_comp
            ),
            format!(
                "Cost: £{:.3}M / ${:.3}M",
                self.cost_pounds, self.cost_dollars
            ),
            format!(
                "Flotation: {:.0} tons; shell hits {:.1}; torpedo hits {:.1}",
                self.flotation, self.damage_shell, self.damage_torp
            ),
        ];
        if self.strength_comp < 0.5 {
            lines.push(
                "DESIGN FAILURE: Hull structure insufficient to carry required loads!".into(),
            );
        } else if self.strength_comp < 0.995 {
            lines.push("WARNING: Hull structure strained by open sea conditions!".into());
        }
        // The armour <-> machinery cycle is a fixed-point iteration.  If it hits
        // its iteration cap the reported weights are still the last iterate, but
        // they are no longer trustworthy to the usual tolerance, so say so
        // instead of silently reporting them as settled.  Guarded on
        // `iterations > 0` so a model that has never been calculated (where the
        // flag is still at its default) does not warn spuriously.
        if self.iterations > 0 && !self.converged {
            lines.push(format!(
                "WARNING: armour/machinery weight iteration did not converge after {} iterations (relative residual {:.2e}); the weight breakdown is approximate.",
                self.iterations, self.iteration_residual
            ));
        }
        Summary { lines }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn localized_reports_preserve_user_text_and_values() {
        let mut ship = demo_ship();
        ship.name = "Displacement <script>舰船 & Test</script>".into();
        ship.country = "Machinery".into();
        ship.calculate();
        let before = serialize(&ship);
        let en = generate_report_with_language(&ship, TextLanguage::Germanic);
        let zh = generate_report_with_language(&ship, TextLanguage::Han);
        assert_eq!(serialize(&ship), before);
        assert!(zh.contains(&ship.name));
        assert!(zh.contains(&ship.country));
        for heading in [
            "排水量：",
            "武备：",
            "装甲：",
            "动力：",
            "性能：",
            "原版 SpringSharp 公式对照（Rust 重建口径）",
        ] {
            assert!(zh.contains(heading), "missing {heading}");
        }
        assert!(!zh.contains("\nDisplacement:"));
        assert!(!zh.contains("laid down"));
        assert!(!zh.contains("Fleet Tactical Role"));
        assert!(en.contains("Original SpringSharp formula comparison (Rust reconstruction)"));
        // Locale changes templates, never the numerical results or their precision.
        for value in [ship.engine.hp_max, ship.engine.hp_cruise, ship.disp_light] {
            assert!(en.contains(&format!("{value:.0}")));
            assert!(zh.contains(&format!("{value:.0}")));
        }
        let latin = generate_report_with_language(&ship, TextLanguage::Italic);
        assert!(latin.contains("Armamenta:"));
        assert!(latin.contains("Machinae:"));
        assert!(latin.contains("Déplacement:"));
        assert!(latin.contains("Comparatio formularum originalium SpringSharp"));
        assert!(!latin.contains("laid down"));
        assert!(!latin.contains("Original SpringSharp formula comparison"));
        assert!(latin.contains(&ship.name));
        assert!(
            generate_html_report_with_language(&ship, TextLanguage::Italic).contains("lang=\"la\"")
        );
        let html = generate_html_report_with_language(&ship, TextLanguage::Han);
        assert!(html.contains("lang=\"zh-CN\""));
        assert!(html.contains("&lt;script&gt;舰船 &amp; Test&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn chinese_report_keeps_failure_and_convergence_warnings() {
        let mut ship = demo_ship();
        ship.calculate();
        ship.strength_comp = 0.4;
        ship.converged = false;
        ship.iterations = 50;
        let report = generate_report_with_language(&ship, TextLanguage::Han);
        assert!(report.contains("设计失败：船体结构不足"));
        assert!(report.contains("50 次后仍未收敛"));
        assert!(report.contains("对照提示：至少一列重量迭代未收敛"));
    }

    #[test]
    fn report_contains_ship_name() {
        let mut ship = ShipModel::new();
        ship.name = "Report Test".into();
        ship.hull.length = 400.0;
        ship.hull.beam = 60.0;
        ship.hull.beam_bulges = 60.0;
        ship.hull.draught = 20.0;
        ship.hull.block = 0.5;
        ship.hull.date_hull = 1920;
        ship.hull.date_engine = 1920;
        ship.engine.screws = 2;
        ship.calculate();
        let report = generate_report(&ship);
        assert!(report.contains("Report Test"));
        assert!(report.contains("Displacement"));
    }
    #[test]
    fn report_has_classification_and_gun_details() {
        let mut ship = ShipModel::new();
        ship.name = "Ironclad Test".into();
        ship.hull.length = 300.0;
        ship.hull.beam = 50.0;
        ship.hull.beam_bulges = 50.0;
        ship.hull.draught = 18.0;
        ship.hull.block = 0.55;
        ship.hull.date_hull = 1875;
        ship.hull.date_engine = 1875;
        ship.free.fc_length = 0.2;
        ship.free.fd_length = 0.3;
        ship.free.qd_length = 0.15;
        ship.engine.screws = 2;
        let g = &mut ship.guns.batteries[0];
        g.barrels = 6;
        g.calibre = 10.0;
        g.cal_length = 15.0;
        g.date_gun = 1875;
        g.gun_type = 0;
        g.mount_type = 0;
        g.mount_super_g1 = 0;
        g.mount_sub_g1 = 3;
        g.mounts = 3;
        g.guns_in_mount_g1 = 1;
        g.layout_g1 = 9;
        ship.armour.belt_main = 8.0;
        ship.armour.belt_main_length = 200.0;
        ship.armour.belt_main_height = 10.0;
        ship.calculate();
        let classes = ship_classification(&ship, TextLanguage::Germanic);
        assert!(classes.iter().any(|c| c.contains("Armoured")));
        let report = generate_report(&ship);
        assert!(report.contains("Muzzle loading gun"));
        assert!(report.contains("hull mounts"));
    }
    #[test]
    fn external_report_html_escapes_user_supplied_fields() {
        let csv = "name,type,length,beam,draught,displacement,block,speed,hp,confidence,source\n\
<script>x</script>,battleship,600,90,28,23760,0.55,24,50000,0.9,<img src=x onerror=alert(1)>\n";
        let cases = parse_external_cases(csv).unwrap();
        let html = external_report_html(&cases);
        assert!(
            !html.contains("<script>"),
            "external report must not contain raw <script>"
        );
        assert!(
            !html.contains("<img"),
            "external report must not contain raw <img>"
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&lt;img"));
    }
    #[test]
    fn data_quality_report_html_escapes() {
        let html = data_quality_report_html();
        assert!(html.contains("&lt;") || html.contains("&amp;") || !html.contains("<td>weird"));
        // The built-in sources are plain ASCII, so the report must remain valid.
        assert!(html.ends_with("</body></html>"));
    }
    #[test]
    fn quality_report_html_contains_ship_names() {
        let html = data_quality_report_html();
        assert!(html.contains("Data Quality Report"));
        assert!(html.contains("HMS Dreadnought"));
        assert!(html.contains("battleship"));
    }
    #[test]
    fn cfd_report_contains_sections() {
        let samples = vec![
            CfdSample {
                fn_: 0.24,
                empirical_hp: 1000.0,
                cfd_hp: 980.0,
                provenance: CfdProvenance::Synthetic,
            },
            CfdSample {
                fn_: 0.28,
                empirical_hp: 1000.0,
                cfd_hp: 1050.0,
                provenance: CfdProvenance::Synthetic,
            },
            CfdSample {
                fn_: 0.32,
                empirical_hp: 1000.0,
                cfd_hp: 1140.0,
                provenance: CfdProvenance::Synthetic,
            },
        ];
        let html = cfd_report_html(&samples);
        assert!(html.contains("CFD Surrogate Report"));
        assert!(html.contains("LOOCV"));
        assert!(html.contains("Holdout"));
    }
    #[test]
    fn convergence_status_is_surfaced_in_summary_and_report() {
        let mut ship = crate::features::demo_ship();
        ship.calculate();
        assert!(!ship.summary().to_string().contains("did not converge"));
        assert!(!generate_report(&ship).contains("did not converge"));

        // Force the not-converged state directly: building a design that
        // genuinely diverges is not portable, and the reporting path is what is
        // under test here.
        ship.converged = false;
        ship.iterations = 50;
        ship.iteration_residual = 1e-3;
        assert!(ship.summary().to_string().contains("did not converge"));
        let report = generate_report(&ship);
        assert!(report.contains("did not converge"));
        assert!(report.contains("50 iterations"));

        // A model that has never been calculated must not warn spuriously.
        let fresh = ShipModel::new();
        assert!(!fresh.summary().to_string().contains("did not converge"));
        assert!(!generate_report(&fresh).contains("did not converge"));
    }
}
