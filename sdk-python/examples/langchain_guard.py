# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Exemple de garde-fou PII (mode block ou redact)."""

from aegis.langchain_tool import AegisPIIGuard


def main() -> None:
    guard = AegisPIIGuard(mode="redact")
    user_input = "Mon email est secret@example.com"
    safe = guard.guard(user_input)
    print("Sortie:", safe)

    try:
        AegisPIIGuard(mode="block").guard(user_input)
    except RuntimeError as e:
        print("Bloqué:", e)


if __name__ == "__main__":
    main()
