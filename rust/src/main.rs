//! Williamson Encoder CLI
//! 
//! Commands:
//!   encode     - Encode a text file (bundled format with interner)
//!   decode     - Decode bundled format back to text
//!   bench      - Benchmark on a text file
//!   verify     - Verify lossless round-trip
//!   load-py-v92 - Load Python v9.2/v9.3 templates JSON
//!
//! Canonical (model-facing) commands:
//!   encode-ids - Encode to pure u32 token stream + slots
//!   decode-ids - Decode from canonical format
//!   roundtrip  - Verify lossless encode->decode in one command

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use williamson_encoder::{bench, Encoder, Interner, Template, TokenizeConfig, AtomKind, EncodeResult, EncTok};
use std::io::{Read, Write, BufReader, BufWriter};

/// Bundled output format: encoded result + interner snapshot for self-contained decode
#[derive(serde::Serialize, serde::Deserialize)]
struct EncodedBundle {
    encoded: EncodeResult,
    interner: Interner,
}

#[derive(Parser)]
#[command(name = "williamson")]
#[command(about = "Williamson Encoder - Structure-aware tokenization that beats BPE", long_about = None)]
#[command(version = "0.93.1")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a text file using a saved encoder (bundled format)
    Encode {
        #[arg(long)]
        encoder: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    /// Decode a bundled encoded file back to text
    Decode {
        #[arg(long)]
        encoder: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    /// Benchmark tokenize+encode on a text file
    Bench {
        #[arg(long)]
        input: String,
        #[arg(long, default_value_t = false)]
        ws_run: bool,
        /// Use saved encoder instead of demo
        #[arg(long)]
        encoder: Option<String>,
    },
    /// Verify lossless round-trip on a text file
    Verify {
        #[arg(long)]
        input: String,
        #[arg(long)]
        encoder: Option<String>,
    },
    /// Load Python v9.2/v9.3 templates JSON and save a Rust encoder.bin
    LoadPyV92 {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    
    // === CANONICAL MODEL-FACING COMMANDS ===
    
    /// Encode to canonical format: pure u32 token IDs + slot strings
    EncodeIds {
        /// Path to lexicon (.bin)
        #[arg(long)]
        lex: String,
        /// Input text file
        #[arg(long)]
        input: String,
        /// Output encoded file (.bin)
        #[arg(long)]
        out: String,
        /// Print first N token IDs to stdout
        #[arg(long)]
        dump: Option<usize>,
    },
    /// Decode from canonical format back to text
    DecodeIds {
        /// Path to lexicon (.bin)
        #[arg(long)]
        lex: String,
        /// Input encoded file (.bin)
        #[arg(long)]
        input: String,
        /// Output text file
        #[arg(long)]
        out: String,
    },
    /// One-command roundtrip verification
    Roundtrip {
        /// Path to lexicon (.bin)
        #[arg(long)]
        lex: String,
        /// Input text file
        #[arg(long)]
        input: String,
    },
}

// === CANONICAL FORMAT ===
// 
// File structure (all little-endian):
//   [4 bytes]   u32: magic (0x57494C4C = "WILL")
//   [4 bytes]   u32: version (1)
//   [8 bytes]   u64: token count (n)
//   [n*4 bytes] u32[]: token IDs
//   [8 bytes]   u64: slot count (m)
//   For each slot:
//     [4 bytes] u32: string byte length (len)
//     [len bytes] UTF-8 string data
//
// Token ID encoding:
//   - Template hits: ID directly (0 to template_count-1)
//   - Literal atoms: template_count + atom_id

const MAGIC: u32 = 0x57494C4C; // "WILL"
const VERSION: u32 = 1;

fn write_canonical(
    path: &str,
    toks: &[u32],
    slots: &[String],
) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut w = BufWriter::new(file);
    
    // Header
    w.write_all(&MAGIC.to_le_bytes())?;
    w.write_all(&VERSION.to_le_bytes())?;
    
    // Tokens
    let n = toks.len() as u64;
    w.write_all(&n.to_le_bytes())?;
    for &t in toks {
        w.write_all(&t.to_le_bytes())?;
    }
    
    // Slots
    let m = slots.len() as u64;
    w.write_all(&m.to_le_bytes())?;
    for s in slots {
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        w.write_all(&len.to_le_bytes())?;
        w.write_all(bytes)?;
    }
    
    w.flush()?;
    Ok(())
}

fn read_canonical(path: &str) -> Result<(Vec<u32>, Vec<String>)> {
    let file = std::fs::File::open(path)?;
    let mut r = BufReader::new(file);
    
    // Header
    let mut buf4 = [0u8; 4];
    let mut buf8 = [0u8; 8];
    
    r.read_exact(&mut buf4)?;
    let magic = u32::from_le_bytes(buf4);
    if magic != MAGIC {
        return Err(anyhow!("Invalid magic: expected 0x{:08X}, got 0x{:08X}", MAGIC, magic));
    }
    
    r.read_exact(&mut buf4)?;
    let version = u32::from_le_bytes(buf4);
    if version != VERSION {
        return Err(anyhow!("Unsupported version: {}", version));
    }
    
    // Tokens
    r.read_exact(&mut buf8)?;
    let n = u64::from_le_bytes(buf8) as usize;
    
    let mut toks = Vec::with_capacity(n);
    for _ in 0..n {
        r.read_exact(&mut buf4)?;
        toks.push(u32::from_le_bytes(buf4));
    }
    
    // Slots
    r.read_exact(&mut buf8)?;
    let m = u64::from_le_bytes(buf8) as usize;
    
    let mut slots = Vec::with_capacity(m);
    for _ in 0..m {
        r.read_exact(&mut buf4)?;
        let len = u32::from_le_bytes(buf4) as usize;
        let mut strbuf = vec![0u8; len];
        r.read_exact(&mut strbuf)?;
        slots.push(String::from_utf8(strbuf)?);
    }
    
    Ok((toks, slots))
}

/// Convert EncodeResult to canonical u32 stream + slot strings
fn to_canonical(enc: &EncodeResult, interner: &Interner, template_count: usize) -> (Vec<u32>, Vec<String>) {
    let mut toks: Vec<u32> = Vec::with_capacity(enc.toks.len());
    
    for t in &enc.toks {
        match t {
            EncTok::Template(tid) => {
                toks.push(*tid);
            }
            EncTok::LiteralAtom(aid) => {
                // Offset literal atoms above template range
                toks.push(template_count as u32 + aid);
            }
        }
    }
    
    let slots: Vec<String> = enc.slots.iter()
        .map(|sv| interner.payload_str(sv.payload).to_string())
        .collect();
    
    (toks, slots)
}

/// Decode from canonical format
fn decode_canonical(
    encoder: &Encoder,
    toks: &[u32],
    slots: &[String],
) -> String {
    let template_count = encoder.templates.len() as u32;
    let mut out = String::new();
    let mut slot_cursor = 0usize;
    
    for &tok in toks {
        if tok < template_count {
            // Template hit
            let t = &encoder.templates[tok as usize];
            for &aid in &t.atoms {
                let (kind, pid) = encoder.interner.atom_info(aid);
                match kind {
                    AtomKind::Lit | AtomKind::Ws | AtomKind::Punc => {
                        out.push_str(encoder.interner.payload_str(pid));
                    }
                    AtomKind::Var | AtomKind::Cap | AtomKind::Num | AtomKind::WsRun => {
                        if slot_cursor < slots.len() {
                            out.push_str(&slots[slot_cursor]);
                            slot_cursor += 1;
                        }
                    }
                }
            }
        } else {
            // Literal atom
            let atom_id = tok - template_count;
            let (kind, pid) = encoder.interner.atom_info(atom_id);
            match kind {
                AtomKind::Lit | AtomKind::Ws | AtomKind::Punc => {
                    out.push_str(encoder.interner.payload_str(pid));
                }
                AtomKind::Var | AtomKind::Cap | AtomKind::Num | AtomKind::WsRun => {
                    if slot_cursor < slots.len() {
                        out.push_str(&slots[slot_cursor]);
                        slot_cursor += 1;
                    }
                }
            }
        }
    }
    
    out
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Encode { encoder, input, output } => {
            println!("Loading encoder from {}...", encoder);
            let mut enc = Encoder::load(&encoder)?;
            
            println!("Reading input from {}...", input);
            let text = std::fs::read_to_string(&input)?;
            
            let cfg = TokenizeConfig::with_default_stopwords();
            let structured = enc.tokenize(&text, &cfg);
            let encoded = enc.encode_stream(&structured);
            
            println!("Encoding complete:");
            println!("  Input atoms: {}", structured.atoms.len());
            println!("  Output tokens: {}", encoded.toks.len());
            println!("  Slots: {}", encoded.slots.len());
            
            let bundle = EncodedBundle {
                encoded,
                interner: enc.interner.clone(),
            };
            
            let bytes = bincode::serialize(&bundle)?;
            std::fs::write(&output, &bytes)?;
            println!("Saved to {} ({} bytes)", output, bytes.len());
            
            Ok(())
        }
        
        Commands::Decode { encoder, input, output } => {
            println!("Loading encoder from {}...", encoder);
            let enc = Encoder::load(&encoder)?;
            
            println!("Reading encoded data from {}...", input);
            let bytes = std::fs::read(&input)?;
            let bundle: EncodedBundle = bincode::deserialize(&bytes)?;
            
            let text = williamson_encoder::decode::decode(&bundle.interner, &enc.templates, &bundle.encoded);
            std::fs::write(&output, &text)?;
            
            println!("Decoded {} chars to {}", text.len(), output);
            Ok(())
        }
        
        Commands::Bench { input, ws_run, encoder: encoder_path } => {
            println!("Reading input from {}...", input);
            let text = std::fs::read_to_string(&input)?;
            println!("Input size: {} chars", text.len());
            
            let mut encoder = if let Some(path) = encoder_path {
                println!("Loading encoder from {}...", path);
                Encoder::load(&path)?
            } else {
                println!("Using demo encoder (no templates - baseline)...");
                create_demo_encoder()
            };
            
            let cfg = TokenizeConfig {
                ws_run,
                strict_frozen: false,
                stopwords: Some(default_stopwords()),
            };
            
            println!("\nRunning benchmark...\n");
            
            let r = bench::bench_encode(&mut encoder, &text, &cfg);
            
            println!("=== BENCHMARK RESULTS ===");
            println!("Chars:           {:>12}", r.chars);
            println!("Atoms:           {:>12}", r.atoms);
            println!("Encoded tokens:  {:>12}", r.encoded_toks);
            println!("Slots:           {:>12}", r.slots);
            println!("Time:            {:>12.4} s", r.seconds);
            println!("Throughput:      {:>12.0} char/s", r.chars_per_sec);
            println!("Positions:       {:>12}", r.positions);
            println!("Trie steps:      {:>12}", r.trie_steps);
            println!("Template hits:   {:>12}", r.template_hits);
            println!("Literal emits:   {:>12}", r.literal_emits);
            println!("Avg steps/pos:   {:>12.2}", r.avg_steps_per_pos);
            println!("Compression:     {:>12.2}x", r.compression_ratio);
            
            println!("\nVerifying lossless round-trip...");
            let is_lossless = bench::verify_lossless(&mut encoder, &text, &cfg);
            if is_lossless {
                println!("✓ Lossless verified");
            } else {
                println!("✗ LOSSLESS CHECK FAILED");
            }
            
            Ok(())
        }
        
        Commands::Verify { input, encoder: encoder_path } => {
            println!("Reading input from {}...", input);
            let text = std::fs::read_to_string(&input)?;
            
            let mut encoder = if let Some(path) = encoder_path {
                Encoder::load(&path)?
            } else {
                create_demo_encoder()
            };
            
            let cfg = TokenizeConfig::with_default_stopwords();
            
            let is_lossless = bench::verify_lossless(&mut encoder, &text, &cfg);
            
            if is_lossless {
                println!("✓ Lossless: decode(encode(text)) == text");
            } else {
                println!("✗ FAILED: decode(encode(text)) != text");
            }
            
            Ok(())
        }
        
        Commands::LoadPyV92 { input, output } => {
            println!("Loading Python v9.2/v9.3 templates from {}...", input);
            let enc = williamson_encoder::loader_v92::load_python_v92_json(&input)?;
            println!("Loaded {} templates, {} atoms in interner", 
                     enc.templates.len(), 
                     enc.interner.atom_count());
            enc.save(&output)?;
            println!("Saved encoder to {}", output);
            Ok(())
        }
        
        // === CANONICAL COMMANDS ===
        
        Commands::EncodeIds { lex, input, out, dump } => {
            let mut encoder = Encoder::load(&lex)?;
            let text = std::fs::read_to_string(&input)?;
            
            let cfg = TokenizeConfig::with_default_stopwords();
            let structured = encoder.tokenize(&text, &cfg);
            let encoded = encoder.encode_stream(&structured);
            
            let (toks, slots) = to_canonical(&encoded, &encoder.interner, encoder.templates.len());
            
            write_canonical(&out, &toks, &slots)?;
            
            println!("Encoded: {} chars -> {} tokens, {} slots", text.len(), toks.len(), slots.len());
            println!("Saved to {}", out);
            
            if let Some(n) = dump {
                let n = n.min(toks.len());
                println!("\nFirst {} token IDs:", n);
                println!("{:?}", &toks[..n]);
            }
            
            Ok(())
        }
        
        Commands::DecodeIds { lex, input, out } => {
            let encoder = Encoder::load(&lex)?;
            let (toks, slots) = read_canonical(&input)?;
            
            let text = decode_canonical(&encoder, &toks, &slots);
            
            std::fs::write(&out, &text)?;
            println!("Decoded: {} tokens, {} slots -> {} chars", toks.len(), slots.len(), text.len());
            println!("Saved to {}", out);
            
            Ok(())
        }
        
        Commands::Roundtrip { lex, input } => {
            let mut encoder = Encoder::load(&lex)?;
            let original = std::fs::read_to_string(&input)?;
            
            let cfg = TokenizeConfig::with_default_stopwords();
            let structured = encoder.tokenize(&original, &cfg);
            let encoded = encoder.encode_stream(&structured);
            
            let (toks, slots) = to_canonical(&encoded, &encoder.interner, encoder.templates.len());
            let decoded = decode_canonical(&encoder, &toks, &slots);
            
            if decoded == original {
                println!("OK");
                std::process::exit(0);
            } else {
                // Find first mismatch
                let orig_bytes = original.as_bytes();
                let dec_bytes = decoded.as_bytes();
                let mut offset = 0;
                for (i, (a, b)) in orig_bytes.iter().zip(dec_bytes.iter()).enumerate() {
                    if a != b {
                        offset = i;
                        break;
                    }
                }
                if offset == 0 && orig_bytes.len() != dec_bytes.len() {
                    offset = orig_bytes.len().min(dec_bytes.len());
                }
                println!("FAIL at offset {}", offset);
                std::process::exit(1);
            }
        }
    }
}

fn create_demo_encoder() -> Encoder {
    let mut interner = williamson_encoder::Interner::default();
    
    let ws = interner.intern_payload(" ");
    let ws_atom = interner.intern_atom(AtomKind::Ws, ws);
    let var_atom = interner.intern_fixed_kind(AtomKind::Var);
    
    let tmpl1 = Template::new(
        vec![ws_atom, var_atom, ws_atom],
        vec![AtomKind::Var],
    );
    
    let tmpl2 = Template::new(
        vec![var_atom, ws_atom, var_atom],
        vec![AtomKind::Var, AtomKind::Var],
    );
    
    Encoder::from_templates(interner, vec![tmpl1, tmpl2])
}

fn default_stopwords() -> std::collections::HashSet<String> {
    [
        "the", "a", "an", "and", "or", "but", "if", "then", "else", "when", "while", "as",
        "of", "to", "in", "on", "at", "by", "for", "with", "from", "into", "over", "under",
        "is", "are", "was", "were", "be", "been", "being", "do", "does", "did", "doing",
        "have", "has", "had", "having", "will", "would", "can", "could", "may", "might",
        "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
        "this", "that", "these", "those", "there", "here",
    ].iter().map(|s| s.to_string()).collect()
}
