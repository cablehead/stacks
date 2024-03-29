## Todo

### Next release

CURRENTLY HERE

- Stack: meta panel
    - show aggregate counts
    - track number of times children changed

- Edit a stack name
    - Should just be an input field, not a textarea
    - change help message to Rename Stack

- GPT:
    - add an action to send a stack to GPT
        - stream response?
        - add response to stack

- Create a new stack

- Add a fork action for stacks

- Rework store_copy_to_clipboard to ignore the clipboard write

- on Delete:
    - make sure parent stack's last_touched is being bumped
    - it'd be nice to animate the parent stack moving to the top of the list

### Stretch

- Ability to order a Stack

- Rework data store to allow for different Stacks to have the same name
    - Want to be able to rename Stacks
    - Bonus: use a backwards compatible serialization format
    - Revert to saving the raw clipboard data, which is mapped to the current
      form

- Investigate macOS clipboard schema when copying files and images in different
  locations

### Also

- Write script for testing the app
- Work out a way to document the script in the source code: preferably in a way
  a parser can check
- Perform the script
- Record performing the script
- Slice up the portions of the recording
- Export to various formats: gif (but right), png, video (youtube)?
- Overlay subitles
- Overlay voice over # stretch
- Stand up cross.stream
- Host a page for stacks.cross.stream
- Overview page of Stacks, the app

### Pipe to command

- perserve terminal colors
- streaming responses
- improve display of stderr
- the piped item should be "touched"
- actions to move command or response to the clipboard
- quick filter for previously used commands
- once a command is working well, the ability apply it to a large number of
  items
- access clipbard / stacks inside command editor: Mike, as you've pointed out,
  we need this for the Editor / New note too


### And then

- Status bar shouldn't show any actions when "no matches"

- Clicking trigger in the Actions Modal doesn't trigger the action

- Add directory stack
    - Inside directory stacks you can run commands

- Surface info about the Stack in Meta Panel

- Going to want multi-select to add

- Meta-N opens choice: Note / Shell command
    - We have Note
    - Now need Shell command

- Editor
    - access clips while editor is open

- Preference panel

- Customize key presses: particularly leader key press
- Add activate Stacks (and document keypress) in the menu bar menu.

- Editor capture should create an xs.add row: with parent set to the id the
  editor was triggered on: this should be merged version that's put on the
  clipboard

- mark a clipboard source as don't track (for password managers, etc).
    - see: http://nspasteboard.org

- Meta panel. add: image info


## Review: Daily/2022-08-11.md

```
"CREATE TABLE IF NOT EXISTS stream (
   id INTEGER PRIMARY KEY,
   source_id INTEGER,
   parent_id INTEGER,

   topic TEXT NOT NULL,

   -- custom to topic (command)
   data TEXT,
   err TEXT,
   code INTEGER NOT NULL
```
