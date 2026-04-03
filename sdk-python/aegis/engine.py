# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Couche Python autour des bindings PyO3 : context manager, pandas, helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Dict, List, Optional, Sequence

from . import _native

if TYPE_CHECKING:
    try:
        import pandas as pd
    except ImportError:
        pd = Any  # type: ignore[misc,assignment]


class AegisEngine:
    """
    Moteur AEGIS (analyse + anonymisation).

    Exemple::

        with AegisEngine() as engine:
            entities = engine.analyze("contact: jane@acme.fr")
    """

    __slots__ = ("_inner",)

    def __init__(
        self,
        config_path: Optional[str] = None,
        languages: Optional[List[str]] = None,
    ) -> None:
        langs = languages if languages is not None else ["en", "fr"]
        self._inner = _native.AegisEngine(config_path, langs)

    def analyze(
        self,
        text: str,
        *,
        language: Optional[str] = None,
        entities: Optional[List[str]] = None,
        score_threshold: float = 0.5,
    ) -> List[_native.Entity]:
        """Détecte les entités PII dans ``text``."""
        return self._inner.analyze(text, language, entities, score_threshold)

    def analyze_full(
        self,
        text: str,
        *,
        language: Optional[str] = None,
        entities: Optional[List[str]] = None,
        score_threshold: float = 0.5,
    ) -> _native.AnalysisResult:
        """Analyse complète (métadonnées + entités)."""
        return self._inner.analyze_full(text, language, entities, score_threshold)

    def anonymize(
        self,
        text: str,
        operators: Optional[Dict[str, Dict[str, Any]]] = None,
    ) -> _native.AnonymizedResult:
        """
        Anonymise le texte. ``operators`` : clés = types d'entité (ex. ``EMAIL``),
        valeurs = ``{"operator_type": "redact", "params": {}}``.
        """
        return self._inner.anonymize(text, operators)

    def analyze_batch(self, texts: Sequence[str]) -> List[List[_native.Entity]]:
        """Analyse chaque texte ; retourne une liste d'entités par entrée."""
        return self._inner.analyze_batch(list(texts))

    def analyze_dataframe(
        self,
        df: "pd.DataFrame",
        columns: Optional[Sequence[str]] = None,
        *,
        language: Optional[str] = None,
        score_threshold: float = 0.5,
    ) -> "pd.DataFrame":
        """
        Applique :meth:`analyze` aux colonnes textuelles.

        Chaque cellule contient une ``list`` d':class:`Entity`.
        """
        import pandas as pd

        if columns is None:
            columns = list(df.select_dtypes(include=["object", "string"]).columns)
        out = df.copy()
        for col in columns:
            out[col] = df[col].map(
                lambda t: self.analyze(
                    "" if t is None or (isinstance(t, float) and pd.isna(t)) else str(t),
                    language=language,
                    score_threshold=score_threshold,
                )
            )
        return out

    def close(self) -> None:
        """Libère le moteur natif."""
        self._inner.close()

    def __enter__(self) -> "AegisEngine":
        return self

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        self._inner.__exit__(exc_type, exc, tb)

    def __repr__(self) -> str:
        return f"<AegisEngine at {id(self):#x}>"


def __getattr__(name: str) -> Any:
    """Ré-export des types natifs pour `from aegis.engine import Entity`."""
    if name in ("Entity", "AnonymizedResult", "AnalysisResult", "TransformationRecord"):
        return getattr(_native, name)
    raise AttributeError(name)
