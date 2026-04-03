# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Build a synthetic multilingual dataset (European PII) in IOB2 format
compatible with Hugging Face `datasets` (tokens + ner_tags).
"""

from __future__ import annotations

import argparse
import random
import string
from typing import List, Sequence, Tuple

from ensure_hf_datasets import load_datasets

_ds = load_datasets()
ClassLabel = _ds.ClassLabel
Dataset = _ds.Dataset
DatasetDict = _ds.DatasetDict
Sequence = _ds.Sequence

# ---------------------------------------------------------------------------
# IOB2 schema (aligned with train_ner.py / evaluate.py)
# ---------------------------------------------------------------------------

LABELS: List[str] = [
    "O",
    "B-PERSON",
    "I-PERSON",
    "B-EMAIL",
    "B-PHONE",
    "I-PHONE",
    "B-IBAN",
    "I-IBAN",
    "B-CREDIT_CARD",
    "I-CREDIT_CARD",
    "B-SSN",
    "I-SSN",
    "B-PASSPORT",
    "B-ADDRESS",
    "I-ADDRESS",
    "B-ORGANIZATION",
    "I-ORGANIZATION",
    "B-LOCATION",
    "I-LOCATION",
    "B-DATE",
    "B-NATIONAL_ID",
    "I-NATIONAL_ID",
    "B-TAX_ID",
    "I-TAX_ID",
    "B-MEDICAL_RECORD",
    "I-MEDICAL_RECORD",
    "B-LICENSE_PLATE",
]

LABEL2ID = {l: i for i, l in enumerate(LABELS)}
ID2LABEL = {i: l for i, l in enumerate(LABELS)}

# Types with no I-* in LABELS: single whitespace-tokenized "word"
SINGLE_CHUNK_TYPES = frozenset(
    {"EMAIL", "DATE", "PASSPORT", "LICENSE_PLATE"}
)

LANGS = [
    "fr",
    "de",
    "it",
    "es",
    "nl",
    "pl",
    "pt",
    "el",
    "sv",
    "fi",
    "ro",
]

DOMAINS = [
    "email_professional",
    "web_form",
    "system_log",
    "support_ticket",
    "news_article",
    "chat",
]


def _iban_checksum(country: str, bban_digits: str) -> str:
    rearr = bban_digits + country + "00"
    expanded = []
    for ch in rearr:
        if ch.isdigit():
            expanded.append(ch)
        else:
            expanded.append(str(ord(ch.upper()) - 55))
    s = "".join(expanded)
    rem = 0
    for i in range(0, len(s), 9):
        rem = int(str(rem) + s[i : i + 9]) % 97
    return f"{98 - rem:02d}"


def synth_iban(lang: str) -> str:
    cc = {"fr": "FR", "de": "DE", "it": "IT", "es": "ES", "nl": "NL"}.get(lang, "FR")
    body = "".join(random.choices(string.digits, k=23 if cc == "FR" else 18))
    cd = _iban_checksum(cc, body)
    raw = cc + cd + body
    return " ".join([raw[i : i + 4] for i in range(0, len(raw), 4)])


def synth_email(local: str, domain: str) -> str:
    return f"{local}@{domain}"


def synth_phone(lang: str) -> str:
    if lang == "fr":
        return f"+33 {random.randint(6,7)} {random.randint(10,99):02d} {random.randint(10,99):02d} {random.randint(10,99):02d} {random.randint(10,99):02d}"
    if lang == "de":
        return f"+49 {random.randint(150,179)} {random.randint(1000000,9999999)}"
    if lang == "it":
        return f"+39 {random.randint(300,399)} {random.randint(1000000,9999999)}"
    if lang == "es":
        return f"+34 {random.randint(600,799)} {random.randint(100,999)} {random.randint(100,999)}"
    if lang == "nl":
        return f"+31 6 {random.randint(10000000,99999999)}"
    return f"+352 {random.randint(100000,999999)}"


def _luhn_valid(full: str) -> bool:
    digits = [int(c) for c in full]
    s = 0
    for i, d in enumerate(reversed(digits)):
        if i % 2 == 1:
            d *= 2
            if d > 9:
                d -= 9
        s += d
    return s % 10 == 0


def _luhn_append_check_digit(body15: str) -> str:
    for c in range(10):
        if _luhn_valid(body15 + str(c)):
            return body15 + str(c)
    return body15 + "0"


def synth_credit_card() -> str:
    body = "".join(str(random.randint(0, 9)) for _ in range(15))
    full = _luhn_append_check_digit(body)
    return " ".join(full[i : i + 4] for i in range(0, 16, 4))


def synth_national_id(lang: str) -> str:
    if lang == "fr":
        return f"{random.randint(1,2)} {random.randint(50,99):02d} {random.randint(1,12):02d} {random.randint(1,95):02d} {random.randint(1,999):03d} {random.randint(1,999):03d} {random.randint(1,97):02d}"
    if lang == "es":
        n = random.randint(10000000, 99999999)
        letters = "TRWAGMYFPDXBNJZSQVHLCKE"
        return f"{n:08d}{letters[n % 23]}"
    if lang == "it":
        cons = "".join(random.choices("BCDFGHJKLMNPQRSTVWXYZ", k=6))
        return f"{cons}{random.randint(50,99):02d}A{random.randint(1,28):02d}A{random.randint(100,999)}X"
    if lang == "de":
        return f"{random.randint(10,31):02d}{random.randint(10,12):02d}{random.randint(50,99):02d} A {random.randint(100,999)} B {random.randint(0,9)}"
    if lang == "nl":
        return f"{random.randint(10000000, 99999999)}"
    return f"ID-{lang.upper()}-{random.randint(100000,999999)}"


def synth_tax_id(lang: str) -> str:
    if lang == "fr":
        return f"FR{random.randint(10,99)} {''.join(random.choices(string.digits, k=11))}"
    if lang == "de":
        return f"{random.randint(10,12)}/{random.randint(100,999)}/{random.randint(10000,99999)}"
    return f"TAX-{lang.upper()}-{random.randint(100000000,999999999)}"


def synth_passport() -> str:
    return f"{random.choice('ABCDEFGHJKLMNPRSTUVWXYZ')}{random.randint(100000,999999)}"


def synth_date(lang: str) -> str:
    d, m, y = random.randint(1, 28), random.randint(1, 12), random.randint(1960, 2005)
    if lang in ("fr", "es", "it", "pt", "pl", "ro", "el"):
        return f"{d:02d}/{m:02d}/{y}"
    if lang in ("de", "fi", "sv"):
        return f"{d:02d}.{m:02d}.{y}"
    return f"{y}-{m:02d}-{d:02d}"


def synth_license_plate(lang: str) -> str:
    if lang == "de":
        return f"B-{random.choice(string.ascii_uppercase)}{random.randint(1000,9999)}"
    if lang == "fr":
        return f"AA-{random.randint(100,999)}-{random.choice(string.ascii_uppercase)}{random.choice(string.ascii_uppercase)}"
    if lang == "it":
        return f"GA {random.randint(100000,999999)}"
    return f"{random.randint(1,9)}{random.choice(string.ascii_uppercase)}{random.randint(100,999)}"


def synth_medical(lang: str) -> str:
    return f"MRN-{lang.upper()}-{random.randint(1000000,9999999)}"


def synth_ssn_generic() -> str:
    return f"{random.randint(100,999)}-{random.randint(10,99)}-{random.randint(1000,9999)}"


def synth_address(lang: str) -> str:
    streets = {
        "fr": ("Rue de la République", "Paris"),
        "de": ("Hauptstraße", "Berlin"),
        "it": ("Via Roma", "Milano"),
        "es": ("Calle Mayor", "Madrid"),
        "nl": ("Hoofdstraat", "Amsterdam"),
        "pl": ("ul. Marszałkowska", "Warszawa"),
        "pt": ("Rua Augusta", "Lisboa"),
        "el": ("Οδός Σύνταγμα", "Αθήνα"),
        "sv": ("Drottninggatan", "Stockholm"),
        "fi": ("Mannerheimintie", "Helsinki"),
        "ro": ("Strada Victoriei", "București"),
    }
    st, city = streets.get(lang, ("Main Street", "City"))
    n = random.randint(1, 120)
    pc = random.randint(10000, 99999)
    return f"{n} {st}, {pc} {city}"


def tokens_tags_from_chunks(chunks: Sequence[Tuple[str, str]]) -> Tuple[List[str], List[str]]:
    """chunks: (raw text, semantic type 'O' or 'PERSON', 'EMAIL', …)."""
    tokens: List[str] = []
    tags: List[str] = []
    for text, kind in chunks:
        parts = text.split()
        if not parts:
            continue
        if kind == "O":
            for p in parts:
                tokens.append(p)
                tags.append("O")
            continue
        if kind in SINGLE_CHUNK_TYPES:
            joined = text.replace(" ", "") if kind in ("EMAIL", "PASSPORT", "LICENSE_PLATE") else text
            if kind == "DATE" and " " in text:
                joined = text
            tp = parts if kind == "DATE" or " " in joined else [joined]
            if len(tp) == 1:
                tokens.append(tp[0])
                tags.append(f"B-{kind}")
            else:
                for i, p in enumerate(tp):
                    tokens.append(p)
                    tags.append(f"B-{kind}" if i == 0 else f"I-{kind}")
            continue
        # Multi-token avec B-/I-
        ent = kind  # e.g. PERSON -> B-PERSON
        for i, p in enumerate(parts):
            tokens.append(p)
            if i == 0:
                tags.append(f"B-{ent}")
            else:
                tags.append(f"I-{ent}")
    return tokens, tags


def _pick_name(rng: random.Random, lang: str) -> Tuple[str, str]:
    pools = {
        "fr": (["Marie", "Jean", "Camille", "Lucas"], ["Dupont", "Martin", "Bernard", "Lefèvre"]),
        "de": (["Anna", "Felix", "Klara", "Jonas"], ["Müller", "Schmidt", "Weber", "Wagner"]),
        "it": (["Giulia", "Marco", "Elena", "Luca"], ["Rossi", "Bianchi", "Romano", "Conti"]),
        "es": (["María", "Carlos", "Lucía", "Javier"], ["García", "López", "Martínez", "Sánchez"]),
        "nl": (["Emma", "Daan", "Sophie", "Lucas"], ["De Jong", "Jansen", "Visser", "Bakker"]),
        "pl": (["Katarzyna", "Piotr", "Anna", "Michał"], ["Nowak", "Wiśniewski", "Kowalczyk", "Lewandowski"]),
        "pt": (["Ana", "João", "Maria", "Pedro"], ["Silva", "Santos", "Oliveira", "Pereira"]),
        "el": (["Ελένη", "Νίκος", "Μαρία", "Γιώργος"], ["Παπαδόπουλος", "Γεωργίου", "Νικολάου"]),
        "sv": (["Elin", "Erik", "Anna", "Johan"], ["Andersson", "Johansson", "Karlsson", "Nilsson"]),
        "fi": (["Sofia", "Matti", "Laura", "Juhani"], ["Virtanen", "Korhonen", "Mäkinen", "Nieminen"]),
        "ro": (["Maria", "Ion", "Elena", "Andrei"], ["Popescu", "Ionescu", "Dumitru", "Stan"]),
    }
    fn, ln = pools.get(lang, pools["fr"])
    return rng.choice(fn), rng.choice(ln)


def build_email_professional(lang: str, rng: random.Random) -> Tuple[List[str], List[str]]:
    fn, ln = _pick_name(rng, lang)
    dom = rng.choice(["acme.eu", "corp.int", "service.gov", "clinic.med"])
    local = f"{fn.lower()}.{ln.lower().replace(' ', '')}"
    org = rng.choice(["ACME Europe", "Nordic Data AB", "MediCare EU", "Banque Centrale EU"])
    loc = rng.choice(["Paris", "Bruxelles", "Berlin", "Amsterdam", "Madrid", "Roma"])
    iban = synth_iban(lang)
    chunks = [
        ("Bonjour", "O"),
        (f"{fn} {ln}", "PERSON"),
        (",", "O"),
        ("merci", "O"),
        ("de", "O"),
        ("confirmer", "O"),
        ("votre", "O"),
        ("RIB", "O"),
        (iban, "IBAN"),
        ("et", "O"),
        ("votre", "O"),
        ("email", "O"),
        (synth_email(local, dom), "EMAIL"),
        (".", "O"),
        ("Cordialement", "O"),
        (",", "O"),
        (org, "ORGANIZATION"),
        (",", "O"),
        (loc, "LOCATION"),
    ]
    if lang not in ("fr",):
        chunks = [
            ("Hello", "O"),
            (f"{fn} {ln}", "PERSON"),
            ("please", "O"),
            ("send", "O"),
            ("IBAN", "O"),
            (iban, "IBAN"),
            ("to", "O"),
            (synth_email(local, dom), "EMAIL"),
            ("Regards", "O"),
            (org, "ORGANIZATION"),
            (loc, "LOCATION"),
        ]
    return tokens_tags_from_chunks(chunks)


def build_web_form(lang: str, rng: random.Random) -> Tuple[List[str], List[str]]:
    fn, ln = _pick_name(rng, lang)
    phone = synth_phone(lang)
    addr = synth_address(lang)
    nid = synth_national_id(lang)
    tax = synth_tax_id(lang)
    bdate = synth_date(lang)
    chunks = [
        ("Form", "O"),
        ("Name", "O"),
        (f"{fn} {ln}", "PERSON"),
        ("Phone", "O"),
        (phone.replace(" ", " "), "PHONE"),
        ("Address", "O"),
        (addr, "ADDRESS"),
        ("NationalId", "O"),
        (nid.replace(" ", " "), "NATIONAL_ID"),
        ("TaxId", "O"),
        (tax.replace(" ", " "), "TAX_ID"),
        ("DOB", "O"),
        (bdate, "DATE"),
    ]
    return tokens_tags_from_chunks(chunks)


def build_system_log(lang: str, rng: random.Random) -> Tuple[List[str], List[str]]:
    uid = f"uid={rng.randint(10000, 99999)}"
    ip = f"{rng.randint(10,223)}.{rng.randint(0,255)}.{rng.randint(0,255)}.{rng.randint(1,254)}"
    card = synth_credit_card()
    chunks = [
        ("INFO", "O"),
        ("auth", "O"),
        (uid, "O"),
        ("ip", "O"),
        (ip, "O"),
        ("card", "O"),
        (card, "CREDIT_CARD"),
        ("session", "O"),
        ("ok", "O"),
    ]
    return tokens_tags_from_chunks(chunks)


def build_support_ticket(lang: str, rng: random.Random) -> Tuple[List[str], List[str]]:
    fn, ln = _pick_name(rng, lang)
    plate = synth_license_plate(lang)
    mrn = synth_medical(lang)
    chunks = [
        ("Ticket", "O"),
        ("#", "O"),
        (str(rng.randint(100000, 999999)), "O"),
        ("from", "O"),
        (f"{fn} {ln}", "PERSON"),
        ("vehicle", "O"),
        (plate.replace(" ", ""), "LICENSE_PLATE"),
        ("patient", "O"),
        (mrn.replace(" ", ""), "MEDICAL_RECORD"),
        ("status", "O"),
        ("open", "O"),
    ]
    return tokens_tags_from_chunks(chunks)


def build_news_article(lang: str, rng: random.Random) -> Tuple[List[str], List[str]]:
    fn, ln = _pick_name(rng, lang)
    org = rng.choice(["Commission Européenne", "European Central Bank", "Parlamento Europeo"])
    loc = rng.choice(["Strasbourg", "Frankfurt", "Bruxelles", "Luxembourg"])
    chunks = [
        (org, "ORGANIZATION"),
        ("announced", "O"),
        ("today", "O"),
        ("in", "O"),
        (loc, "LOCATION"),
        ("spokesperson", "O"),
        (f"{fn} {ln}", "PERSON"),
        ("added", "O"),
        ("details", "O"),
        ("follow", "O"),
    ]
    return tokens_tags_from_chunks(chunks)


def build_chat(_lang: str, rng: random.Random) -> Tuple[List[str], List[str]]:
    pp = f"{rng.choice('ABCDEFGHJKLMNPRSTUVWXYZ')}{rng.randint(100000, 999999)}"
    ssn = f"{rng.randint(100, 999)}-{rng.randint(10, 99)}-{rng.randint(1000, 9999)}"
    chunks = [
        ("user1:", "O"),
        ("my", "O"),
        ("passport", "O"),
        ("is", "O"),
        (pp, "PASSPORT"),
        ("user2:", "O"),
        ("ssn", "O"),
        (ssn, "SSN"),
    ]
    return tokens_tags_from_chunks(chunks)


BUILDERS = {
    "email_professional": build_email_professional,
    "web_form": build_web_form,
    "system_log": build_system_log,
    "support_ticket": build_support_ticket,
    "news_article": build_news_article,
    "chat": build_chat,
}


def generate_example(rng: random.Random) -> dict:
    lang = rng.choice(LANGS)
    domain = rng.choice(DOMAINS)
    tokens, tags = BUILDERS[domain](lang, rng)
    # Validation des labels
    for t in tags:
        if t not in LABEL2ID:
            raise ValueError(t)
    return {"tokens": tokens, "ner_tags": tags, "lang": lang, "domain": domain}


def build_dataset(num_examples: int, seed: int = 42) -> DatasetDict:
    rng = random.Random(seed)
    rows = [generate_example(rng) for _ in range(num_examples)]
    str_ds = Dataset.from_list(rows)
    str_ds = str_ds.cast_column(
        "ner_tags",
        Sequence(ClassLabel(names=LABELS)),
    )
    split = str_ds.train_test_split(test_size=0.05, seed=seed)
    return DatasetDict(train=split["train"], validation=split["test"])


def main() -> None:
    parser = argparse.ArgumentParser(description="Build synthetic EU PII NER dataset (IOB2).")
    parser.add_argument("--num_examples", type=int, default=50_000)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--output", type=str, default="./data/eu_pii_synthetic")
    args = parser.parse_args()

    ds = build_dataset(args.num_examples, args.seed)
    ds.save_to_disk(args.output)
    print(f"Saved {args.num_examples} examples (train/val split) to {args.output}")
    print(ds)


if __name__ == "__main__":
    main()
