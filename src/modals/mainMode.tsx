import { Icon } from "../ui/icons";

import { Modes } from "./types";

import { default as actionsMode } from "./actionsMode";

import { currStack} from "../stacks";

export default {
  name: "Clipboard",
  hotKeys: (modes: Modes) => [
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

    ...(currStack.value.filter.dirty()
      ? [
        {
          name: "Clear filter",
          keys: ["ESC"],
          onMouseDown: () => {
            currStack.value.filter.clear();
          },
        },
      ]
      : []),
  ],
};
