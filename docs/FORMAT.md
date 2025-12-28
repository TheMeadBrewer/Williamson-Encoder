# Williamson v93 Encoded File Format

**Version:** 1  
**Magic:** `0x57494C4C` ("WILL" in ASCII)

---

## Overview

The canonical encoded output is a binary file containing:

1. A header with magic number and version
2. A stream of u32 token IDs
3. A list of UTF-8 slot strings

This format is designed for lossless round-trip encoding/decoding using only the lexicon file.

---

## Binary Layout

All integers are **little-endian**.

```
Offset  Size     Type      Description
------  ----     ----      -----------
0       4        u32       Magic number (0x57494C4C)
4       4        u32       Format version (1)
8       8        u64       Token count (n)
16      n*4      u32[]     Token IDs
16+n*4  8        u64       Slot count (m)
24+n*4  variable slot[]    Slot strings (length-prefixed)
```

### Slot String Format

Each slot is stored as:

```
[4 bytes]  u32 LE: byte length of string (len)
[len bytes] UTF-8 string data (no null terminator)
```

---

## Token ID Semantics

Token IDs are **not word indices**. They are structural template indices.

- IDs `0` to `template_count - 1`: Template matches
- IDs `≥ template_count`: Literal atom fallbacks (offset by template_count)

The actual words/numbers that fill variable positions (VAR, CAP, NUM) are stored in the **slots** section, consumed in order as templates are decoded.

---

## Example

Input: `"restrictions"`

```
Header:
  Magic:   0x57494C4C
  Version: 1

Tokens:
  Count: 1
  IDs:   [84876]

Slots:
  Count: 1
  [0]:   "restrictions" (12 bytes)
```

Total file size: `4 + 4 + 8 + 4 + 8 + 4 + 12 = 44 bytes`

---

## Validation

A well-formed encoded file must satisfy:

1. Magic equals `0x57494C4C`
2. Version equals `1` (or supported version)
3. File size ≥ `16 + 4*n + 8` bytes (header + tokens + slot count)
4. All slot strings are valid UTF-8
5. `decode(encode(text)) == text` (lossless round-trip)

---

## Decoding Algorithm

```
1. Read header, verify magic and version
2. Read token count (n), then n token IDs
3. Read slot count (m), then m slot strings
4. Initialize slot_cursor = 0
5. For each token ID:
   a. If ID < template_count:
      - Look up template by ID
      - For each atom in template:
        - If atom is fixed (LIT, WS, PUNC): emit payload from interner
        - If atom is slot (VAR, CAP, NUM): emit slots[slot_cursor++]
   b. Else (literal atom fallback):
      - atom_id = ID - template_count
      - Emit atom payload (or consume slot if slot-type)
6. Return concatenated output
```

---

## Notes

- The slot section exists because variable content (actual words, numbers) cannot be stored in template IDs alone.
- If you need integer-only output for embeddings, you must define your own slot encoding scheme.
- This format is specific to v93. Future versions may use different magic/version.

---

## Reference Implementation

See `rust/src/main.rs`:

- `write_canonical()` - Encoding
- `read_canonical()` - Decoding
- `decode_canonical()` - Token stream to text
