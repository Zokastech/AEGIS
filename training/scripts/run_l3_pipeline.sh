#!/usr/bin/env bash
# AEGIS — pipeline NER niveau 3 : jeu synthétique → fine-tune → export ONNX (+ INT8).
# Variables (optionnelles) :
#   AEGIS_L3_EXAMPLES, AEGIS_L3_MAX_STEPS, AEGIS_L3_MODEL_NAME
#   AEGIS_L3_GOLD_DIR (défaut ../datasets/training/l3_regression) : letter_*.jsonl + corpus_expert_*.jsonl
#   AEGIS_L3_GOLD_JSONL : un seul fichier JSONL (remplace la concaténation des deux jeux or)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

AEGIS_L3_EXAMPLES="${AEGIS_L3_EXAMPLES:-12000}"
AEGIS_L3_MAX_STEPS="${AEGIS_L3_MAX_STEPS:-800}"
AEGIS_L3_MODEL_NAME="${AEGIS_L3_MODEL_NAME:-xlm-roberta-base}"
DATA_DIR="${AEGIS_L3_DATA_DIR:-./data/ci_l3}"
OUT_DIR="${AEGIS_L3_OUT_DIR:-./outputs/ci_l3}"
EXPORT_DIR="${AEGIS_L3_EXPORT_DIR:-./exports/ci_onnx}"

echo "[run_l3_pipeline] dataset → ${DATA_DIR} (${AEGIS_L3_EXAMPLES} exemples)"
python dataset_builder.py --num_examples "${AEGIS_L3_EXAMPLES}" --output "${DATA_DIR}"

GOLD_DIR="${AEGIS_L3_GOLD_DIR:-../datasets/training/l3_regression}"
GOLD_A="${GOLD_DIR}/letter_fr_golden.jsonl"
GOLD_B="${GOLD_DIR}/corpus_expert_composite_fr_golden.jsonl"
mkdir -p ./data
if [[ -n "${AEGIS_L3_GOLD_JSONL:-}" ]]; then
  echo "[run_l3_pipeline] golden unique (AEGIS_L3_GOLD_JSONL=${AEGIS_L3_GOLD_JSONL})"
  python jsonl_to_hf_dataset.py "${AEGIS_L3_GOLD_JSONL}" --output ./data/l3_gold --val_ratio 0.12
  python merge_hf_datasets.py "${DATA_DIR}" ./data/l3_gold --output ./data/merged_l3
  DATA_TRAIN="./data/merged_l3"
elif [[ -f "${GOLD_A}" && -f "${GOLD_B}" ]]; then
  echo "[run_l3_pipeline] fusion jeux or : lettre FR + corpus expert composite"
  cat "${GOLD_A}" "${GOLD_B}" > ./data/combined_l3_gold.jsonl
  python jsonl_to_hf_dataset.py ./data/combined_l3_gold.jsonl --output ./data/l3_gold --val_ratio 0.12
  python merge_hf_datasets.py "${DATA_DIR}" ./data/l3_gold --output ./data/merged_l3
  DATA_TRAIN="./data/merged_l3"
elif [[ -f "${GOLD_A}" ]]; then
  echo "[run_l3_pipeline] fusion jeu or lettre FR seul (${GOLD_A})"
  python jsonl_to_hf_dataset.py "${GOLD_A}" --output ./data/l3_gold --val_ratio 0.12
  python merge_hf_datasets.py "${DATA_DIR}" ./data/l3_gold --output ./data/merged_l3
  DATA_TRAIN="./data/merged_l3"
elif [[ -f "${GOLD_B}" ]]; then
  echo "[run_l3_pipeline] fusion jeu or corpus expert seul (${GOLD_B})"
  python jsonl_to_hf_dataset.py "${GOLD_B}" --output ./data/l3_gold --val_ratio 0.12
  python merge_hf_datasets.py "${DATA_DIR}" ./data/l3_gold --output ./data/merged_l3
  DATA_TRAIN="./data/merged_l3"
else
  echo "[run_l3_pipeline] pas de golden JSONL — entraînement sur synthétique seul"
  DATA_TRAIN="${DATA_DIR}"
fi

echo "[run_l3_pipeline] train (CPU, max_steps=${AEGIS_L3_MAX_STEPS})"
python train_ner.py \
  --dataset "${DATA_TRAIN}" \
  --output_dir "${OUT_DIR}" \
  --model_name "${AEGIS_L3_MODEL_NAME}" \
  --cpu \
  --max_steps "${AEGIS_L3_MAX_STEPS}" \
  --per_device_train_batch_size 2 \
  --per_device_eval_batch_size 2 \
  --max_seq_length 256 \
  --gradient_checkpointing \
  --disable_early_stopping \
  --learning_rate 5e-5

echo "[run_l3_pipeline] export ONNX"
python export_onnx.py \
  --model_dir "${OUT_DIR}/best_hf" \
  --out_dir "${EXPORT_DIR}" \
  --seq_len 256 \
  --skip_benchmark

echo "[run_l3_pipeline] terminé → ${EXPORT_DIR} (model_int8.onnx, tokenizer_hf/tokenizer.json)"
