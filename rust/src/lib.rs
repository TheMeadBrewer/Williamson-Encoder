//! Williamson Encoder - Structure-aware, lossless tokenization
//! 
//! A tokenizer that achieves better compression than BPE by recognizing
//! grammatical patterns rather than just counting byte frequencies.
//!
//! Built by: Matthew Williamson, Claude, GPT, Grok
//! December 2025

pub mod interner;
pub mod template;
pub mod trie;
pub mod tokenize;
pub mod encode;
pub mod decode;
pub mod bench;
pub mod loader_v92;

use anyhow::Result;

pub use interner::{AtomId, AtomKind, Interner, PayloadId, SlotValue};
pub use template::Template;
pub use trie::Trie;
pub use encode::{EncTok, EncodeResult, EncodeStats};
pub use tokenize::{StructuredInput, TokenizeConfig};
pub use loader_v92::load_python_v92_json;

/// Full encoder/decoder bundle (templates + trie + intern tables).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Encoder {
    pub interner: Interner,
    pub templates: Vec<Template>,
    pub trie: Trie,
}

impl Encoder {
    /// Build an Encoder from already-prepared templates and a frozen interner.
    pub fn from_templates(interner: Interner, templates: Vec<Template>) -> Self {
        let trie = Trie::build(&templates);
        Self { interner, templates, trie }
    }

    /// Tokenize text into (atoms, slots). Uses mutable interner (can grow).
    pub fn tokenize(&mut self, text: &str, cfg: &TokenizeConfig) -> StructuredInput {
        tokenize::tokenize(text, cfg, &mut self.interner)
    }

    /// Tokenize with a frozen interner.
    pub fn tokenize_frozen(&self, text: &str, cfg: &TokenizeConfig) -> StructuredInput {
        tokenize::tokenize_frozen(text, cfg, &self.interner)
    }

    /// Encode a pre-tokenized stream (hot path).
    pub fn encode_stream(&self, input: &StructuredInput) -> EncodeResult {
        encode::encode_stream(&self.trie, &self.templates, input)
    }

    /// Decode an encoded result back to text (lossless).
    pub fn decode(&self, encoded: &EncodeResult) -> String {
        decode::decode(&self.interner, &self.templates, encoded)
    }

    /// Save encoder (includes intern tables + templates + trie).
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let bytes = bincode::serialize(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load encoder.
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Ok(bincode::deserialize(&bytes)?)
    }
}
