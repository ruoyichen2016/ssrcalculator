//! User-facing labels are deliberately outside the numerical core.
use crate::superstructure::{Archetype, Conflict, Plan};

pub fn archetype_name(kind: Archetype) -> &'static str {
    match kind {
        Archetype::Auto => "Auto",
        Archetype::LowDistributed => "Low distributed",
        Archetype::Compact => "Compact central",
        Archetype::Split => "Fore / aft split",
        Archetype::Tower => "Concentrated tower",
        Archetype::AftCitadel => "Aft-shifted citadel",
        Archetype::FlotillaStreamlined => "Flotilla streamlined",
        Archetype::Island => "Side island",
    }
}

pub fn planning_notes(plan: &Plan) -> Vec<String> {
    let mut notes = vec![format!(
        "Superstructure: {}",
        archetype_name(plan.archetype)
    )];
    notes.push(format!("Mast: {}", mast_name(plan.mast)));
    for issue in &plan.conflicts {
        notes.push(match issue {
            Conflict::DeckBoundary(i) => format!("Part {} exceeds usable deck bounds.", i + 1),
            Conflict::PartOverlap(i, j) => format!("Parts {} and {} overlap.", i + 1, j + 1),
            Conflict::BatteryClearance(i) => format!(
                "Part {} intrudes into a battery clearance zone; check firing arcs.",
                i + 1
            ),
            Conflict::ExhaustNearBridge => {
                "Exhaust is close to the bridge / director; check smoke interference.".into()
            }
            Conflict::HeightLimit(i) => format!(
                "Part {} is unusually tall; review windage and stability.",
                i + 1
            ),
            Conflict::IslandOnGunship => {
                "Side island selected on a non-carrier design; review deck access.".into()
            }
        });
    }
    notes
}

pub fn mast_name(style: crate::superstructure::MastStyle) -> &'static str {
    use crate::superstructure::MastStyle::*;
    match style {
        Auto => "Auto",
        Pole => "Pole",
        Tripod => "Tripod",
        Lattice => "Lattice",
        Pagoda => "Pagoda",
        Integrated => "Integrated tower",
    }
}
