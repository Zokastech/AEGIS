# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Charge le paquet pip **datasets** (Hugging Face), pas le dossier `./datasets/` du dépôt
(benchmarks), lorsque le cwd ou sys.path[0] pointe vers la racine du monorepo.

Sans cela : ``ImportError: cannot import name 'ClassLabel' from 'datasets' (unknown location)``.
"""

from __future__ import annotations

import importlib
import site
import sys
from types import ModuleType


def _drop_shallow_datasets_module() -> None:
    mod = sys.modules.get("datasets")
    if mod is None:
        return
    if hasattr(mod, "ClassLabel") and hasattr(mod, "load_from_disk"):
        return
    del sys.modules["datasets"]
    for key in list(sys.modules):
        if key.startswith("datasets."):
            del sys.modules[key]


def load_datasets() -> ModuleType:
    """
    Retourne le module ``datasets`` de Hugging Face (installé via pip).
    """
    _drop_shallow_datasets_module()
    if "datasets" in sys.modules:
        return sys.modules["datasets"]

    inserted: list[str] = []
    for p in (*site.getsitepackages(), site.getusersitepackages()):
        if not p or p in sys.path:
            continue
        sys.path.insert(0, p)
        inserted.append(p)
    try:
        return importlib.import_module("datasets")
    finally:
        for p in inserted:
            if sys.path and sys.path[0] == p:
                sys.path.pop(0)
