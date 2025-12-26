//! Loader for Python v9.2/v9.3 template JSON files.

use crate::{AtomId, AtomKind, Encoder, Interner, Template, Trie};
use anyhow::{anyhow, Result};
use hashbrown::HashMap;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct PyV92File {
    version: String,
    str_to_id: HashMap<String, u32>,
    id_to_template: HashMap<String, Vec<String>>,
}

pub fn load_python_v92_json<P: AsRef<Path>>(path: P) -> Result<Encoder> {
    let bytes = std::fs::read(path)?;
    let parsed: PyV92File = serde_json::from_slice(&bytes)?;

    if !["9.2", "9.3"].contains(&parsed.version.as_str()) {
        return Err(anyhow!("Unsupported template file version: {}", parsed.version));
    }

    let mut interner = build_interner_preserve_atom_ids(&parsed.str_to_id)?;
    let templates = build_templates(&interner, &parsed.id_to_template)?;
    let trie = Trie::build(&templates);

    Ok(Encoder { interner, templates, trie })
}

fn build_interner_preserve_atom_ids(str_to_id: &HashMap<String, u32>) -> Result<Interner> {
    let mut atoms: Vec<(&String, u32)> = str_to_id.iter().map(|(s, id)| (s, *id)).collect();
    atoms.sort_by_key(|(_, id)| *id);

    let mut interner = Interner::default();

    for (s, wanted_id) in atoms {
        let (kind, payload) = parse_atom_string(s)?;
        let got_id: AtomId = match kind {
            AtomKind::Var | AtomKind::Cap | AtomKind::Num => {
                let empty = interner.intern_payload("");
                interner.intern_atom(kind, empty)
            }
            AtomKind::Lit | AtomKind::Ws | AtomKind::Punc => {
                let pid = interner.intern_payload(payload);
                interner.intern_atom(kind, pid)
            }
            AtomKind::WsRun => return Err(anyhow!("WS_RUN not expected in v9.2: {s}")),
        };

        if got_id != wanted_id as AtomId {
            return Err(anyhow!("AtomId mismatch: atom='{s}', wanted={wanted_id}, got={got_id}"));
        }
    }

    interner.freeze();
    Ok(interner)
}

fn build_templates(interner: &Interner, id_to_template: &HashMap<String, Vec<String>>) -> Result<Vec<Template>> {
    let mut items: Vec<(usize, &Vec<String>)> = Vec::with_capacity(id_to_template.len());
    for (tid_str, atoms) in id_to_template {
        let tid = parse_template_id(tid_str)?;
        items.push((tid, atoms));
    }
    items.sort_by_key(|(tid, _)| *tid);

    if items.is_empty() { return Err(anyhow!("No templates found")); }
    for (i, (tid, _)) in items.iter().enumerate() {
        if *tid != i { return Err(anyhow!("Template IDs not contiguous: expected {i}, found {tid}")); }
    }

    let mut out: Vec<Template> = Vec::with_capacity(items.len());

    for (_tid, atoms_strs) in items {
        let mut atoms: Vec<AtomId> = Vec::with_capacity(atoms_strs.len());
        let mut slot_kinds: Vec<AtomKind> = Vec::new();

        for s in atoms_strs {
            let (kind, payload) = parse_atom_string(s)?;
            let aid: AtomId = match kind {
                AtomKind::Var | AtomKind::Cap | AtomKind::Num => {
                    let empty = interner.payload_id("").ok_or_else(|| anyhow!("Missing empty payload"))?;
                    interner.atom_id(kind, empty).ok_or_else(|| anyhow!("Missing atom: {s}"))?
                }
                AtomKind::Lit | AtomKind::Ws | AtomKind::Punc => {
                    let pid = interner.payload_id(payload).ok_or_else(|| anyhow!("Missing payload: {payload}"))?;
                    interner.atom_id(kind, pid).ok_or_else(|| anyhow!("Missing atom: {s}"))?
                }
                AtomKind::WsRun => return Err(anyhow!("WS_RUN not expected: {s}")),
            };

            if matches!(kind, AtomKind::Var | AtomKind::Cap | AtomKind::Num) {
                slot_kinds.push(kind);
            }
            atoms.push(aid);
        }
        out.push(Template::new(atoms, slot_kinds));
    }

    Ok(out)
}

fn parse_atom_string(s: &str) -> Result<(AtomKind, &str)> {
    if s == "VAR" { return Ok((AtomKind::Var, "")); }
    if s == "CAP" { return Ok((AtomKind::Cap, "")); }
    if s == "NUM" { return Ok((AtomKind::Num, "")); }

    let open = s.find('(').ok_or_else(|| anyhow!("Bad atom: {s}"))?;
    let close = s.rfind(')').ok_or_else(|| anyhow!("Bad atom: {s}"))?;
    if close <= open { return Err(anyhow!("Bad atom: {s}")); }

    let kind_str = &s[..open];
    let mut payload = &s[open + 1..close];

    let kind = match kind_str {
        "LIT" => AtomKind::Lit,
        "WS" => AtomKind::Ws,
        "PUNC" => AtomKind::Punc,
        _ => return Err(anyhow!("Unknown atom kind: {kind_str}")),
    };

    if kind == AtomKind::Ws && payload.len() >= 2 && payload.starts_with('\'') && payload.ends_with('\'') {
        payload = &payload[1..payload.len() - 1];
    }

    Ok((kind, payload))
}

fn parse_template_id(s: &str) -> Result<usize> {
    if !s.starts_with("<T") || !s.ends_with('>') { return Err(anyhow!("Bad template id: {s}")); }
    let inner = &s[2..s.len() - 1];
    inner.parse::<usize>().map_err(|_| anyhow!("Bad template id: {s}"))
}
