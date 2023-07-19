import { Icon } from "../ui/icons";

import { Modes } from "./types";

import { default as actionsMode } from "./actionsMode";
import { default as addToStackMode } from "./addToStackMode";

import { Stack } from "../types";
import { createStack, currStack, triggerCopy } from "../stacks";

import { actions } from "../actions";

export default {
  name: "Clipboard",
  hotKeys: (stack: Stack, modes: Modes) => {
    let ret = [];

    if (!stack.parent) {
      if (stack.item.value?.content_type == "Stack") {
        ret.push({
          name: "Enter stack",
          keys: ["TAB"],
          onMouseDown: () => {
            const item = stack.item.value;
            if (item && item.content_type == "Stack") {
              const subStack = createStack(item.stack, stack);
              currStack.value = subStack;
              return;
            }
          },
        });
      } else {
        ret.push({
          name: "Add to stack",
          keys: ["TAB"],
          onMouseDown: () => {
            modes.activate(currStack.value, addToStackMode);
          },
        });
      }
    } else {
      ret.push({
        name: "Leave stack",
        keys: ["SHIFT", "TAB"],
        onMouseDown: () => {
          if (currStack.value.parent) {
            currStack.value = currStack.value.parent;
            return;
          }
        },
      });
    }

    ret.push({
      name: "Copy",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
        triggerCopy();
      },
    });

    let action = actions.find((action) => action.name === "Copy entire stack");
    if (action && action.canApply(stack)) {
      ret.push({
        name: action.name,
        keys: action.keys,
        onMouseDown: () => action.trigger(stack),
      });
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
