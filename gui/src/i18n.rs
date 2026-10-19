//! Native GUI localization helpers.
//! Translates interface elements, navigation tabs, section cards, metric titles,
//! and field labels into Chinese when `TextLanguage::Han` is active, while preserving
//! ship names, historical presets, dimensionless ratios, and standard engineering units.

use springsharp::presentation::fonts::TextLanguage;

pub fn tr<'a>(lang: TextLanguage, en: &'a str, zh: &'a str) -> &'a str {
    if lang == TextLanguage::Han {
        zh
    } else {
        en
    }
}

pub fn translate_field(label: &str) -> &str {
    match label {
        // Hull
        "Length WL (ft)" => "水线长 (ft)",
        "Beam (ft)" => "型宽 (ft)",
        "Beam over bulges (ft)" => "含防雷突唇型宽 (ft)",
        "Draught (ft)" => "吃水 (ft)",
        "Block coefficient" => "方形系数 Cb",
        "Hull year" => "船体设计年代",
        "Engine year" => "主机设计年代",
        "LCB (% L)" => "浮心纵向位置 LCB (% L)",
        "Midship coeff Cm" => "舯剖面系数 Cm",
        "Bulb area (sq ft)" => "球鼻艏剖面积 (sq ft)",
        "Bulb centre height (ft)" => "球鼻艏中心高 (ft)",
        "Transom area (sq ft)" => "艉封板浸水面积 (sq ft)",
        "Appendage area (sq ft)" => "附体浸水面积 (sq ft)",
        "Stern shape Cstern" => "艉部形状系数 Cstern",

        // Freeboard
        "Bow shape" => "艏部类型",
        "Stern shape" => "艉部类型",
        "Transom width fraction" => "艉封板宽度比",
        "Bow overhang (ft)" => "艏部前倾外伸 (ft)",
        "Ram length (ft)" => "撞角长度 (ft)",
        "Stern overhang (ft)" => "艉部外伸长 (ft)",
        "Forecastle length fraction" => "艏楼长度比",
        "Forward deck length fraction" => "前甲板长度比",
        "Quarter deck length fraction" => "后甲板长度比",
        "FC freeboard forward (ft)" => "艏楼前部干舷 (ft)",
        "FC freeboard aft (ft)" => "艏楼后部干舷 (ft)",
        "FD freeboard forward (ft)" => "前甲板前部干舷 (ft)",
        "FD freeboard aft (ft)" => "前甲板后部干舷 (ft)",
        "AD freeboard forward (ft)" => "舯后甲板前部干舷 (ft)",
        "AD freeboard aft (ft)" => "舯后甲板后部干舷 (ft)",
        "QD freeboard forward (ft)" => "后甲板前部干舷 (ft)",
        "QD freeboard aft (ft)" => "后甲板后部干舷 (ft)",

        // Armour
        "Main belt (in)" => "主装甲带厚度 (in)",
        "Main belt length (ft)" => "主装甲带长度 (ft)",
        "Main belt height (ft)" => "主装甲带高度 (ft)",
        "Upper belt (in)" => "上层装甲带 (in)",
        "Upper belt length (ft)" => "上层装甲带长度 (ft)",
        "Upper belt height (ft)" => "上层装甲带高度 (ft)",
        "Belt ends (in)" => "艏艉装甲带 (in)",
        "Belt ends length (ft)" => "艏艉装甲带长度 (ft)",
        "Belt ends height (ft)" => "艏艉装甲带高度 (ft)",
        "Belt incline (deg)" => "主装甲带倾角 (deg)",
        "Bulkhead (in)" => "装甲横舱壁 (in)",
        "Bulkhead length (ft)" => "横舱壁长度 (ft)",
        "Bulkhead height (ft)" => "横舱壁高度 (ft)",
        "Bulkhead beam (ft)" => "横舱壁宽度 (ft)",
        "Bulkhead type" => "装甲横舱壁类型",
        "Deck (in)" => "主装甲甲板 (in)",
        "Deck FC (in)" => "艏楼装甲甲板 (in)",
        "Deck QD (in)" => "后甲板装甲甲板 (in)",
        "Deck coverage" => "装甲甲板布置模式",
        "Conning forward (in)" => "前司令塔装甲 (in)",
        "Conning aft (in)" => "后司令塔装甲 (in)",
        "Bulge (ft)" => "防雷突唇宽度 (ft)",
        "Bulge length (ft)" => "防雷突唇长度 (ft)",
        "Bulge height (ft)" => "防雷突唇高度 (ft)",

        // Guns
        "Calibre (in)" => "火炮口径 (in)",
        "Calibre length" => "身管倍径 (cal)",
        "Barrels" => "身管数 (门)",
        "Gun year" => "火炮研制年代",
        "Gun type" => "火炮种类",
        "Mount type" => "炮塔/炮架型式",
        "Mounts" => "炮座/炮塔总数",
        "Magazine rounds" => "单门备弹量 (发)",
        "Armour face (in)" => "炮塔正面装甲 (in)",
        "Armour back (in)" => "炮塔背/侧装甲 (in)",
        "Armour bottom (in)" => "炮塔顶/底装甲 (in)",
        "Layout G1" => "第 1 组炮塔布局",
        "Guns/mount G1" => "第 1 组单座管数",
        "Sub mounts G1" => "第 1 组水下/廓装座数",
        "Deck mounts G1" => "第 1 组甲板座数",
        "Super mounts G1" => "第 1 组背负式座数",
        "Layout G2" => "第 2 组炮塔布局",
        "Guns/mount G2" => "第 2 组单座管数",
        "Sub mounts G2" => "第 2 组水下/廓装座数",
        "Deck mounts G2" => "第 2 组甲板座数",
        "Super mounts G2" => "第 2 组背负式座数",

        // Weapons & Misc
        "Weapon kind:" => "副武器种类:",
        "Number of tubes / weapons" => "武器/发射管数量",
        "Loads / reloads" => "单具再装填备弹数",
        "Size (in diameter / lb charge)" => "口径/装药 (in直径/lb)",
        "Length (ft, torpedo)" => "鱼雷长度 (ft)",
        "Location" => "安装布置位置",
        "Vital spaces (tons)" => "核心舱容积 (tons)",
        "Hull spaces (tons)" => "舰体容积 (tons)",
        "Deck spaces (tons)" => "甲板容积 (tons)",
        "Above-water spaces (tons)" => "水上舱室容积 (tons)",
        "Void weight (tons)" => "空舱重量 (tons)",

        // Engine
        "Speed max (kt)" => "最大航速 (kt)",
        "Speed cruise (kt)" => "巡航航速 (kt)",
        "Range (nm)" => "设计续航力 (nm)",
        "Screws" => "推进轴/螺旋桨数",
        "Percent coal" => "燃煤比例 (%)",
        "Power factor" => "功率修正系数",
        "Machinery weight factor" => "机械重量系数",
        "Original power factor" => "原版功率系数",
        "Blend weight" => "阻力混合权重",
        "Resistance model" => "阻力估算模型",
        "Holtrop viscous factor" => "Holtrop 粘性阻力因子",
        "Holtrop wave factor" => "Holtrop 兴波阻力因子",
        "Holtrop correlation factor" => "Holtrop 相关阻力因子",

        // Superstructure & Solvers
        "Length" => "长度 (ft)",
        "Height" => "高度 (ft)",
        "Position" => "中心纵向位置 (ft)",
        "Relative shift" => "相对位移 (ft)",
        "Length multiplier" => "长度缩放倍率",
        "Height multiplier" => "高度缩放倍率",
        "Funnel spacing (0 = Auto)" => "烟囱间距 (0 = 自动)",
        "Ensemble structural weight" => "上层建筑结构重 (tons)",
        "Ensemble reduced weight" => "优化后结构重 (tons)",
        "Target vertical CG (fraction of depth)" => "目标重心高度 (KG/D)",
        "Planing calibration factor" => "滑行艇标定因子",
        "Planing min speed (kt)" => "滑行最低航速 (kt)",
        "Planing max displacement (tons)" => "滑行最大排水量 (tons)",
        "Power calibration factor" => "功率校核因子",

        _ => label,
    }
}
