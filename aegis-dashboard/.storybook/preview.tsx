// AEGIS — zokastech.fr — Apache 2.0 / MIT

import type { Preview } from "@storybook/react";
import "../src/index.css";

const preview: Preview = {
  parameters: {
    layout: "padded",
    backgrounds: {
      default: "aegis",
      values: [{ name: "aegis", value: "hsl(222 47% 6%)" }],
    },
    docs: {
      toc: true,
    },
  },
  decorators: [
    (Story) => (
      <div className="dark min-h-[200px] w-full max-w-5xl bg-background p-4 text-foreground antialiased">
        <Story />
      </div>
    ),
  ],
};

export default preview;
