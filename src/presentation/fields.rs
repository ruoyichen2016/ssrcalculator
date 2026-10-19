//! Field display metadata stays outside core and is shared by both adapters.
use crate::presentation::fonts::TextLanguage;

pub const CATALOG: &str = include_str!("../../assets/fields.tsv");

pub fn label(key: &str) -> &str {
    label_for(key, TextLanguage::Germanic)
}

pub fn label_for(key: &str, lang: TextLanguage) -> &str {
    if lang == TextLanguage::Han {
        if let Some(zh) = han_label(key) {
            return zh;
        }
    }
    CATALOG
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(id, _)| *id == key)
        .map(|(_, label)| super::latin_ui::text(lang, label))
        .unwrap_or(key)
}

pub fn han_label(key: &str) -> Option<&'static str> {
    Some(match key {
        "meta.name" => "舰名",
        "meta.country" => "国家/海军",
        "meta.type" => "舰种类型",
        "hull.length" => "水线长 (ft)",
        "hull.beam" => "型宽 (ft)",
        "hull.beam_bulges" => "含防雷突唇型宽 (ft)",
        "hull.draught" => "吃水 (ft)",
        "hull.block" => "方形系数 Cb",
        "hull.date_hull" => "船体设计年代",
        "hull.date_engine" => "主机设计年代",
        "hull.lcb_percent" => "浮心纵向位置 LCB (% L)",
        "hull.midship_coefficient" => "舯剖面系数 Cm",
        "hull.bulb_area_sqft" => "球鼻艏剖面积 (sq ft)",
        "hull.bulb_centre_height_ft" => "球鼻艏中心高 (ft)",
        "hull.transom_area_sqft" => "艉封板面积 (sq ft)",
        "hull.appendage_area_sqft" => "附体面积 (sq ft)",
        "hull.stern_shape" => "艉部形状系数 Cstern",
        "free.bow" => "艏部类型",
        "free.stern" => "艉部类型",
        "free.transom_width" => "艉封板宽度比",
        "free.length_bow" => "艏倾角/外伸长 (ft)",
        "free.length_ram" => "撞角长度 (ft)",
        "free.length_stern" => "艉部外伸长 (ft)",
        "free.fc_length" => "艏楼长度比",
        "free.fd_length" => "前甲板长度比",
        "free.qd_length" => "后甲板长度比",
        "free.free_fc_for" => "艏楼前部干舷 (ft)",
        "free.free_fc_aft" => "艏楼后部干舷 (ft)",
        "free.free_fd_for" => "前甲板前部干舷 (ft)",
        "free.free_fd_aft" => "前甲板后部干舷 (ft)",
        "free.free_ad_for" => "舯后甲板前部干舷 (ft)",
        "free.free_ad_aft" => "舯后甲板后部干舷 (ft)",
        "free.free_qd_for" => "后甲板前部干舷 (ft)",
        "free.free_qd_aft" => "后甲板后部干舷 (ft)",
        "arm.belt_main" => "主装甲带厚度 (in)",
        "arm.belt_main_length" => "主装甲带长度 (ft)",
        "arm.belt_main_height" => "主装甲带高度 (ft)",
        "arm.belt_ends" => "艏艉装甲带 (in)",
        "arm.belt_ends_length" => "艏艉装甲带长度 (ft)",
        "arm.belt_ends_height" => "艏艉装甲带高度 (ft)",
        "arm.belt_upper" => "上层装甲带 (in)",
        "arm.belt_upper_length" => "上层装甲带长度 (ft)",
        "arm.belt_upper_height" => "上层装甲带高度 (ft)",
        "arm.belts_incline" => "装甲带倾角 (deg)",
        "arm.bulkhead" => "装甲横舱壁 (in)",
        "arm.bulkhead_length" => "横舱壁长度 (ft)",
        "arm.bulkhead_height" => "横舱壁高度 (ft)",
        "arm.bulkhead_beam" => "横舱壁宽度 (ft)",
        "arm.bulkhead_type" => "装甲横舱壁类型",
        "arm.bulge" => "防雷突唇宽度 (ft)",
        "arm.bulge_length" => "防雷突唇长度 (ft)",
        "arm.bulge_height" => "防雷突唇高度 (ft)",
        "arm.deck" => "主装甲甲板 (in)",
        "arm.deck_fc" => "艏楼装甲甲板 (in)",
        "arm.deck_qd" => "后甲板装甲甲板 (in)",
        "arm.deck_coverage" => "装甲甲板布置模式",
        "arm.conning_for" => "前司令塔装甲 (in)",
        "arm.conning_aft" => "后司令塔装甲 (in)",
        "engine.speed_max" => "最大航速 (kt)",
        "engine.speed_cruise" => "巡航航速 (kt)",
        "engine.range" => "设计续航力 (nm)",
        "engine.screws" => "螺旋桨轴数",
        "engine.pct_coal" => "燃煤比例 (%)",
        "engine.power_factor" => "功率修正系数",
        "engine.machinery_weight_factor" => "机械重量系数",
        "engine.original_power_factor" => "原版功率系数",
        "engine.blend_weight" => "阻力混合权重",
        "engine.resistance_model" => "阻力估算模型",
        "engine.holtrop_viscous_factor" => "Holtrop 粘性阻力因子",
        "engine.holtrop_wave_factor" => "Holtrop 兴波阻力因子",
        "engine.holtrop_correlation_factor" => "Holtrop 相关阻力因子",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn latin_covers_existing_chinese_fields_without_expanding_chinese() {
        for (key, en) in CATALOG.lines().filter_map(|line| line.split_once('\t')) {
            if let Some(zh) = han_label(key) {
                assert_eq!(label_for(key, TextLanguage::Han), zh);
                assert_ne!(
                    label_for(key, TextLanguage::Italic),
                    en,
                    "missing Latin field {key}"
                );
            } else {
                assert_eq!(label_for(key, TextLanguage::Han), en);
            }
        }
        assert_eq!(label_for("custom.key", TextLanguage::Italic), "custom.key");
    }
}
