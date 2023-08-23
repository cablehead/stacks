import { invoke } from "@tauri-apps/api/tauri";

import { overlay, vars } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Stack } from "../types";

const state = (() => {
  return {
    accept_meta: async (_: Stack, modes: Modes) => {
      console.log("SAVE");
      const args = {
        // ...
      };
      await invoke("store_settings_save", args);
      modes.deactivate();
    },
  };
})();

export default {
  name: (_: Stack) => "Settings",
  hotKeys: (stack: Stack, modes: Modes) => [
    {
      name: "Save",
      keys: [
        <Icon name="IconCommandKey" />,
        <Icon name="IconReturnKey" />,
      ],
      onMouseDown: () => state.accept_meta(stack, modes),
      matchKeyEvent: (event: KeyboardEvent) =>
        event.metaKey && event.key === "Enter",
    },
    {
      name: "Discard",
      keys: ["ESC"],
      onMouseDown: () => {
          console.log("DISCARD");
          modes.deactivate()
      },
      matchKeyEvent: (event: KeyboardEvent) =>
        event.key === "Escape",
    },
  ],
  Modal: ({ stack, modes }: { stack: Stack; modes: Modes }) => {
    console.log(stack, modes);

    return (
      <div
        className={overlay}
        style={{
          position: "absolute",
          overflow: "auto",
          fontSize: "0.9rem",
          bottom: "2ch",
          right: "2ch",
          left: "2ch",
          top: "2ch",
          padding: "1ch 2ch 1ch 2ch",
          borderRadius: "0.5rem",
          zIndex: 1000,
        }}
      >
        <p>OpenAI API Access</p>
        <form onSubmit={() => {}}>
          <div
            style={{
              display: "flex",
              gap: "1ch",
              alignItems: "center",
              textAlign: "right",
              marginBottom: "0.25lh",
            }}
          >
            <label style={{ width: "15ch" }}>Access Token</label>
            <input
              type="text"
              placeholder="sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
              style={{
                flex: 1,
                maxWidth: "52ch",
                outline: "none",
                borderColor: vars.borderColor,
                borderWidth: "1px",
                borderStyle: "solid",
                borderRadius: "0.25rem",
              }}
              name="accessToken"
              value={""}
              onChange={() => {}}
            />
          </div>
          <div
            style={{
              display: "flex",
              gap: "1ch",
              alignItems: "center",
              textAlign: "right",
            }}
          >
            <label style={{ width: "15ch" }}>Preferred Model</label>
            <select
              name="selectedModel"
              value={"davinci"}
              onChange={() => {}}
              style={{
                flex: 1,
                maxWidth: "20ch",
                outline: "none",
                borderColor: vars.borderColor,
                borderWidth: "1px",
                borderStyle: "solid",
                borderRadius: "0.25rem",
                appearance: "none",
              }}
            >
              <option value="gpt-4">gpt-4</option>
              <option value="gpt-3.5-turbo">gpt-3.5-turbo</option>
              <option value="gpt-3.5-turbo-16k">gpt-3.5-turbo-16k</option>
            </select>
          </div>
        </form>
      </div>
    );
  },
};
