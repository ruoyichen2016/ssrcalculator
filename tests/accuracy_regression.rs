//! Accuracy regression guard.
//!
//! These tests freeze the *out-of-sample* baseline (LOOCV and holdout) plus the
//! provenance/quality gate, so future changes cannot silently regress accuracy
//! or reintroduce self-declared "verified" data.
//!
//! Thresholds are baseline + tolerance.  If a change is a deliberate accuracy
//! improvement, update the constants and the baseline in `docs/VALIDATION.md`.

use springsharp::{
    data_replacement_progress, holdout_validation, loocv_by_type, quality_gate_with_loocv,
    run_loocv, select_lasso_lambda, select_ridge_lambda,
};

/// Overall ensemble LOOCV MAPE baseline was ~10.5% when frozen.
const OVERALL_LOOCV_ENSEMBLE_MAX: f64 = 0.12;
const OVERALL_LOOCV_REDUCED_MAX: f64 = 0.142;
const OVERALL_LOOCV_STRUCTURAL_MAX: f64 = 0.165;

/// Holdout (28 train / 9 held-out) baseline was 16.5% original, 10.9% Holtrop.
const HOLDOUT_ORIGINAL_MAX: f64 = 0.185;
const HOLDOUT_HOLTROP_MAX: f64 = 0.145;

fn per_type(mape: &[(String, usize, f64)], ship_type: &str) -> f64 {
    mape.iter()
        .find(|(t, _, _)| t == ship_type)
        .map(|(_, _, m)| *m)
        .unwrap_or_else(|| panic!("no LOOCV entry for ship type {ship_type}"))
}

#[test]
fn loocv_overall_does_not_regress() {
    let loocv = run_loocv();
    assert!(
        loocv.ensemble_mape <= OVERALL_LOOCV_ENSEMBLE_MAX,
        "ensemble LOOCV MAPE {:.2}% exceeded frozen baseline (max {:.2}%)",
        loocv.ensemble_mape * 100.0,
        OVERALL_LOOCV_ENSEMBLE_MAX * 100.0
    );
    assert!(
        loocv.reduced_mape <= OVERALL_LOOCV_REDUCED_MAX,
        "reduced LOOCV MAPE {:.2}% exceeded frozen baseline",
        loocv.reduced_mape * 100.0
    );
    assert!(
        loocv.structural_mape <= OVERALL_LOOCV_STRUCTURAL_MAX,
        "structural LOOCV MAPE {:.2}% exceeded frozen baseline",
        loocv.structural_mape * 100.0
    );
    assert!(loocv.optimal_reduced_weight >= 0.0 && loocv.optimal_reduced_weight <= 1.0);
}

#[test]
fn loocv_per_type_does_not_regress() {
    let by_type = loocv_by_type();
    let total: usize = by_type.iter().map(|(_, n, _)| *n).sum();
    assert_eq!(
        total,
        springsharp::historical_cases().len(),
        "every historical case must appear once"
    );

    // Baselines: battleship 11.1%, carrier 12.6%, cruiser 12.3%, destroyer 5.3%.
    assert!(per_type(&by_type, "battleship") <= 0.126);
    assert!(per_type(&by_type, "carrier") <= 0.141);
    assert!(per_type(&by_type, "cruiser") <= 0.138);
    assert!(per_type(&by_type, "destroyer") <= 0.105);

    for (ty, n, mape) in &by_type {
        assert!(*n >= 2, "{ty} has too few cases for a per-type estimate");
        assert!(
            mape.is_finite() && *mape >= 0.0,
            "{ty} produced an invalid LOOCV MAPE {mape}"
        );
    }
}

#[test]
fn holdout_does_not_regress() {
    let h = holdout_validation();
    assert!(
        h.holdout_cases >= 8,
        "holdout must contain at least 8 cases"
    );
    assert_eq!(
        h.train_cases + h.holdout_cases,
        springsharp::historical_cases().len()
    );
    assert!(
        h.original_mape <= HOLDOUT_ORIGINAL_MAX,
        "original holdout MAPE {:.2}% exceeded frozen baseline",
        h.original_mape * 100.0
    );
    assert!(
        h.holtrop_mape <= HOLDOUT_HOLTROP_MAX,
        "Holtrop holdout MAPE {:.2}% exceeded frozen baseline",
        h.holtrop_mape * 100.0
    );
}

#[test]
fn provenance_gate_is_honest() {
    let p = data_replacement_progress();
    assert_eq!(
        p.verified, 0,
        "no built-in case is independently verified yet"
    );
    assert_eq!(p.traceable, 0, "no built-in case is traceable yet");
    assert_eq!(p.provenance_coverage, 1.0);

    let gate = quality_gate_with_loocv(run_loocv().ensemble_mape);
    assert!(gate.official_model_fraction == 0.0);
    assert!(gate.traceability_fraction == 0.0);
    assert!(
        gate.composite < 0.6,
        "composite gate {:.3} should not pass without official/traceable data",
        gate.composite
    );
}

#[test]
fn regularization_grid_is_not_a_noop() {
    let ridge = select_ridge_lambda();
    let lasso = select_lasso_lambda();
    let distinct = |sel: &springsharp::LambdaSelection| {
        sel.results
            .iter()
            .map(|(_, m)| (m * 1e6).round() as i64)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    };
    assert!(
        distinct(&ridge) > 1,
        "ridge lambda grid is a no-op: {:?}",
        ridge.results
    );
    assert!(
        distinct(&lasso) > 1,
        "lasso lambda grid is a no-op: {:?}",
        lasso.results
    );
    assert!(ridge.best_lambda > 0.0 && lasso.best_lambda > 0.0);
    assert!(ridge.best_loocv_mape.is_finite() && lasso.best_loocv_mape.is_finite());
}

// ---------------------------------------------------------------------------
// Regime-aware modern model: Holtrop + per-ship-type calibration factor.
// Baseline nested LOOCV ~10.6% with the expanded 123-case dataset (2026-09-27).
// ---------------------------------------------------------------------------

const REGIME_OVERALL_MAX: f64 = 0.115;

#[test]
fn regime_nested_loocv_does_not_regress() {
    let r = springsharp::run_loocv_regime();
    assert!(
        r.overall_mape <= REGIME_OVERALL_MAX,
        "regime nested LOOCV {:.2}% exceeded frozen baseline (max {:.2}%)",
        r.overall_mape * 100.0,
        REGIME_OVERALL_MAX * 100.0
    );
    assert!(
        r.macro_mape.is_finite() && r.macro_mape <= 0.115,
        "regime type-averaged MAPE {:.2}% exceeded baseline",
        r.macro_mape * 100.0
    );
    assert!(r.curated_cases > 0);
    assert!(
        r.curated_mape.is_finite() && r.curated_mape <= 0.095,
        "curated-core MAPE {:.2}% exceeded baseline",
        r.curated_mape * 100.0
    );
    // The modern regime model must beat both comparison baselines.
    assert!(
        r.overall_mape < r.original_baseline_mape,
        "regime {:.2}% should beat original+factor {:.2}%",
        r.overall_mape * 100.0,
        r.original_baseline_mape * 100.0
    );
    assert!(
        r.overall_mape < r.holtrop_uncalibrated_mape,
        "regime {:.2}% should beat uncalibrated Holtrop {:.2}%",
        r.overall_mape * 100.0,
        r.holtrop_uncalibrated_mape * 100.0
    );

    let lookup = |ty: &str| {
        r.per_type
            .iter()
            .find(|(t, _, _)| t == ty)
            .map(|(_, _, m)| *m)
            .unwrap_or_else(|| panic!("no regime entry for {ty}"))
    };
    assert!(lookup("battleship") <= 0.120);
    assert!(lookup("carrier") <= 0.130);
    assert!(lookup("cruiser") <= 0.105);
    assert!(lookup("destroyer") <= 0.130);
}

#[test]
fn regime_table_is_well_formed_and_applicable() {
    let table = springsharp::fit_regime_calibration();
    assert!(!table.is_empty());
    for row in &table {
        assert_eq!(row.model, springsharp::ResistanceModel::HoltropMennen);
        assert!(row.count > 0);
        assert!(row.factor > 0.0 && row.factor.is_finite());
        assert!(row.train_mape.is_finite());
    }

    let mut ship = springsharp::ShipModel::new();
    ship.ship_type = "Battleship".to_string();
    ship.engine.resistance_model = springsharp::ResistanceModel::Original;
    let applied = springsharp::apply_regime_calibration(&mut ship);
    assert!(
        applied.is_some(),
        "demo-like battleship should match the regime table"
    );
    assert_eq!(
        ship.engine.resistance_model,
        springsharp::ResistanceModel::HoltropMennen
    );
    assert!(ship.engine.power_factor > 0.0);
}
