import { Modes } from "./types";
import { Stack } from "../types";

import { createModal } from "./topBarBase";

export default createModal(
  {
    name: () => "Filter by content type",

    options: [
      { name: "All" },
      { name: "Links" },
      { name: "Images" },
      { name: "Markdown" },
      { name: "Source Code" },
    ],

    rightOffset: (() => {
      const element = document.getElementById("filter-content-type");
      if (element && element.parentElement) {
        const elementRect = element.getBoundingClientRect();
        const parentRect = element.parentElement.getBoundingClientRect();
        return parentRect.right - elementRect.right;
      }

      return 300;
    }),

    accept: (stack: Stack, modes: Modes, chosen: string) => {
      stack.filter.content_type.value = chosen;
      modes.deactivate();
    },

    activate: (stack: Stack, state: any) => {
      const idx = state.options.indexOf(stack.filter.content_type.value);
      state.selected.value = idx == -1 ? 0 : idx;
    },
  },
);
