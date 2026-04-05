#!/usr/bin/env bash
# AEGIS — pipeline NER niveau 3 : jeu synthétique → fine-tune → export ONNX (+ INT8).
# Variables (optionnelles) : AEGIS_L3_EXAMPLES, AEGIS_L3_MAX_STEPS, AEGIS_L3_MODEL_NAME
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

AEGIS_L3_EXAMPLES="${AEGIS_L3_EXAMPLES:-120}"
AEGIS_L3_MAX_STEPS="${AEGIS_L3_MAX_STEPS:-80}"
AEGIS_L3_MODEL_NAME="${AEGIS_L3_MODEL_NAME:-xlm-roberta-base}"
DATA_DIR="${AEGIS_L3_DATA_DIR:-./data/ci_l3}"
OUT_DIR="${AEGIS_L3_OUT_DIR:-./outputs/ci_l3}"
EXPORT_DIR="${AEGIS_L3_EXPORT_DIR:-./exports/ci_onnx}"

echo "[run_l3_pipeline] dataset → ${DATA_DIR} (${AEGIS_L3_EXAMPLES} exemples)"
python dataset_builder.py --num_examples "${AEGIS_L3_EXAMPLES}" --output "${DATA_DIR}"

echo "[run_l3_pipeline] train (CPU, max_steps=${AEGIS_L3_MAX_STEPS})"
python train_ner.py \
  --dataset "${DATA_DIR}" \
  --output_dir "${OUT_DIR}" \
  --model_name "${AEGIS_L3_MODEL_NAME}" \
  --cpu \
  --max_steps "${AEGIS_L3_MAX_STEPS}" \
  --per_device_train_batch_size 2 \
  --per_device_eval_batch_size 2 \
  --max_seq_length 128 \
  --gradient_checkpointing \
  --disable_early_stopping \
  --learning_rate 5e-5

echo "[run_l3_pipeline] export ONNX"
python export_onnx.py \
  --model_dir "${OUT_DIR}/best_hf" \
  --out_dir "${EXPORT_DIR}" \
  --seq_len 128 \
  --skip_benchmark

echo "[run_l3_pipeline] terminé → ${EXPORT_DIR} (model_int8.onnx, tokenizer_hf/tokenizer.json)"
