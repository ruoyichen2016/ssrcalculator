//! Scratch: does Fn-segmented per-type factor beat a single per-type factor?
//! Run: cargo run --example segment_experiment

use springsharp::{froude_number, historical_cases, simulate_case_with_model, ResistanceModel};

struct Row {
    holt: f64,
    reference: f64,
    fn_: f64,
    ty: &'static str,
}

fn mean_factor(rows: &[&Row]) -> f64 {
    let mut s = 0.0;
    let mut n = 0.0;
    for r in rows {
        if r.holt > 0.0 && r.reference > 0.0 {
            s += r.reference / r.holt;
            n += 1.0;
        }
    }
    if n > 0.0 {
        s / n
    } else {
        1.0
    }
}

fn nested(
    rows: &[Row],
    segmented: bool,
    min_train: usize,
) -> (f64, Vec<(&'static str, usize, f64)>) {
    let mut errs: Vec<(&'static str, f64)> = Vec::new();
    for i in 0..rows.len() {
        let test = &rows[i];
        let train: Vec<&Row> = rows
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, r)| r)
            .collect();
        let same: Vec<&Row> = train.iter().copied().filter(|r| r.ty == test.ty).collect();
        let pool: Vec<&Row> = if same.len() >= min_train {
            same.clone()
        } else {
            train.clone()
        };
        let factor = if segmented && same.len() >= min_train {
            let mut fns: Vec<f64> = same.iter().map(|r| r.fn_).collect();
            fns.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = fns[fns.len() / 2];
            let band: Vec<&Row> = if test.fn_ <= median {
                same.iter().copied().filter(|r| r.fn_ <= median).collect()
            } else {
                same.iter().copied().filter(|r| r.fn_ > median).collect()
            };
            if band.len() >= 2 {
                mean_factor(&band)
            } else {
                mean_factor(&same)
            }
        } else {
            mean_factor(&pool)
        };
        let pred = test.holt * factor;
        errs.push((test.ty, ((pred - test.reference) / test.reference).abs()));
    }
    let overall = errs.iter().map(|(_, e)| e).sum::<f64>() / errs.len() as f64;
    let mut types: Vec<&'static str> = errs.iter().map(|(t, _)| *t).collect();
    types.sort_unstable();
    types.dedup();
    let per = types
        .into_iter()
        .map(|ty| {
            let es: Vec<f64> = errs
                .iter()
                .filter(|(t, _)| *t == ty)
                .map(|(_, e)| *e)
                .collect();
            (ty, es.len(), es.iter().sum::<f64>() / es.len() as f64)
        })
        .collect();
    (overall, per)
}

fn main() {
    let cases = historical_cases();
    let rows: Vec<Row> = cases
        .iter()
        .map(|c| Row {
            holt: simulate_case_with_model(c, ResistanceModel::HoltropMennen)
                .engine
                .hp_max,
            reference: c.reference_hp,
            fn_: froude_number(c.speed_kt, c.length_ft),
            ty: c.ship_type,
        })
        .collect();
    for (seg, min_train, label) in [
        (false, 4usize, "single factor (n>=4)"),
        (true, 4, "Fn-segmented (n>=4)"),
        (true, 8, "Fn-segmented (n>=8)"),
        (true, 12, "Fn-segmented (n>=12)"),
    ] {
        let (overall, per) = nested(&rows, seg, min_train);
        println!("{label:<26} overall {:.2}%", overall * 100.0);
        for (ty, n, m) in per {
            println!("    {ty:<12} n={n:<2} {:.2}%", m * 100.0);
        }
    }
}
