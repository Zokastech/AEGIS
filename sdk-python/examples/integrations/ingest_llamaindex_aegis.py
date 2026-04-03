# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""
Ingestion LlamaIndex : découpage puis anonymisation AEGIS des nœuds (PII non
stockées en clair dans l’index si vous enchaînez ensuite des embeddings).

Prérequis : ``pip install 'aegis-pii[llamaindex]'``.
"""

from __future__ import annotations

import json

from aegis.integrations.llamaindex import AegisTransformation


def main() -> None:
    from llama_index.core import Document
    from llama_index.core.ingestion import IngestionPipeline
    from llama_index.core.node_parser import SentenceSplitter

    docs = [
        Document(
            text="Client VIP : email contact@client-test.invalid, téléphone +33699887766."
        )
    ]

    pipeline = IngestionPipeline(
        transformations=[
            SentenceSplitter(chunk_size=64, chunk_overlap=0),
            AegisTransformation(attach_metadata=True),
        ]
    )

    nodes = pipeline.run(documents=docs)
    for i, n in enumerate(nodes):
        print(f"--- Node {i} ---")
        print(n.get_content(metadata_mode="none"))
        raw = (n.metadata or {}).get("aegis_anonymization")
        if raw:
            meta = json.loads(raw)
            print("metadata aegis_anonymization keys:", list(meta.keys()))


if __name__ == "__main__":
    main()
