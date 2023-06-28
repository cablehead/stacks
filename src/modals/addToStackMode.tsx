import { Icon } from "../ui/icons";

import { Modes } from "./types";

export default {
  name: "Add to stack",
  hotKeys: (modes: Modes) => [
    {
      name: "Select",
      keys: [<Icon name="IconReturnKey" />],
      onMouseDown: () => {
      },
    },
    {
      name: "Create new",
      keys: [
        <Icon name="IconCommandKey" />,
        <Icon name="IconReturnKey" />,
      ],
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
