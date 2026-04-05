# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""
Agent minimal : une étape « outil base de données » renvoie du JSON avec PII ;
le texte est anonymisé avant le prompt LLM ; la réponse du modèle factice répète
le JSON masqué, puis AEGIS restaure les valeurs réelles pour l’utilisateur.

Prérequis : ``pip install 'aegis-pii[langchain]'``.
"""

from __future__ import annotations

import json
import logging

from aegis.integrations.langchain import AegisPIIGuard

logging.basicConfig(level=logging.INFO)


def fake_db_lookup(client_id: str) -> str:
    """Simule SELECT * avec email / téléphone en clair."""
    return json.dumps(
        {
            "client_id": client_id,
            "email": "vip.client@banque-test.invalid",
            "phone": "+33123456789",
            "balance_eur": 4200,
        },
        ensure_ascii=False,
    )


def main() -> None:
    from langchain_core.language_models.fake_chat_models import FakeListChatModel
    from langchain_core.prompts import ChatPromptTemplate
    from langchain_core.runnables import RunnableLambda, RunnablePassthrough

    guard = AegisPIIGuard(mode="roundtrip")  # aller-retour obligatoire pour la restauration

    client_id = "C-99"
    db_raw = fake_db_lookup(client_id)
    db_safe = guard.anonymize_for_llm(db_raw)

    llm = FakeListChatModel(
        responses=[
            f"Réponse assistant : voici la ligne telle qu’en base (masquée) :\n{db_safe}"
        ]
    )

    prompt = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                "Tu réponds en t’appuyant sur la ligne suivante (ne invente rien).\n{db_row}",
            ),
            ("human", "{user_query}"),
        ]
    )

    def _inject_db_row(x: dict) -> dict:
        return {**x, "db_row": db_safe}

    chain = (
        RunnablePassthrough()
        | RunnableLambda(_inject_db_row)
        | prompt
        | llm
        | guard.as_runnable_deanonymize(apply_to_ai_message=True)
    )

    out = chain.invoke(
        {
            "user_query": f"Quelle est la fiche pour {client_id} ?",
        }
    )
    print(out.content)


if __name__ == "__main__":
    main()
