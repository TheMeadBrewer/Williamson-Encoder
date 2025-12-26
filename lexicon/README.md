# Lexicon Files

This directory contains the trained template lexicons.

## v93.json

The v93 lexicon contains 84,702 templates trained on:
- WikiText-103 (prose)
- Multi-domain corpora (JSON, Python, Rust, Markdown)

### Format

```json
{
  "version": "9.2",
  "str_to_id": {
    "VAR": 0,
    "CAP": 1,
    "NUM": 2,
    "LIT(the)": 3,
    ...
  },
  "id_to_template": {
    "<T0>": ["LIT(the)", "WS(' ')", "VAR"],
    "<T1>": ["VAR", "WS(' ')", "LIT(and)", "WS(' ')", "VAR"],
    ...
  }
}
```

### Loading

**Python:**
```python
import json
with open("v93.json") as f:
    data = json.load(f)
```

**Rust:**
```bash
williamson load-py-v92 --input v93.json --output v93.bin
```

## v93.bin

Binary format for the Rust implementation. Generated from v93.json using the `load-py-v92` command.
