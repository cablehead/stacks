import createActionMode from "./actionBaseMode";
import { Signal} from "@preact/signals";
import { Stack } from "../types";
import { Modes } from "./types";

const setContentType = createActionMode(
  (_stack: Stack) => "Set content type",
  (_stack: Stack, availOptions: Signal<string[]>) => {
    availOptions.value = ["Markdown", "Rust"];
  },
  (_stack: Stack, _modes: Modes, selected: string) => {
    console.log(`Content type set to: ${selected}`);
  }
);

export default setContentType;
