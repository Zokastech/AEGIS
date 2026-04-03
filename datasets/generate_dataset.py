#!/usr/bin/env python3
# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Générateur de textes synthétiques annotés (PII) pour benchmark AEGIS.
Sortie : JSONL avec champs id, language, category, text, entities, tags.
"""

from __future__ import annotations

import argparse
import base64
import json
import random
import string
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# 10 langues UE (couverture large)
EU_LANGS = ("en", "fr", "de", "es", "it", "nl", "pl", "pt", "ro", "sv")

CATEGORIES = (
    "email_pro",
    "chat",
    "system_log",
    "form",
    "press",
    "medical",
    "contract",
    "invoice",
    "support_ticket",
)

# Poids relatifs de catégories (distribution réaliste : plus d’emails / support que rapports médicaux)
CATEGORY_WEIGHTS = {
    "email_pro": 18,
    "chat": 14,
    "system_log": 12,
    "form": 11,
    "press": 10,
    "support_ticket": 14,
    "invoice": 8,
    "contract": 7,
    "medical": 6,
}

# Types d’entités AEGIS (clés config) — pondérations par catégorie (clé = type, valeur = poids relatif)
ENTITY_PROFILES: dict[str, dict[str, float]] = {
    "email_pro": {
        "PERSON": 3,
        "EMAIL": 5,
        "PHONE": 2,
        "ORGANIZATION": 2,
        "DATE": 1,
        "URL": 1,
    },
    "chat": {
        "PERSON": 4,
        "PHONE": 2,
        "EMAIL": 2,
        "LOCATION": 2,
        "DATE": 1,
        "URL": 1,
    },
    "system_log": {
        "IP_ADDRESS": 4,
        "EMAIL": 2,
        "URL": 2,
        "PERSON": 1,
        "DATE": 2,
        "PHONE": 1,
    },
    "form": {
        "PERSON": 3,
        "EMAIL": 2,
        "PHONE": 2,
        "ADDRESS": 3,
        "DATE": 2,
        "NATIONAL_ID": 1,
        "TAX_ID": 1,
    },
    "press": {
        "PERSON": 3,
        "ORGANIZATION": 3,
        "LOCATION": 3,
        "DATE": 2,
        "URL": 1,
    },
    "medical": {
        "PERSON": 2,
        "DATE": 2,
        "MEDICAL_RECORD": 3,
        "PHONE": 2,
        "ADDRESS": 1,
        "EMAIL": 1,
    },
    "contract": {
        "PERSON": 3,
        "ORGANIZATION": 3,
        "ADDRESS": 2,
        "DATE": 3,
        "EMAIL": 1,
        "TAX_ID": 2,
        "IBAN": 1,
    },
    "invoice": {
        "ORGANIZATION": 2,
        "PERSON": 1,
        "ADDRESS": 2,
        "EMAIL": 2,
        "IBAN": 3,
        "TAX_ID": 2,
        "DATE": 2,
        "CREDIT_CARD": 0.3,
    },
    "support_ticket": {
        "PERSON": 3,
        "EMAIL": 3,
        "PHONE": 2,
        "URL": 2,
        "IP_ADDRESS": 2,
        "ORGANIZATION": 1,
        "DATE": 1,
    },
}

# Estimation du gabarit « long » par catégorie (évite hard-case si dépassement de target)
_CATEGORY_BASE_ENTITIES: dict[str, int] = {
    "email_pro": 3,
    "chat": 3,
    "system_log": 4,
    "form": 4,
    "press": 5,
    "medical": 5,
    "contract": 5,
    "invoice": 3,
    "support_ticket": 4,
}

LOCALE = {
    "en": {
        "greet": ("Dear", "Hi", "Hello"),
        "regards": ("Best regards", "Kind regards", "Thanks"),
        "company": ("Acme Ltd", "Northwind Inc", "Globex Corp"),
        "city": ("London", "Manchester", "Bristol"),
        "street": ("High Street", "Church Road", "Station Road"),
    },
    "fr": {
        "greet": ("Bonjour", "Madame, Monsieur", "Cher partenaire"),
        "regards": ("Cordialement", "Bien à vous", "Salutations"),
        "company": ("SARL Dupont", "Acme France SAS", "Tech Seine SA"),
        "city": ("Paris", "Lyon", "Toulouse"),
        "street": ("rue de la République", "avenue des Champs", "boulevard Haussmann"),
    },
    "de": {
        "greet": ("Sehr geehrte Damen und Herren", "Hallo", "Guten Tag"),
        "regards": ("Mit freundlichen Grüßen", "Viele Grüße"),
        "company": ("GmbH Berlin", "Müller AG", "Schmidt Technologies"),
        "city": ("Berlin", "München", "Hamburg"),
        "street": ("Hauptstraße", "Bahnhofstraße", "Gartenweg"),
    },
    "es": {
        "greet": ("Estimado", "Hola", "Buenos días"),
        "regards": ("Un saludo", "Atentamente"),
        "company": ("SL Madrid", "Ibérica SA", "Soluciones SL"),
        "city": ("Madrid", "Barcelona", "Valencia"),
        "street": ("Calle Mayor", "Avenida Libertad", "Plaza Central"),
    },
    "it": {
        "greet": ("Gentile", "Buongiorno", "Egregio"),
        "regards": ("Cordiali saluti", "Distinti saluti"),
        "company": ("S.r.l. Milano", "SpA Roma", "Italia Tech"),
        "city": ("Milano", "Roma", "Torino"),
        "street": ("Via Roma", "Corso Italia", "Piazza Duomo"),
    },
    "nl": {
        "greet": ("Beste", "Geachte", "Hallo"),
        "regards": ("Met vriendelijke groet", "Hoogachtend"),
        "company": ("BV Amsterdam", "Rotterdam Tech BV"),
        "city": ("Amsterdam", "Rotterdam", "Utrecht"),
        "street": ("Hoofdstraat", "Kerkweg", "Stationsplein"),
    },
    "pl": {
        "greet": ("Szanowni Państwo", "Dzień dobry", "Witam"),
        "regards": ("Pozdrawiam", "Z poważaniem"),
        "company": ("Sp. z o.o. Warszawa", "Tech Polska SA"),
        "city": ("Warszawa", "Kraków", "Gdańsk"),
        "street": ("ul. Marszałkowska", "ul. Długa", "al. Niepodległości"),
    },
    "pt": {
        "greet": ("Exmo.", "Olá", "Bom dia"),
        "regards": ("Com os melhores cumprimentos", "Atenciosamente"),
        "company": ("Lda Lisboa", "SA Porto", "Tech Portugal"),
        "city": ("Lisboa", "Porto", "Braga"),
        "street": ("Rua Augusta", "Avenida da Liberdade", "Praça do Comércio"),
    },
    "ro": {
        "greet": ("Stimate", "Bună ziua", "Salut"),
        "regards": ("Cu stimă", "Toate cele bune"),
        "company": ("SRL București", "Tech România"),
        "city": ("București", "Cluj-Napoca", "Timișoara"),
        "street": ("Str. Victoriei", "Bd. Unirii", "Calea Dorobanților"),
    },
    "sv": {
        "greet": ("Hej", "Bästa", "God dag"),
        "regards": ("Vänliga hälsningar", "Med vänlig hälsning"),
        "company": ("AB Stockholm", "Nordic Tech AB"),
        "city": ("Stockholm", "Göteborg", "Malmö"),
        ("street",): ("Storgatan", "Drottninggatan", "Kungsgatan"),
    },
}


def _fix_sv_locale() -> None:
    if "sv" in LOCALE and "street" not in LOCALE["sv"]:
        LOCALE["sv"]["street"] = ("Storgatan", "Drottninggatan", "Kungsgatan")


_fix_sv_locale()

FIRST_NAMES = (
    "Alex",
    "Jordan",
    "Marie",
    "Luca",
    "Sofia",
    "Jan",
    "Elena",
    "Ivan",
    "Anna",
    "Lars",
    "Pierre",
    "Carmen",
    "Marta",
    "Oliver",
    "Zoe",
)
LAST_NAMES = (
    "Müller",
    "Rossi",
    "García",
    "Novak",
    "Andersson",
    "Popescu",
    "Silva",
    "Kowalski",
    "Bernard",
    "Schmidt",
)


def _luhn_checksum(digits: str) -> int:
    s = 0
    alt = False
    for d in reversed(digits):
        n = int(d)
        if alt:
            n *= 2
            if n > 9:
                n -= 9
        s += n
        alt = not alt
    return (10 - (s % 10)) % 10


def fake_credit_card(rng: random.Random) -> str:
    prefix = "4532"  # Visa-like test
    body = "".join(rng.choice(string.digits) for _ in range(11))
    partial = prefix + body
    check = _luhn_checksum(partial)
    return partial + str(check)


def fake_iban(lang: str, rng: random.Random) -> str:
    country = {"de": "DE", "fr": "FR", "es": "ES", "it": "IT", "nl": "NL"}.get(lang, "FR")
    bban = "".join(rng.choice(string.digits) for _ in range(18))
    return f"{country}79{bban[:20]}"[:34]


def fake_phone(lang: str, rng: random.Random) -> str:
    if lang == "fr":
        return f"+33 {rng.randint(6,7)} {rng.randint(10,99):02d} {rng.randint(10,99):02d} {rng.randint(10,99):02d} {rng.randint(10,99):02d}"
    if lang == "de":
        return f"+49 {rng.randint(150,179)} {rng.randint(1000000,9999999)}"
    if lang in ("pl", "ro"):
        return f"+{48 if lang=='pl' else 40} {rng.randint(500,899)} {rng.randint(100,999)} {rng.randint(100,999)}"
    return f"+44 {rng.randint(7700,7999)} {rng.randint(100000,999999)}"


def fake_email(first: str, last: str, rng: random.Random) -> str:
    dom = rng.choice(("mail.eu", "corp.io", "contact.fr", "team.io", "example.net"))
    user = f"{first.lower()}.{last.lower()}"[:20].replace(" ", "")
    return f"{user}@{dom}"


def fake_date(rng: random.Random) -> str:
    return f"{rng.randint(1,28):02d}/{rng.randint(1,12):02d}/{rng.randint(2020,2025)}"


def fake_ip(rng: random.Random) -> str:
    return f"192.168.{rng.randint(0,255)}.{rng.randint(1,254)}"


def fake_url(rng: random.Random) -> str:
    return f"https://portal-{rng.randint(100,999)}.internal.example.org/track?id={rng.randint(1000,9999)}"


def fake_tax_id(lang: str, rng: random.Random) -> str:
    if lang == "fr":
        return f"FR{rng.randint(10,99)}{''.join(rng.choice(string.digits) for _ in range(11))}"
    if lang == "de":
        return f"DE{rng.randint(100000000,999999999)}"
    return f"EU-VAT-{rng.randint(10000000,99999999)}"


def fake_national_id(lang: str, rng: random.Random) -> str:
    if lang == "pl":
        return f"{rng.randint(800101,991231):06d}{rng.choice(string.ascii_uppercase)}{rng.randint(100,999)}"
    return f"ID-{lang.upper()}-{rng.randint(100000,999999)}"


def fake_medical_ref(rng: random.Random) -> str:
    return f"MRN-{rng.randint(2020,2025)}-{rng.randint(100000,999999)}"


def fake_passport(rng: random.Random) -> str:
    return f"{rng.choice('ABCDEFGHJKLMNPRSTUVWXYZ')}{rng.randint(10000000,99999999)}"


def typo_text(s: str, rng: random.Random) -> str:
    if len(s) < 4 or rng.random() > 0.5:
        return s
    i = rng.randint(1, len(s) - 2)
    t = list(s)
    t[i], t[i + 1] = t[i + 1], t[i]
    return "".join(t)


@dataclass
class TextBuilder:
    parts: list[str] = field(default_factory=list)
    entities: list[dict[str, Any]] = field(default_factory=list)

    def raw(self, s: str) -> None:
        self.parts.append(s)

    def ent(self, text: str, entity_type: str) -> None:
        t = str(text)
        start = sum(len(p) for p in self.parts)
        self.parts.append(t)
        self.entities.append(
            {"entity_type": entity_type, "start": start, "end": start + len(t), "text": t}
        )

    def build(self) -> tuple[str, list[dict[str, Any]]]:
        return "".join(self.parts), list(self.entities)


def pick_entity_value(etype: str, lang: str, first: str, last: str, rng: random.Random) -> str:
    loc = LOCALE.get(lang, LOCALE["en"])
    if etype == "PERSON":
        return f"{first} {last}"
    if etype == "EMAIL":
        return fake_email(first, last, rng)
    if etype == "PHONE":
        return fake_phone(lang, rng)
    if etype == "CREDIT_CARD":
        return fake_credit_card(rng)
    if etype == "IBAN":
        return fake_iban(lang, rng)
    if etype == "IP_ADDRESS":
        return fake_ip(rng)
    if etype == "URL":
        return fake_url(rng)
    if etype == "DATE":
        return fake_date(rng)
    if etype == "ADDRESS":
        n = rng.randint(1, 120)
        return f"{n} {rng.choice(loc['street'])}, {rng.choice(loc['city'])}"
    if etype == "ORGANIZATION":
        return rng.choice(loc["company"])
    if etype == "LOCATION":
        return rng.choice(loc["city"])
    if etype == "MEDICAL_RECORD":
        return fake_medical_ref(rng)
    if etype == "NATIONAL_ID":
        return fake_national_id(lang, rng)
    if etype == "TAX_ID":
        return fake_tax_id(lang, rng)
    if etype == "BANK_ACCOUNT":
        return f"ACC-{rng.randint(10000000,99999999)}"
    if etype == "PASSPORT":
        return fake_passport(rng)
    return f"REF-{etype}-{rng.randint(1000,9999)}"


def weighted_pick(weights: dict[str, float], rng: random.Random, k: int) -> list[str]:
    types = list(weights.keys())
    w = [max(0.01, weights[t]) for t in types]
    return rng.choices(types, weights=w, k=k)


def add_hard_case(tb: TextBuilder, lang: str, rng: random.Random, tags: list[str]) -> None:
    r = rng.random()
    if r < 0.35:
        # Homonyme : Paris (personne) vs ville
        tb.raw("Meeting scheduled with ")
        tb.ent("Paris", "PERSON")
        tb.raw(" at the ")
        tb.ent("Paris", "LOCATION")
        tb.raw(" office next week.\n")
        tags.append("homonym_paris")
    elif r < 0.55:
        # Abréviation + entreprise
        tb.raw("Contact ")
        tb.ent("Dr. " + rng.choice(LAST_NAMES), "PERSON")
        tb.raw(" (")
        tb.ent("EU HQ", "ORGANIZATION")
        tb.raw(").\n")
        tags.append("abbreviation_title")
    elif r < 0.75:
        # Email avec faute d’orthographe (annotation = forme canonique dans le texte fautif)
        first, last = rng.choice(FIRST_NAMES), rng.choice(LAST_NAMES)
        good = fake_email(first, last, rng)
        bad = typo_text(good, rng)
        tb.raw("Reply-to: ")
        tb.ent(bad, "EMAIL")
        tags.append("typo_email")
    else:
        # PII encodée Base64 (le segment annoté est le bloc base64 complet)
        secret = fake_email(rng.choice(FIRST_NAMES), rng.choice(LAST_NAMES), rng)
        b64 = base64.b64encode(secret.encode()).decode()
        tb.raw("Legacy field b64:")
        tb.ent(b64, "EMAIL")
        tb.raw(" (decoded legacy).\n")
        tags.append("base64_payload")


def generate_document(
    lang: str,
    category: str,
    rng: random.Random,
    hard_case: bool,
) -> tuple[str, list[dict[str, Any]], list[str]]:
    tags: list[str] = []
    tb = TextBuilder()
    first, last = rng.choice(FIRST_NAMES), rng.choice(LAST_NAMES)
    loc = LOCALE.get(lang, LOCALE["en"])
    profile = ENTITY_PROFILES[category]
    target = rng.randint(3, 15)
    # Cas courts : gabarit compact pour ne pas dépasser la cible
    if target <= 5:
        tb.raw(f"[{category}] ")
        for i in range(target):
            if i:
                tb.raw(rng.choice((" | ", "; ", "\n")))
            et = weighted_pick(profile, rng, 1)[0]
            val = pick_entity_value(et, lang, first, last, rng)
            if et == "EMAIL" and rng.random() < 0.25:
                val = typo_text(val, rng)
                tags.append("typo_email")
            tb.ent(val, et)
        text, entities = tb.build()
        return text, _dedupe_entities(entities), tags

    base_n = _CATEGORY_BASE_ENTITIES.get(category, 4)
    if hard_case and target >= 7 and base_n + 3 <= target:
        add_hard_case(tb, lang, rng, tags)

    if category == "email_pro":
        tb.raw(rng.choice(loc["greet"]) + " " + first + ",\n\n")
        tb.raw("Following our call, please send the draft to ")
        tb.ent(pick_entity_value("EMAIL", lang, first, last, rng), "EMAIL")
        tb.raw(".\nDirect line: ")
        tb.ent(pick_entity_value("PHONE", lang, first, last, rng), "PHONE")
        tb.raw(".\n\n")
        tb.raw(rng.choice(loc["regards"]) + ",\n")
        tb.ent(f"{first} {last}", "PERSON")
        tb.raw("\n")
    elif category == "chat":
        tb.raw("[10:02] ")
        tb.ent(f"{first}{rng.randint(1,99)}", "PERSON")
        tb.raw(": did you see the invoice?\n[10:03] support: yes, IBAN ")
        tb.ent(pick_entity_value("IBAN", lang, first, last, rng), "IBAN")
        tb.raw("\n")
    elif category == "system_log":
        tb.raw(f"[{fake_date(rng)} {rng.randint(0,23):02d}:{rng.randint(0,59):02d}:00] ")
        tb.raw("auth ok user=")
        tb.ent(fake_email(first, last, rng), "EMAIL")
        tb.raw(" from ")
        tb.ent(fake_ip(rng), "IP_ADDRESS")
        tb.raw(" session=")
        tb.ent(fake_url(rng), "URL")
        tb.raw("\n")
    elif category == "form":
        tb.raw("Registration — Name: ")
        tb.ent(f"{first} {last}", "PERSON")
        tb.raw(" | Addr: ")
        tb.ent(pick_entity_value("ADDRESS", lang, first, last, rng), "ADDRESS")
        tb.raw(" | ID: ")
        tb.ent(pick_entity_value("NATIONAL_ID", lang, first, last, rng), "NATIONAL_ID")
        tb.raw("\n")
    elif category == "press":
        tb.raw("Interview: ")
        tb.ent(f"{first} {last}", "PERSON")
        tb.raw(", spokesperson for ")
        tb.ent(pick_entity_value("ORGANIZATION", lang, first, last, rng), "ORGANIZATION")
        tb.raw(" in ")
        tb.ent(pick_entity_value("LOCATION", lang, first, last, rng), "LOCATION")
        tb.raw(" (")
        tb.ent(fake_date(rng), "DATE")
        tb.raw(").\n")
    elif category == "medical":
        tb.raw("Patient ")
        tb.ent(f"{first} {last}", "PERSON")
        tb.raw(", record ")
        tb.ent(pick_entity_value("MEDICAL_RECORD", lang, first, last, rng), "MEDICAL_RECORD")
        tb.raw(", follow-up ")
        tb.ent(pick_entity_value("DATE", lang, first, last, rng), "DATE")
        tb.raw(". Tel ")
        tb.ent(pick_entity_value("PHONE", lang, first, last, rng), "PHONE")
        tb.raw("\n")
    elif category == "contract":
        tb.raw("Between ")
        tb.ent(pick_entity_value("ORGANIZATION", lang, first, last, rng), "ORGANIZATION")
        tb.raw(" and ")
        tb.ent(f"{first} {last}", "PERSON")
        tb.raw(", dated ")
        tb.ent(pick_entity_value("DATE", lang, first, last, rng), "DATE")
        tb.raw(". Tax ref ")
        tb.ent(pick_entity_value("TAX_ID", lang, first, last, rng), "TAX_ID")
        tb.raw("\n")
    elif category == "invoice":
        tb.raw("Bill to ")
        tb.ent(pick_entity_value("ORGANIZATION", lang, first, last, rng), "ORGANIZATION")
        tb.raw(". IBAN ")
        tb.ent(pick_entity_value("IBAN", lang, first, last, rng), "IBAN")
        tb.raw(" Contact ")
        tb.ent(fake_email(first, last, rng), "EMAIL")
        tb.raw("\n")
    else:  # support_ticket
        tb.raw("Ticket #")
        tb.raw(str(rng.randint(10000, 99999)))
        tb.raw(" from ")
        tb.ent(fake_email(first, last, rng), "EMAIL")
        tb.raw(" IP ")
        tb.ent(fake_ip(rng), "IP_ADDRESS")
        tb.raw(" URL ")
        tb.ent(fake_url(rng), "URL")
        tb.raw("\n")

    # Compléter jusqu’à `target` entités (sans dépasser)
    attempts = 0
    while len(tb.entities) < target and attempts < 48:
        attempts += 1
        et = weighted_pick(profile, rng, 1)[0]
        sep = rng.choice((" | ", "; ", "\n", " — ", " · "))
        tb.raw(sep)
        val = pick_entity_value(et, lang, first, last, rng)
        if et == "EMAIL" and rng.random() < 0.15:
            val = typo_text(val, rng)
            tags.append("extra_typo")
        tb.ent(val, et)

    text, entities = tb.build()
    # Dédupliquer chevauchements exacts
    entities = _dedupe_entities(entities)
    return text, entities, tags


def _dedupe_entities(entities: list[dict[str, Any]]) -> list[dict[str, Any]]:
    entities.sort(key=lambda e: (e["start"], -(e["end"] - e["start"])))
    out: list[dict[str, Any]] = []
    for e in entities:
        if any(o["start"] < e["end"] and o["end"] > e["start"] for o in out):
            continue
        out.append(e)
    out.sort(key=lambda x: x["start"])
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Génère un JSONL de textes PII synthétiques (AEGIS benchmark).")
    ap.add_argument("--output", type=Path, default=Path("generated/synthetic_pii.jsonl"))
    ap.add_argument("--n", type=int, default=10_000, help="Nombre de documents")
    ap.add_argument("--seed", type=int, default=42)
    args = ap.parse_args()
    rng = random.Random(args.seed)
    args.output.parent.mkdir(parents=True, exist_ok=True)

    cat_list = list(CATEGORY_WEIGHTS.keys())
    cat_w = [CATEGORY_WEIGHTS[c] for c in cat_list]

    with args.output.open("w", encoding="utf-8") as f:
        for i in range(args.n):
            lang = rng.choice(EU_LANGS)
            category = rng.choices(cat_list, weights=cat_w, k=1)[0]
            hard_case = rng.random() < 0.18
            text, entities, tags = generate_document(lang, category, rng, hard_case)
            rec = {
                "id": f"syn_{i:05d}",
                "language": lang,
                "category": category,
                "text": text,
                "entities": entities,
                "tags": tags,
            }
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")

    print(f"Wrote {args.n} records to {args.output}")


if __name__ == "__main__":
    main()
