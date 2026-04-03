#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
"Protected" RAG: before indexing documents or sending context to an LLM,
each text chunk is anonymized via AEGIS (/v1/anonymize).

Two modes:
  1) No LangChain: minimal pipeline (doc list → anonymize → mock answer).
  2) With LangChain: FAISS + fake embeddings + prompt chain (if packages installed).

HTTP requirement: AEGIS_BASE_URL
Optional: pip install langchain-community langchain-core faiss-cpu
"""

from __future__ import annotations

import json
import os
import sys
from typing import List

import requests

BASE = os.environ.get("AEGIS_BASE_URL", "http://127.0.0.1:8080").rstrip("/")

# Synthetic corpus (intentional fake PII-like patterns).
DOCS = [
    "Alice Smith reached us from alice@patient.zokastech.com.",
    "Billing contact: +33 6 11 22 33 44 for invoice #9921.",
    "Dr. Noemie Dupont — nir 1 85 08 75 123 456 78 (synthetic).",
]

ANON_CFG = json.dumps(
    {
        "operators_by_entity": {
            "EMAIL": {"operator_type": "replace", "params": {}},
            "PHONE": {"operator_type": "redact", "params": {"replacement": "[PHONE]"}},
        },
        "default_operator": {"operator_type": "redact", "params": {"replacement": "[PII]"}},
    }
)


def aegis_anonymize(text: str) -> str:
    try:
        r = requests.post(
            f"{BASE}/v1/anonymize",
            json={"text": text, "config_json": ANON_CFG},
            timeout=60,
        )
        r.raise_for_status()
        payload = r.json()
        raw = payload.get("result", payload)
        if isinstance(raw, str):
            raw = json.loads(raw)
        anon = raw.get("anonymized") or raw
        if isinstance(anon, dict):
            return str(anon.get("text", ""))
        return str(anon)
    except requests.RequestException:
        # No gateway: coarse masking only for notebook / laptop demo.
        return text.replace("@", " [at] ").replace(".", " [dot] ")


def rag_minimal() -> None:
    """No LangChain: shows sanitize-then-concatenate-context step."""
    safe_chunks: List[str] = []
    for d in DOCS:
        safe_chunks.append(aegis_anonymize(d))
    context = "\n---\n".join(safe_chunks)
    print("=== Sanitized context for LLM ===\n", context[:1500])
    # Here: call OpenAI / Ollama with `context` — never raw documents.
    print("\n=== Mock LLM answer ===\nBased on internal policy, no raw PII is in context.")


def rag_langchain() -> None:
    """Requires langchain-community + faiss-cpu (or equivalent)."""
    try:
        from langchain_community.embeddings import FakeEmbeddings
        from langchain_community.vectorstores import FAISS
        from langchain_core.documents import Document
        from langchain_core.prompts import ChatPromptTemplate
    except ImportError as exc:
        raise ImportError(
            "pip install langchain-community langchain-core faiss-cpu"
        ) from exc

    safe_docs = [Document(page_content=aegis_anonymize(t)) for t in DOCS]
    emb = FakeEmbeddings(size=32)
    store = FAISS.from_documents(safe_docs, emb)
    q = "What contact email was mentioned?"
    q_safe = aegis_anonymize(q)
    hits = store.similarity_search(q_safe, k=2)
    prompt = ChatPromptTemplate.from_messages(
        [
            ("system", "Answer using only the context. Do not invent PII."),
            ("human", "Context:\n{ctx}\n\nQuestion: {q}"),
        ]
    )
    ctx = "\n".join(h.page_content for h in hits)
    msg = prompt.format_messages(ctx=ctx, q=q_safe)
    print("=== LangChain prompt (sanitized) ===")
    for m in msg:
        print(m.type, ":", m.content[:400])


def main() -> None:
    if os.environ.get("AEGIS_USE_LANGCHAIN", "").lower() in ("1", "true", "yes"):
        try:
            rag_langchain()
        except ImportError:
            print(
                "LangChain path requested but packages missing. Install:\n"
                "  pip install langchain-community langchain-core faiss-cpu",
                file=sys.stderr,
            )
            rag_minimal()
    else:
        rag_minimal()


if __name__ == "__main__":
    try:
        main()
    except requests.RequestException as exc:
        print("HTTP error:", exc, file=sys.stderr)
        sys.exit(1)
