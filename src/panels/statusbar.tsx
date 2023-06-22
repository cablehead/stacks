import { JSXInternal } from "preact/src/jsx";

import { Icon, RenderKeys } from "../ui/icons";
import { borderRight, footer } from "../ui/app.css";

import { actions, editor, filter, themeMode, triggerCopy } from "../state";
import * as stacks from "./stacks";

export function StatusBar() {
  if (stacks.state.show.value) return <StacksBar />;
  if (editor.show.value) return <EditorBar />;
  if (actions.show.value) return <ActionBar />;
  return <MainBar />;
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

const Theme = () => (
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

const EditorBar = () => {
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
        <HotKey
          name="Discard"
          keys={["ESC"]}
          onMouseDown={() => editor.show.value = false}
        />

        <VertDiv />
        <HotKey
          name="Capture"
          keys={[
            <Icon name="IconCommandKey" />,
            <Icon name="IconReturnKey" />,
          ]}
          onMouseDown={editor.save}
        />

        <VertDiv />
        <Theme />
      </div>
    </footer>
  );
};

const ActionBar = () => {
  return (
    <footer className={footer}>
      <div style="">
        Actions
      </div>
      <div style="
        display: flex;
        align-items: center;
        gap: 0.5ch;
      ">
        {
          /*
        <div onClick={() => actions.show.value = false} class="hoverable">
          Back&nbsp;
          <span className={iconStyle}>
            ESC
          </span>
        </div>

        <VertDiv />
        <div
          onMouseDown={() => editor.save()}
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
        */
        }

        <HotKey
          name="Trigger"
          keys={[<Icon name="IconReturnKey" />]}
          onMouseDown={() => undefined}
        />

        <VertDiv />
        <HotKey
          name="Back"
          keys={["ESC"]}
          onMouseDown={() => {
            actions.show.value = !actions.show.value;
          }}
        />

        <VertDiv />
        <Theme />
      </div>
    </footer>
  );
};

const MainBar = () => {
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
        {!filter.show.value
          ? (
            <HotKey
              name="Filter"
              keys={["/"]}
              onMouseDown={() => filter.show.value = true}
            />
          )
          : (
            <HotKey
              name="Clear Filter"
              keys={["ESC"]}
              onMouseDown={() => filter.show.value = false}
            />
          )}

        <VertDiv />
        <HotKey
          name="Copy"
          keys={[<Icon name="IconReturnKey" />]}
          onMouseDown={triggerCopy}
        />

        <VertDiv />
        <HotKey
          name="Actions"
          keys={[<Icon name="IconCommandKey" />, "K"]}
          onMouseDown={() => {
            actions.show.value = !actions.show.value;
          }}
        />

        <VertDiv />
        <Theme />
      </div>
    </footer>
  );
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

const StacksBar = () => {
  return (
    <footer className={footer}>
      <div style="">
        Add to stack
      </div>
      <div style="
        display: flex;
        align-items: center;
        gap: 0.5ch;
      ">

        <HotKey
          name="Select"
          keys={[<Icon name="IconReturnKey" />]}
          onMouseDown={() => undefined}
        />

        <VertDiv />
        <HotKey
          name="Create new"
          keys={[
            <Icon name="IconCommandKey" />,
            <Icon name="IconReturnKey" />,
          ]}
          onMouseDown={() => undefined}
        />

        <VertDiv />
        <Theme />
      </div>
    </footer>
  );
};
