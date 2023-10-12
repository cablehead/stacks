import { Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { overlay, vars } from "../ui/app.css";
import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Stack } from "../types";

const state = (() => {
  const form: Signal<HTMLFormElement | undefined> = signal(undefined);
  return {
    form,
    accept_meta: async (_: Stack, modes: Modes) => {
      if (!form.value) {
        console.error("Form is not available", form.value);
        return;
      }
      const formData = new FormData(form.value);
      const settings = Object.fromEntries(formData.entries());
      console.log("save", settings);
      if (settings.openai_access_token === "") return;
      await invoke("store_settings_save", { settings: settings });
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
      onMouseDown: () => modes.deactivate(),
      matchKeyEvent: (event: KeyboardEvent) => event.key === "Escape",
    },
  ],
  Modal: ({}: { stack: Stack; modes: Modes }) => {
    const formRef = useRef<HTMLFormElement>(null);

    useEffect(() => {
      if (formRef.current != null) {
        (formRef.current.elements[0] as HTMLElement).focus();
        state.form.value = formRef.current;
        invoke<Record<string, string>>("store_settings_get", {}).then(
          (settings: Record<string, string>) => {
              console.log("settings", settings);
            if (formRef.current) {
              for (const key in settings) {
                (formRef.current.elements.namedItem(key) as HTMLInputElement)
                  .value = settings[key];
              }
            }
          },
        );
      }
    }, []);

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
        <form ref={formRef}>
          <p>OpenAI API access</p>
          <div
            style={{
              display: "flex",
              gap: "1ch",
              alignItems: "center",
              textAlign: "right",
              marginBottom: "0.25lh",
            }}
          >
            <label style={{ width: "15ch" }}>Access token</label>
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
              name="openai_access_token"
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
            <label style={{ width: "15ch" }}>Preferred model</label>
            <select
              name="openai_selected_model"
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

          <div
            style={{
              margin: "2ch 0",
              borderTop: "1px solid",
              borderColor: vars.borderColor,
            }}
          />

          <p>cross.stream garden</p>
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
              placeholder="1234"
              style={{
                flex: 1,
                maxWidth: "52ch",
                outline: "none",
                borderColor: vars.borderColor,
                borderWidth: "1px",
                borderStyle: "solid",
                borderRadius: "0.25rem",
              }}
              name="cross_stream_access_token"
              value={""}
              onChange={() => {}}
            />
          </div>
        </form>
      </div>
    );
  },
};
