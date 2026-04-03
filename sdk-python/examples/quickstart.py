# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Analyse et anonymisation de base."""

from aegis import AegisEngine


def main() -> None:
    text = "Envoyer le rapport à alice@example.com avant demain."
    with AegisEngine() as engine:
        entities = engine.analyze(text, score_threshold=0.35)
        print("Entités:", entities)
        redacted = engine.anonymize(
            text,
            operators={
                "EMAIL": {"operator_type": "redact", "params": {}},
            },
        )
        print("Anonymisé:", redacted.text)


if __name__ == "__main__":
    main()
