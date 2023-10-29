import { Modes } from "./types";
import { Stack } from "../types";

import { createModal } from "./topBarBase";

export default createModal(
  {
    name: () => "Filter by content type",

    options: ["All", "Links", "Images", "Markdown"],

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
  },
);
