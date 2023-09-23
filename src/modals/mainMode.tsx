import { Icon } from "../ui/icons";

import { HotKey, Modes } from "./types";

import { default as actionsMode } from "./actionsMode";
// import { default as addToStackMode } from "./addToStackMode";

import { Stack } from "../types";

import { actions } from "../actions";

import { borderRight } from "../ui/app.css";

const VertDiv = () => (
  <div
    className={borderRight}
    style={{
      width: "1px",
      height: "1.5em",
    }}
  />
);

const Lock = () => (
  <div class="hoverable">
    <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
      <Icon name="IconLockOpen" />
    </span>
  </div>
);

const SortOrder = () => (
  <div class="hoverable">
    <span style="
            display: inline-block;
            width: 1.5em;
            height: 1.5em;
            text-align: center;
            border-radius: 5px;
            ">
      <Icon name="IconStack" />
    </span>
  </div>
);

export default {
  name: (stack: Stack) => {
    const selected = stack.nav.value.root?.selected;
    const terse = selected ? selected.terse : "";
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "0.5ch",
          marginLeft: "-1ch",
        }}
      >
        <Lock />
        <VertDiv />
        <SortOrder />
        <VertDiv />
        <div>
          {terse}
        </div>
      </div>
    );
  },
  hotKeys: (stack: Stack, modes: Modes) => {
    let ret = [];

    ret.push({
      name: "Copy",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
        stack.triggerCopy();
      },
    });

    let action = actions.find((action) => action.name === "Copy entire stack");
    if (action && action.canApply && action.canApply(stack)) {
      ret.push({
        name: action.name,
        keys: action.keys,
        onMouseDown: () => {
          if (action && action.trigger) action.trigger(stack);
        },
      } as HotKey);
    }

    ret.push({
      name: "Actions",
      keys: [<Icon name="IconCommandKey" />, "K"],
      onMouseDown: () => {
        modes.toggle(stack, actionsMode);
      },
    });

    if (stack.filter.dirty()) {
      ret.push({
        name: "Clear filter",
        keys: ["ESC"],
        onMouseDown: () => {
          stack.filter.clear();
        },
      });
    }

    return ret;
  },
};
