use crate::workbench::Outcome;
pub fn command_message(result: Outcome) -> String {
    match result {
        Outcome::Calculated => "Calculated submitted parameters".into(),
        Outcome::DemoLoaded => "Demo design loaded".into(),
        Outcome::Loaded(name) => format!("Loaded {name}"),
        Outcome::Imported => "Design file loaded".into(),
        Outcome::Regime { model, factor } => format!("Regime applied: {model} × {factor:.3}"),
        Outcome::Blended => "Blended resistance model applied".into(),
        Outcome::Reset => "Original resistance model restored".into(),
        Outcome::Speed(n) => format!("Solved maximum speed: {n:.2} kt"),
        Outcome::Range(n) => format!("Solved range: {n:.0} nm"),
        Outcome::Displacement(n) => format!("Scaled displacement: {n:.0} tons"),
        Outcome::Pinned => "Submitted design pinned as baseline".into(),
        Outcome::Comparing(name) => format!("Comparing with {name}"),
        Outcome::ComparisonCleared => "Comparison cleared".into(),
        Outcome::Arranged => "Arrangement optimized for submitted design".into(),
    }
}
