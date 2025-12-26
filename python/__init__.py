"""
Williamson Encoder - Structure-aware, lossless tokenization that beats BPE.

Usage:
    from williamson import PatternSlotV9, tokenize_stream
    
    encoder = PatternSlotV9()
    encoder.load_templates(templates_list)
    
    tokens = encoder.encode("Hello world!")
    text = encoder.decode(tokens)
"""

from .williamson import (
    PatternSlotV9,
    tokenize_stream,
    classify,
    STOPWORDS,
)

__version__ = "0.93.0"
__all__ = ["PatternSlotV9", "tokenize_stream", "classify", "STOPWORDS"]
