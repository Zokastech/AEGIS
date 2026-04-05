# AEGIS — zokastech.fr — Apache 2.0 / MIT

"""Scan PII sur un DataFrame pandas."""

from __future__ import annotations

import pandas as pd

from aegis import AegisEngine


def main() -> None:
    df = pd.DataFrame(
        {
            "name": ["Alice", "Bob"],
            "email": ["alice@corp.test", "bob@corp.test"],
            "note": ["RAS", "Appeler +33 6 00 00 00 00"],
        }
    )
    with AegisEngine() as engine:
        scanned = engine.analyze_dataframe(df, columns=["email", "note"])
    print(scanned[["email", "note"]])


if __name__ == "__main__":
    main()
