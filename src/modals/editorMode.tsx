import { Icon } from "../ui/icons";

import { Modes } from "./types";
import { Stack } from "../types";

export default {
  name: "Editor",
  hotKeys: (_: Stack, modes: Modes) => [
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
};
