//! Tokenization: text -> structured input (atoms + slots).

use crate::{AtomId, AtomKind, Interner, PayloadId, SlotValue};

#[derive(Clone, Debug)]
pub struct StructuredInput {
    pub atoms: Vec<AtomId>,
    pub slots: Vec<SlotValue>,
}

#[derive(Clone, Debug)]
pub struct TokenizeConfig {
    pub ws_run: bool,
    pub strict_frozen: bool,
    pub stopwords: Option<std::collections::HashSet<String>>,
}

impl Default for TokenizeConfig {
    fn default() -> Self {
        Self { ws_run: false, strict_frozen: false, stopwords: None }
    }
}

impl TokenizeConfig {
    pub fn with_default_stopwords() -> Self {
        let stopwords: std::collections::HashSet<String> = [
            "the", "a", "an", "and", "or", "but", "if", "then", "else", "when", "while", "as",
            "of", "to", "in", "on", "at", "by", "for", "with", "from", "into", "over", "under",
            "is", "are", "was", "were", "be", "been", "being", "do", "does", "did", "doing",
            "have", "has", "had", "having", "will", "would", "can", "could", "may", "might",
            "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
            "this", "that", "these", "those", "there", "here",
        ].iter().map(|s| s.to_string()).collect();
        Self { ws_run: false, strict_frozen: false, stopwords: Some(stopwords) }
    }
}

pub fn tokenize(text: &str, cfg: &TokenizeConfig, interner: &mut Interner) -> StructuredInput {
    tokenize_impl(text, cfg, interner)
}

pub fn tokenize_frozen(text: &str, cfg: &TokenizeConfig, interner: &Interner) -> StructuredInput {
    let mut tmp = interner.clone();
    tokenize_impl(text, cfg, &mut tmp)
}

fn tokenize_impl(text: &str, cfg: &TokenizeConfig, interner: &mut Interner) -> StructuredInput {
    let mut atoms: Vec<AtomId> = Vec::new();
    let mut slots: Vec<SlotValue> = Vec::new();

    let var_atom = interner.intern_fixed_kind(AtomKind::Var);
    let cap_atom = interner.intern_fixed_kind(AtomKind::Cap);
    let num_atom = interner.intern_fixed_kind(AtomKind::Num);
    let wsrun_atom = interner.intern_fixed_kind(AtomKind::WsRun);

    let bytes = text.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let b = bytes[i];

        if b.is_ascii_whitespace() {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
            let s = &text[start..i];

            if cfg.ws_run {
                atoms.push(wsrun_atom);
                let pid = interner.intern_payload(s);
                slots.push(SlotValue { kind: AtomKind::WsRun, payload: pid });
            } else {
                let pid = interner.intern_payload(s);
                let aid = interner.intern_atom(AtomKind::Ws, pid);
                atoms.push(aid);
            }
            continue;
        }

        if b.is_ascii_punctuation() && b != b'\'' {
            let s = &text[i..i + 1];
            let pid = interner.intern_payload(s);
            let aid = interner.intern_atom(AtomKind::Punc, pid);
            atoms.push(aid);
            i += 1;
            continue;
        }

        let start = i;
        i += 1;
        while i < bytes.len() 
            && !bytes[i].is_ascii_whitespace() 
            && !(bytes[i].is_ascii_punctuation() && bytes[i] != b'\'') 
        { i += 1; }
        let s = &text[start..i];

        let is_num = s.bytes().all(|c| c.is_ascii_digit() || c == b'.');
        let is_stopword = cfg.stopwords.as_ref().map(|sw| sw.contains(&s.to_lowercase())).unwrap_or(false);
        let is_cap = s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

        if is_num {
            atoms.push(num_atom);
            let pid = interner.intern_payload(s);
            slots.push(SlotValue { kind: AtomKind::Num, payload: pid });
        } else if is_stopword {
            let pid = interner.intern_payload(s);
            let aid = interner.intern_atom(AtomKind::Lit, pid);
            atoms.push(aid);
        } else if is_cap {
            atoms.push(cap_atom);
            let pid = interner.intern_payload(s);
            slots.push(SlotValue { kind: AtomKind::Cap, payload: pid });
        } else {
            atoms.push(var_atom);
            let pid = interner.intern_payload(s);
            slots.push(SlotValue { kind: AtomKind::Var, payload: pid });
        }
    }

    StructuredInput { atoms, slots }
}
