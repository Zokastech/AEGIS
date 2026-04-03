// AEGIS — zokastech.fr — Apache 2.0 / MIT

import type { Meta, StoryObj } from "@storybook/react";
import { TextHighlighter } from "./TextHighlighter";
import type { PlaygroundEntity } from "./types";

const text = "Contact : Jane Doe <jane@acme.fr> — IBAN FR76 3000 6000 0112 3456 7890 189";

const entities: PlaygroundEntity[] = [
  {
    id: "e1",
    entityType: "PERSON",
    start: 10,
    end: 18,
    text: "Jane Doe",
    score: 0.91,
    recognizer: "ner_fr_v2",
    decisionTrace: {
      pipelineLevel: 3,
      steps: [
        { name: "regex_pre", passed: true, detail: "aucun motif strict" },
        { name: "ner_span", passed: true, detail: "B-PER / I-PER" },
        { name: "score_gate", passed: true },
      ],
    },
  },
  {
    id: "e2",
    entityType: "EMAIL",
    start: 20,
    end: 32,
    text: "jane@acme.fr",
    score: 0.97,
    recognizer: "email_rfc",
    decisionTrace: {
      pipelineLevel: 1,
      steps: [
        { name: "email_pattern", passed: true },
        { name: "score_gate", passed: true },
      ],
    },
  },
  {
    id: "e3",
    entityType: "IBAN",
    start: 37,
    end: 73,
    text: "FR76 3000 6000 0112 3456 7890 189",
    score: 0.99,
    recognizer: "eu_iban",
    decisionTrace: {
      pipelineLevel: 1,
      steps: [
        { name: "iban_checksum", passed: true },
        { name: "country_fr", passed: true },
        { name: "score_gate", passed: true },
      ],
    },
  },
];

const meta = {
  title: "Playground/TextHighlighter",
  component: TextHighlighter,
  tags: ["autodocs"],
  parameters: {
    docs: {
      description: {
        component:
          "Semantic highlighting by entity type (PERSON blue, EMAIL green, IBAN orange) with tooltip: score, recognizer, decision trace.",
      },
    },
  },
  args: {
    text,
    entities,
  },
} satisfies Meta<typeof TextHighlighter>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const SansTrace: Story = {
  args: {
    entities: entities.map((e) => {
      const copy = { ...e };
      delete copy.decisionTrace;
      return copy;
    }),
  },
};
