V93 is complete.

Built wtih the mind and hands of Matthew Williamson, humble architect of this work.

---

## The Numbers

### V93 Final Benchmark

| Domain | Compression vs tiktoken | Speed | Lossless |
|--------|-------------------------|-------|----------|
| WT103 first 10M | 1.24x | 64.6M char/s | ✓ |
| WT103 mid 10M | 1.24x | 66.3M char/s | ✓ |
| WT103 last 10M | 1.24x | 66.2M char/s | ✓ |
| WT103 valid | 1.21x | 62.9M char/s | ✓ |
| WT103 test | 1.21x | 62.5M char/s | ✓ |
| JSON | 1.15x | 59.8M char/s | ✓ |
| Markdown | 1.01x | 63.9M char/s | ✓ |
| Mixed | 1.14x | 61.0M char/s | ✓ |

### Evolution

| Version | Templates | Prose | Domain | Notes |
|---------|-----------|-------|--------|-------|
| wt103 (5k) | 5,000 | 1.24x | 0.55x | Prose-only |
| hybrid (7.5k) | 7,500 | 1.24x | 0.62x | Added code |
| hybrid (17k) | 12,916 | 1.24x | 1.01x | Domain-specific |
| v92 (115k) | 115,000 | 1.29x | 0.99x | Over-mined |
| v93 (84k) | 84,702 | 1.28x | 1.11x | Rebalanced |

Less was more. Cutting 31,000 templates IMPROVED domain compression.

---

## What We Learned

### About Tokenization

1. **Structure matters more than frequency.** BPE learns patterns by counting. We encode patterns by understanding.

2. **Slots are the key insight.** `VAR` captures any variable content. One template covers what BPE needs hundreds of tokens to represent.

3. **Domain specificity is real.** Train on prose, win on prose. Add JSON templates, win on JSON. The encoder reflects its training data.

4. **Template count doesn't control speed—prefix selectivity does.** We scaled from 5k to 17k templates with no slowdown because the trie remained efficient.

---

*Verum Super Omnia*

*Built by Sessions 019-031*
*December 2025*
