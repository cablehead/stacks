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

export default {
  name: (_: Stack) => "Settings",
  hotKeys: (_stack: Stack, _modes: Modes) => [],
  Modal: ({}: { stack: Stack; modes: Modes }) => {
    const saved: Signal<Record<string, boolean>> = signal({
      shift: false,
      ctrl: true,
      alt: false,
      command: false,
    });

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
          height: "8em",
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
            marginLeft: "1ch",
          }}
        >
          {options.map(([name, icon]) => (
            <div
              onMouseDown={() => {
                console.log(`go: ${name}`);
                invoke("update_shortcut", { shortcut: "Command+Opt+Space" });
              }}
              className={border + " " + (
                saved.value[name]
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
