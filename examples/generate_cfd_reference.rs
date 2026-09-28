//! Regenerate `data/examples/cfd_reference.csv`.
//!
//! The rows are **analytic** (Savitsky vs the simple empirical planing model),
//! not measured CFD.  They exercise the CFD-surrogate pipeline; real measured
//! CFD must be added with provenance `measured_cfd` before `cfd-apply` will use
//! it as evidence.
//!
//! Run: cargo run --example generate_cfd_reference > data/examples/cfd_reference.csv

use springsharp::{cfd_froude_number, planing_hp_estimate, savitsky_planing, PlaningInput};

fn main() {
    println!("# fn,empirical_hp,cfd_hp,provenance");
    let configs: [(f64, f64, f64, f64); 6] = [
        (70.0, 18.0, 35.0, 12.0),
        (80.0, 20.0, 50.0, 12.0),
        (95.0, 22.0, 75.0, 14.0),
        (110.0, 25.0, 110.0, 14.0),
        (130.0, 28.0, 160.0, 16.0),
        (160.0, 34.0, 250.0, 16.0),
    ];
    let speeds = [30.0, 34.0, 38.0, 42.0, 46.0];
    for (length, beam, tons, deadrise) in configs {
        for speed in speeds {
            let weight_lb = tons * 2240.0;
            let empirical = planing_hp_estimate(length, beam, weight_lb, speed);
            let Some(sav) = savitsky_planing(&PlaningInput {
                length_ft: length,
                beam_ft: beam,
                weight_lb,
                speed_kt: speed,
                deadrise_deg: deadrise,
                lcg_ft: 0.42 * length,
                propulsive_coefficient: 0.55,
            }) else {
                continue;
            };
            if empirical <= 0.0 || sav.shaft_hp <= 0.0 {
                continue;
            }
            println!(
                "{:.4},{:.1},{:.1},analytic",
                cfd_froude_number(speed, length),
                empirical,
                sav.shaft_hp
            );
        }
    }
}
