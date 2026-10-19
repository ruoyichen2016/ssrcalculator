//! Shared option catalogs for discrete `ShipModel` fields.
//!
//! The index of each label is the stored integer value. Labels follow the
//! original SpringSharp 3.0 WinForms combo boxes so the native GUI, the web
//! workbench and reports all describe the same value the same way.

use crate::model::WeaponKind;
use crate::presentation::fonts::TextLanguage;

pub const BOW_TYPES: &[&str] = &[
    "Normal bow",
    "Bulbous bow - straight",
    "Bulbous bow - forward",
    "Ram bow",
];

pub const BOW_TYPES_HAN: &[&str] = &[
    "常规型船艏 (Normal)",
    "球鼻艏 - 直线型 (Bulbous straight)",
    "球鼻艏 - 前伸型 (Bulbous forward)",
    "撞角艏 (Ram bow)",
];

pub const STERN_TYPES: &[&str] = &["Cruiser stern", "Transom stern", "Round stern"];
pub const STERN_TYPES_HAN: &[&str] = &["巡洋舰艉 (Cruiser)", "方艉 (Transom)", "圆艉 (Round)"];

pub const GUN_TYPES: &[&str] = &[
    "Muzzle loading gun",
    "Breech loading gun",
    "Quick firing gun",
    "Anti-air gun",
    "Dual purpose gun",
    "Auto rapid fire gun",
    "Machine gun",
];

pub const GUN_TYPES_HAN: &[&str] = &[
    "前装滑膛炮 (Muzzle loading)",
    "后装线膛炮 (Breech loading)",
    "速射炮 (Quick firing)",
    "高射炮 (Anti-air)",
    "两用平高炮 (Dual purpose)",
    "全自动火炮 (Auto rapid fire)",
    "机枪/机炮 (Machine gun)",
];

pub const MOUNT_TYPES: &[&str] = &[
    "Broadside",
    "Coles/Ericsson turret",
    "Open barbette",
    "Turret on barbette",
    "Deck and hoist",
    "Deck mount",
    "Casemate",
];

pub const MOUNT_TYPES_HAN: &[&str] = &[
    "舷侧炮架 (Broadside)",
    "科尔斯/埃里克森炮塔 (Turret)",
    "开放式露炮台 (Open barbette)",
    "座圈重型炮塔 (Turret on barbette)",
    "甲板炮架带扬弹机 (Deck & hoist)",
    "简易甲板炮架 (Deck mount)",
    "装甲廓炮 (Casemate)",
];

pub const GUNS_IN_MOUNT: &[&str] = &[
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

pub const GUNS_IN_MOUNT_HAN: &[&str] = &[
    "单装 (1)",
    "双联上下双层 (2)",
    "四联上下四层 (4)",
    "双联装 (2)",
    "双发并联 (2)",
    "四联双层 (4)",
    "三联装 (3)",
    "三发并联 (3)",
    "六联双层 (6)",
    "四联装 (4)",
    "四发并联 (4)",
    "八联双层 (8)",
    "五联装 (5)",
    "五发并联 (5)",
    "十联双层 (10)",
];

pub const MOUNT_LAYOUTS: &[&str] = &[
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

pub const MOUNT_LAYOUTS_HAN: &[&str] = &[
    "中轴线 - 均布",
    "中轴线 - 艏艉对称 (前>=后)",
    "中轴线 - 艏艉对称 (后>=前)",
    "中轴线 - 前甲板最前",
    "中轴线 - 前甲板",
    "中轴线 - 前甲板后部",
    "中轴线 - 后甲板前部",
    "中轴线 - 后甲板",
    "中轴线 - 后甲板最后",
    "两舷侧 - 均布",
    "两舷侧 - 艏艉配置 (前>=后)",
    "两舷侧 - 艏艉配置 (后>=前)",
    "两舷侧 - 前甲板最前",
    "两舷侧 - 前甲板",
    "两舷侧 - 前甲板后部",
    "两舷侧 - 后甲板前部",
    "两舷侧 - 后甲板",
    "两舷侧 - 后甲板最后",
    "无 (None)",
];

/// `ArmourControl.bulkheadType` in SpringSharp 3.0.
pub const BULKHEAD_TYPES: &[&str] = &["Additional bulkheads", "Strengthened bulkheads"];
pub const BULKHEAD_TYPES_HAN: &[&str] = &["附加横舱壁 (Additional)", "加强横舱壁 (Strengthened)"];

/// `ArmourControl.deckCoverage` in SpringSharp 3.0.
pub const DECK_COVERAGE: &[&str] = &[
    "Armoured deck - multiple decks",
    "Armoured deck - single deck",
    "Protected deck - multiple decks",
    "Protected deck - single deck",
    "Box over machinery",
    "Box over magazines",
    "Box over machinery & magazines",
];

pub const DECK_COVERAGE_HAN: &[&str] = &[
    "装甲甲板 - 多层甲板 (Multiple)",
    "装甲甲板 - 单层甲板 (Single)",
    "防护甲板 - 多层甲板 (Multiple)",
    "防护甲板 - 单层甲板 (Single)",
    "装甲盒 - 仅动力舱 (Machinery)",
    "装甲盒 - 仅弹药库 (Magazines)",
    "装甲盒 - 动力与弹药库 (Citadel)",
];

pub const TORPEDO_LOCATIONS: &[&str] = &[
    "Deck mounted carriage/fixed tube",
    "Deck mounted side rotating tube",
    "Deck mounted centre rotating tube",
    "Deck mounted reload",
    "Submerged bow tube",
    "Submerged stern tube",
    "Submerged bow & stern tube",
    "Submerged side tube",
    "Below water reload",
];

pub const TORPEDO_LOCATIONS_HAN: &[&str] = &[
    "甲板固定发射架/管",
    "甲板舷侧旋转发射管",
    "甲板中轴旋转发射管",
    "甲板再装填装置",
    "水下艏部发射管",
    "水下艉部发射管",
    "水下艏艉发射管",
    "水下舷侧发射管",
    "水下再装填装置",
];

pub const MINE_LOCATIONS: &[&str] = &[
    "Above water - Stern racks/rails",
    "Below water - bow tubes",
    "Below water - stern tubes",
    "Below water - side tubes",
];

pub const MINE_LOCATIONS_HAN: &[&str] = &[
    "水上 - 艉部布雷滑轨",
    "水下 - 艏部布雷管",
    "水下 - 艉部布雷管",
    "水下 - 舷侧布雷管",
];

pub const DC_LOCATIONS: &[&str] = &[
    "Stern depth charge racks",
    "Depth charge throwers",
    "Hedgehog style A/S mortars",
    "Squid style A/S mortars",
];

pub const DC_LOCATIONS_HAN: &[&str] = &[
    "艉部深弹投放轨",
    "深弹抛射炮 (K炮)",
    "刺猬式反潜迫击炮",
    "乌贼式反潜臼炮",
];

pub fn bow_types(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => BOW_TYPES_HAN,
        TextLanguage::Italic => BOW_TYPES_ITALIC,
        TextLanguage::Germanic => BOW_TYPES,
    }
}

pub fn stern_types(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => STERN_TYPES_HAN,
        TextLanguage::Italic => STERN_TYPES_ITALIC,
        TextLanguage::Germanic => STERN_TYPES,
    }
}

pub fn bulkhead_types(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => BULKHEAD_TYPES_HAN,
        TextLanguage::Italic => BULKHEAD_TYPES_ITALIC,
        TextLanguage::Germanic => BULKHEAD_TYPES,
    }
}

pub fn deck_coverage(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => DECK_COVERAGE_HAN,
        TextLanguage::Italic => DECK_COVERAGE_ITALIC,
        TextLanguage::Germanic => DECK_COVERAGE,
    }
}

pub fn gun_types(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => GUN_TYPES_HAN,
        TextLanguage::Italic => GUN_TYPES_ITALIC,
        TextLanguage::Germanic => GUN_TYPES,
    }
}

pub fn mount_types(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => MOUNT_TYPES_HAN,
        TextLanguage::Italic => MOUNT_TYPES_ITALIC,
        TextLanguage::Germanic => MOUNT_TYPES,
    }
}

pub fn guns_in_mount(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => GUNS_IN_MOUNT_HAN,
        TextLanguage::Italic => GUNS_IN_MOUNT_ITALIC,
        TextLanguage::Germanic => GUNS_IN_MOUNT,
    }
}

pub fn mount_layouts(lang: TextLanguage) -> &'static [&'static str] {
    match lang {
        TextLanguage::Han => MOUNT_LAYOUTS_HAN,
        TextLanguage::Italic => MOUNT_LAYOUTS_ITALIC,
        TextLanguage::Germanic => MOUNT_LAYOUTS,
    }
}

/// Location choices depend on the weapon kind (`WeaponControl.cs`).
pub fn weapon_locations(kind: WeaponKind) -> &'static [&'static str] {
    weapon_locations_lang(kind, TextLanguage::Germanic)
}

pub fn weapon_locations_lang(kind: WeaponKind, lang: TextLanguage) -> &'static [&'static str] {
    if lang == TextLanguage::Han {
        match kind {
            WeaponKind::Torpedo => TORPEDO_LOCATIONS_HAN,
            WeaponKind::Mine => MINE_LOCATIONS_HAN,
            WeaponKind::DepthCharge => DC_LOCATIONS_HAN,
        }
    } else if lang == TextLanguage::Italic {
        match kind {
            WeaponKind::Torpedo => TORPEDO_LOCATIONS_ITALIC,
            WeaponKind::Mine => MINE_LOCATIONS_ITALIC,
            WeaponKind::DepthCharge => DC_LOCATIONS_ITALIC,
        }
    } else {
        match kind {
            WeaponKind::Torpedo => TORPEDO_LOCATIONS,
            WeaponKind::Mine => MINE_LOCATIONS,
            WeaponKind::DepthCharge => DC_LOCATIONS,
        }
    }
}

/// Label for `value` in `options`, or a neutral fallback for out-of-range data.
pub fn label(options: &[&'static str], value: i32) -> &'static str {
    usize::try_from(value)
        .ok()
        .and_then(|i| options.get(i).copied())
        .unwrap_or("Other")
}

/// Catalog for a web form field key, when that field is a fixed integer enum.
pub fn for_field(key: &str) -> Option<&'static [&'static str]> {
    for_field_lang(key, TextLanguage::Germanic)
}

pub fn for_field_lang(key: &str, lang: TextLanguage) -> Option<&'static [&'static str]> {
    let suffix = key.rsplit_once('.').map_or(key, |(_, s)| s);
    Some(match (key, suffix) {
        ("free.bow", _) => bow_types(lang),
        ("free.stern", _) => stern_types(lang),
        ("arm.bulkhead_type", _) => bulkhead_types(lang),
        ("arm.deck_coverage", _) => deck_coverage(lang),
        (k, "gun_type") if k.starts_with("gun") => gun_types(lang),
        (k, "mount_type") if k.starts_with("gun") => mount_types(lang),
        (k, "layout_g1" | "layout_g2") if k.starts_with("gun") => mount_layouts(lang),
        (k, "guns_in_mount_g1" | "guns_in_mount_g2") if k.starts_with("gun") => guns_in_mount(lang),
        _ => return None,
    })
}

pub const BOW_TYPES_ITALIC: &[&str] = &[
    "Prora ordinaria",
    "Prora bulbosa recta",
    "Prora bulbosa prominens",
    "Prora rostrata",
];

pub const STERN_TYPES_ITALIC: &[&str] = &["Poupe de croiseur", "Puppis truncata", "Puppis rotunda"];

pub const GUN_TYPES_ITALIC: &[&str] = &[
    "Canon à chargement par la bouche",
    "Canon à chargement par la culasse",
    "Canon à tir rapide",
    "Canon antiaérien",
    "Canon à double usage",
    "Canon automatique à tir rapide",
    "Mitrailleuse",
];

pub const MOUNT_TYPES_ITALIC: &[&str] = &[
    "Affût de bordée",
    "Tourelle Coles/Ericsson",
    "Affût en barbette ouverte",
    "Tourelle sur barbette",
    "Affût de pont avec monte-munitions",
    "Affût de pont",
    "Affût en casemate",
];

pub const GUNS_IN_MOUNT_ITALIC: &[&str] = &[
    "Affût simple (1)",
    "Affût double à deux étages (2)",
    "Affût quadruple à quatre étages (4)",
    "Affût jumelé (2)",
    "Affût à deux canons (2)",
    "Affût quadruple à deux étages (4)",
    "Affût triple (3)",
    "Affût à trois canons (3)",
    "Affût sextuple à deux étages (6)",
    "Affût quadruple (4)",
    "Affût à quatre canons (4)",
    "Affût octuple à deux étages (8)",
    "Affût quintuple (5)",
    "Affût à cinq canons (5)",
    "Affût décuple à deux étages (10)",
];

pub const BULKHEAD_TYPES_ITALIC: &[&str] = &["Clausurae additae", "Clausurae roboratae"];

pub const DECK_COVERAGE_ITALIC: &[&str] = &[
    "Tabulata loricata — plura",
    "Tabulatum loricatum — unum",
    "Ponts protégés — plusieurs",
    "Pont protégé — unique",
    "Arca loricata super machinas",
    "Arca loricata super cellas munitionum",
    "Arca loricata super machinas et cellas munitionum",
];

pub const TORPEDO_LOCATIONS_ITALIC: &[&str] = &[
    "Tube fixe sur affût de pont",
    "Tube orientable latéral sur pont",
    "Tube orientable axial sur pont",
    "Recharge sur pont",
    "Tube immergé à la proue",
    "Tube immergé à la poupe",
    "Tubes immergés à la proue et à la poupe",
    "Tube immergé latéral",
    "Recharge sous-marine",
];

pub const MINE_LOCATIONS_ITALIC: &[&str] = &[
    "Supra aquam — rails de mouillage à la poupe",
    "Sub aqua — tubes de mouillage à la proue",
    "Sub aqua — tubes de mouillage à la poupe",
    "Sub aqua — tubes de mouillage latéraux",
];

pub const DC_LOCATIONS_ITALIC: &[&str] = &[
    "Rails de grenadage à la poupe",
    "Lance-grenades sous-marines",
    "Mortiers ASM type Hedgehog",
    "Mortiers ASM type Squid",
];

pub const MOUNT_LAYOUTS_ITALIC: &[&str] = &[
    "In linea media — distributa",
    "In linea media — in extremitatibus (prora >= puppis)",
    "In linea media — in extremitatibus (puppis >= prora)",
    "In linea media — in parte anteriore tabulati anterioris",
    "In linea media — in tabulato anteriore",
    "In linea media — in parte posteriore tabulati anterioris",
    "In linea media — in parte anteriore tabulati posterioris",
    "In linea media — in tabulato posteriore",
    "In linea media — in parte posteriore tabulati posterioris",
    "Ad latera — distributa",
    "Ad latera — in extremitatibus (prora >= puppis)",
    "Ad latera — in extremitatibus (puppis >= prora)",
    "Ad latera — in parte anteriore tabulati anterioris",
    "Ad latera — in tabulato anteriore",
    "Ad latera — in parte posteriore tabulati anterioris",
    "Ad latera — in parte anteriore tabulati posterioris",
    "Ad latera — in tabulato posteriore",
    "Ad latera — in parte posteriore tabulati posterioris",
    "Nullum",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localized_catalogs_preserve_every_stored_index() {
        for catalog in [
            bow_types,
            stern_types,
            bulkhead_types,
            deck_coverage,
            gun_types,
            mount_types,
            guns_in_mount,
            mount_layouts,
        ] {
            let en = catalog(TextLanguage::Germanic);
            for lang in [TextLanguage::Han, TextLanguage::Italic] {
                let localized = catalog(lang);
                assert_eq!(localized.len(), en.len());
                assert!(localized.iter().all(|label| !label.is_empty()));
            }
        }
        for kind in [
            WeaponKind::Torpedo,
            WeaponKind::Mine,
            WeaponKind::DepthCharge,
        ] {
            let en = weapon_locations(kind);
            for lang in TextLanguage::ALL {
                assert_eq!(weapon_locations_lang(kind, lang).len(), en.len());
            }
        }
        assert_eq!(
            gun_types(TextLanguage::Italic)[1],
            "Canon à chargement par la culasse"
        );
        assert_eq!(
            mount_types(TextLanguage::Italic)[3],
            "Tourelle sur barbette"
        );
        assert_eq!(mount_layouts(TextLanguage::Italic)[18], "Nullum");
    }

    #[test]
    fn labels_are_bounds_safe() {
        assert_eq!(label(BULKHEAD_TYPES, 1), "Strengthened bulkheads");
        assert_eq!(label(BULKHEAD_TYPES, -1), "Other");
        assert_eq!(label(DECK_COVERAGE, 99), "Other");
        assert_eq!(weapon_locations(WeaponKind::Torpedo).len(), 9);
        assert_eq!(weapon_locations(WeaponKind::Mine).len(), 4);
        assert_eq!(for_field("arm.deck_coverage").map(<[_]>::len), Some(7));
        assert_eq!(
            for_field_lang("arm.deck_coverage", TextLanguage::Han).map(<[_]>::len),
            Some(7)
        );
    }
}
