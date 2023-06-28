import { Icon } from "../ui/icons";

import { Modes } from "./types";

export default {
  name: "Filter by content type",
  hotKeys: (modes: Modes) => [
    {
      name: "Select",
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
