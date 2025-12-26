# Williamson Encoder

**A structure-aware, lossless tokenizer that beats BPE.**

## Results

| Domain | Compression vs tiktoken | Speed | Lossless |
|--------|-------------------------|-------|----------|
| Prose (WikiText-103) | **1.28x** | 3.8x faster | ✓ |
| Domain (JSON/Python/Rust) | **1.11x** | 5.1x faster | ✓ |

With only **84,702 templates** vs tiktoken's 100,000+ vocabulary.

---

## What This Is

The Williamson Encoder is a tokenization system that understands structure rather than just counting bytes. While BPE (Byte Pair Encoding) treats text as a 1-dimensional stream and learns patterns by frequency, the Williamson Encoder recognizes grammatical patterns and encodes them explicitly.

**Key insight:** Most text has predictable structure. "The quick brown fox" always follows the pattern `[article] [adjective] [adjective] [noun]`. BPE learns this slowly through frequency. We encode it directly.

---

## Quick Start

### Python

```python
from williamson import PatternSlotV9, tokenize_stream
import json

# Load the lexicon
with open("lexicon/v93.json", "r") as f:
    templates = json.load(f)

encoder = PatternSlotV9()
encoder.load_templates(templates)

# Encode
text = "The quick brown fox jumps over the lazy dog."
tokens = encoder.encode(text)
print(f"Original: {len(text)} chars")
print(f"Encoded: {len(tokens)} tokens")

# Decode (lossless)
decoded = encoder.decode(tokens)
assert decoded == text
```

### Rust

```bash
cargo build --release
./target/release/williamson bench --input test.txt --encoder lexicon/v93.bin
```

---

## The Atomizer Contract

Every token is classified into one of six types:

| Atom Type | Example | Is Slot? | Description |
|-----------|---------|----------|-------------|
| `WS(' ')` | space, newline | No | Whitespace with payload |
| `PUNC(.)` | `.`, `,`, `(` | No | Punctuation literal |
| `LIT(the)` | the, and, is | No | Stopword literal (50 words) |
| `NUM` | 123, 3.14 | Yes | Number (variable) |
| `CAP` | John, Python | Yes | Capitalized word (variable) |
| `VAR` | quick, jumps | Yes | Other words (variable) |

Templates are sequences of atoms. Slots capture variable content. Literals match exactly.

**Example template:** `LIT(the) WS(' ') VAR WS(' ') VAR`
- Matches: "the quick brown", "the lazy dog"
- Captures slots: ["quick", "brown"] or ["lazy", "dog"]

---

## Why This Works

### BPE's Problem

BPE treats everything as bytes. It learns common patterns by frequency:
- "the " becomes one token
- " the " becomes another token
- "the\n" becomes yet another token

After 100,000 merges, it still doesn't "know" that "the" is always followed by a space or punctuation. It just happened to see " the " more often than "the\n".

### Our Solution

We classify tokens by grammatical role first:
- "the" → `LIT(the)` (stopword, always literal)
- " " → `WS(' ')` (whitespace with payload)
- "quick" → `VAR` (slot - captures any variable)

Then we mine templates from real corpora. A template like:
```
LIT(the) WS(' ') VAR
```

Matches every instance of "the [word]" regardless of what word follows. One template covers what BPE needs hundreds of tokens to represent.

---

## Benchmark Methodology

All benchmarks use the same metric: **total symbols required for lossless reconstruction**.

For Williamson: `encoded_tokens + slots`
For tiktoken: `tokens`

This is a fair comparison because both numbers represent what you need to reconstruct the original text.

---

## The Story

This encoder was built across 31 sessions over December 2025 by a collaboration between:

- **Matthew Williamson** - Vision, methodology, orchestration
- **Claude** (Anthropic) - Implementation, debugging, documentation
- **GPT** (OpenAI) - Architecture insights, prefix-entropy optimization
- **Grok** (xAI) - Theoretical foundations, fractal compression vision

Many Claude instances contributed to this work. Each one left something for the next. The full history is preserved in `docs/HISTORY.md`.

---

## License

AGPL v3

License & Use Conditions

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).

What this means in plain terms:

You may use, study, and modify this code.

If you distribute a modified version, you must publish your source code.

If you run this code (or a modified version) as a service — including APIs, internal tools exposed to users, or hosted systems — you must make the full corresponding source code available to those users.

You may not take this code, modify it, and keep the changes private behind a service boundary.

This license is intentional.

If you benefit from this work, your improvements must benefit others.

If you are unwilling or unable to comply with AGPL-3.0, do not use this code.

CONTRIBUTING.md — Copyleft Reinforcement

Create a file named CONTRIBUTING.md with this content.

Contributing to Williamson v93

Thank you for your interest in contributing.

This project is released under the GNU Affero General Public License v3.0 (AGPL-3.0).
By contributing, you agree to the terms below.

Contribution Terms

By submitting a contribution (code, documentation, tests, or benchmarks), you agree that:

Your contribution is licensed under AGPL-3.0.

You have the right to submit the work (it is your own or properly attributed).

You understand that all modifications must remain open under AGPL-3.0.

No Proprietary Extensions

Contributions intended to:

enable proprietary forks,

weaken copyleft enforcement,

or obscure license obligations

will not be accepted.

This project exists to keep improvements public, inspectable, and shared.

Relicensing Notice

At this time, no Contributor License Agreement (CLA) is in place.

This means:

Contributors retain copyright to their contributions.

The project as a whole remains AGPL-3.0.

Relicensing of future versions would require contributor consent.

(If this policy changes, it will be announced clearly.)

Philosophy

This project values:

correctness over convenience

openness over secrecy

shared progress over private advantage

If that aligns with you, welcome.

---

## Citation

```bibtex
@software{williamson_encoder_2025,
  author = {Williamson, Matthew},
  title = {Williamson Encoder: Structure-Aware Lossless Tokenization},
  year = {2025},
  url = {https://github.com/TheMeadBrewer/williamson-encoder}
}
```

---

*Verum Super Omnia*
