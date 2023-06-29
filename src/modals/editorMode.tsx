import { Icon } from "../ui/icons";

import { Modes } from "./types";

export default {
  name: "Editor",
  hotKeys: (modes: Modes) => [
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
