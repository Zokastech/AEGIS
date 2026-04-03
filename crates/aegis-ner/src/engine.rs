// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Moteur NER ONNX : session `ort`, tokenizer Hugging Face, post-traitement IOB.

use std::collections::HashMap;

use ndarray::s;
use ndarray::Array2;
#[cfg(feature = "ort-coreml")]
use ort::CoreMLExecutionProvider;
#[cfg(feature = "ort-cuda")]
use ort::CUDAExecutionProvider;
#[cfg(feature = "ort-tensorrt")]
use ort::TensorRTExecutionProvider;
use ort::{
    CPUExecutionProvider, ExecutionProviderDispatch, GraphOptimizationLevel, Session, Tensor,
};

use tokenizers::Tokenizer;
use tokenizers::{TruncationParams, TruncationStrategy};

use crate::config::{default_id2label_map, NerConfig, NerDevice};
use crate::error::{NerError, Result};
use crate::postprocess::{merge_token_predictions, TokenPrediction};
use crate::prediction::NerPrediction;

fn execution_providers(device: NerDevice) -> Vec<ExecutionProviderDispatch> {
    let mut eps = Vec::new();
    match device {
        NerDevice::Cpu => {}
        NerDevice::Cuda { .. } => {
            #[cfg(feature = "ort-cuda")]
            eps.push(CUDAExecutionProvider::default().build());
        }
        NerDevice::TensorRt => {
            #[cfg(feature = "ort-tensorrt")]
            eps.push(TensorRTExecutionProvider::default().build());
        }
        NerDevice::CoreML => {
            #[cfg(feature = "ort-coreml")]
            eps.push(CoreMLExecutionProvider::default().build());
        }
    }
    eps.push(CPUExecutionProvider::default().build());
    eps
}

fn build_session(model_path: &str, config: &NerConfig) -> Result<Session> {
    let eps = execution_providers(config.device);
    Ok(Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(config.intra_threads)?
        .with_execution_providers(eps)?
        .commit_from_file(model_path)?)
}

fn resolve_token_type_input(session: &Session, config: &NerConfig) -> Option<String> {
    if let Some(ref n) = config.token_type_ids_input_name {
        return Some(n.clone());
    }
    if session.inputs.len() == 3 {
        return Some(session.inputs[2].name.clone());
    }
    None
}

fn stable_softmax_argmax(row: &[f32]) -> (usize, f64) {
    if row.is_empty() {
        return (0, 0.0);
    }
    let m = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0f32;
    let exps: Vec<f32> = row.iter().map(|&x| (x - m).exp()).collect();
    for &e in &exps {
        sum += e;
    }
    let probs: Vec<f64> = exps.iter().map(|&e| (e / sum) as f64).collect();
    probs
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, &p)| (i, p))
        .unwrap_or((0, 0.0))
}

struct EncodedRow {
    ids: Vec<u32>,
    mask: Vec<u32>,
    offsets: Vec<(usize, usize)>,
    tokens: Vec<String>,
}

/// Token-classification inference (BERT / DistilBERT / XLM-RoBERTa exported to ONNX).
pub struct NerEngine {
    session: Session,
    tokenizer: Tokenizer,
    config: NerConfig,
    id2label: HashMap<u32, String>,
    token_type_input_name: Option<String>,
}

impl NerEngine {
    /// Load ONNX model and `tokenizer.json`.
    pub fn new(model_path: &str, tokenizer_path: &str, mut config: NerConfig) -> Result<Self> {
        let mut tokenizer = Tokenizer::from_file(tokenizer_path)?;
        let _ = tokenizer.with_truncation(Some(TruncationParams {
            max_length: config.max_sequence_length,
            strategy: TruncationStrategy::LongestFirst,
            stride: 0,
            ..Default::default()
        }));

        let id2label = if let Some(ref p) = config.hf_config_json_path {
            NerConfig::load_hf_id2label(p)?
        } else if let Some(m) = config.id2label.take() {
            m
        } else {
            default_id2label_map()
        };

        let session = build_session(model_path, &config)?;
        let token_type_input_name = resolve_token_type_input(&session, &config);

        Ok(Self {
            session,
            tokenizer,
            config,
            id2label,
            token_type_input_name,
        })
    }

    /// Run inference on a single text.
    pub fn predict(&self, text: &str) -> Result<Vec<NerPrediction>> {
        Ok(self.predict_batch(&[text])?.into_iter().next().unwrap_or_default())
    }

    /// Batch inference (max size [`NerConfig::batch_size`]).
    pub fn predict_batch(&self, texts: &[&str]) -> Result<Vec<Vec<NerPrediction>>> {
        let mut all = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(self.config.batch_size.max(1)) {
            let batch_out = self.run_ort_batch(chunk)?;
            all.extend(batch_out);
        }
        Ok(all)
    }

    fn run_ort_batch(&self, texts: &[&str]) -> Result<Vec<Vec<NerPrediction>>> {
        let pad_id = self
            .tokenizer
            .token_to_id("<redacted_PAD>")
            .or_else(|| self.tokenizer.token_to_id("<pad>"))
            .unwrap_or(0);

        let mut rows: Vec<EncodedRow> = Vec::with_capacity(texts.len());
        for t in texts {
            let enc = self
                .tokenizer
                .encode(*t, true)
                .map_err(|e| NerError::Tokenizer(e.to_string()))?;
            let ids = enc.get_ids().to_vec();
            let mask = enc.get_attention_mask().to_vec();
            let offsets: Vec<(usize, usize)> = enc.get_offsets().iter().copied().collect();
            let tokens = enc.get_tokens().iter().cloned().collect();
            rows.push(EncodedRow {
                ids,
                mask,
                offsets,
                tokens,
            });
        }

        let max_len = rows
            .iter()
            .map(|r| r.ids.len())
            .max()
            .unwrap_or(0)
            .min(self.config.max_sequence_length);

        for r in &mut rows {
            while r.ids.len() < max_len {
                r.ids.push(pad_id);
                r.mask.push(0);
                r.offsets.push((0, 0));
                r.tokens.push("<redacted_PAD>".into());
            }
            if r.ids.len() > max_len {
                r.ids.truncate(max_len);
                r.mask.truncate(max_len);
                r.offsets.truncate(max_len);
                r.tokens.truncate(max_len);
            }
        }

        let batch = rows.len();
        let seq = max_len;
        let mut ids_flat: Vec<i64> = Vec::with_capacity(batch * seq);
        let mut mask_flat: Vec<i64> = Vec::with_capacity(batch * seq);
        for r in &rows {
            for &id in &r.ids {
                ids_flat.push(id as i64);
            }
            for &m in &r.mask {
                mask_flat.push(m as i64);
            }
        }

        let input_ids = Array2::from_shape_vec((batch, seq), ids_flat)
            .map_err(|e| NerError::Shape(e.to_string()))?;
        let attention = Array2::from_shape_vec((batch, seq), mask_flat)
            .map_err(|e| NerError::Shape(e.to_string()))?;

        let outputs = if let Some(ref tt_name) = self.token_type_input_name {
            let tt = Array2::<i64>::zeros((batch, seq));
            self.session
                .run(ort::inputs![
                    self.config.input_ids_input_name.as_str() => Tensor::from_array(input_ids.clone())?,
                    self.config.attention_mask_input_name.as_str() => Tensor::from_array(attention.clone())?,
                    tt_name.as_str() => Tensor::from_array(tt)?,
                ]?)
                .map_err(|e| NerError::Onnx(e.to_string()))?
        } else {
            self.session
                .run(ort::inputs![
                    self.config.input_ids_input_name.as_str() => Tensor::from_array(input_ids)?,
                    self.config.attention_mask_input_name.as_str() => Tensor::from_array(attention)?,
                ]?)
                .map_err(|e| NerError::Onnx(e.to_string()))?
        };

        let logits_tensor = outputs
            .get(self.config.logits_output_name.as_str())
            .ok_or_else(|| {
                let keys: Vec<_> = outputs.keys().copied().collect();
                NerError::Shape(format!(
                    "sortie `{}` absente; sorties: {keys:?}",
                    self.config.logits_output_name
                ))
            })?;

        let arr = logits_tensor
            .try_extract_tensor::<f32>()
            .map_err(|e| NerError::Onnx(e.to_string()))?
            .to_owned();
        let shape = arr.shape();
        if shape.len() != 3 {
            return Err(NerError::Shape(format!(
                "logits attendus [batch, seq, labels], forme {:?}",
                shape
            )));
        }
        let bsz = shape[0];
        let slen = shape[1];
        if bsz != batch || slen != seq {
            return Err(NerError::Shape(format!(
                "batch/seq ONNX {:?} != encodé {batch}/{seq}",
                shape
            )));
        }

        let mut batch_preds = Vec::with_capacity(batch);
        for b in 0..batch {
            let text = texts[b];
            let row = &rows[b];
            let mut toks = Vec::with_capacity(seq);
            for s in 0..seq {
                if self.config.skip_edge_special_tokens && (s == 0 || s + 1 == seq) {
                    toks.push(TokenPrediction {
                        raw_label: "O".into(),
                        score: 1.0,
                        start: 0,
                        end: 0,
                        token_text: row.tokens.get(s).cloned().unwrap_or_default(),
                        attention: row.mask.get(s).copied().unwrap_or(0),
                    });
                    continue;
                }

                let row_logits = arr.slice(s![b, s, ..]);
                let logits: Vec<f32> = row_logits.iter().copied().collect();
                let (lid, sc) = stable_softmax_argmax(&logits);
                let lid = lid as u32;
                let raw_label = self
                    .id2label
                    .get(&lid)
                    .cloned()
                    .unwrap_or_else(|| format!("LABEL_{lid}"));
                let (start, end) = row.offsets.get(s).copied().unwrap_or((0, 0));
                let attention = row.mask.get(s).copied().unwrap_or(0);
                toks.push(TokenPrediction {
                    raw_label,
                    score: sc,
                    start,
                    end,
                    token_text: row.tokens.get(s).cloned().unwrap_or_default(),
                    attention,
                });
            }
            let merged = merge_token_predictions(
                text,
                &toks,
                &self.config.label_to_entity,
                self.config.score_aggregation,
            );
            batch_preds.push(merged);
        }

        Ok(batch_preds)
    }

    pub fn config(&self) -> &NerConfig {
        &self.config
    }
}
