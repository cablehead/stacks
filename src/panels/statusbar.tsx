import { Signal } from "@preact/signals";

import { Icon } from "../ui/icons";
import { borderRight, footer, iconStyle } from "../ui/app.css";

import { editor } from "../state";

export function StatusBar(
  { themeMode, showFilter, showActions, triggerCopy }: {
    themeMode: Signal<string>;
    showFilter: Signal<boolean>;
    showActions: Signal<boolean>;
    triggerCopy: () => void;
  },
) {
  if (editor.show.value) return <EditorStatusBar themeMode={themeMode} />;

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
        <div onClick={async () => await triggerCopy()} class="hoverable">
          Copy&nbsp;
          <span className={iconStyle}>
            <Icon name="IconReturnKey" />
          </span>
        </div>

        <VertDiv />
        <div
          class="hoverable"
          onMouseDown={() => {
            showActions.value = !showActions.value;
          }}
        >
          Actions&nbsp;
          <span className={iconStyle} style="margin-right: 0.25ch;">
            <Icon name="IconCommandKey" />
          </span>
          <span className={iconStyle}>
            K
          </span>
        </div>

        <VertDiv />
        <Theme themeMode={themeMode} />
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
    onMouseDown={() => {
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

const EditorStatusBar = (
  { themeMode }: {
    themeMode: Signal<string>;
  },
) => {
  return (
    <footer className={footer}>
      <div style="">
        Editor
      </div>
      <div style="
        display: flex;
        align-items: center;
        gap: 0.5ch;
      ">
        <div onClick={() => editor.show.value = false} class="hoverable">
          Discard&nbsp;
          <span className={iconStyle}>
            ESC
          </span>
        </div>

        <VertDiv />
        <div
          onMouseDown={() => {
            editor.save();
            editor.show.value = false;
          }}
          class="hoverable"
        >
          Capture&nbsp;
          <span className={iconStyle} style="margin-right: 0.25ch;">
            <Icon name="IconCommandKey" />
          </span>
          <span className={iconStyle}>
            <Icon name="IconReturnKey" />
          </span>
        </div>

        <VertDiv />
        <Theme themeMode={themeMode} />
      </div>
    </footer>
  );
};
