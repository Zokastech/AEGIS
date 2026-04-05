// AEGIS — zokastech.fr — Apache 2.0 / MIT

import type { Meta, StoryObj } from "@storybook/react";
import { AnonymizationPreview } from "./AnonymizationPreview";

const originalText = "Mme Jane Doe, email jane@acme.fr, tel +33 6 12 34 56 78";
const anonymizedText = "Mme [PERSON], email [EMAIL], tel [PHONE]";

const meta = {
  title: "Playground/AnonymizationPreview",
  component: AnonymizationPreview,
  tags: ["autodocs"],
  parameters: {
    docs: {
      description: {
        component:
          "Side-by-side original / anonymized with SVG link curves between matching segments (same `id` on each side).",
      },
    },
  },
  args: {
    originalText,
    anonymizedText,
    links: [
      {
        id: "l1",
        entityType: "PERSON",
        originalStart: 4,
        originalEnd: 12,
        anonymizedStart: 4,
        anonymizedEnd: 12,
      },
      {
        id: "l2",
        entityType: "EMAIL",
        originalStart: 20,
        originalEnd: 32,
        anonymizedStart: 20,
        anonymizedEnd: 27,
      },
      {
        id: "l3",
        entityType: "PHONE",
        originalStart: 38,
        originalEnd: 56,
        anonymizedStart: 33,
        anonymizedEnd: 40,
      },
    ],
    minHeight: 120,
  },
} satisfies Meta<typeof AnonymizationPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
