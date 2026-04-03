// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useState, useMemo, useCallback, useEffect } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { analyzeText, anonymizeText } from "@/lib/api";
import {
  analyzeResultUsesMockEngine,
  anonymizedTextFromResponse,
  buildAnalysisConfigJson,
  extractEntitiesFromAnalyzeResult,
} from "@/lib/playground/analysis";
import {
  buildPlaygroundAnonymizeConfigJson,
  DEFAULT_PLAYGROUND_ANONYMIZE_OPERATOR,
  PLAYGROUND_SAMPLE_TEXT,
  type PlaygroundAnonymizeOperatorId,
} from "@/lib/playground/anonymizeOperators";
import { policiesListQueryOptions } from "@/queries/gateway-query-options";
import { useAuthCredential } from "@/hooks/useAuthCredential";
import type { PlaygroundEntity, PipelineLevel, AnonymizationLink } from "@/components/playground";

export function usePlaygroundPage() {
  const credential = useAuthCredential();
  const policiesQuery = useQuery(policiesListQueryOptions(credential));

  const [text, setText] = useState("");
  const [lang, setLang] = useState("fr");
  const [pipeline, setPipeline] = useState<PipelineLevel>(2);
  /** Matches engine default (0.5). Above ~0.75, L1 filter drops many spans before L2/NER → often only emails remain. */
  const [threshold, setThreshold] = useState(0.5);
  const [policy, setPolicy] = useState("");
  const [anonOperator, setAnonOperator] = useState<PlaygroundAnonymizeOperatorId>(DEFAULT_PLAYGROUND_ANONYMIZE_OPERATOR);
  const [entities, setEntities] = useState<PlaygroundEntity[]>([]);
  const [anonymizedBlock, setAnonymizedBlock] = useState("");
  const [anonLinks, setAnonLinks] = useState<AnonymizationLink[] | null>(null);
  const [falsePositiveIds, setFalsePositiveIds] = useState<Set<string>>(() => new Set());
  const [analyzeError, setAnalyzeError] = useState("");
  const [anonymError, setAnonymError] = useState("");
  const [gatewayUsesMockEngine, setGatewayUsesMockEngine] = useState(false);

  const configJson = useMemo(() => buildAnalysisConfigJson(lang, pipeline, threshold), [lang, pipeline, threshold]);
  const anonymizeConfigJson = useMemo(
    () => buildPlaygroundAnonymizeConfigJson(lang, pipeline, threshold, anonOperator),
    [lang, pipeline, threshold, anonOperator]
  );

  /** Anonymization config changed: previous anonymized text no longer matches — avoids confusion. */
  useEffect(() => {
    setAnonymizedBlock("");
    setAnonLinks(null);
  }, [anonOperator, lang, pipeline, threshold, policy]);

  const insertSampleText = useCallback(() => {
    const sample = PLAYGROUND_SAMPLE_TEXT[lang] ?? PLAYGROUND_SAMPLE_TEXT.en;
    setText(sample);
    setAnonymizedBlock("");
    setAnonLinks(null);
    setEntities([]);
    setAnalyzeError("");
    setAnonymError("");
  }, [lang]);

  const onFalsePositiveChange = useCallback((id: string, v: boolean) => {
    setFalsePositiveIds((prev) => {
      const next = new Set(prev);
      if (v) next.add(id);
      else next.delete(id);
      return next;
    });
  }, []);

  const analyzeMut = useMutation({
    mutationFn: async () =>
      analyzeText(text, { analysisConfigJson: configJson, policy: policy || undefined }, credential),
    onMutate: () => {
      setGatewayUsesMockEngine(false);
    },
    onSuccess: (data) => {
      setAnalyzeError("");
      setEntities(extractEntitiesFromAnalyzeResult(data));
      setGatewayUsesMockEngine(analyzeResultUsesMockEngine(data));
    },
    onError: (e: Error) => {
      setAnalyzeError(e.message || String(e));
      setEntities([]);
      setGatewayUsesMockEngine(false);
    },
  });

  const anonymMut = useMutation({
    mutationFn: async () =>
      anonymizeText(text, anonymizeConfigJson, { policy: policy || undefined }, credential),
    onSuccess: (data) => {
      setAnonymError("");
      setAnonLinks(null);
      setAnonymizedBlock(anonymizedTextFromResponse(data));
    },
    onError: (e: Error) => {
      setAnonymError(e.message || String(e));
      setAnonLinks(null);
    },
  });

  const onTextChange = useCallback((v: string) => {
    setText(v);
    setAnonLinks(null);
  }, []);

  const scoreSamples = useMemo(() => entities.map((e) => ({ score: e.score })), [entities]);
  const policyNames = policiesQuery.data?.policies ?? [];

  return {
    policiesQuery,
    text,
    lang,
    pipeline,
    threshold,
    policy,
    anonOperator,
    entities,
    anonymizedBlock,
    anonLinks,
    falsePositiveIds,
    analyzeError,
    anonymError,
    gatewayUsesMockEngine,
    analyzeMut,
    anonymMut,
    setLang,
    setPipeline,
    setThreshold,
    setPolicy,
    setAnonOperator,
    onTextChange,
    onFalsePositiveChange,
    scoreSamples,
    policyNames,
    configJson,
    anonymizeConfigJson,
    insertSampleText,
  };
}
