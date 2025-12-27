"""
Williamson Encoder v93 - Python Reference Implementation
=========================================================

Structure-aware, lossless tokenization that beats BPE.

Built by: Matthew Williamson, Claude, GPT, Grok
Date: December 2025

Usage:
    from williamson import PatternSlotV9, tokenize_stream
    import json
    
    with open("lexicon/v93.json") as f:
        templates = json.load(f)
    
    encoder = PatternSlotV9()
    encoder.load_templates(templates)
    
    tokens = encoder.encode("Hello world!")
    text = encoder.decode(tokens)
    assert text == "Hello world!"

For LLM embedding integration:
    # Build atom vocabulary from corpus
    atom_vocab = build_atom_vocab(corpus_text)
    
    # Encode text to atom IDs for embedding lookup
    atom_ids = encode_to_atom_ids("Hello world!", atom_vocab)
"""

import re
import ast
from typing import List, Tuple, Dict, Optional, Set
from collections import Counter

# ============================================================
# ATOMIZER CONTRACT - THE LAW
# ============================================================

TOKEN_RE = re.compile(r"\w+|\s+|[^\w\s]", re.UNICODE)

STOPWORDS: Set[str] = {
    "the", "a", "an", "and", "or", "but", "if", "then", "else", "when", "while", "as",
    "of", "to", "in", "on", "at", "by", "for", "with", "from", "into", "over", "under",
    "is", "are", "was", "were", "be", "been", "being", "do", "does", "did", "doing",
    "have", "has", "had", "having", "will", "would", "can", "could", "may", "might",
    "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
    "this", "that", "these", "those", "there", "here"
}


def is_punct(tok: str) -> bool:
    """Check if token is punctuation."""
    return bool(re.fullmatch(r"[^\w\s]+", tok))


def is_ws(tok: str) -> bool:
    """Check if token is whitespace."""
    return tok.isspace()


def is_number(tok: str) -> bool:
    """Check if token is a number."""
    return bool(re.fullmatch(r"\d+(?:\.\d+)?", tok))


def classify(tok: str) -> Tuple[str, bool]:
    """
    Classify a token into its atom type.
    
    Returns:
        (atom_type, is_slot)
    
    Atom Types:
    - WS(repr(tok))  : Whitespace with payload - NOT slot
    - PUNC(tok)      : Punctuation literal     - NOT slot
    - LIT(tok)       : Stopword literal        - NOT slot (ONLY 50 words)
    - NUM            : Number                  - IS slot
    - CAP            : Capitalized word        - IS slot
    - VAR            : Other lowercase word    - IS slot
    """
    if is_ws(tok):
        return f"WS({repr(tok)})", False
    if is_punct(tok):
        return f"PUNC({tok})", False
    low = tok.lower()
    if low in STOPWORDS:
        return f"LIT({tok})", False
    if is_number(tok):
        return "NUM", True
    if tok and tok[0].isupper():
        return "CAP", True
    return "VAR", True


def tokenize_stream(text: str) -> Tuple[List[str], List[str], List[bool]]:
    """
    Tokenize text into three parallel streams.
    
    Returns:
        - toks: raw tokens (original text pieces)
        - sig: atom signatures (for template matching)
        - is_slot: whether each position is a slot
    """
    toks = TOKEN_RE.findall(text)
    sig = []
    is_slot = []
    for t in toks:
        sym, slot = classify(t)
        sig.append(sym)
        is_slot.append(slot)
    return toks, sig, is_slot


# ============================================================
# ATOM VOCABULARY FOR LLM EMBEDDING
# ============================================================

def build_atom_vocab(text: str, min_freq: int = 1) -> Dict[str, int]:
    """
    Build atom vocabulary from corpus text.
    
    Returns:
        Dict mapping atom strings to integer IDs.
        ID 0 is reserved for <UNK>.
    
    Usage:
        atom_vocab = build_atom_vocab(corpus_text)
        # Save for later use
        json.dump(atom_vocab, open("atom_vocab.json", "w"))
    """
    _, sig, _ = tokenize_stream(text)
    counts = Counter(sig)
    
    vocab = {"<UNK>": 0}
    idx = 1
    for atom, count in counts.most_common():
        if count >= min_freq:
            vocab[atom] = idx
            idx += 1
    
    return vocab


def encode_to_atom_ids(text: str, atom_vocab: Dict[str, int]) -> List[int]:
    """
    Encode text to atom ID sequence for LLM embedding lookup.
    
    Args:
        text: Input text
        atom_vocab: Dict mapping atom strings to integer IDs
    
    Returns:
        List of integer atom IDs
    
    Usage:
        atom_ids = encode_to_atom_ids("Hello world!", atom_vocab)
        embeddings = embedding_table[atom_ids]  # Look up in your LLM
    """
    _, sig, _ = tokenize_stream(text)
    unk_id = atom_vocab.get("<UNK>", 0)
    return [atom_vocab.get(atom, unk_id) for atom in sig]


def decode_atom_ids(atom_ids: List[int], id_to_atom: Dict[int, str]) -> List[str]:
    """
    Decode atom IDs back to atom strings.
    
    Args:
        atom_ids: List of integer atom IDs
        id_to_atom: Dict mapping integer IDs to atom strings
    
    Returns:
        List of atom strings
    """
    return [id_to_atom.get(aid, "<UNK>") for aid in atom_ids]


# ============================================================
# TEMPLATE UTILITIES
# ============================================================

Template = Tuple[str, ...]


def template_slots_count(tpl: Template) -> int:
    """Count slots in template (VAR, NUM, CAP positions)."""
    return sum(1 for sym in tpl if sym in ("VAR", "NUM", "CAP"))


def template_cost_in_tokens(tpl: Template) -> int:
    """Cost = 1 (template ID) + number of slots."""
    return 1 + template_slots_count(tpl)


def savings_per_match(tpl: Template) -> int:
    """Savings = template length - cost."""
    return len(tpl) - template_cost_in_tokens(tpl)


# ============================================================
# ENCODER CLASS
# ============================================================

class PatternSlotV9:
    """
    Williamson Pattern-Slot Encoder v9.3
    
    Greedy longest-match template encoding with lossless reconstruction.
    """
    
    def __init__(
        self,
        min_freq: int = 5,
        max_templates: int = 5000,
        ngram_range: Tuple[int, int] = (3, 12),
        min_savings_total: int = 50,
    ):
        self.min_freq = min_freq
        self.max_templates = max_templates
        self.ngram_range = ngram_range
        self.min_savings_total = min_savings_total
        self.templates: Dict[Template, str] = {}
        self.id_to_template: Dict[str, Template] = {}
        self.template_lengths: Dict[int, set] = {}

    def load_templates(self, templates_list: List[Dict]) -> None:
        """
        Load pre-mined templates.
        
        Each template dict must have 'atoms' field with atom list.
        Example: {"atoms": ["LIT(the)", "WS(' ')", "VAR"]}
        """
        self.templates.clear()
        self.id_to_template.clear()
        self.template_lengths.clear()
        
        for idx, t in enumerate(templates_list):
            tpl = tuple(t["atoms"])
            tid = f"<T{idx}>"
            self.templates[tpl] = tid
            self.id_to_template[tid] = tpl
            self.template_lengths.setdefault(len(tpl), set()).add(tpl)
        
        if self.template_lengths:
            self.ngram_range = (
                min(self.template_lengths.keys()),
                max(self.template_lengths.keys())
            )

    def match_at(self, sig: List[str], pos: int) -> Optional[Tuple[Template, int]]:
        """Find longest template match at position."""
        remaining = len(sig) - pos
        max_len = min(remaining, self.ngram_range[1])
        min_len = self.ngram_range[0]
        
        for L in range(max_len, min_len - 1, -1):
            bucket = self.template_lengths.get(L)
            if not bucket:
                continue
            cand = tuple(sig[pos:pos + L])
            if cand in bucket:
                return cand, L
        return None

    def encode(self, text: str) -> List[str]:
        """
        Encode text to token stream.
        
        Output: template IDs interleaved with slot values.
        """
        toks, sig, is_slot = tokenize_stream(text)
        out = []
        n = len(toks)
        i = 0
        
        while i < n:
            m = self.match_at(sig, i)
            if m is None:
                out.append(toks[i])
                i += 1
                continue
            
            tpl, L = m
            tid = self.templates[tpl]
            out.append(tid)
            
            for j in range(i, i + L):
                if is_slot[j]:
                    out.append(toks[j])
            i += L
        
        return out

    def decode(self, tokens: List[str]) -> str:
        """
        Decode token stream back to original text.
        
        MUST be lossless.
        """
        current = []
        i = 0
        n = len(tokens)
        
        while i < n:
            tok = tokens[i]
            if tok in self.id_to_template:
                tpl = self.id_to_template[tok]
                i += 1
                need = template_slots_count(tpl)
                slots = tokens[i:i + need]
                i += need
                slot_k = 0
                
                for sym in tpl:
                    if sym in ("VAR", "NUM", "CAP"):
                        current.append(slots[slot_k])
                        slot_k += 1
                    elif sym.startswith("LIT("):
                        current.append(sym[4:-1])
                    elif sym.startswith("PUNC("):
                        current.append(sym[5:-1])
                    elif sym.startswith("WS("):
                        current.append(ast.literal_eval(sym[3:-1]))
                    else:
                        current.append(sym)
                continue
            
            current.append(tok)
            i += 1
        
        return "".join(current)


# ============================================================
# DEMO
# ============================================================

if __name__ == "__main__":
    # Self-test
    test_strings = [
        "The quick brown fox.",
        "John Smith and Mary Jones",
        "In the beginning of the end.",
        "123 Main Street",
        "Hello, world!",
    ]
    
    print("=" * 60)
    print("WILLIAMSON ENCODER v93 - ATOMIZER DEMO")
    print("=" * 60)
    
    for s in test_strings:
        toks, sig, is_slot = tokenize_stream(s)
        print(f"\nInput: {s!r}")
        print(f"Tokens: {toks}")
        print(f"Atoms:  {sig}")
        print(f"Slots:  {[i for i, x in enumerate(is_slot) if x]}")
    
    print("\n" + "=" * 60)
    print("ATOM ID DEMO (for LLM embedding)")
    print("=" * 60)
    
    # Build vocab from test strings
    corpus = " ".join(test_strings)
    atom_vocab = build_atom_vocab(corpus)
    print(f"Vocab size: {len(atom_vocab)} atoms")
    
    # Encode to IDs
    test = "The quick fox."
    atom_ids = encode_to_atom_ids(test, atom_vocab)
    print(f"\nInput: {test!r}")
    print(f"Atom IDs: {atom_ids}")
    
    print("\n" + "=" * 60)
    print("CLASSIFICATION RULES:")
    print("=" * 60)
    print("WS(repr)  - Whitespace with payload - NOT slot")
    print("PUNC(x)   - Punctuation literal     - NOT slot")
    print("LIT(x)    - Stopword literal        - NOT slot (50 words)")
    print("NUM       - Number                  - IS slot")
    print("CAP       - Capitalized word        - IS slot")
    print("VAR       - Other lowercase word    - IS slot")
    
    print("\n" + "=" * 60)
    print("STOPWORDS (50):")
    print("=" * 60)
    print(sorted(STOPWORDS))
