// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useCallback, useEffect, useRef, useState, type CSSProperties } from "react";
import "./App.css";

/** Documentation intégrée au site ZokasTech ; miroir GitHub Pages si besoin. */
const DOCS_URL = "https://zokastech.fr/aegis/docs";
const GITHUB_REPO = "https://github.com/zokastech/aegis";
const API_GITHUB = "https://api.github.com/repos/zokastech/aegis";

/** Hook : ajoute `in-view` quand l’élément entre dans le viewport (animation scroll). */
function useReveal<T extends HTMLElement>(options?: IntersectionObserverInit) {
  const ref = useRef<T | null>(null);
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const obs = new IntersectionObserver(
      ([e]) => {
        if (e?.isIntersecting) {
          setVisible(true);
          obs.disconnect();
        }
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.08, ...options }
    );
    obs.observe(el);
    return () => obs.disconnect();
  }, []);
  return { ref, visible };
}

const PYTHON_SNIPPET = `import requests, json
r = requests.post("http://127.0.0.1:8080/v1/analyze",
    json={"text": "Contact: alice@example.com"})
print(json.dumps(r.json(), indent=2)[:400])`;

const TERMINAL_LINES = [
  { prompt: true, text: 'aegis analyze "Patient: bob@clinic.eu — NIR 1 85 08 75 123 456 78"' },
  { prompt: false, text: "entities: EMAIL, NATIONAL_ID (synthetic demo)" },
  { prompt: false, text: "latency: ~1.2 ms (L1) · pipeline: L1+L2+L3 ready" },
];

/** Carte UE stylisée : positions % pour des pastilles pays. */
const EU_COUNTRIES = [
  { id: "FR", cx: 46, cy: 38, r: 3.2 },
  { id: "DE", cx: 52, cy: 32, r: 3.4 },
  { id: "ES", cx: 38, cy: 48, r: 3.2 },
  { id: "IT", cx: 54, cy: 48, r: 3 },
  { id: "NL", cx: 48, cy: 28, r: 2.2 },
  { id: "BE", cx: 47, cy: 30, r: 1.8 },
  { id: "AT", cx: 54, cy: 36, r: 2.2 },
  { id: "PL", cx: 58, cy: 30, r: 3.2 },
  { id: "PT", cx: 32, cy: 48, r: 2.4 },
  { id: "SE", cx: 54, cy: 18, r: 2.8 },
  { id: "IE", cx: 36, cy: 28, r: 2 },
  { id: "GR", cx: 58, cy: 54, r: 2.4 },
  { id: "RO", cx: 62, cy: 42, r: 2.6 },
  { id: "CZ", cx: 56, cy: 34, r: 2 },
  { id: "DK", cx: 50, cy: 24, r: 1.8 },
];

function ShieldIcon({ className, style }: { className?: string; style?: CSSProperties }) {
  return (
    <svg
      className={className}
      style={style}
      viewBox="0 0 64 72"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden
    >
      <defs>
        <linearGradient id="sg" x1="8" y1="4" x2="56" y2="68" gradientUnits="userSpaceOnUse">
          <stop stopColor="#ff8a00" />
          <stop offset="0.5" stopColor="#e52e71" />
          <stop offset="1" stopColor="#4361ee" />
        </linearGradient>
        <linearGradient id="sg2" x1="20" y1="20" x2="48" y2="52" gradientUnits="userSpaceOnUse">
          <stop stopColor="#ff8a00" stopOpacity="0.2" />
          <stop offset="1" stopColor="#4361ee" stopOpacity="0" />
        </linearGradient>
      </defs>
      <path
        d="M32 4L8 16v20c0 14 10 26 24 32 14-6 24-18 24-32V16L32 4z"
        fill="url(#sg)"
        stroke="#e52e71"
        strokeWidth="1.2"
        strokeOpacity="0.45"
      />
      <path d="M32 14L18 22v12c0 8 6 15 14 18 8-3 14-10 14-18V22L32 14z" fill="url(#sg2)" />
      <path
        d="M28 34h8v-8h-8v8zm0 4h8v8h-8v-8z"
        fill="#ffffff"
        fillOpacity="0.35"
      />
    </svg>
  );
}

function EuMap({ lit }: { lit: boolean }) {
  return (
    <svg className="eu-map-svg" viewBox="0 0 100 72" role="img" aria-label="Carte Europe — pays supportés">
      <title>Europe — couverture recognizers AEGIS</title>
      <path
        d="M12 38 Q25 22 48 20 Q72 18 88 32 Q92 48 78 58 Q58 68 38 64 Q18 58 12 38 Z"
        fill="#e2e8f0"
        stroke="rgba(67,97,238,0.2)"
        strokeWidth="0.5"
      />
      {EU_COUNTRIES.map((c) => (
        <circle
          key={c.id}
          className={`country-dot ${lit ? "lit" : ""}`}
          cx={c.cx}
          cy={c.cy}
          r={c.r}
        />
      ))}
    </svg>
  );
}

function TerminalDemo() {
  const [text, setText] = useState("");
  const full = TERMINAL_LINES.map((l) => (l.prompt ? "$ " : "") + l.text).join("\n");

  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) {
      setText(full);
      return;
    }
    let i = 0;
    const id = window.setInterval(() => {
      i += 1;
      setText(full.slice(0, i));
      if (i >= full.length) clearInterval(id);
    }, 22);
    return () => clearInterval(id);
  }, [full]);

  return (
    <div className="code-panel" id="terminal-demo">
      <header>
        <div className="code-dots" aria-hidden>
          <span />
          <span />
          <span />
        </div>
        <span>aegis-cli · synthetic</span>
      </header>
      <div className="terminal-body">
        <pre style={{ margin: 0, whiteSpace: "pre-wrap", wordBreak: "break-word" }}>
          <span className="out">{text}</span>
          {text.length < full.length && <span className="prompt">▌</span>}
        </pre>
      </div>
    </div>
  );
}

export default function App() {
  const whyRef = useReveal<HTMLDivElement>();
  const compareRef = useReveal<HTMLDivElement>();
  const archRef = useReveal<HTMLDivElement>();
  const euRef = useReveal<HTMLDivElement>();
  const [stars, setStars] = useState<number | null>(null);
  const [copied, setCopied] = useState(false);
  const [euLit, setEuLit] = useState(false);

  useEffect(() => {
    fetch(API_GITHUB)
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => (d && typeof d.stargazers_count === "number" ? setStars(d.stargazers_count) : setStars(null)))
      .catch(() => setStars(null));
  }, []);

  useEffect(() => {
    if (!euRef.visible) return;
    const t = window.setTimeout(() => setEuLit(true), 200);
    return () => clearTimeout(t);
  }, [euRef.visible]);

  const copySnippet = useCallback(() => {
    void navigator.clipboard.writeText(PYTHON_SNIPPET);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);
  }, []);

  return (
    <div className="app">
      <header className="site-header">
        <a className="brand-mini" href="#top" id="top">
          <ShieldIcon className="shield-svg" style={{ width: 32, height: "auto" }} />
          AEGIS
        </a>
        <nav className="nav-links" aria-label="Principal">
          <a href="#why">Why AEGIS</a>
          <a href="#architecture">Architecture</a>
          <a href="#europe">Europe</a>
          <a className="nav-hide-sm" href="#code">
            Code
          </a>
          <a href={DOCS_URL} rel="noopener noreferrer">
            Docs
          </a>
          <a href={GITHUB_REPO} rel="noopener noreferrer">
            GitHub
          </a>
        </nav>
      </header>

      <section className="hero" aria-labelledby="hero-title">
        <div className="hero-visual">
          <div className="shield-wrap">
            <ShieldIcon className="shield-svg" />
          </div>
        </div>
        <h1 id="hero-title" className="hero-logo-text">
          AEGIS
        </h1>
        <p className="hero-tagline">Le bouclier européen pour vos données sensibles</p>
        <p className="hero-sub">
          Détection et anonymisation de PII open source, haute performance, RGPD-native — par{" "}
          <a href="https://zokastech.fr" rel="noopener noreferrer">
            zokastech.fr
          </a>
        </p>
        <div className="hero-cta">
          <a className="btn btn-primary" href={DOCS_URL} rel="noopener noreferrer">
            Get Started
          </a>
          <a className="btn btn-ghost" href={GITHUB_REPO} rel="noopener noreferrer">
            GitHub
          </a>
          <a className="btn btn-accent" href="#code">
            Try Demo
          </a>
        </div>
      </section>

      <section id="why" aria-labelledby="why-title">
        <h2 id="why-title" className="section-title">
          Why AEGIS?
        </h2>
        <p className="section-lead">
          Une alternative européenne aux stacks lourds : moteur Rust, passerelle durcie, politiques
          déclaratives — sans sacrifier la précision.
        </p>

        <div ref={whyRef.ref} className="why-grid">
          {[
            {
              stat: "50×",
              sub: "ordre de grandeur",
              title: "Plus rapide",
              body: "Sur charges analytiques typiques vs interpréteur Python seul — à valider sur votre hardware.",
              gold: false,
            },
            {
              stat: "95%",
              sub: "objectif rappel",
              title: "Pipeline multi-niveaux",
              body: "Regex + contexte + NER ONNX pour maximiser le rappel tout en contrôlant la latence.",
              gold: true,
            },
            {
              stat: "30+",
              sub: "variantes & contextes",
              title: "Europe d’abord",
              body: "IBAN, TVA multi-pays, identifiants nationaux, lexiques FR/EN/DE/ES/IT/NL/PT/PL…",
              gold: false,
            },
            {
              stat: "TLS",
              sub: "RBAC · limites",
              title: "Secure by default",
              body: "Passerelle avec en-têtes OWASP, clés API, rate limiting — pas d’analyseur nu sur le réseau.",
              gold: false,
            },
          ].map((c, i) => (
            <article key={i} className={`why-card ${whyRef.visible ? "in-view" : ""}`}>
              <p className={`why-card-stat ${c.gold ? "accent" : ""}`}>
                {c.stat}
                <small style={{ fontSize: "0.45em", opacity: 0.85, display: "block" }}>{c.sub}</small>
              </p>
              <h3>{c.title}</h3>
              <p>{c.body}</p>
            </article>
          ))}
        </div>

        <div ref={compareRef.ref} className={`compare-wrap ${compareRef.visible ? "in-view" : ""}`}>
          <table className="compare-table">
            <thead>
              <tr>
                <th>Capacité</th>
                <th>Microsoft Presidio</th>
                <th className="aegis-col">AEGIS</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>Moteur d’analyse</td>
                <td>Python</td>
                <td className="aegis-col">
                  <span className="win">Rust</span> (latence prévisible)
                </td>
              </tr>
              <tr>
                <td>Focus UE</td>
                <td>Générique + extensions</td>
                <td className="aegis-col">
                  <span className="win">Natif</span> (IBAN, TVA, SIREN…)
                </td>
              </tr>
              <tr>
                <td>API production</td>
                <td>Souvent à composer</td>
                <td className="aegis-col">
                  <span className="win">Gateway</span> durcie (opt.)
                </td>
              </tr>
              <tr>
                <td>Licence</td>
                <td>MIT</td>
                <td className="aegis-col">
                  <span className="win">Apache 2 + MIT</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section id="architecture" className="arch-section" aria-labelledby="arch-title">
        <h2 id="arch-title" className="section-title">
          Architecture
        </h2>
        <p className="section-lead">
          Pipeline en trois niveaux : détection rapide, scoring contextuel, NER optionnel — fusion des scores
          et seuils configurables.
        </p>
        <div
          ref={archRef.ref}
          className={`pipeline ${archRef.visible ? "in-view-partial" : ""}`}
          role="list"
        >
          {[
            {
              level: "Niveau 1",
              title: "Regex & validateurs",
              desc: "E-mail, téléphone, IBAN, cartes, URL, dates — avec lexiques et deny-lists.",
            },
            {
              level: "Niveau 2",
              title: "Contexte & quasi-ID",
              desc: "Mots-clés métier, combinaisons DATE+LOCATION, réduction des faux positifs.",
            },
            {
              level: "Niveau 3",
              title: "NER ONNX",
              desc: "Modèle embarqué pour entités difficiles ; budgets temps et court-circuit L1.",
            },
          ].flatMap((s, i) => [
            <div key={`step-${i}`} className={`pipe-step ${archRef.visible ? "in-view" : ""}`} role="listitem">
              <div className="pipe-level">{s.level}</div>
              <h3>{s.title}</h3>
              <p>{s.desc}</p>
            </div>,
            ...(i < 2
              ? [
                  <div key={`arr-${i}`} className="pipe-arrow" aria-hidden="true">
                    →
                  </div>,
                ]
              : []),
          ])}
        </div>
      </section>

      <section id="europe" aria-labelledby="eu-title">
        <h2 id="eu-title" className="section-title">
          Europe First
        </h2>
        <p className="section-lead">
          Recognizers et politiques pensés pour les formats européens — extensible à vos pays cibles.
        </p>
        <div ref={euRef.ref} className="eu-block">
          <EuMap lit={euLit && euRef.visible} />
          <div>
            <h3 className="section-title" style={{ fontSize: "1.2rem", textAlign: "left", marginBottom: "1rem" }}>
              Formats nationaux & financiers (extraits)
            </h3>
            <ul className="eu-list">
              <li>
                <strong>IBAN</strong> — validation mod-97, contexte multilingue
              </li>
              <li>
                <strong>France</strong> — SIREN, SIRET, NIR (synthèse / checksums)
              </li>
              <li>
                <strong>TVA intracommunautaire</strong> — FR, DE, IT, ES, …
              </li>
              <li>
                <strong>Identités nationales UE</strong> — pack par pays (Rust) filtrable par langue
              </li>
              <li>
                <strong>RGPD</strong> — politiques YAML, DPIA, minimisation (voir docs gateway)
              </li>
            </ul>
          </div>
        </div>
      </section>

      <section id="code" className="code-section" aria-labelledby="code-title">
        <h2 id="code-title" className="section-title">
          Intégration en quelques lignes
        </h2>
        <p className="section-lead">
          Appel HTTP vers la passerelle — ou SDK Python / FFI selon votre stack. Données d’exemple
          synthétiques.
        </p>
        <div className="code-grid">
          <div className="code-panel">
            <header>
              <div className="code-dots" aria-hidden>
                <span />
                <span />
                <span />
              </div>
              <span>quick_scan.py</span>
              <button type="button" className="copy-btn" onClick={copySnippet}>
                {copied ? "Copié ✓" : "Copier"}
              </button>
            </header>
            <pre className="code-snippet">
              <code>
                <span className="kw">import</span> requests, json{"\n"}
                r = requests.<span className="fn">post</span>(
                <span className="str">&quot;http://127.0.0.1:8080/v1/analyze&quot;</span>,{"\n"}
                {"    "}json={"{"}
                <span className="str">&quot;text&quot;</span>: <span className="str">&quot;Contact: alice@example.com&quot;</span>
                {"}"}){"\n"}
                <span className="fn">print</span>(json.dumps(r.json(), indent=<span className="str">2</span>)[:<span className="str">400</span>])
              </code>
            </pre>
          </div>
          <TerminalDemo />
        </div>
      </section>

      <section className="community" id="community" aria-labelledby="comm-title">
        <h2 id="comm-title" className="section-title">
          Sponsors &amp; communauté
        </h2>
        <p className="built">
          Built by{" "}
          <a href="https://zokastech.fr" rel="noopener noreferrer">
            zokastech.fr
          </a>
        </p>
        <div className="stars-pill" aria-live="polite">
          <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden>
            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
          </svg>
          <span>
            GitHub :{" "}
            <a href={GITHUB_REPO} rel="noopener noreferrer">
              {stars !== null ? `${stars.toLocaleString("fr-FR")} stars` : "zokastech/aegis"}
            </a>
          </span>
        </div>
        <p className="section-lead" style={{ marginTop: "1.5rem" }}>
          PR, issues et retours d’expérience sont les bienvenus — consultez les guidelines du dépôt.
        </p>
        <a className="btn btn-primary" href={`${GITHUB_REPO}/blob/main/CONTRIBUTING.md`} rel="noopener noreferrer">
          Contribuer
        </a>
      </section>

      <footer className="site-footer">
        <div className="footer-inner">
          <nav className="footer-links" aria-label="Liens de pied de page">
            <a href={DOCS_URL} rel="noopener noreferrer">
              Documentation
            </a>
            <a href={GITHUB_REPO} rel="noopener noreferrer">
              GitHub
            </a>
            <a href="https://github.com/zokastech/aegis/discussions" rel="noopener noreferrer">
              Discord / Discussions
            </a>
            <a href="https://twitter.com/zokastech" rel="noopener noreferrer">
              Twitter
            </a>
          </nav>
          <span
            className="badge-rgpd"
            title="Fonctionnalités et documentation orientées conformité RGPD — validation juridique auprès de votre DPO."
          >
            RGPD Compliant
          </span>
          <p className="footer-tagline">Made with 🛡️ in Europe by zokastech.fr · Apache 2.0 / MIT</p>
        </div>
      </footer>
    </div>
  );
}
