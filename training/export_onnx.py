# AEGIS — zokastech.fr — Apache 2.0 / MIT
"""
Export ONNX (optimisations + quantification INT8) et benchmark latence PyTorch / ONNX.
Sauvegarde tokenizer au format Hugging Face (tokenizer.json utilisable depuis la crate Rust `tokenizers`).
"""

from __future__ import annotations

import argparse
import inspect
import os
import statistics
import time
from typing import Any, Dict, List, Tuple

import numpy as np
import onnx
import torch
from onnxruntime import GraphOptimizationLevel, InferenceSession, SessionOptions
from onnxruntime.quantization import QuantType, quantize_dynamic
from transformers import AutoModelForTokenClassification, AutoTokenizer

from dataset_builder import LABELS


def export_torch_to_onnx(
    model: torch.nn.Module,
    tokenizer: Any,
    onnx_path: str,
    opset: int = 14,
) -> None:
    model.eval()
    device = next(model.parameters()).device
    dummy = tokenizer(
        ["EU PII test sentence for export."],
        return_tensors="pt",
        padding="max_length",
        truncation=True,
        max_length=128,
    )
    input_ids = dummy["input_ids"].to(device)
    attention_mask = dummy["attention_mask"].to(device)

    class _Wrap(torch.nn.Module):
        def __init__(self, m):
            super().__init__()
            self.m = m

        def forward(self, input_ids, attention_mask):
            return self.m(input_ids=input_ids, attention_mask=attention_mask).logits

    wrapped = _Wrap(model).eval()

    # PyTorch 2.5+ : l’exporteur Dynamo + onnxscript peut appeler le version_converter ONNX
    # et échouer sur LayerNormalization vers opset 14 (« No Previous Version of LayerNormalization »).
    # L’export TorchScript classique (`dynamo=False`) produit un graphe compatible opset 14.
    export_kw: Dict[str, Any] = {
        "input_names": ["input_ids", "attention_mask"],
        "output_names": ["logits"],
        "dynamic_axes": {
            "input_ids": {0: "batch", 1: "sequence"},
            "attention_mask": {0: "batch", 1: "sequence"},
            "logits": {0: "batch", 1: "sequence"},
        },
        "opset_version": opset,
        "do_constant_folding": True,
    }
    if "dynamo" in inspect.signature(torch.onnx.export).parameters:
        export_kw["dynamo"] = False

    torch.onnx.export(
        wrapped,
        (input_ids, attention_mask),
        onnx_path,
        **export_kw,
    )


def optimize_onnx_graph(src: str, dst: str) -> None:
    opts = SessionOptions()
    opts.graph_optimization_level = GraphOptimizationLevel.ORT_ENABLE_ALL
    opts.optimized_model_filepath = dst
    InferenceSession(src, opts, providers=["CPUExecutionProvider"])


def benchmark(
    run_fn,
    warmup: int = 5,
    runs: int = 30,
) -> Tuple[float, float]:
    for _ in range(warmup):
        run_fn()
    times: List[float] = []
    for _ in range(runs):
        t0 = time.perf_counter()
        run_fn()
        times.append(time.perf_counter() - t0)
    return statistics.mean(times), statistics.stdev(times) if len(times) > 1 else 0.0


def main() -> None:
    parser = argparse.ArgumentParser(description="Export NER model to ONNX + INT8 + latency bench.")
    parser.add_argument("--model_dir", type=str, default="./outputs/ner-xlmr-eu-pii/best_hf")
    parser.add_argument("--out_dir", type=str, default="./exports/onnx_ner")
    parser.add_argument("--seq_len", type=int, default=128)
    parser.add_argument("--bench_runs", type=int, default=30)
    parser.add_argument(
        "--skip_benchmark",
        action="store_true",
        help="N’exécute pas le bench latence PyTorch/ONNX (CI plus rapide).",
    )
    args = parser.parse_args()

    model_dir = os.path.abspath(os.path.expanduser(args.model_dir))
    if not os.path.isdir(model_dir):
        raise SystemExit(
            f"model_dir introuvable : {model_dir}\n"
            "Indiquez le dossier du checkpoint Hugging Face (ex. …/outputs/ner-…/best_hf) "
            "après train_ner.py."
        )

    os.makedirs(args.out_dir, exist_ok=True)
    tok_path = os.path.join(args.out_dir, "tokenizer_hf")
    onnx_fp32 = os.path.join(args.out_dir, "model.onnx")
    onnx_opt = os.path.join(args.out_dir, "model_optimized.onnx")
    onnx_int8 = os.path.join(args.out_dir, "model_int8.onnx")

    # local_files_only: prevents HF Hub from treating an absolute path as repo_id (recent transformers/hub).
    tokenizer = AutoTokenizer.from_pretrained(model_dir, local_files_only=True)
    model = AutoModelForTokenClassification.from_pretrained(model_dir, local_files_only=True)
    model.eval()
    model.cpu()

    tokenizer.save_pretrained(tok_path)

    export_torch_to_onnx(model, tokenizer, onnx_fp32)
    onnx_model = onnx.load(onnx_fp32)
    onnx.checker.check_model(onnx_model)

    optimize_onnx_graph(onnx_fp32, onnx_opt)
    # Quantifier depuis le graphe ONNX d’origine (l’optimiseur ORT peut casser shape_inference).
    quantize_dynamic(onnx_fp32, onnx_int8, weight_type=QuantType.QUInt8)

    if not args.skip_benchmark:
        text = "Contact: Marie Dupont email m.dupont@acme.eu IBAN FR76 3000 6000 0112 3456 7890 189"
        enc = tokenizer(
            [text],
            return_tensors="pt",
            padding="max_length",
            truncation=True,
            max_length=args.seq_len,
        )
        input_ids = enc["input_ids"].numpy().astype(np.int64)
        attention_mask = enc["attention_mask"].numpy().astype(np.int64)

        def pt_run():
            with torch.no_grad():
                _ = model(
                    input_ids=torch.from_numpy(input_ids),
                    attention_mask=torch.from_numpy(attention_mask),
                ).logits.numpy()

        sess_fp32 = InferenceSession(onnx_fp32, providers=["CPUExecutionProvider"])
        sess_opt = InferenceSession(onnx_opt, providers=["CPUExecutionProvider"])
        sess_q = InferenceSession(onnx_int8, providers=["CPUExecutionProvider"])

        def ort_run(sess: InferenceSession):
            _ = sess.run(
                None,
                {"input_ids": input_ids, "attention_mask": attention_mask},
            )

        results: Dict[str, Tuple[float, float]] = {}
        results["pytorch_fp32"] = benchmark(pt_run, runs=args.bench_runs)
        results["onnx_fp32"] = benchmark(lambda: ort_run(sess_fp32), runs=args.bench_runs)
        results["onnx_optimized"] = benchmark(lambda: ort_run(sess_opt), runs=args.bench_runs)
        results["onnx_int8"] = benchmark(lambda: ort_run(sess_q), runs=args.bench_runs)

        report = os.path.join(args.out_dir, "latency_benchmark.txt")
        lines = [
            f"seq_len={args.seq_len} runs={args.bench_runs}",
            f"num_labels={len(LABELS)}",
            "",
        ]
        for k, (m, s) in results.items():
            lines.append(f"{k}: mean_s={m:.6f} stdev_s={s:.6f}")
        with open(report, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))

        print("\n".join(lines))

    print(f"\nTokenizer (Rust `tokenizers`): {tok_path}/tokenizer.json")
    print(f"ONNX INT8: {onnx_int8}")


if __name__ == "__main__":
    main()
