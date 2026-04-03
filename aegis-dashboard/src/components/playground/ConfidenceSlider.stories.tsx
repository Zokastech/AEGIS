// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useState, useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { ConfidenceSlider } from "./ConfidenceSlider";
import type { PlaygroundEntity } from "./types";

const meta = {
  title: "Playground/ConfidenceSlider",
  component: ConfidenceSlider,
  tags: ["autodocs"],
  parameters: {
    docs: {
      description: {
        component:
          "Confidence threshold + Recharts curves: entities kept vs filtered at each threshold (axis sampling).",
      },
    },
  },
} satisfies Meta<typeof ConfidenceSlider>;

export default meta;
type Story = StoryObj<typeof meta>;

const mixedScores: PlaygroundEntity[] = [
  { id: "1", entityType: "X", start: 0, end: 1, score: 0.52 },
  { id: "2", entityType: "X", start: 0, end: 1, score: 0.61 },
  { id: "3", entityType: "X", start: 0, end: 1, score: 0.72 },
  { id: "4", entityType: "X", start: 0, end: 1, score: 0.81 },
  { id: "5", entityType: "X", start: 0, end: 1, score: 0.9 },
  { id: "6", entityType: "X", start: 0, end: 1, score: 0.96 },
];

export const Interactif: Story = {
  render: () => {
    const [t, setT] = useState(0.75);
    const scores = useMemo(() => mixedScores.map(({ score }) => ({ score })), []);
    return <ConfidenceSlider value={t} onChange={setT} entities={scores} />;
  },
};

export const PeuDEntites: Story = {
  render: () => {
    const [t, setT] = useState(0.8);
    const few = useMemo(
      () =>
        [
          { id: "a", entityType: "X", start: 0, end: 1, score: 0.6 },
          { id: "b", entityType: "X", start: 0, end: 1, score: 0.85 },
          { id: "c", entityType: "X", start: 0, end: 1, score: 0.92 },
        ] as PlaygroundEntity[],
      []
    );
    return <ConfidenceSlider value={t} onChange={setT} entities={few} />;
  },
};
