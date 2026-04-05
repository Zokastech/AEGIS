# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Intégration LlamaIndex : post-processeur de nœuds et transformation d’ingestion."""

from __future__ import annotations

import json
import logging
from typing import Any, Dict, List, Optional

from aegis.engine import AegisEngine

try:
    from llama_index.core import QueryBundle
    from llama_index.core.postprocessor.types import BaseNodePostprocessor
    from llama_index.core.schema import BaseNode, TransformComponent
    from llama_index.core.schema import NodeWithScore
except ImportError as e:  # pragma: no cover
    raise ImportError(
        "Installez llama-index-core : pip install 'aegis-pii[llamaindex]'"
    ) from e


def _serialize_anon_metadata(result: Any) -> Dict[str, Any]:
    """Représentation JSON-friendly des transformations (métadonnées nœud)."""
    return {
        "text": result.text,
        "transformations": [
            {
                "replacement": t.replacement,
                "original_text": t.original_text,
                "entity_type": t.entity_type,
            }
            for t in result.transformations
        ],
        "mapping_hints": dict(getattr(result, "mapping_hints", {}) or {}),
    }


def _set_node_text(node: BaseNode, text: str) -> None:
    """Écrit le texte du nœud selon la version de LlamaIndex."""
    if hasattr(node, "set_content"):
        node.set_content(text)  # type: ignore[no-untyped-call]
    elif hasattr(node, "text"):
        node.text = text  # type: ignore[misc]
    else:
        raise TypeError(f"Nœud non supporté pour AEGIS: {type(node)!r}")


class AegisNodePostprocessor(BaseNodePostprocessor):
    """
    Post-processeur : anonymise le texte des nœuds récupérés avant synthèse LLM.

    Si ``store_mapping_in_metadata=True``, enregistre le mapping sous
    ``node.metadata["aegis_anonymization"]`` pour une restauration ultérieure.
    """

    def __init__(
        self,
        engine: Optional[AegisEngine] = None,
        *,
        only_entity_types: Optional[List[str]] = None,
        operators: Optional[Dict[str, Dict[str, Any]]] = None,
        store_mapping_in_metadata: bool = False,
        log_detections: bool = False,
        callback_manager: Any = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(callback_manager=callback_manager, **kwargs)
        self._aegis_engine = engine or AegisEngine()
        self._only_entity_types = only_entity_types
        self._operators = operators
        self._store_mapping_in_metadata = store_mapping_in_metadata
        self._log_detections = log_detections
        self._log = logging.getLogger("aegis.llamaindex.postprocessor")

    @classmethod
    def class_name(cls) -> str:
        return "AegisNodePostprocessor"

    def _postprocess_nodes(
        self,
        nodes: List[NodeWithScore],
        query_bundle: Optional[QueryBundle] = None,
    ) -> List[NodeWithScore]:
        for nws in nodes:
            node = nws.node
            text = node.get_content(metadata_mode="none")
            if self._log_detections:
                ents = self._aegis_engine.analyze(
                    text, entities=self._only_entity_types
                )
                if ents:
                    self._log.warning(
                        "AEGIS nodes: %s entités avant anonymisation", len(ents)
                    )
            ar = self._aegis_engine.anonymize(text, self._operators)
            _set_node_text(node, ar.text)
            if self._store_mapping_in_metadata:
                meta = dict(node.metadata or {})
                meta["aegis_anonymization"] = json.dumps(_serialize_anon_metadata(ar))
                node.metadata = meta
        return nodes


class AegisTransformation(TransformComponent):
    """
    Transformation d’ingestion : anonymise chaque nœud (ex. après chunking).

    Placez-la typiquement après un découpeur de phrases et avant les embeddings
    si l’index ne doit pas stocker de PII en clair.
    """

    def __init__(
        self,
        engine: Optional[AegisEngine] = None,
        *,
        operators: Optional[Dict[str, Dict[str, Any]]] = None,
        attach_metadata: bool = True,
    ) -> None:
        super().__init__()
        self._aegis_engine = engine or AegisEngine()
        self._operators = operators
        self._attach_metadata = attach_metadata

    @classmethod
    def class_name(cls) -> str:
        return "AegisTransformation"

    def __call__(self, nodes: List[BaseNode], **kwargs: Any) -> List[BaseNode]:
        for node in nodes:
            text = node.get_content(metadata_mode="none")
            ar = self._aegis_engine.anonymize(text, self._operators)
            _set_node_text(node, ar.text)
            if self._attach_metadata:
                meta = dict(node.metadata or {})
                meta["aegis_anonymization"] = json.dumps(_serialize_anon_metadata(ar))
                node.metadata = meta
        return nodes
