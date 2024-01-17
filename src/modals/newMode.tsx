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
        name: "Stack & Clip",
        keys: [
          <Icon name="IconShiftKey" />,
          <Icon name="IconAltKey" />,
          <Icon name="IconCommandKey" />,
          "N",
        ],
      },
      {
        name: "Stack",
        keys: [
          <Icon name="IconAltKey" />,
          <Icon name="IconCommandKey" />,
          "N",
        ],
      },
      {
        name: "Clip",
        keys: [
          <Icon name="IconShiftKey" />,
          <Icon name="IconCommandKey" />,
          "N",
        ],
      },
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
      // https://github.com/cablehead/stacks/issues/40

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

      if (chosen == "Stack & Clip") {
        (async () => {
          await invoke("store_new_stack", {
            name: dn(),
          });
          modes.activate(stack, newNoteMode);
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
