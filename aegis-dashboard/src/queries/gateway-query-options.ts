// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { queryOptions } from "@tanstack/react-query";
import {
  fetchAuditExportLines,
  fetchMetricsText,
  fetchPolicyDpiaMarkdown,
  listPolicies,
  listRecognizers,
} from "@/lib/api";
import { parsePrometheusText } from "@/lib/prometheus";
import { queryKeys } from "@/queries/keys";

const RECOGNIZERS_STALE_MS = 30_000;
const AUDIT_STALE_MS = 15_000;

export function gatewayMetricsQueryOptions(pollMs: number, credential: string | null) {
  return queryOptions({
    queryKey: queryKeys.gatewayMetrics(credential),
    queryFn: async () => parsePrometheusText(await fetchMetricsText(credential)),
    refetchInterval: pollMs,
    retry: 2,
    staleTime: Math.min(pollMs, 10_000),
  });
}

export function policiesListQueryOptions(credential: string | null) {
  return queryOptions({
    queryKey: queryKeys.policies(credential),
    queryFn: () => listPolicies(credential),
  });
}

export function policyDpiaQueryOptions(credential: string | null, policyName: string) {
  return queryOptions({
    queryKey: queryKeys.policyDpia(credential, policyName),
    queryFn: () => fetchPolicyDpiaMarkdown(policyName, credential),
    enabled: policyName.length > 0,
    retry: false,
  });
}

export function recognizersListQueryOptions(credential: string | null) {
  return queryOptions({
    queryKey: queryKeys.recognizers(credential),
    queryFn: () => listRecognizers(credential),
    staleTime: RECOGNIZERS_STALE_MS,
  });
}

export function auditExportQueryOptions(credential: string | null) {
  return queryOptions({
    queryKey: queryKeys.auditExport(credential),
    queryFn: () => fetchAuditExportLines(credential),
    staleTime: AUDIT_STALE_MS,
  });
}
