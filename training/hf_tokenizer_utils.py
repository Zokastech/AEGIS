# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""Chargement AutoTokenizer avec options récentes (ex. regex Mistral) et rétrocompatibilité."""

from __future__ import annotations

from typing import Any


def load_autotokenizer_pretrained(pretrained: str, **kwargs: Any) -> Any:
    """
    Enveloppe `AutoTokenizer.from_pretrained` : passe `fix_mistral_regex=True` si supporté
    (évite une tokenisation incorrecte sur les checkpoints Mistral / dérivés).
    """
    from transformers import AutoTokenizer

    try:
        return AutoTokenizer.from_pretrained(pretrained, **kwargs, fix_mistral_regex=True)
    except TypeError:
        return AutoTokenizer.from_pretrained(pretrained, **kwargs)
