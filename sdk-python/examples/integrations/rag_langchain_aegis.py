# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""
RAG minimal avec garde-fou AEGIS (LCEL) : contexte récupéré anonymisé pour le LLM,
réponse désanonymisée pour l’utilisateur.

Prérequis : ``pip install 'aegis-pii[langchain]'`` (wheel AEGIS + langchain-core).
"""

from __future__ import annotations

import logging

from aegis.integrations.langchain import AegisCallbackHandler, AegisPIIGuard

logging.basicConfig(level=logging.INFO)


def fake_retrieve(_question: str) -> str:
    """Simule un chunk RAG contenant des PII."""
    return (
        "Dossier client : Marie Dupont, email marie.dupont@entreprise-test.invalid, "
        "tél. +33601020304."
    )


def main() -> None:
    from langchain_core.language_models.fake_chat_models import FakeListChatModel
    from langchain_core.prompts import ChatPromptTemplate
    from langchain_core.runnables import RunnablePassthrough

    guard = AegisPIIGuard(mode="roundtrip")
    cb = AegisCallbackHandler(log_level=logging.INFO)

    llm = FakeListChatModel(
        responses=[
            "D’après le dossier, l’email du contact est marie.dupont@entreprise-test.invalid."
        ]
    )

    prompt = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                "Tu es un assistant. Réponds en t’appuyant uniquement sur le contexte.\n\n"
                "Contexte :\n{context}",
            ),
            ("human", "{question}"),
        ]
    )

    chain = (
        RunnablePassthrough.assign(
            context=lambda x: fake_retrieve(x["question"]),
        )
        | guard.as_lcel_passthrough("context")
        | prompt
        | llm
        | guard.as_runnable_deanonymize(apply_to_ai_message=True)
    )

    out = chain.invoke(
        {"question": "Quel est l’email du client ?"},
        config={"callbacks": [cb]},
    )
    print("Réponse (PII restaurées côté utilisateur) :", out.content)


if __name__ == "__main__":
    main()
