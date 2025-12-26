//! Encoding: structured input -> compressed token stream.

use crate::{AtomId, SlotValue, StructuredInput, Template, Trie};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum EncTok {
    Template(u32),
    LiteralAtom(AtomId),
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct EncodeStats {
    pub positions: u64,
    pub trie_steps: u64,
    pub template_hits: u64,
    pub literal_emits: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EncodeResult {
    pub toks: Vec<EncTok>,
    pub slots: Vec<SlotValue>,
    pub stats: EncodeStats,
}

pub fn encode_stream(trie: &Trie, templates: &[Template], input: &StructuredInput) -> EncodeResult {
    let atoms = &input.atoms;
    let mut slot_cursor = 0usize;
    let mut out_slots: Vec<SlotValue> = Vec::with_capacity(input.slots.len());
    let mut toks: Vec<EncTok> = Vec::new();
    let mut stats = EncodeStats::default();

    let mut pos = 0usize;
    while pos < atoms.len() {
        stats.positions += 1;
        let (best_len, best_tid, steps) = trie.match_longest(atoms, pos);
        stats.trie_steps += steps as u64;

        if best_tid >= 0 {
            let tid = best_tid as usize;
            toks.push(EncTok::Template(tid as u32));
            stats.template_hits += 1;
            let sc = templates[tid].slot_count as usize;
            for _ in 0..sc {
                if slot_cursor < input.slots.len() {
                    out_slots.push(input.slots[slot_cursor].clone());
                    slot_cursor += 1;
                }
            }
            pos += best_len.max(1);
        } else {
            toks.push(EncTok::LiteralAtom(atoms[pos]));
            stats.literal_emits += 1;
            pos += 1;
        }
    }

    while slot_cursor < input.slots.len() {
        out_slots.push(input.slots[slot_cursor].clone());
        slot_cursor += 1;
    }

    EncodeResult { toks, slots: out_slots, stats }
}
