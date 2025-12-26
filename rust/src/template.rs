//! Template representation for pattern matching.

use crate::{AtomId, AtomKind};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Template {
    pub atoms: Vec<AtomId>,
    pub slot_count: u8,
    pub slot_kinds: Vec<AtomKind>,
}

impl Template {
    pub fn new(atoms: Vec<AtomId>, slot_kinds: Vec<AtomKind>) -> Self {
        let slot_count = slot_kinds.len() as u8;
        Self { atoms, slot_count, slot_kinds }
    }

    pub fn len(&self) -> usize { self.atoms.len() }
    pub fn is_empty(&self) -> bool { self.atoms.is_empty() }
}
