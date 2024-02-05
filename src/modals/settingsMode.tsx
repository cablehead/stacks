import { Signal, signal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";

import { invoke } from "@tauri-apps/api/tauri";

import { overlay, vars } from "../ui/app.css";
import { Icon, RenderKeys } from "../ui/icons";

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
  hotKeys: (stack: Stack, modes: Modes) => [],
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
          bottom: "0",
          right: "4ch",
          padding: "1ch 2ch 1ch 2ch",
          borderRadius: "0.5rem 0.5rem 0 0",
          zIndex: 1000,
        }}
      >
        <p>Activation Shortcut</p>
        <div
          style={{
            display: "flex",
            gap: "1ch",
            alignItems: "center",
            textAlign: "right",
            marginBottom: "0.25lh",
          }}
        >
          <RenderKeys
            keys={[
              <Icon name="IconShiftKey" />,
              <Icon name="IconAltKey" />,
              <Icon name="IconCommandKey" />,
            ]}
          />
        </div>
      </div>
    );
  },
};
