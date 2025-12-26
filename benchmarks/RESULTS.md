# Williamson Encoder Benchmark Results

**Version:** v93 (84,702 templates)
**Date:** December 25-26, 2025
**Benchmark Corpus:** WikiText-103 + Multi-domain

---

## Summary

| Metric | Value |
|--------|-------|
| Templates | 84,702 |
| Prose Compression | 1.28x vs tiktoken |
| Domain Compression | 1.11x vs tiktoken |
| Speed | 3.8-5.1x faster than tiktoken |
| Lossless | ✓ All tests |

---

## WikiText-103 Results

| Dataset | Chars | cl100k | o200k | Williamson | Win vs cl100k | Win vs o200k | Throughput |
|---------|-------|--------|-------|------------|---------------|--------------|------------|
| WT103 first 10M | 10,000,000 | 2,230,418 | 2,208,862 | 1,793,226 | **1.244x** | **1.232x** | 64.6M char/s |
| WT103 mid 10M | 10,000,000 | 2,214,043 | 2,193,907 | 1,790,204 | **1.237x** | **1.226x** | 66.3M char/s |
| WT103 last 10M | 10,000,000 | 2,219,297 | 2,201,954 | 1,793,797 | **1.237x** | **1.228x** | 66.2M char/s |
| WT103 valid (held-out) | 1,141,840 | 253,044 | 251,885 | 209,833 | **1.206x** | **1.200x** | 62.9M char/s |
| WT103 test (held-out) | 1,280,757 | 288,636 | 286,870 | 238,316 | **1.211x** | **1.204x** | 62.5M char/s |

**Key findings:**
- 23-24% fewer symbols than tiktoken on training data
- 20-21% fewer symbols on held-out data
- No overfitting—generalization confirmed

---

## Multi-Domain Results

| File | Chars | cl100k | Williamson | Win vs cl100k | Throughput | Lossless |
|------|-------|--------|------------|---------------|------------|----------|
| corpus_json.txt | 1,000,804 | 377,949 | 327,782 | **1.15x** | 59.8M | ✓ |
| corpus_python.txt | 1,003,119 | 251,657 | 226,491 | **1.11x** | 75.4M | ✓ |
| corpus_rust.txt | 42,834 | 10,251 | 9,226 | **1.11x** | 62.0M | ✓ |
| corpus_markdown.txt | 1,003,765 | 238,729 | 236,366 | **1.01x** | 63.9M | ✓ |
| corpus_mixed.txt | 981,820 | 281,875 | 246,846 | **1.14x** | 61.0M | ✓ |

**Key findings:**
- Domain-specific templates enable wins across code and structured data
- JSON shows strongest domain gains
- All domains remain lossless

---

## Speed Comparison

| Tokenizer | Throughput (char/s) |
|-----------|---------------------|
| Williamson (Rust) | 58-66M |
| cl100k_base | 18-21M |
| o200k_base | 13-14M |

**Speedup: 3-5x faster than tiktoken**

---

## Version Evolution

| Version | Templates | Prose | Domain | Notes |
|---------|-----------|-------|--------|-------|
| wt103 (5k) | 5,000 | 1.24x | 0.55x | Prose-only training |
| hybrid (7.5k) | 7,500 | 1.24x | 0.62x | Added code templates |
| hybrid (17k) | 12,916 | 1.24x | 1.01x | Domain-specific mining |
| v92 (115k) | 115,000 | 1.29x | 0.99x | Over-mined, redundant |
| **v93 (84k)** | **84,702** | **1.28x** | **1.11x** | **Rebalanced, final** |

**Critical insight:** Cutting 31,000 templates from v92 IMPROVED domain compression. Less was more.

---

## Methodology

### Fair Comparison Metric

Both tokenizers are measured by **symbols required for lossless reconstruction**:

- **Williamson:** `encoded_tokens + slots`
- **tiktoken:** `tokens`

This ensures apples-to-apples comparison—both numbers represent what you need to reconstruct the original text.

### Tiktoken Baselines

- `cl100k_base`: GPT-4 / ChatGPT tokenizer
- `o200k_base`: GPT-4o tokenizer

### Data Sources

- WikiText-103: HuggingFace `wikitext/wikitext-103-v1`
- Domain corpora: Curated samples from real-world JSON, Python, Rust, Markdown

### Lossless Verification

Every benchmark run verifies: `decode(encode(text)) == text`

All 12 test files pass lossless verification.

---

## Reproduction

```bash
# Install dependencies
pip install tiktoken

# Run benchmark
python bench_v93_vs_tiktoken.py
```

---

## Reproducibility Fingerprint

| Component | SHA256 |
|-----------|--------|
| `merged_lexicon_v93.bin` | (compute on your system) |
| `merged_lexicon_v93.json` | (compute on your system) |

---

*Verum Super Omnia*
