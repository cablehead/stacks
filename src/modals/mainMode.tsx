import { Icon } from "../ui/icons";

import { HotKey, Modes } from "./types";

import { default as actionsMode } from "./actionsMode";
// import { default as addToStackMode } from "./addToStackMode";

import { Stack } from "../types";

import { actions } from "../actions";

export default {
  name: (stack: Stack) => {
    const selected = stack.nav.value.root?.selected;
    const terse = selected ? selected.terse : "";
    return (
      <div style="
          display: flex;
          gap: 0.75ch;
          align-items: center;
          overflow: hidden;
          ">
        <div
          style={{
            flexShrink: 0,
            marginTop: "-2px",
            width: "2ch",
            whiteSpace: "nowrap",
            overflow: "hidden",
          }}
        >
          <Icon name="IconStack" />
        </div>
        {terse}
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
