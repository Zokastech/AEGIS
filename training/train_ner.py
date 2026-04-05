# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Fine-tune XLM-RoBERTa-base for token classification (EU PII NER).
"""

from __future__ import annotations

import argparse
import inspect
import os
from typing import Any, Dict, List, Optional

# Before any torch import (via transformers): PyTorch reads these at MPS init.
# High "other allocations" + OOM in backward/optimizer: prefer --cpu or free RAM.
_mps_ratio = os.environ.get("AEGIS_MPS_HIGH_WATERMARK_RATIO", "").strip()
if os.environ.get("AEGIS_RELAX_MPS_MEMORY_CAP", "").lower() in ("1", "true", "yes"):
    os.environ["PYTORCH_MPS_HIGH_WATERMARK_RATIO"] = "0.0"
elif _mps_ratio:
    os.environ["PYTORCH_MPS_HIGH_WATERMARK_RATIO"] = _mps_ratio

import numpy as np
from ensure_hf_datasets import load_datasets

_ds = load_datasets()
DatasetDict = _ds.DatasetDict
load_from_disk = _ds.load_from_disk
from seqeval.metrics import f1_score, precision_score, recall_score
from transformers import (
    AutoModelForTokenClassification,
    AutoTokenizer,
    DataCollatorForTokenClassification,
    EarlyStoppingCallback,
    Trainer,
    TrainingArguments,
)

from dataset_builder import ID2LABEL, LABEL2ID, LABELS

MODEL_NAME_DEFAULT = "xlm-roberta-base"


def tokenize_and_align_labels(
    examples: Dict[str, List],
    tokenizer: Any,
    label2id: Dict[str, int],
    max_length: int = 512,
) -> Dict[str, List]:
    tokenized = tokenizer(
        examples["tokens"],
        is_split_into_words=True,
        truncation=True,
        max_length=max_length,
        padding=False,
    )
    all_labels: List[List[int]] = []
    for i, labels in enumerate(examples["ner_tags"]):
        word_ids = tokenized.word_ids(batch_index=i)
        label_ids: List[int] = []
        prev_wid: Optional[int] = None
        for j, wid in enumerate(word_ids):
            if wid is None:
                label_ids.append(-100)
            elif wid != prev_wid:
                lab = labels[wid]
                if isinstance(lab, str):
                    label_ids.append(label2id[lab])
                else:
                    label_ids.append(int(lab))
            else:
                # Continuation subwords: ignored by loss (first-subword alignment).
                label_ids.append(-100)
            prev_wid = wid
        all_labels.append(label_ids)
    tokenized["labels"] = all_labels
    return tokenized


def compute_metrics_builder(id2label: Dict[int, str]):
    def compute_metrics(eval_pred) -> Dict[str, float]:
        logits, labels = eval_pred
        preds = np.argmax(logits, axis=-1)
        true_preds: List[List[str]] = []
        true_labs: List[List[str]] = []
        for pred_row, lab_row in zip(preds, labels):
            pr: List[str] = []
            lb: List[str] = []
            for p_i, l_i in zip(pred_row, lab_row):
                if l_i == -100:
                    continue
                pr.append(id2label[int(p_i)])
                lb.append(id2label[int(l_i)])
            true_preds.append(pr)
            true_labs.append(lb)
        return {
            "precision": precision_score(true_labs, true_preds),
            "recall": recall_score(true_labs, true_preds),
            "f1": f1_score(true_labs, true_preds),
        }

    return compute_metrics


def main() -> None:
    parser = argparse.ArgumentParser(description="Fine-tune XLM-RoBERTa for EU PII NER.")
    parser.add_argument("--dataset", type=str, default="./data/eu_pii_synthetic")
    parser.add_argument("--output_dir", type=str, default="./outputs/ner-xlmr-eu-pii")
    parser.add_argument("--model_name", type=str, default=MODEL_NAME_DEFAULT)
    parser.add_argument("--num_train_epochs", type=float, default=4.0)
    parser.add_argument("--per_device_train_batch_size", type=int, default=16)
    parser.add_argument("--per_device_eval_batch_size", type=int, default=16)
    parser.add_argument("--learning_rate", type=float, default=5e-5)
    parser.add_argument("--weight_decay", type=float, default=0.01)
    parser.add_argument("--warmup_ratio", type=float, default=0.1)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--fp16", action="store_true", help="Use mixed precision (CUDA).")
    parser.add_argument("--gradient_accumulation_steps", type=int, default=1)
    parser.add_argument(
        "--cpu",
        action="store_true",
        help="Forcer l’entraînement sur CPU (utile si MPS/CUDA manque de mémoire).",
    )
    parser.add_argument(
        "--max_seq_length",
        type=int,
        default=512,
        help="Longueur max des séquences après tokenisation (256–384 sur Mac 16 Go pour limiter la RAM).",
    )
    parser.add_argument(
        "--gradient_checkpointing",
        action="store_true",
        help="Ré-active les activations pendant la rétropropagation : moins de pic mémoire, entraînement un peu plus lent.",
    )
    parser.add_argument(
        "--adafactor",
        action="store_true",
        help="Optimiseur Adafactor au lieu d’AdamW (états optimiseur plus légers) — utile si OOM MPS sur Mac 16 Go.",
    )
    parser.add_argument(
        "--max_steps",
        type=int,
        default=-1,
        help="Si > 0, limite l’entraînement à ce nombre de steps (prioritaire sur num_train_epochs). "
        "Active eval/save par steps (adapté au CI).",
    )
    parser.add_argument(
        "--disable_early_stopping",
        action="store_true",
        help="Désactive EarlyStoppingCallback (recommandé avec --max_steps court en CI).",
    )
    args = parser.parse_args()

    if not args.cpu:
        try:
            import torch

            if getattr(torch.backends, "mps", None) and torch.backends.mps.is_available():
                print(
                    "[train_ner] MPS actif. Si OOM (backward ou AdamW), utilisez --cpu ou "
                    "libérez de la RAM ; optionnel : AEGIS_RELAX_MPS_MEMORY_CAP=1 (risque swap). "
                    "Voir training/README.md."
                )
        except ImportError:
            pass

    os.makedirs(args.output_dir, exist_ok=True)

    raw: DatasetDict = load_from_disk(args.dataset)

    tokenizer = AutoTokenizer.from_pretrained(args.model_name, use_fast=True)
    model = AutoModelForTokenClassification.from_pretrained(
        args.model_name,
        num_labels=len(LABELS),
        id2label=ID2LABEL,
        label2id=LABEL2ID,
        ignore_mismatched_sizes=True,
    )
    if args.gradient_checkpointing:
        model.gradient_checkpointing_enable()
        model.config.use_cache = False

    def _tok(batch):
        return tokenize_and_align_labels(
            batch, tokenizer, LABEL2ID, max_length=args.max_seq_length
        )

    cols = raw["train"].column_names
    tokenized = raw.map(_tok, batched=True, remove_columns=cols)
    data_collator = DataCollatorForTokenClassification(tokenizer)

    id2label = {int(k): v for k, v in ID2LABEL.items()}
    compute_metrics = compute_metrics_builder(id2label)

    ta_params = inspect.signature(TrainingArguments.__init__).parameters
    device_kw: Dict[str, Any] = {}
    if args.cpu:
        if "use_cpu" in ta_params:
            device_kw["use_cpu"] = True
        else:
            device_kw["no_cuda"] = True
        device_kw["use_mps_device"] = False

    eval_kw: Dict[str, Any]
    save_strategy: str
    save_steps_kw: Dict[str, Any] = {}
    max_steps_kw: Dict[str, Any] = {}
    log_steps = 50
    if args.max_steps > 0:
        eval_steps = max(1, min(25, args.max_steps // 3))
        log_steps = max(1, min(10, args.max_steps))
        if "eval_strategy" in ta_params:
            eval_kw = {"eval_strategy": "steps", "eval_steps": eval_steps}
        else:
            eval_kw = {"evaluation_strategy": "steps", "eval_steps": eval_steps}
        save_strategy = "steps"
        save_steps_kw = {"save_steps": eval_steps}
        max_steps_kw = {"max_steps": args.max_steps}
    else:
        if "eval_strategy" in ta_params:
            eval_kw = {"eval_strategy": "epoch"}
        else:
            eval_kw = {"evaluation_strategy": "epoch"}
        save_strategy = "epoch"

    ta_kw: Dict[str, Any] = dict(
        output_dir=args.output_dir,
        save_strategy=save_strategy,
        learning_rate=args.learning_rate,
        per_device_train_batch_size=args.per_device_train_batch_size,
        per_device_eval_batch_size=args.per_device_eval_batch_size,
        num_train_epochs=args.num_train_epochs,
        weight_decay=args.weight_decay,
        warmup_ratio=args.warmup_ratio,
        load_best_model_at_end=True,
        metric_for_best_model="f1",
        greater_is_better=True,
        save_total_limit=3,
        seed=args.seed,
        logging_steps=log_steps,
        fp16=args.fp16 and not args.cpu,
        gradient_accumulation_steps=args.gradient_accumulation_steps,
        report_to=[],
        **device_kw,
        **eval_kw,
        **save_steps_kw,
        **max_steps_kw,
    )
    if args.gradient_checkpointing and "gradient_checkpointing" in ta_params:
        ta_kw["gradient_checkpointing"] = True
    if args.adafactor and "optim" in ta_params:
        ta_kw["optim"] = "adafactor"
    training_args = TrainingArguments(**ta_kw)

    callbacks = []
    if not args.disable_early_stopping:
        callbacks.append(EarlyStoppingCallback(early_stopping_patience=2))
    trainer_kw: Dict[str, Any] = dict(
        model=model,
        args=training_args,
        train_dataset=tokenized["train"],
        eval_dataset=tokenized["validation"],
        data_collator=data_collator,
        compute_metrics=compute_metrics,
        callbacks=callbacks,
    )
    _ts = inspect.signature(Trainer.__init__).parameters
    if "processing_class" in _ts:
        trainer_kw["processing_class"] = tokenizer
    else:
        trainer_kw["tokenizer"] = tokenizer
    trainer = Trainer(**trainer_kw)

    trainer.train()
    trainer.save_model(os.path.join(args.output_dir, "best_hf"))
    tokenizer.save_pretrained(os.path.join(args.output_dir, "best_hf"))
    print(f"Best model saved under {args.output_dir}/best_hf")


if __name__ == "__main__":
    main()
