//! Williamson Encoder CLI
//! 
//! Commands:
//!   encode  - Encode a text file
//!   decode  - Decode back to text
//!   bench   - Benchmark on a text file
//!   load-py-v92 - Load Python v9.2 templates JSON

use anyhow::Result;
use clap::{Parser, Subcommand};
use williamson_encoder::{bench, Encoder, Interner, Template, TokenizeConfig, AtomKind, EncodeResult};

/// Bundled output format: encoded result + interner snapshot for self-contained decode
#[derive(serde::Serialize, serde::Deserialize)]
struct EncodedBundle {
    encoded: EncodeResult,
    interner: Interner,
}

#[derive(Parser)]
#[command(name = "williamson")]
#[command(about = "Williamson Encoder - Structure-aware tokenization that beats BPE", long_about = None)]
#[command(version = "0.93.0")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a text file using a saved encoder
    Encode {
        #[arg(long)]
        encoder: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: String,
    },
    /// Decode an encoded file back to text
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
    }
}

fn create_demo_encoder() -> Encoder {
    let mut interner = Interner::default();
    
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
