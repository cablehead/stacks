import { Icon } from "../ui/icons";

import { Modes } from "./types";

import { default as actionsMode } from "./actionsMode";

import { Stack } from "../types";
import { triggerCopy } from "../stacks";

export default {
  name: "Clipboard",
  hotKeys: (stack: Stack, modes: Modes) => [
    {
      name: "Copy",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
          triggerCopy();
      },
    },

    {
      name: "Actions",
      keys: [<Icon name="IconCommandKey" />, "K"],
      onMouseDown: () => {
        modes.toggle(stack, actionsMode);
      },
    },

    ...(stack.filter.dirty()
      ? [
        {
          name: "Clear filter",
          keys: ["ESC"],
          onMouseDown: () => {
            stack.filter.clear();
          },
        },
      ]
      : []),
  ],
};
