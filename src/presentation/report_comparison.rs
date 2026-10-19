//! Comparison with the reconstructed, uncalibrated original resistance formula.
//! This does not execute or claim bit-for-bit parity with the original program.
use super::fonts::TextLanguage;
use crate::model::{ResistanceModel, ShipModel};
use std::fmt::Write;

fn original_baseline(ship: &ShipModel) -> ShipModel {
    let mut baseline = ship.clone();
    baseline.engine.resistance_model = ResistanceModel::Original;
    baseline.engine.original_power_factor = 1.0;
    baseline.engine.power_factor = 1.0;
    baseline.engine.blend_weight = 1.0;
    baseline.engine.cfd_proxy_enabled = false;
    baseline.calculate();
    baseline
}

fn difference(current: f64, original: f64) -> String {
    if !current.is_finite() || !original.is_finite() || original.abs() < 1e-12 {
        "—".into()
    } else {
        format!("{:+.2}%", (current - original) / original.abs() * 100.0)
    }
}

pub(super) fn append_comparison(out: &mut String, ship: &ShipModel, language: TextLanguage) {
    use super::report::report_text as tr;
    if ship.iterations == 0 {
        out.push_str(tr(
            language,
            "\nOriginal formula comparison: calculate the current design first.\n",
            "\n原版公式对照：请先计算当前设计。\n",
            "\nComparatio formularum originalium: primum consilium navis computandum est.\n",
        ));
        return;
    }
    let baseline = original_baseline(ship);
    out.push_str(tr(language,
        "\nOriginal SpringSharp formula comparison (Rust reconstruction)\n\
         These are not outputs from the original executable. The same calculated design's hull,\n\
         armament, armour, speed and range inputs are retained. The baseline uses the original\n\
         empirical resistance formula, unit power factors, no blending and no CFD correction,\n\
         then recalculates. Other weight, stability, combat-system and iteration algorithms\n\
         remain this project's implementation; this is not full original-software parity.\n\
         Difference = (current - baseline) / |baseline| * 100%; zero/nonfinite baseline shows —.\n\
         Mass is in long tons (1 long ton = 1.01605 metric tonnes). Speed/range inputs are held fixed.\n\n\
         Metric | Current | Original formula | Difference\n",
        "\n原版 SpringSharp 公式对照（Rust 重建口径）\n\
         本表不是原版程序的实际运行结果。使用同一已计算设计的船体、武备、装甲、航速和航程输入；\n\
         对照列切换至原版经验阻力公式，功率校准系数均为 1，禁用混合模型和 CFD 修正，再重算。\n\
         其余重量、稳性、战斗系统及迭代算法仍采用本项目实现，并非完整恢复原版软件。\n\
         差异 =（当前值 − 对照值）/ |对照值| × 100%；基准为零或非有限值时显示 —。\n\
         重量单位为长吨（1 长吨 = 1.01605 公吨）；输入相同的航速和航程不作为计算差异。\n\n\
         指标 | 当前计算 | 原版公式重算 | 差异\n",
        "\nComparatio formularum originalium SpringSharp (reconstructio in Rust)\n\
         Hi numeri non sunt exitus programmatis originalis. Data eiusdem consilii de corpore navis,\n\
         armamentis, lorica, celeritate et spatio navigationis servantur. Columna comparationis\n\
         formulam empiricam originalem resistentiae adhibet: factores potentiae = 1, sine mixtura\n\
         exemplarium et sine correctione CFD; deinde omnia denuo computantur. Ceteri algorithmi\n\
         ponderum, stabilitatis, systematum bellicorum et iterationis huius operis manent;\n\
         non est restitutio integra programmatis originalis.\n\
         Differentia = (valor praesens - valor comparationis) / |valor comparationis| * 100%;\n\
         si valor comparationis est zero vel non finitus, — ostenditur.\n\
         Pondera: tonnes longues (1 = 1.01605 tonnes métriques). Celeritas et spatium servantur.\n\n\
         Magnitudo | Computatio praesens | Formula originalis | Differentia\n"));
    let rows = [
        (
            "Maximum shaft power (hp)",
            "最大轴功率 (hp)",
            "Potentia maxima in axe (hp)",
            ship.engine.hp_max,
            baseline.engine.hp_max,
        ),
        (
            "Cruise shaft power (hp)",
            "巡航轴功率 (hp)",
            "Potentia itineraria in axe (hp)",
            ship.engine.hp_cruise,
            baseline.engine.hp_cruise,
        ),
        (
            "Machinery (long tons)",
            "动力装置重量（长吨）",
            "Machinae (tonnes longues)",
            ship.engine.engine_weight,
            baseline.engine.engine_weight,
        ),
        (
            "Normal fuel (long tons)",
            "正常燃料储量（长吨）",
            "Combustibile ordinarium (tonnes longues)",
            ship.engine.bunker,
            baseline.engine.bunker,
        ),
        (
            "Armour (long tons)",
            "装甲重量（长吨）",
            "Lorica (tonnes longues)",
            ship.armour.weight,
            baseline.armour.weight,
        ),
        (
            "Structural hull (long tons)",
            "船体结构重量（长吨）",
            "Structura navis (tonnes longues)",
            ship.engine.hull_weight,
            baseline.engine.hull_weight,
        ),
        (
            "Light displacement (long tons)",
            "空载排水量（长吨）",
            "Déplacement lège (tonnes longues)",
            ship.disp_light,
            baseline.disp_light,
        ),
        (
            "Standard displacement (long tons)",
            "标准排水量（长吨）",
            "Déplacement standard (tonnes longues)",
            ship.disp_standard,
            baseline.disp_standard,
        ),
        (
            "Full-load displacement (long tons)",
            "满载排水量（长吨）",
            "Déplacement à pleine charge (tonnes longues)",
            ship.disp_max_load,
            baseline.disp_max_load,
        ),
        (
            "Metacentric height (ft)",
            "初稳性高 (ft)",
            "Hauteur métacentrique (ft)",
            ship.metacentre,
            baseline.metacentre,
        ),
        (
            "Stability",
            "稳性",
            "Stabilitas",
            ship.stability_adj,
            baseline.stability_adj,
        ),
        (
            "Seakeeping",
            "耐波性",
            "Tenue à la mer",
            ship.seakeeping,
            baseline.seakeeping,
        ),
        (
            "Composite strength",
            "综合强度",
            "Firmitas composita",
            ship.strength_comp,
            baseline.strength_comp,
        ),
        (
            "Cost (million GBP)",
            "造价（百万英镑）",
            "Pretium (milliones GBP)",
            ship.cost_pounds,
            baseline.cost_pounds,
        ),
    ];
    for (en, cn, la, current, original) in rows {
        let label = tr(language, en, cn, la);
        let _ = writeln!(
            out,
            "{label} | {current:.3} | {original:.3} | {}",
            difference(current, original)
        );
    }
    let yes = tr(language, "yes", "是", "ita");
    let no = tr(language, "no", "否", "non");
    let _ = writeln!(
        out,
        "{}: {} / {}; {}: {:.2e} / {:.2e}",
        tr(
            language,
            "Weight convergence (current / baseline)",
            "重量迭代收敛（当前／对照）",
            "Convergentia ponderum (praesens / comparatio)"
        ),
        if ship.converged { yes } else { no },
        if baseline.converged { yes } else { no },
        tr(language, "Relative residual", "相对残差", "Résidu relatif"),
        ship.iteration_residual,
        baseline.iteration_residual
    );
    if !ship.converged || !baseline.converged {
        out.push_str(tr(language,
            "Comparison caution: at least one weight iteration did not converge; related weights and performance are approximate.\n",
            "对照提示：至少一列重量迭代未收敛，相关重量及性能仅供近似参考。\n",
            "Monitum comparationis: saltem una iteratio ponderum non convergit; pondera et proprietates pertinentes approximata sunt.\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparison_removes_calibration_blending_and_cfd_without_mutating_input() {
        let mut ship = crate::demo_ship();
        ship.engine.resistance_model = ResistanceModel::HoltropMennen;
        ship.engine.power_factor = 1.4;
        ship.engine.original_power_factor = 1.3;
        ship.engine.blend_weight = 0.5;
        ship.engine.cfd_proxy_enabled = true;
        ship.engine.cfd_proxy_viscous = 1.2;
        ship.calculate();
        let saved = crate::serialize(&ship);
        let baseline = original_baseline(&ship);
        assert_eq!(crate::serialize(&ship), saved);
        assert_eq!(baseline.engine.speed_max, ship.engine.speed_max);
        assert_eq!(baseline.engine.range, ship.engine.range);
        assert_eq!(baseline.hull.length, ship.hull.length);
        // Independent evaluation of the original formula at the requested speed.
        let h = &baseline.hull;
        let speed = baseline.engine.speed_max;
        let length = if speed > 25.0 {
            h.length_effect
        } else if speed > 15.0 {
            (h.length_effect - h.length) * ((speed - 20.0) / 5.0) + h.length
        } else {
            h.length - (h.length_effect - h.length)
        };
        let era = if h.date_engine < 1890 {
            1.0 + (1890 - h.date_engine) as f64 / 100.0
        } else {
            1.0
        };
        let expected =
            (h.displacement.powf(0.6666666865348816) / length * h.co_sharp * speed.powi(4)
                + 0.01 * h.wetted_surface * speed.powf(1.83))
                * speed
                / 184.1666667
                * era;
        assert!((baseline.engine.hp_max - expected).abs() < expected.abs() * 1e-10);
        assert!((baseline.engine.hp_max - ship.engine.hp_max).abs() > 1.0);
    }

    #[test]
    fn uncalibrated_original_design_matches_its_baseline() {
        let mut ship = crate::demo_ship();
        ship.engine.resistance_model = ResistanceModel::Original;
        ship.engine.original_power_factor = 1.0;
        ship.engine.blend_weight = 1.0;
        ship.engine.cfd_proxy_enabled = false;
        ship.calculate();
        let mut report = String::new();
        append_comparison(&mut report, &ship, TextLanguage::Germanic);
        for line in report.lines().filter(|line| line.contains(" | ")).skip(1) {
            assert!(
                line.ends_with("+0.00%") || line.ends_with("-0.00%") || line.ends_with('—'),
                "{line}"
            );
        }
    }

    #[test]
    fn zero_nonfinite_and_equal_baselines_have_defined_differences() {
        assert_eq!(difference(1.0, 0.0), "—");
        assert_eq!(difference(f64::NAN, 2.0), "—");
        assert_eq!(difference(2.0, f64::INFINITY), "—");
        assert_eq!(difference(2.0, 2.0), "+0.00%");
        assert_eq!(difference(3.0, 2.0), "+50.00%");
    }
}
