# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""
Chatbot : anonymisation « transparente » du dernier message utilisateur avant LLM,
puis restauration dans la réponse affichée.

Utilise un modèle factice (pas d’API clé). Prérequis : ``aegis-pii[langchain]``.
"""

from __future__ import annotations

import logging

from aegis.integrations.langchain import AegisPIIGuard

logging.basicConfig(level=logging.INFO)


def main() -> None:
    from langchain_core.language_models.fake_chat_models import FakeListChatModel
    from langchain_core.messages import AIMessage, HumanMessage
    from langchain_core.prompts import ChatPromptTemplate
    from langchain_core.runnables import RunnableLambda, RunnablePassthrough

    guard = AegisPIIGuard(mode="roundtrip")

    def _last_user_text(msgs: list) -> str:
        for m in reversed(msgs):
            if isinstance(m, HumanMessage) and isinstance(m.content, str):
                return m.content
        return ""

    def _anonymize_last_turn(x: dict) -> dict:
        msgs = list(x["messages"])
        raw = _last_user_text(msgs)
        if not raw:
            return x
        safe = guard.anonymize_for_llm(raw)
        for i in range(len(msgs) - 1, -1, -1):
            if isinstance(msgs[i], HumanMessage) and msgs[i].content == raw:
                msgs[i] = HumanMessage(content=safe)
                break
        return {"messages": msgs}

    llm = FakeListChatModel(
        responses=[
            "J’ai noté votre email secret@example.com pour le suivi."
        ]
    )

    prompt = ChatPromptTemplate.from_messages([("placeholder", "{messages}")])

    chain = (
        RunnablePassthrough.assign(messages=lambda d: d["messages"])
        | RunnableLambda(_anonymize_last_turn)
        | prompt
        | llm
        | guard.as_runnable_deanonymize(apply_to_ai_message=True)
    )

    user_msg = HumanMessage(
        content="Bonjour, mon email est secret@example.com pour la facture."
    )
    ai: AIMessage = chain.invoke({"messages": [user_msg]})
    print("Réponse :", ai.content)


if __name__ == "__main__":
    main()
