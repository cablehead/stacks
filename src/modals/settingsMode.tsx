import { Signal, signal } from "@preact/signals";

import { invoke } from "@tauri-apps/api/tauri";

import {
  border,
  enchantedForestGradient,
  enchantedForestGradientActive,
  overlay,
} from "../ui/app.css";

import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Stack } from "../types";

const saved: Signal<Record<string, boolean> | null> = signal(null);
(async () => {
  saved.value = await invoke("spotlight_get_shortcut");
})();

export default {
  name: (_: Stack) => "Settings",
  hotKeys: (_stack: Stack, _modes: Modes) => [],
  Modal: ({}: { stack: Stack; modes: Modes }) => {
    if (!saved.value) return;
    const options = [
      ["shift", "IconShiftKey"],
      ["ctrl", "IconCtrlKey"],
      ["alt", "IconAltKey"],
      ["command", "IconCommandKey"],
    ];

    return (
      <div
        className={overlay}
        style={{
          position: "absolute",
          overflow: "auto",
          width: "40ch",
          height: "9em",
          fontSize: "0.9rem",
          bottom: "0",
          right: "4ch",
          padding: "1ch 2ch 1ch 2ch",
          borderRadius: "0.5rem 0.5rem 0 0",
          display: "flex",
          flexDirection: "column",
          gap: "1ch",
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
            marginLeft: "1ch",
          }}
        >
          {options.map(([name, icon]) => (
            <div
              onMouseDown={() => {
                let update = saved.peek() ?? {};
                update[name] = !update[name];
                console.log(`${name}: `, update);
                invoke("spotlight_update_shortcut", { shortcut: update });
                saved.value = { ...update };
              }}
              className={border + " " + (
                saved.value && saved.value[name]
                  ? enchantedForestGradientActive
                  : enchantedForestGradient
              )}
            >
              <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
                {<Icon name={icon} />}
              </span>
            </div>
          ))}
          + SPACE
        </div>
      </div>
    );
  },
};
