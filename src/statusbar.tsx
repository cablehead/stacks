import { Signal } from "@preact/signals";

import { Icon } from "./icons.tsx";

import { borderRight, footer, iconStyle } from "./app.css.ts";

export function StatusBar(
  { themeMode, showFilter, triggerCopy, triggerDelete }: {
    themeMode: Signal<string>;
    showFilter: Signal<boolean>;
    triggerCopy: () => void;
    triggerDelete: () => void;
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
        <Filter showFilter={showFilter} />
        <VertDiv />

        <div onClick={async (e) => await triggerCopy()} class="hoverable">
          Copy&nbsp;
          <span className={iconStyle}>
            <Icon name="IconReturnKey" />
          </span>
        </div>
        <VertDiv />

        <div onClick={async (e) => await triggerDelete()} class="hoverable">
          Delete&nbsp;
          <span className={iconStyle}>
            Ctrl + DEL
          </span>
        </div>
        <VertDiv />

        <Theme themeMode={ themeMode } />
      </div>
    </footer>
  );
}

const VertDiv = () => (
  <div
    className={borderRight}
    style={{
      width: "1px",
      height: "1.5em",
    }}
  />
);

const Filter = (
  { showFilter }: {
    showFilter: Signal<boolean>;
  },
) =>
  !showFilter.value
    ? (
      <div onClick={() => showFilter.value = true} class="hoverable">
        Filter&nbsp;
        <span className={iconStyle}>
          /
        </span>
      </div>
    )
    : (
      <div onClick={() => showFilter.value = false} class="hoverable">
        Clear Filter&nbsp;
        <span className={iconStyle}>
          ESC
        </span>
      </div>
    );

const Theme = ({ themeMode }: { themeMode: Signal<string> }) => (
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
);
