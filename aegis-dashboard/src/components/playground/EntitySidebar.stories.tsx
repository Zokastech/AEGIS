// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useState, useCallback } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { EntitySidebar } from "./EntitySidebar";
import type { PlaygroundEntity } from "./types";

const entities: PlaygroundEntity[] = [
  {
    id: "a",
    entityType: "PERSON",
    start: 0,
    end: 8,
    text: "Alice M.",
    score: 0.88,
    recognizer: "ner_fr_v2",
  },
  {
    id: "b",
    entityType: "PERSON",
    start: 20,
    end: 28,
    text: "Bob Martin",
    score: 0.82,
    recognizer: "ner_fr_v2",
  },
  {
    id: "c",
    entityType: "EMAIL",
    start: 40,
    end: 55,
    text: "bob@corp.test",
    score: 0.95,
    recognizer: "email_rfc",
  },
];

const meta = {
  title: "Playground/EntitySidebar",
  component: EntitySidebar,
  tags: ["autodocs"],
  parameters: {
    docs: {
      description: {
        component:
          "List grouped by type with score and false-positive toggle for the quality feedback loop.",
      },
    },
  },
  args: {
    entities,
  },
} satisfies Meta<typeof EntitySidebar>;

export default meta;
type Story = StoryObj<typeof meta>;

function InteractiveSidebar(props: { entities: PlaygroundEntity[] }) {
  const [fp, setFp] = useState<Set<string>>(() => new Set());
  const onFp = useCallback((id: string, v: boolean) => {
    setFp((prev) => {
      const next = new Set(prev);
      if (v) next.add(id);
      else next.delete(id);
      return next;
    });
  }, []);
  return <EntitySidebar entities={props.entities} falsePositiveIds={fp} onFalsePositiveChange={onFp} className="max-w-sm" />;
}

export const AvecFeedback: Story = {
  render: (args) => <InteractiveSidebar entities={args.entities} />,
};

export const LectureSeule: Story = {
  args: {
    entities,
  },
  render: (args) => <EntitySidebar {...args} className="max-w-sm" />,
};

export const Vide: Story = {
  args: { entities: [] },
};
