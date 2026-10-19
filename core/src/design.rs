//! Editing state and immutable, versioned calculation snapshots.
//!
//! ShipModel is still the numerical engine's mutable working model. Frontends
//! must render results from Evaluation, never from a partially edited draft.
use std::sync::Arc;

use crate::{blueprint::BlueprintLayout, io::serialize, model::ShipModel};

#[derive(Debug)]
pub struct Evaluation {
    pub revision: u64,
    pub ship: ShipModel,
    pub blueprint: BlueprintLayout,
    pub systems: crate::systems::SystemsEvaluation,
}

impl Evaluation {
    pub fn from_calculated(ship: ShipModel, revision: u64) -> Self {
        Self {
            revision,
            blueprint: BlueprintLayout::from_ship(&ship),
            systems: ship.systems.evaluate(&ship),
            ship,
        }
    }
}

pub struct DesignSession {
    pub draft: ShipModel,
    revision: u64,
    observed_inputs: String,
    result: Arc<Evaluation>,
}

impl DesignSession {
    pub fn new(mut draft: ShipModel) -> Self {
        draft.calculate();
        Self {
            observed_inputs: serialize(&draft),
            result: Arc::new(Evaluation::from_calculated(draft.clone(), 0)),
            draft,
            revision: 0,
        }
    }

    /// Track persisted inputs, ignoring changes to derived engine fields.
    /// Called at UI transaction boundaries, including before explicit actions.
    pub fn sync_edits(&mut self) -> bool {
        let inputs = serialize(&self.draft);
        if inputs == self.observed_inputs {
            return false;
        }
        self.observed_inputs = inputs;
        self.revision += 1;
        true
    }

    pub fn is_dirty(&self) -> bool {
        self.revision != self.result.revision
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn result(&self) -> Arc<Evaluation> {
        Arc::clone(&self.result)
    }

    pub fn calculate(&mut self) {
        let mut ship = self.draft.clone();
        ship.calculate();
        self.publish(ship);
    }

    /// Commit an already calculated application transaction without recalculating.
    pub(crate) fn publish(&mut self, ship: ShipModel) {
        self.draft = ship.clone();
        self.sync_edits();
        // Publish normalized inputs and their result as one transaction.
        self.observed_inputs = serialize(&ship);
        self.result = Arc::new(Evaluation::from_calculated(ship, self.revision));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_preserves_snapshot_until_calculation() {
        let mut session = DesignSession::new(crate::features::demo_ship());
        let old = session.result();
        session.draft.hull.length += 30.0;
        assert!(session.sync_edits());
        assert!(session.is_dirty());
        assert_eq!(session.result().ship.hull.length, old.ship.hull.length);
        session.calculate();
        assert!(!session.is_dirty());
        assert!(!session.sync_edits());
        assert_eq!(session.result().revision, old.revision + 1);
        assert_ne!(session.result().ship.hull.length, old.ship.hull.length);
    }
}
