//! Benchmarking utilities.

use crate::{Encoder, StructuredInput, TokenizeConfig};

#[derive(Clone, Debug)]
pub struct BenchResult {
    pub chars: usize,
    pub atoms: usize,
    pub encoded_toks: usize,
    pub slots: usize,
    pub seconds: f64,
    pub chars_per_sec: f64,
    pub positions: u64,
    pub trie_steps: u64,
    pub template_hits: u64,
    pub literal_emits: u64,
    pub avg_steps_per_pos: f64,
    pub compression_ratio: f64,
}

pub fn bench_encode(encoder: &mut Encoder, text: &str, cfg: &TokenizeConfig) -> BenchResult {
    let start = std::time::Instant::now();
    let input = encoder.tokenize(text, cfg);
    let encoded = encoder.encode_stream(&input);
    let secs = start.elapsed().as_secs_f64();

    let chars = text.len();
    let atoms = input.atoms.len();
    let encoded_toks = encoded.toks.len();
    let slots = encoded.slots.len();
    let cps = (chars as f64) / secs.max(1e-9);
    
    let positions = encoded.stats.positions;
    let trie_steps = encoded.stats.trie_steps;
    let template_hits = encoded.stats.template_hits;
    let literal_emits = encoded.stats.literal_emits;
    let avg = if positions > 0 { (trie_steps as f64) / (positions as f64) } else { 0.0 };
    
    let total_output = encoded_toks + slots;
    let compression_ratio = if total_output > 0 { (atoms as f64) / (total_output as f64) } else { 1.0 };

    BenchResult {
        chars, atoms, encoded_toks, slots, seconds: secs, chars_per_sec: cps,
        positions, trie_steps, template_hits, literal_emits, avg_steps_per_pos: avg, compression_ratio,
    }
}

pub fn verify_lossless(encoder: &mut Encoder, text: &str, cfg: &TokenizeConfig) -> bool {
    let input = encoder.tokenize(text, cfg);
    let encoded = encoder.encode_stream(&input);
    let decoded = encoder.decode(&encoded);
    decoded == text
}
