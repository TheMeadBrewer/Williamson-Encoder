# Williamson Encoder v93

**Status:** Frozen release  
**License:** GNU Affero General Public License v3.0 (AGPL-3.0)

Williamson v93 is a lossless text encoding system built around large-scale structural lexicon matching. It is the final result of a first-generation design approach and is published as a stable, complete artifact.

This repository exists to preserve that work as it actually stood: measured, benchmarked, and finished.

---

## Results

| Domain | Compression vs tiktoken | Speed | Lossless |
|--------|-------------------------|-------|----------|
| Prose (WikiText-103) | **1.28x** | 3.8x faster | ✓ |
| Domain (JSON/Python/Rust) | **1.11x** | 5.1x faster | ✓ |

With only **84,702 templates** vs tiktoken's 100,000+ vocabulary.

---

## Model-Facing Token IDs (Canonical Interface)

The canonical interface produces pure integer token streams suitable for downstream model consumption.

**Commands:**

```bash
# Encode text to token IDs
williamson encode-ids --lex merged_lexicon_v93.bin --in input.txt --out encoded.bin

# Decode token IDs back to text
williamson decode-ids --lex merged_lexicon_v93.bin --in encoded.bin --out decoded.txt

# One-command lossless verification
williamson roundtrip --lex merged_lexicon_v93.bin --in input.txt
```

**File Format (encoded.bin):**
```
[4 bytes]   u32 LE: magic (0x57494C4C = "WILL")
[4 bytes]   u32 LE: version (1)
[8 bytes]   u64 LE: token count (n)
[n*4 bytes] u32 LE: token IDs
[8 bytes]   u64 LE: slot count (m)
[variable]  m length-prefixed UTF-8 strings (slot values)
```

**Important:** Token IDs are structural template indices, not word indices. The slot values section contains the actual words/numbers that fill VAR/CAP/NUM positions. See [ARCHITECTURE.md](ARCHITECTURE.md) for the full explanation.

---

## Overview

Most modern tokenizers rely on byte-pair encoding (BPE) or closely related variants. These approaches optimize for frequency, gradually merging adjacent symbols and hoping structure emerges over time.

Williamson takes a different path.

Instead of merging symbols, it identifies explicit multi-token patterns and substitutes them directly using a fixed lexicon. The result is a compact, lossless representation with predictable, linear-time performance.

Version 93 ("v93") represents the point at which this approach was fully explored and brought to completion.

---

## How It Works

Williamson v93 operates on a flat, one-dimensional atom stream.

Text is first atomized into a sequence of basic units (literals, identifiers, punctuation, whitespace, and so on). A large lexicon—approximately 85,000 templates—captures frequent structural patterns across these atoms.

During encoding, templates are matched greedily in linear time and replaced with compact identifiers. Decoding reverses the process and reproduces the original byte sequence exactly.

Note: In v93, whitespace is treated as payload. This is a deliberate architectural choice and a known limitation of the design, documented here for completeness.

---

## Design Constraints

From the beginning, v93 was developed under a small set of non-negotiable rules:

- Lossless or fail
- Benchmarks decide
- Deterministic builds
- JSON artifacts only
- No hidden state
- No silent phases

Every optimization was required to survive measurement. Anything that did not was removed.

---

## Scale

- Approximately 85,000 templates
- Template lengths up to 30 atoms
- SAM-based deduplication
- Deterministic lexicon construction
- Fully reproducible builds

This scale is intentional: large enough to capture structure, bounded enough to remain fast.

---

## Reproducibility

All published results can be reproduced using the material in this repository.

**Artifacts**
- Lexicon: `lexicon/v93.json`
- Benchmarks: `benchmarks/`

**Commands**
```bash
# Python benchmarks
python bench_v93_vs_tiktoken.py

# Rust benchmarks
cargo run --release -- bench
```

Benchmark outputs and historical measurements are recorded in `docs/HISTORY.md`.

---

## Known Limits

Williamson v93 operates entirely within a flat, one-dimensional stream model.

As a result:

- Structure is implicit rather than explicit
- Layout is inferred rather than factored
- Higher-order relationships are not first-class entities

These are architectural limits, not implementation bugs.

---

## Why v93 Is Frozen

v93 is frozen because it is complete within its design space.

All meaningful optimizations compatible with this architecture were explored, benchmarked, and exhausted. Further gains would be marginal and duplicative.

Freezing the release preserves reproducibility, benchmark integrity, and historical clarity.

---

## Repository Contents

This repository contains everything required to understand and reproduce v93:

- Encoder and decoder implementations (Python and Rust)
- Lexicon artifacts
- Benchmark harnesses
- Build and validation scripts
- Historical notes in `docs/HISTORY.md`

---

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

In practical terms:

- You may use, study, and modify this code.
- If you distribute a modified version, you must publish the corresponding source.
- If you run this code (or a modified version) as a service, you must make the source available to users of that service.

This license is intentional.

If you benefit from this work, your improvements must remain public.

See the LICENSE file for the full terms.

---

## Acknowledgements

Williamson v93 is the product of sustained iteration, disagreement, correction, and testing.

This repository exists to preserve that work honestly—both its successes and its limits.

---

## Final Note

Williamson v93 does exactly what it claims to do.

Nothing more.  
Nothing less.
