//! Trie structure for fast template matching.

use crate::{AtomId, Template};
use hashbrown::HashMap;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub term_tid: i32,
    pub edges_start: u32,
    pub edges_len: u16,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Edge {
    pub tok: AtomId,
    pub next: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Trie {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Trie {
    pub fn build(templates: &[Template]) -> Self {
        let mut children: Vec<HashMap<AtomId, u32>> = vec![HashMap::new()];
        let mut term: Vec<i32> = vec![-1];

        for (tid, t) in templates.iter().enumerate() {
            let mut node = 0u32;
            for &a in &t.atoms {
                let next = if let Some(&nx) = children[node as usize].get(&a) {
                    nx
                } else {
                    let nx = children.len() as u32;
                    children.push(HashMap::new());
                    term.push(-1);
                    children[node as usize].insert(a, nx);
                    nx
                };
                node = next;
            }
            term[node as usize] = tid as i32;
        }

        let mut nodes: Vec<Node> = Vec::with_capacity(children.len());
        let mut edges: Vec<Edge> = Vec::new();

        for i in 0..children.len() {
            let start = edges.len() as u32;
            let mut es: Vec<(AtomId, u32)> = children[i].iter().map(|(&k, &v)| (k, v)).collect();
            es.sort_by_key(|(k, _)| *k);
            for (tok, next) in es {
                edges.push(Edge { tok, next });
            }
            let len = (edges.len() as u32 - start) as u16;
            nodes.push(Node { term_tid: term[i], edges_start: start, edges_len: len });
        }

        Self { nodes, edges }
    }

    #[inline]
    pub fn match_longest(&self, stream: &[AtomId], pos: usize) -> (usize, i32, usize) {
        let mut node = 0u32;
        let mut depth = 0usize;
        let mut best_len = 0usize;
        let mut best_tid = -1i32;
        let mut steps = 0usize;

        while pos + depth < stream.len() {
            let tok = stream[pos + depth];
            let n = &self.nodes[node as usize];
            let start = n.edges_start as usize;
            let end = start + n.edges_len as usize;
            let slice = &self.edges[start..end];

            let found = self.binary_search_edge(slice, tok);
            steps += 1;

            let Some(next) = found else { break };
            node = next;
            depth += 1;

            let tn = self.nodes[node as usize].term_tid;
            if tn >= 0 {
                best_len = depth;
                best_tid = tn;
            }
        }

        (best_len, best_tid, steps)
    }

    #[inline]
    fn binary_search_edge(&self, slice: &[Edge], tok: AtomId) -> Option<u32> {
        let mut lo = 0usize;
        let mut hi = slice.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            let m = slice[mid].tok;
            if m == tok { return Some(slice[mid].next); }
            else if m < tok { lo = mid + 1; }
            else { hi = mid; }
        }
        None
    }
}
