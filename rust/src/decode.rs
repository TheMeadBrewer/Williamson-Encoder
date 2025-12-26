//! Decoding: compressed token stream -> original text (lossless).

use crate::{AtomKind, EncTok, EncodeResult, Interner, SlotValue, Template};

pub fn decode(interner: &Interner, templates: &[Template], encoded: &EncodeResult) -> String {
    let mut out = String::new();
    let mut slot_cursor = 0usize;

    for tok in &encoded.toks {
        match *tok {
            EncTok::Template(tid) => {
                let t = &templates[tid as usize];
                for &aid in &t.atoms {
                    let (kind, pid) = interner.atom_info(aid);
                    match kind {
                        AtomKind::Lit | AtomKind::Ws | AtomKind::Punc => {
                            out.push_str(interner.payload_str(pid));
                        }
                        AtomKind::Var | AtomKind::Cap | AtomKind::Num | AtomKind::WsRun => {
                            let slot = encoded.slots.get(slot_cursor).cloned().unwrap_or(
                                SlotValue { kind, payload: interner.payload_id("").unwrap_or(0) }
                            );
                            slot_cursor += 1;
                            out.push_str(interner.payload_str(slot.payload));
                        }
                    }
                }
            }
            EncTok::LiteralAtom(aid) => {
                let (kind, pid) = interner.atom_info(aid);
                match kind {
                    AtomKind::Lit | AtomKind::Ws | AtomKind::Punc => {
                        out.push_str(interner.payload_str(pid));
                    }
                    AtomKind::Var | AtomKind::Cap | AtomKind::Num | AtomKind::WsRun => {
                        let slot = encoded.slots.get(slot_cursor).cloned().unwrap_or(
                            SlotValue { kind, payload: interner.payload_id("").unwrap_or(0) }
                        );
                        slot_cursor += 1;
                        out.push_str(interner.payload_str(slot.payload));
                    }
                }
            }
        }
    }

    out
}
