# WILLIAMSON ENCODER ARCHITECTURE

## This Is Not BPE

If you're coming from tiktoken, SentencePiece, or other BPE-family tokenizers, **forget everything you know.** The Williamson Encoder works on fundamentally different principles. Most "bugs" filed against this repo are actually misunderstandings of the architecture.

---

## Core Concept: Structural Templates, Not Substrings

### BPE (What You Know)
- Vocabulary = list of substrings ("ing", "tion", "the", " re")
- Encoding = greedy match longest substrings
- Token ID = index into substring vocabulary
- Every unique word fragment needs its own vocab entry

### Williamson (What This Is)
- Vocabulary = list of **structural templates** (patterns of atom types)
- Encoding = match structural patterns, emit template ID + slot values
- Token ID = index into template vocabulary
- Words are **slot values**, not vocabulary entries

---

## The Atom Layer

Text is first converted to an atom stream. Atoms are structural classifications:

| Atom | Meaning | Example | Is Slot? |
|------|---------|---------|----------|
| `VAR` | Lowercase word | "restrictions", "hello" | ✓ Yes |
| `CAP` | Capitalized word | "Matthew", "Python" | ✓ Yes |
| `NUM` | Number | "42", "3.14" | ✓ Yes |
| `PUNC(x)` | Punctuation | `PUNC(.)`, `PUNC(,)` | No |
| `LIT(x)` | Stopword | `LIT(the)`, `LIT(and)` | No |
| `WS(x)` | Whitespace | `WS(' ')`, `WS('\n')` | No |

**Critical:** The actual text "restrictions" becomes the atom `VAR`. The word itself is stored separately as a **slot value**.

---

## Templates

A template is a sequence of atoms that recurs frequently. Examples:

```
Template 1042: [VAR, PUNC(.), VAR]
  Matches: "hello.world", "foo.bar", "request.body"
  
Template 3891: [LIT(the), VAR, LIT(of), LIT(the), VAR]
  Matches: "the king of the castle", "the start of the journey"
  
Template 7722: [CAP, PUNC((), VAR, PUNC()), PUNC(:)]
  Matches: "Initialize(config):", "Process(data):"
```

---

## Encoding Output

When we encode text, we output:

1. **Template IDs** - which structural pattern matched
2. **Slot values** - the actual words/numbers that filled VAR/CAP/NUM positions

```
Input:  "the king of the castle"
Atoms:  [LIT(the), VAR, LIT(of), LIT(the), VAR]
Output: Template 3891, slots=["king", "castle"]
```

The token stream is template IDs. The slot values go to a separate interner.

---

## FAQ: "Bugs" That Aren't Bugs

### "I can't find 'restrictions' in the vocabulary"
**Correct behavior.** Words aren't in the vocabulary. Templates are. The word "restrictions" fills a VAR slot in whatever template matches its context.

### "Whitespace seems wrong / missing"
**Correct behavior.** Whitespace is structural. `WS(' ')` and `WS('\n')` are atoms. They participate in templates. They're not stripped or normalized.

### "I can't get embeddings for tokens"
**Out of scope.** This is a tokenizer/compressor, not an embedding model. The output is integer IDs for structural patterns. If you want embeddings, train them downstream.

### "Token count is different from tiktoken"
**Correct behavior.** Different architecture = different count. We consistently produce fewer tokens because templates capture recurring structure that BPE treats as independent substrings.

### "The Python and Rust implementations differ"
**By design.** Python is reference/prototyping. Rust is production. Both follow the same atomizer contract. Use Rust for benchmarks.

---

## Architecture Diagram

```
                    WILLIAMSON ENCODER
                    
Input Text ──────────────────────────────────────────────►
     │
     ▼
┌─────────────┐
│  Atomizer   │  "the cat sat" → [LIT(the), VAR, VAR]
└─────────────┘
     │
     ▼
┌─────────────┐
│  Template   │  Match against template vocabulary
│  Matcher    │  
└─────────────┘
     │
     ├──────────────────┬────────────────────►
     ▼                  ▼
┌──────────┐     ┌──────────────┐
│ Template │     │ Slot Values  │
│ IDs      │     │ (Interner)   │
│ [3891,   │     │ ["cat",      │
│  1042,   │     │  "sat", ...] │
│  ...]    │     └──────────────┘
└──────────┘
     │
     ▼
Encoded Output (lossless, decode reverses exactly)
```

---

## Why This Beats BPE

1. **Structural compression**: Common patterns like `function(arg):` become single tokens regardless of the actual function/arg names.

2. **Generalization**: Templates learned from Wikipedia compress Don Quixote (not in training) at 1.85x.

3. **Semantic coherence**: A template represents a structural concept, not an arbitrary substring boundary.

4. **Lossless**: Round-trip verified. Decode(Encode(text)) == text. Always.

---

## Benchmarks

| Corpus | Williamson V93 | tiktoken o200k | Ratio |
|--------|----------------|----------------|-------|
| Don Quixote (2.4MB) | 249,775 | 575,441 | 2.30x fewer |
| WikiText-103 test | See benchmarks/ | - | ~1.3-1.5x fewer |
| Python code | See benchmarks/ | - | ~1.2-1.4x fewer |

---

## Contributing

Before filing an issue, ask yourself: "Am I assuming BPE behavior?"

If yes, re-read this document.

If you've found an actual bug (crash, incorrect decode, lossless failure), please include:
- Input text that triggers the issue
- Expected vs actual output
- Which binary/script you used

---

## License

MIT. Use it, extend it, learn from it. Just don't file BPE-assumption bugs.
