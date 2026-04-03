// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { PipelineLevelSelector, type PipelineLevel } from "./PipelineLevelSelector";

const meta = {
  title: "Playground/PipelineLevelSelector",
  component: PipelineLevelSelector,
  tags: ["autodocs"],
  parameters: {
    docs: {
      description: {
        component:
          "Visual L1 (regex) → L2 (context) → L3 (ML/NER) selection with indicative latency range.",
      },
    },
  },
} satisfies Meta<typeof PipelineLevelSelector>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Interactif: Story = {
  render: () => {
    const [v, setV] = useState<PipelineLevel>(2);
    return <PipelineLevelSelector value={v} onChange={setV} />;
  },
};

export const Niveau1: Story = {
  render: () => {
    const [v, setV] = useState<PipelineLevel>(1);
    return <PipelineLevelSelector value={v} onChange={setV} />;
  },
};
