import { Signal } from "@preact/signals";

import { Icon } from "./icons.tsx";

import {
  borderRight,
  footer,
  iconStyle,
} from "./app.css.ts";

export function StatusBar(
  { themeMode, showFilter, triggerCopy }: {
    themeMode: Signal<string>;
    showFilter: Signal<boolean>;
    triggerCopy: () => void;
  },
) {
  return (
    <footer className={footer}>
      <div style="">
        Clipboard
      </div>

      <div style="
    display: flex;
        align-items: center;
    gap: 0.5ch;
    ">
        {!showFilter.value &&
          (
            <div onClick={() => showFilter.value = true} class="hoverable">
              Filter&nbsp;
              <span className={iconStyle}>
                /
              </span>
            </div>
          )}

        {showFilter.value &&
          (
            <div onClick={() => showFilter.value = false} class="hoverable">
              Clear Filter&nbsp;
              <span className={iconStyle}>
                ESC
              </span>
            </div>
          )}

        <div
          className={borderRight}
          style={{
            width: "1px",
            height: "1.5em",
          }}
        />

        <div onClick={async (e) => await triggerCopy()} class="hoverable">
          Copy&nbsp;
          <span className={iconStyle}>
            <Icon name="IconReturnKey" />
          </span>
        </div>

        <div
          className={borderRight}
          style={{
            width: "1px",
            height: "1.5em",
          }}
        />

        <div
          onClick={() => {
            themeMode.value = themeMode.value === "light" ? "dark" : "light";
          }}
          class="hoverable"
        >
          <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
            {themeMode.value == "light"
              ? <Icon name="IconMoon" />
              : <Icon name="IconSun" />}
          </span>
        </div>
      </div>
    </footer>
  );
}
