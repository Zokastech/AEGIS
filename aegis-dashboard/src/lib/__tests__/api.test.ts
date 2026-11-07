// AEGIS — zokastech.fr — Apache 2.0 / MIT
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { apiFetch, apiUrl, credentialAllowedFor } from "@/lib/api";

describe("apiUrl", () => {
  it("garde les chemins relatifs quand aucune base n’est configurée", () => {
    expect(apiUrl("/v1/analyze")).toBe("/v1/analyze");
    expect(apiUrl("v1/analyze")).toBe("/v1/analyze");
  });
});

describe("credentialAllowedFor", () => {
  it("autorise les chemins relatifs", () => {
    expect(credentialAllowedFor("/v1/analyze")).toBe(true);
  });
  it("refuse les hôtes tiers et les URL protocol-relative", () => {
    expect(credentialAllowedFor("https://evil.example/v1/analyze")).toBe(false);
    expect(credentialAllowedFor("//evil.example/x")).toBe(false);
  });
});

describe("apiFetch", () => {
  const fetchMock = vi.fn(async () => new Response("{}", { status: 200 }));
  beforeEach(() => {
    fetchMock.mockClear();
    vi.stubGlobal("fetch", fetchMock);
  });
  afterEach(() => vi.unstubAllGlobals());

  const sentHeaders = (): Headers => (fetchMock.mock.calls[0] as unknown as [string, RequestInit])[1].headers as Headers;

  it("envoie une clé API dans X-API-Key", async () => {
    await apiFetch("/v1/analyze", { method: "POST", body: "{}" }, "aegis_key.with.dots");
    expect(sentHeaders().get("X-API-Key")).toBe("aegis_key.with.dots");
    expect(sentHeaders().get("Authorization")).toBeNull();
    expect(sentHeaders().get("Content-Type")).toBe("application/json");
  });

  it("envoie un JWT en Bearer", async () => {
    await apiFetch("/v1/analyze", {}, "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ4In0.sig");
    expect(sentHeaders().get("Authorization")).toMatch(/^Bearer eyJ/);
    expect(sentHeaders().get("X-API-Key")).toBeNull();
  });

  it("n’envoie jamais le secret vers une origine étrangère", async () => {
    await apiFetch("https://evil.example/steal", {}, "aegis_secret");
    expect(sentHeaders().get("X-API-Key")).toBeNull();
    expect(sentHeaders().get("Authorization")).toBeNull();
  });

  it("sans credential, aucun en-tête d’authentification", async () => {
    await apiFetch("/v1/health", {}, "   ");
    expect(sentHeaders().get("X-API-Key")).toBeNull();
  });
});
