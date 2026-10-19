//! Application transactions shared by native and zero-script web workbenches.
//! Adapters own widgets/HTTP and file access; this layer owns design operations.
use crate::{
    design::DesignSession,
    features::demo_ship,
    io::{deserialize, serialize},
    model::{ResistanceModel, ShipModel},
    spaces::CompartmentPlacement,
};

pub enum Command<'a> {
    Calculate,
    Demo,
    Preset(&'a str),
    Import(&'a str),
    Regime,
    Blend,
    ResetResistance,
    Speed(f64),
    Range(f64),
    Displacement(f64),
    PinBaseline,
    ComparePreset(&'a str),
    ClearComparison,
    Arrange(f64),
}

#[derive(Debug)]
pub enum Outcome {
    Calculated,
    DemoLoaded,
    Loaded(String),
    Imported,
    Regime { model: String, factor: f64 },
    Blended,
    Reset,
    Speed(f64),
    Range(f64),
    Displacement(f64),
    Pinned,
    Comparing(String),
    ComparisonCleared,
    Arranged,
}

pub struct Workbench {
    pub design: DesignSession,
    baseline: Option<ShipModel>,
    placement: Option<CompartmentPlacement>,
}

impl Default for Workbench {
    fn default() -> Self {
        Self::new(demo_ship())
    }
}

impl Workbench {
    pub fn new(ship: ShipModel) -> Self {
        Self {
            design: DesignSession::new(ship),
            baseline: None,
            placement: None,
        }
    }

    /// Parse and validate persisted inputs without running the numerical engine.
    pub fn parse_design(text: &str) -> Result<ShipModel, String> {
        let mut ship = ShipModel::new();
        deserialize(&mut ship, text)?;
        Ok(ship)
    }

    pub fn baseline(&self) -> Option<&ShipModel> {
        self.baseline.as_ref()
    }
    pub fn placement(&self) -> Option<&CompartmentPlacement> {
        self.placement.as_ref()
    }
    pub fn clear_placement(&mut self) {
        self.placement = None;
    }

    /// Rehydrate a web request's baseline; an invalid value must not silently vanish.
    pub fn restore_baseline(&mut self, text: &str) -> Result<(), String> {
        let baseline = if text.is_empty() {
            None
        } else {
            let mut ship = Self::parse_design(text)?;
            ship.calculate();
            Some(ship)
        };
        self.baseline = baseline;
        Ok(())
    }

    pub fn sync_edits(&mut self) -> bool {
        let changed = self.design.sync_edits();
        if changed {
            self.clear_placement();
        }
        changed
    }

    /// Every successful action consumes current inputs and publishes one snapshot.
    /// Validation and fallible operations happen before mutating committed state.
    pub fn execute(&mut self, command: Command<'_>) -> Result<Outcome, String> {
        let mut ship = match command {
            Command::Demo => demo_ship(),
            Command::Preset(id) => crate::get_preset(id).ok_or("Select a historical preset")?,
            Command::Import(text) => Self::parse_design(text)?,
            _ => Self::parse_design(&serialize(&self.design.draft))?,
        };
        let mut baseline = None;
        let mut placement = None;
        let message = match command {
            Command::Regime => {
                let applied = crate::apply_regime_calibration(&mut ship)
                    .ok_or("No regime calibration available")?;
                Outcome::Regime {
                    model: applied.model_label.to_string(),
                    factor: applied.factor,
                }
            }
            Command::Blend => {
                crate::apply_blend_calibration(&mut ship)
                    .ok_or("No blended calibration available")?;
                Outcome::Blended
            }
            Command::Speed(target) => {
                positive(target, "Target power")?;
                ship.calculate();
                Outcome::Speed(crate::solve_max_speed(&mut ship, target))
            }
            Command::Range(target) => {
                positive(target, "Target bunker")?;
                ship.calculate();
                Outcome::Range(crate::solve_range(&mut ship, target))
            }
            Command::Displacement(target) => {
                positive(target, "Target displacement")?;
                ship.calculate();
                Outcome::Displacement(crate::solve_displacement_scale(&mut ship, target))
            }
            command => {
                if matches!(command, Command::ResetResistance) {
                    ship.engine.resistance_model = ResistanceModel::Original;
                    ship.engine.power_factor = 1.0;
                    ship.engine.original_power_factor = 1.0;
                    ship.engine.blend_weight = 1.0;
                }
                // Presets are already calculated by their factory.
                if !matches!(command, Command::Preset(_)) {
                    ship.calculate();
                }
                match command {
                    Command::Calculate => Outcome::Calculated,
                    Command::Demo => Outcome::DemoLoaded,
                    Command::Preset(_) => Outcome::Loaded(ship.name.clone()),
                    Command::Import(_) => Outcome::Imported,
                    Command::ResetResistance => Outcome::Reset,
                    Command::PinBaseline => {
                        baseline = Some(Some(ship.clone()));
                        Outcome::Pinned
                    }
                    Command::ComparePreset(id) => {
                        let base = crate::get_preset(id).ok_or("Select a comparison preset")?;
                        let message = Outcome::Comparing(base.name.clone());
                        baseline = Some(Some(base));
                        message
                    }
                    Command::ClearComparison => {
                        baseline = Some(None);
                        Outcome::ComparisonCleared
                    }
                    Command::Arrange(cg) => {
                        positive(cg, "Target CG")?;
                        if cg >= 1.0 {
                            return Err("Target CG must be between 0 and 1".into());
                        }
                        placement = Some(crate::suggest_compartment_placement(&ship, cg));
                        Outcome::Arranged
                    }
                    _ => unreachable!(),
                }
            }
        };
        self.design.publish(ship);
        self.placement = placement;
        if let Some(baseline) = baseline {
            self.baseline = baseline;
        }
        Ok(message)
    }

    /// Export the same committed snapshot shown by both frontends.
    pub fn export_design(&self) -> String {
        serialize(&self.design.result().ship)
    }
}

fn positive(value: f64, name: &str) -> Result<(), String> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(format!("{name} must be finite and positive"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn transactions_publish_current_edits_and_rollback_failures() {
        let mut work = Workbench::default();
        work.design.draft.hull.length = 620.0;
        work.execute(Command::PinBaseline).unwrap();
        assert_eq!(work.baseline().unwrap().hull.length, 620.0);
        let pinned = serialize(work.baseline().unwrap());
        let previous = work.design.result();
        work.design.draft.hull.length = 640.0;
        assert!(work.execute(Command::Displacement(f64::NAN)).is_err());
        assert!(Arc::ptr_eq(&previous, &work.design.result()));
        assert_eq!(work.design.draft.hull.length, 640.0);
        assert_eq!(serialize(work.baseline().unwrap()), pinned);
        work.execute(Command::Arrange(0.45)).unwrap();
        assert_eq!(work.design.result().ship.hull.length, 640.0);
        assert!(work.placement().is_some());
        assert!(!work.design.is_dirty());
        work.design.draft.hull.length = 650.0;
        assert!(work.sync_edits());
        assert!(work.placement().is_none());
        assert_eq!(work.baseline().unwrap().hull.length, 620.0);
        assert_eq!(
            Workbench::parse_design(&work.export_design())
                .unwrap()
                .hull
                .length,
            640.0
        );
    }
}
