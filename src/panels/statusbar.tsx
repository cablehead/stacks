import { JSXInternal } from "preact/src/jsx";

import { Icon, RenderKeys } from "../ui/icons";
import { borderRight, footer } from "../ui/app.css";

import { default as theme } from "../theme";

import { modes } from "../modals";
import { Mode } from "../modals/types";
import { Stack } from "../types";

const VertDiv = () => (
  <div
    className={borderRight}
    style={{
      width: "1px",
      height: "1.5em",
    }}
  />
);

const Theme = () => (
  <div
    onMouseDown={() => {
      theme.value = theme.value === "light"
        ? "dark"
        : "light";
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
      {theme.value == "light"
        ? <Icon name="IconMoon" />
        : <Icon name="IconSun" />}
    </span>
  </div>
);

const ModeBar = ({ stack, mode }: { stack: Stack; mode: Mode }) => {
  return (
    <footer className={footer}>
      <div style="">
        {mode.name}
      </div>
      <div style="
        display: flex;
        align-items: center;
        gap: 0.5ch;
      ">
        {mode.hotKeys(stack, modes).map((hotKey) => (
          <>
            <HotKey
              name={hotKey.name}
              keys={hotKey.keys}
              onMouseDown={hotKey.onMouseDown}
            />
            <VertDiv />
          </>
        ))}
        <Theme />
      </div>
    </footer>
  );
};

export const StatusBar = ({ stack }: { stack: Stack }) => {
  return <ModeBar stack={stack} mode={modes.active.value} />;
};

const HotKey = ({ name, keys, onMouseDown }: {
  name: string;
  keys: (string | JSXInternal.Element)[];
  onMouseDown: (event: any) => void;
}) => {
  return (
    <div
      class="hoverable"
      style={{
        display: "flex",
        gap: "0.75ch",
      }}
      onMouseDown={onMouseDown}
    >
      <div>{name}</div>
      <RenderKeys
        keys={keys}
      />
    </div>
  );
};
