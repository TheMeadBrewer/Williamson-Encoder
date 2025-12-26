//! Token interning for compact, fast comparison.

use hashbrown::HashMap;

pub type AtomId = u32;
pub type PayloadId = u32;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AtomKind {
    Lit = 0,
    Ws = 1,
    Punc = 2,
    Var = 3,
    Cap = 4,
    Num = 5,
    WsRun = 6,
}

impl AtomKind {
    pub fn is_slot(&self) -> bool {
        matches!(self, AtomKind::Var | AtomKind::Cap | AtomKind::Num | AtomKind::WsRun)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SlotValue {
    pub kind: AtomKind,
    pub payload: PayloadId,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Interner {
    payload_to_id: HashMap<String, PayloadId>,
    id_to_payload: Vec<String>,
    atom_to_id: HashMap<(AtomKind, PayloadId), AtomId>,
    id_to_atom: Vec<(AtomKind, PayloadId)>,
    frozen: bool,
}

impl Default for Interner {
    fn default() -> Self {
        Self {
            payload_to_id: HashMap::new(),
            id_to_payload: Vec::new(),
            atom_to_id: HashMap::new(),
            id_to_atom: Vec::new(),
            frozen: false,
        }
    }
}

impl Interner {
    pub fn new() -> Self { Self::default() }
    pub fn freeze(&mut self) { self.frozen = true; }
    pub fn is_frozen(&self) -> bool { self.frozen }
    pub fn payload_count(&self) -> usize { self.id_to_payload.len() }
    pub fn atom_count(&self) -> usize { self.id_to_atom.len() }

    pub fn intern_payload(&mut self, s: &str) -> PayloadId {
        if let Some(&id) = self.payload_to_id.get(s) { return id; }
        let id = self.id_to_payload.len() as PayloadId;
        self.id_to_payload.push(s.to_owned());
        self.payload_to_id.insert(s.to_owned(), id);
        id
    }

    pub fn payload_str(&self, id: PayloadId) -> &str {
        &self.id_to_payload[id as usize]
    }

    pub fn intern_atom(&mut self, kind: AtomKind, payload: PayloadId) -> AtomId {
        if let Some(&id) = self.atom_to_id.get(&(kind, payload)) { return id; }
        let id = self.id_to_atom.len() as AtomId;
        self.id_to_atom.push((kind, payload));
        self.atom_to_id.insert((kind, payload), id);
        id
    }

    pub fn atom_info(&self, id: AtomId) -> (AtomKind, PayloadId) {
        self.id_to_atom[id as usize]
    }

    pub fn intern_fixed_kind(&mut self, kind: AtomKind) -> AtomId {
        let empty = self.intern_payload("");
        self.intern_atom(kind, empty)
    }

    pub fn atom_id(&self, kind: AtomKind, payload: PayloadId) -> Option<AtomId> {
        self.atom_to_id.get(&(kind, payload)).copied()
    }

    pub fn payload_id(&self, s: &str) -> Option<PayloadId> {
        self.payload_to_id.get(s).copied()
    }
}
