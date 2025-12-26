Williamson Encoder v93

Status: Frozen release
License: GNU Affero General Public License v3.0 (AGPL-3.0)
Guarantees: Lossless • Linear-time • Benchmarked

Overview

Williamson v93 is a lossless text encoding system based on large-scale structural lexicon matching.

Instead of byte-pair merges, Williamson identifies and substitutes frequent multi-token patterns using a fixed lexicon, producing compact encodings with predictable, linear-time performance.

v93 is a complete, first-generation system.
It is published as a stable baseline and is intentionally frozen.

Key Results
Compression

Measured against modern BPE tokenizers on real corpora:

Prose: ~1.28× improvement

Domain text (code / JSON / markdown): up to 1.11× improvement

Lossless: ✓ byte-for-byte reconstruction

Performance

Encode: ≥ 3× faster than cl100k

Decode: faster than encode

Streaming, single-pass operation

No backtracking, no regex engines, no heuristics

How It Works (High Level)

Williamson v93 operates on a flat 1D atom stream:

Text is atomized into literals, variables, punctuation, whitespace, etc.

A large lexicon (~85k templates) captures frequent structural patterns.

Templates are matched greedily in linear time.

Encoded output substitutes templates with compact IDs.

Decoding restores the original text exactly.

Note: v93 treats whitespace as payload.
This is an explicit architectural choice and a known limit of the design.

Design Constraints

v93 was built under strict rules:

Lossless or fail

Benchmarks decide

Deterministic builds

JSON artifacts only

No hidden state

No silent phases

Anything that did not survive benchmarking was removed.

Scale

~85,000 templates

Template lengths up to 30 atoms

SAM-based deduplication

Deterministic lexicon builds

Reproducible results

Reproducibility

All published results can be reproduced using the included scripts.

Artifacts

Lexicon: lexicon/lexicon_v93.json

Benchmarks: bench/

Commands

# Python benchmarks
python bench_100k_vs_tiktoken.py

# Rust benchmarks
cargo run --release -- bench


Benchmark logs and historical results are recorded in HISTORY.md.

Known Limits

Williamson v93 operates entirely in a 1D stream model.

As a result:

Structure is implicit, not explicit

Layout is inferred, not factored

Higher-order relationships are not first-class

These are architectural limits, not bugs.

Why v93 Is Frozen

v93 is frozen because it is complete within its design space.

All major optimizations compatible with this architecture were explored, benchmarked, and exhausted. Further gains would be marginal or duplicative.

Freezing v93 preserves:

Reproducibility

Benchmark integrity

Historical clarity

Repository Contents

Encoder / decoder implementations (Python + Rust)

Lexicon artifacts

Benchmark harnesses

Build and validation scripts

Historical notes (HISTORY.md)

Everything required to reproduce published results is included.

License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).

In plain terms:

You may use, study, and modify this code.

If you distribute modified versions, you must publish your source.

If you run this code (or a modified version) as a service, you must make the source available to users of that service.

This license is intentional.

If you benefit from this work, your improvements must remain public.

See the LICENSE file for full terms.

Acknowledgements

Williamson v93 is the product of sustained iteration, correction, and testing.

This repository exists to preserve that work honestly — including both its successes and its limits.

Final Note

Williamson v93 does exactly what it claims to do.

Nothing more.
Nothing less.
