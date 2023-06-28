import { Icon } from "../ui/icons";

import { Modes } from "./types";

export default {
  name: "Actions",
  hotKeys: (modes: Modes) => [
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
