import { Modes } from "./types";
import { Stack } from "../types";

import { invoke } from "@tauri-apps/api/tauri";

import { dn } from "../utils";

import { default as newNoteMode } from "./newNoteMode";

import { createModal } from "./topBarBase";

export default createModal(
  {
    name: () => "New ...",
    options: ["Note", "Stack"],
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
      if (chosen == "Note") {
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
  },
);


/*
  activate: (_: Stack) => {
    state.selected.value = 0;
  },

            onKeyDown={(event) => {
              event.stopPropagation();
              switch (true) {
                case event.key === "Escape":
                  event.preventDefault();
                  modes.deactivate();
                  break;

                case (event.metaKey && event.key === "n"):
                  event.preventDefault();
                  modes.deactivate();
                  break;

                case (event.ctrlKey && event.key === "n") ||
                  event.key === "ArrowDown":
                  event.preventDefault();
                  selected.value += 1;
                  break;

                case event.ctrlKey && event.key === "p" ||
                  event.key === "ArrowUp":
                  event.preventDefault();
                  selected.value -= 1;
                  break;

                case event.key === "Enter":
                  event.preventDefault();
                  state.accept(stack, modes);
                  break;
              }
            }}
*/
