import { Modes } from "./types";
import { Stack } from "../types";
import { Icon } from "../ui/icons";

import { invoke } from "@tauri-apps/api/tauri";

import { dn } from "../utils";

import { default as newNoteMode } from "./newNoteMode";

import { createModal } from "./topBarBase";

export default createModal(
  {
    name: () => "New ...",

    options: [
      {
        name: "Clip",
        keys: [
          <Icon name="IconCommandKey" />,
          <Icon name="IconShiftKey" />,
          "N",
        ],
      },
      { name: "Stack" },
    ],

    rightOffset: (() => {
      const element = document.getElementById("trigger-new");
      if (element && element.parentElement) {
        const elementRect = element.getBoundingClientRect();
        const parentRect = element.parentElement.getBoundingClientRect();
        return parentRect.right - elementRect.right - 5;
      }
      return 300;
    }),

    accept: (stack: Stack, modes: Modes, chosen: string) => {
      if (chosen == "Clip") {
        modes.activate(stack, newNoteMode);
        return;
      }

      if (chosen == "Stack") {
        (async () => {
          await invoke("store_new_stack", {
            name: dn(),
          });
          modes.deactivate();
        })();
        return;
      }

      modes.deactivate();
    },

    activate: (_: Stack, state: any) => {
      state.selected.value = 0;
    },
  },
);
