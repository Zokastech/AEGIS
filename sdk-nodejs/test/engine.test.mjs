// AEGIS — zokastech.fr — Apache 2.0 / MIT
// Tests du SDK Node.js (node:test, aucun framework requis).
// Prérequis : `npm run build` (addon natif + dist/).

import { test } from 'node:test'
import assert from 'node:assert/strict'
import { Readable } from 'node:stream'
import { AegisEngine, nativeAddonVersion } from '../dist/index.js'

const KEY_HEX = '000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f'
const PII = 'Écrire à jean.dupont@example.fr avant lundi.'

test('version du module natif', () => {
  assert.match(nativeAddonVersion(), /\d+\.\d+\.\d+/)
})

test('analyze détecte un e-mail', async () => {
  const eng = new AegisEngine()
  const ents = await eng.analyze(PII, { scoreThreshold: 0.2 })
  assert.ok(ents.some((e) => e.entityType === 'EMAIL'), JSON.stringify(ents))
})

test('analyzeFull et analyzeBatch', async () => {
  const eng = new AegisEngine()
  const full = await eng.analyzeFull(PII)
  assert.ok(Array.isArray(full.entities))
  const batch = await eng.analyzeBatch([PII, 'rien ici'])
  assert.equal(batch.length, 2)
})

for (const op of [
  { operator_type: 'redact', params: {} },
  { operator_type: 'hash', params: { salt: 's' } },
  { operator_type: 'encrypt', params: { key_hex: KEY_HEX } },
]) {
  test(`anonymize (${op.operator_type}) ne laisse pas fuir l’e-mail`, async () => {
    const eng = new AegisEngine()
    const r = await eng.anonymize(PII, { EMAIL: op })
    assert.ok(!r.text.includes('jean.dupont@example.fr'), r.text)
  })
}

test('anonymize accepte une chaîne JSON et rejette un JSON invalide', async () => {
  const eng = new AegisEngine()
  const r = await eng.anonymize(PII, JSON.stringify({ EMAIL: { operator_type: 'redact', params: {} } }))
  assert.ok(!r.text.includes('@example.fr'))
  await assert.rejects(eng.anonymize(PII, '{not json'))
})

test('config_path inexistant → erreur', () => {
  assert.throws(() => new AegisEngine('/nonexistent/aegis.yaml'))
})

test('analyzeStream par ligne', async () => {
  const eng = new AegisEngine()
  const stream = Readable.from([`a@b.co\nrien\n`, `c@d.fr\n`])
  let n = 0
  for await (const chunk of eng.analyzeStream(stream, { byLine: true })) {
    assert.ok(chunk)
    n++
  }
  assert.ok(n >= 2, `chunks=${n}`)
})

test('entrées volumineuses et unicode', async () => {
  const eng = new AegisEngine()
  const big = 'Élodie ✉ a@b.co 🚀 '.repeat(4000)
  const ents = await eng.analyze(big)
  assert.ok(Array.isArray(ents))
})

// Regression: options were sent in snake_case and silently ignored by the native layer.
test('les options entities/scoreThreshold sont appliquées', async () => {
  const eng = new AegisEngine()
  const text = 'mail a@b.co tel +33 6 12 34 56 78'
  const onlyPhone = await eng.analyze(text, { entities: ['PHONE'], scoreThreshold: 0.1 })
  assert.ok(onlyPhone.every((e) => e.entityType === 'PHONE'), JSON.stringify(onlyPhone))
  const strict = await eng.analyze(text, { scoreThreshold: 0.999 })
  const loose = await eng.analyze(text, { scoreThreshold: 0.1 })
  assert.ok(strict.length <= loose.length)
})

test('les transformations exposent entityType', async () => {
  const eng = new AegisEngine()
  const r = await eng.anonymize(PII, { EMAIL: { operator_type: 'redact', params: {} } })
  assert.ok(r.transformations.length > 0)
  assert.ok(r.transformations.every((t) => typeof t.entityType === 'string' && t.entityType.length > 0))
})
