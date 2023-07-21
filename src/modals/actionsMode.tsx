import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Stack } from "../types";

export default {
  name: () => "Actions",
  hotKeys: (_: Stack, modes: Modes) => [
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
