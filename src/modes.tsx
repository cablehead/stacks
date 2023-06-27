import { Signal, signal } from "@preact/signals";
import { JSXInternal } from "preact/src/jsx";

import { writeText } from "@tauri-apps/api/clipboard";
import { hide } from "tauri-plugin-spotlight-api";

import { Icon } from "./ui/icons";

interface HotKey {
  name: string;
  keys: (string | JSXInternal.Element)[];
  onMouseDown: (event: any) => void;
}

export interface Mode {
  name: string;
  hotKeys: HotKey[];
}

const defaultMode = {
  name: "Clipboard",
  hotKeys: [
    /*
     * TODO:
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
          name="Clear filter"
          keys={["ESC"]}
          onMouseDown={() => filter.show.value = false}
        />
      )}
      */
    {
      name: "Copy",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
      },
    },
    {
      name: "Actions",
      keys: [<Icon name="IconCommandKey" />, "K"],
      onMouseDown: () => {
        modes.toggle(actionsMode);
      },
    },
  ],
};

export const actionsMode = {
  name: "Actions",
  hotKeys: [
    {
      name: "Trigger",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
      },
    },
    {
      name: "Back",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],
};

export const editorMode = {
  name: "Editor",
  hotKeys: [
    {
      name: "Capture",
      keys: [
        <Icon name="IconCommandKey" />,
        <Icon name="IconReturnKey" />,
      ],
      onMouseDown: () => {
          // onMouseDown={editor.save}
      },
    },
    {
      name: "Discard",
      keys: ["ESC"],
      onMouseDown: () => modes.deactivate(),
    },
  ],

  content: "",
  get save() {
    return () => {
      writeText(this.content);
      hide();
    };
  },
};


/*
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
        <HotKey
          name="Back"
          keys={["ESC"]}
          onMouseDown={() => {
            stacks.state.show.value = !stacks.state.show.value;
          }}
        />

        <VertDiv />
        <Theme />
      </div>
    </footer>
  );
};
*/

export const modes = {
  modes: [defaultMode, actionsMode, editorMode] as Mode[],
  prev: defaultMode as Mode,
  active: signal(defaultMode) as Signal<Mode>,
  isActive(mode: Mode) {
    return mode == this.active.value;
  },
  activate(mode: Mode) {
    this.active.value = mode;
  },
  toggle(mode: Mode) {
    if (mode == this.active.value) {
      this.deactivate();
      return;
    }
    this.activate(mode);
  },
  deactivate() {
    if (this.active.value == defaultMode) {
      hide();
      return;
    }
    this.active.value = defaultMode;
  },
  get(name: string) {
    return this.modes.find((mode) => mode.name === name) || defaultMode;
  },
};
