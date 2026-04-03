# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Intégrations frameworks AI (LangChain, LlamaIndex) — dépendances optionnelles."""

from __future__ import annotations

__all__: list[str] = []

try:
    from .langchain import AegisCallbackHandler, AegisPIIGuard

    __all__.extend(["AegisPIIGuard", "AegisCallbackHandler"])
except ImportError:
    pass

try:
    from .llamaindex import AegisNodePostprocessor, AegisTransformation

    __all__.extend(["AegisNodePostprocessor", "AegisTransformation"])
except ImportError:
    pass
