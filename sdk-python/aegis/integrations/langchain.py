# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Intégration LangChain : garde PII, LCEL, callbacks."""

from __future__ import annotations

import contextvars
import logging
from typing import Any, Callable, Dict, List, Optional

from aegis.engine import AegisEngine


def _require_langchain() -> None:
    try:
        import langchain_core  # noqa: F401
    except ImportError as e:  # pragma: no cover
        raise ImportError(
            "Installez langchain-core : pip install 'aegis-pii[langchain]'"
        ) from e


def deanonymize_text(text: str, anonymized: Any) -> str:
    """
    Réinjecte les valeurs d’origine à partir d’un :class:`aegis.AnonymizedResult`.
    Remplace les ``replacement`` les plus longs en premier pour limiter les collisions.
    """
    pairs: List[tuple[str, str]] = []
    for t in anonymized.transformations:
        rep = t.replacement
        orig = t.original_text
        if rep and orig:
            pairs.append((rep, orig))
    hints = getattr(anonymized, "mapping_hints", None) or {}
    for k, v in hints.items():
        if k and v:
            pairs.append((k, v))
    pairs.sort(key=lambda p: len(p[0]), reverse=True)
    out = text
    for rep, orig in pairs:
        out = out.replace(rep, orig)
    return out


_ctx_last_anon: contextvars.ContextVar[Optional[Any]] = contextvars.ContextVar(
    "aegis_last_anon", default=None
)


class AegisPIIGuard:
    """
    Garde-fou AEGIS pour chaînes LangChain : analyse, blocage, anonymisation,
    aller-retour anonymiser → LLM → désanonymiser.

    Modes :

    - ``block`` : lève ``RuntimeError`` si PII détectée (défaut, rétro-compat).
    - ``redact`` : renvoie le texte anonymisé (sans conserver le mapping).
    - ``roundtrip`` : anonymise pour le LLM ; utilisez :meth:`deanonymize_output`
      ou :meth:`as_runnable_deanonymize` sur la sortie.
    - ``log`` : ne modifie pas le texte ; journalise les détections via ``logger``.

    **LCEL** : :meth:`as_lcel_passthrough` fournit le même usage qu’un
    ``RunnablePassthrough.assign`` qui applique AEGIS sur une clé.
    """

    def __init__(
        self,
        engine: Optional[AegisEngine] = None,
        *,
        mode: str = "block",
        only_entity_types: Optional[List[str]] = None,
        operators: Optional[Dict[str, Dict[str, Any]]] = None,
        logger: Optional[logging.Logger] = None,
    ) -> None:
        self._engine = engine or AegisEngine()
        self._mode = mode
        self._only_entity_types = only_entity_types
        self._operators = operators
        self._logger = logger or logging.getLogger("aegis.langchain")

    @property
    def engine(self) -> AegisEngine:
        return self._engine

    def check(self, text: str) -> List[Any]:
        """Liste des entités détectées (peut être vide)."""
        return self._engine.analyze(text, entities=self._only_entity_types)

    def guard(self, text: str) -> str:
        """API historique : bloque, anonymise ou laisse passer selon ``mode``."""
        ents = self._engine.analyze(text, entities=self._only_entity_types)
        if not ents:
            return text
        if self._mode == "block":
            types = {e.entity_type for e in ents}
            raise RuntimeError(f"AEGIS: PII détecté ({types})")
        if self._mode == "redact":
            return self._engine.anonymize(text, self._operators).text
        if self._mode == "roundtrip":
            ar = self._engine.anonymize(text, self._operators)
            _ctx_last_anon.set(ar)
            return ar.text
        if self._mode == "log":
            self._logger.warning("AEGIS: PII dans le texte (%s)", ents)
            return text
        raise ValueError(f"mode inconnu: {self._mode}")

    def anonymize_for_llm(self, text: str) -> str:
        """Anonymise et mémorise le mapping (contextvars, compatible asyncio)."""
        if self._mode == "log":
            ents = self.check(text)
            if ents:
                self._logger.warning("AEGIS PII (prompt): %s", ents)
            return text
        if self._mode == "block":
            return self.guard(text)
        ar = self._engine.anonymize(text, self._operators)
        _ctx_last_anon.set(ar)
        return ar.text

    def deanonymize_output(self, text: str) -> str:
        """Restaure les valeurs d’origine après le dernier anonymize (même contexte)."""
        ar = _ctx_last_anon.get()
        if ar is None:
            return text
        return deanonymize_text(text, ar)

    def as_runnable_anonymize(
        self,
        *,
        text_key: str = "input",
        extract: Optional[Callable[[Any], str]] = None,
    ) -> RunnableLambda:
        _require_langchain()
        from langchain_core.runnables import RunnableConfig, RunnableLambda

        def _pull(x: Any) -> str:
            if extract is not None:
                return extract(x)
            if isinstance(x, dict):
                v = x.get(text_key, "")
                return v if isinstance(v, str) else str(v)
            return str(x)

        guard = self

        def _fn(x: Any, config: Optional[RunnableConfig] = None) -> Any:
            raw = _pull(x)
            safe = guard.anonymize_for_llm(raw)
            if isinstance(x, dict):
                return {**x, text_key: safe}
            return safe

        return RunnableLambda(_fn)

    def as_runnable_deanonymize(
        self,
        *,
        text_key: str = "output",
        apply_to_ai_message: bool = True,
    ) -> Any:
        _require_langchain()
        from langchain_core.runnables import RunnableConfig, RunnableLambda

        guard = self

        def _fn(x: Any, config: Optional[RunnableConfig] = None) -> Any:
            if apply_to_ai_message:
                try:
                    from langchain_core.messages import AIMessage

                    if isinstance(x, AIMessage) and isinstance(x.content, str):
                        new_c = guard.deanonymize_output(x.content)
                        if hasattr(x, "model_copy"):
                            return x.model_copy(update={"content": new_c})
                        return AIMessage(
                            content=new_c,
                            additional_kwargs=getattr(
                                x, "additional_kwargs", None
                            )
                            or {},
                            response_metadata=getattr(x, "response_metadata", None)
                            or {},
                        )
                except ImportError:
                    pass
            if isinstance(x, dict) and text_key in x:
                v = x[text_key]
                if isinstance(v, str):
                    return {**x, text_key: guard.deanonymize_output(v)}
            if isinstance(x, str):
                return guard.deanonymize_output(x)
            return x

        return RunnableLambda(_fn)

    def as_lcel_passthrough(self, key: str = "input") -> Any:
        """
        Même rôle qu’un ``RunnablePassthrough.assign`` : enrichit l’entrée en
        remplaçant ``key`` par sa version anonymisée pour le LLM.
        """
        _require_langchain()
        from langchain_core.runnables import RunnablePassthrough

        guard = self

        def _map_val(d: Dict[str, Any]) -> str:
            v = d.get(key, "")
            return guard.anonymize_for_llm(v if isinstance(v, str) else str(v))

        return RunnablePassthrough.assign(**{key: _map_val})

    @classmethod
    def as_langchain_tool(cls, **guard_kwargs: Any) -> Any:
        _require_langchain()
        from langchain_core.tools import StructuredTool

        g = cls(**guard_kwargs)

        def _run(user_text: str) -> str:
            return g.guard(user_text)

        return StructuredTool.from_function(
            name="aegis_pii_guard",
            description="Vérifie ou anonymise les données personnelles (AEGIS / zokastech.fr).",
            func=_run,
        )


try:
    from langchain_core.callbacks.base import BaseCallbackHandler
    from langchain_core.outputs import LLMResult

    class AegisCallbackHandler(BaseCallbackHandler):
        """
        Callback LangChain : analyse les prompts et les générations pour journaliser les PII.

        N’anonymise pas automatiquement ; utile en observabilité / audit.
        """

        def __init__(
            self,
            engine: Optional[AegisEngine] = None,
            *,
            logger: Optional[logging.Logger] = None,
            log_level: int = logging.WARNING,
            only_entity_types: Optional[List[str]] = None,
        ) -> None:
            super().__init__()
            self._engine = engine or AegisEngine()
            self._logger = logger or logging.getLogger("aegis.langchain.callback")
            self._log_level = log_level
            self._only_entity_types = only_entity_types

        def _log_text(self, phase: str, text: str) -> None:
            if not text:
                return
            ents = self._engine.analyze(text, entities=self._only_entity_types)
            if ents:
                self._logger.log(
                    self._log_level,
                    "AEGIS [%s] PII détectée (%d) : %s",
                    phase,
                    len(ents),
                    [(e.entity_type, e.text[:32]) for e in ents],
                )

        def on_llm_start(
            self,
            serialized: Dict[str, Any],
            prompts: List[str],
            **kwargs: Any,
        ) -> None:
            for p in prompts:
                self._log_text("llm_prompt", p)

        def on_llm_end(self, response: LLMResult, **kwargs: Any) -> None:
            for gen_list in response.generations:
                for gen in gen_list:
                    text = getattr(gen, "text", None) or ""
                    if not text and hasattr(gen, "message"):
                        msg = gen.message
                        content = getattr(msg, "content", "")
                        text = content if isinstance(content, str) else str(content)
                    self._log_text("llm_output", text)

        def on_chat_model_start(
            self,
            serialized: Dict[str, Any],
            messages: List[List[Any]],
            **kwargs: Any,
        ) -> None:
            for convo in messages:
                for m in convo:
                    content = getattr(m, "content", m)
                    if isinstance(content, str):
                        self._log_text("chat_message", content)
                    elif isinstance(content, list):
                        for part in content:
                            if isinstance(part, dict) and part.get("type") == "text":
                                self._log_text(
                                    "chat_message", str(part.get("text", ""))
                                )

except ImportError:  # pragma: no cover

    class AegisCallbackHandler:  # type: ignore[no-redef]
        """Réservé si langchain-core n’est pas installé."""

        def __init__(self, *args: Any, **kwargs: Any) -> None:
            raise ImportError(
                "AegisCallbackHandler nécessite langchain-core : pip install 'aegis-pii[langchain]'"
            )
