# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""SDK Python **aegis-pii** : détection et anonymisation PII via le moteur AEGIS (Rust / PyO3)."""

from . import _native
from .engine import AegisEngine

AnalysisResult = _native.AnalysisResult
AnonymizedResult = _native.AnonymizedResult
Entity = _native.Entity
TransformationRecord = _native.TransformationRecord

__version__: str = getattr(_native, "__version__", "0.1.0")

__all__ = [
    "AegisEngine",
    "AnalysisResult",
    "AnonymizedResult",
    "Entity",
    "TransformationRecord",
    "__version__",
]
