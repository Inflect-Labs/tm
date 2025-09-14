# tm - minimal task manager cli

a minimal and powerful task management cli, built for devs and coding agents.

## installation

```bash
# install from source
cargo install --path .

# or build manually
cargo build --release
sudo cp target/release/tm /usr/local/bin/
```

## features

- ✅ nested subtasks with unlimited depth
- 🗂️ project organization and switching
- 📝 simple add, complete, delete operations
- 🔄 flexible task reordering and moving
- 🧹 bulk clearing of completed or all tasks
- ⬆️ automatic updates
- 🗑️ clean uninstall with data removal

## usage

### basic task management

#### add a task
```bash
tm add "buy groceries"           # add root level task
tm a "walk the dog"              # short alias
```

#### add subtasks
```bash
tm add "get milk" 0              # add subtask to item 0
tm add "get bread" 0             # add another subtask to item 0
tm add "whole wheat bread" 0 1   # add sub-subtask to item 0's subtask 1
```

#### list tasks
```bash
tm list                          # list all tasks
tm l                             # short alias
tm ls                            # another alias
```

output format: `[status] index: text` with nested indentation

#### complete tasks
```bash
tm check 0                       # complete main item 0
tm c 0 1                         # complete subtask 1 of item 0
tm check 0 1 2                   # complete deeply nested item
```

#### delete tasks
```bash
tm delete 1                      # delete main item 1
tm d 0 1                         # delete subtask 1 of item 0
tm rm 2                          # alternative alias
```

### task organization

#### move tasks around
```bash
tm move 0 --up                   # move item 0 up one position
tm m 0 -u                        # short form
tm move 0 --down                 # move item 0 down one position
tm m 0 -d                        # short form
tm move 0 --top                  # move item 0 to top
tm m 0 -t                        # short form
tm move 0 --bottom               # move item 0 to bottom
tm m 0 -b                        # short form
tm move 0 --position 3           # move item 0 to specific position 3
tm m 0 -p 3                      # short form
```

#### bulk operations
```bash
tm clear                         # remove all completed items
tm cl                            # short alias
tm clear-all                     # remove ALL items (careful!)
tm ca                            # short alias
```

### project management

#### create and switch projects
```bash
tm create-project work           # create a new project called "work"
tm cp personal                   # create "personal" project (short alias)
tm switch-project work           # switch to "work" project
tm sp personal                   # switch to "personal" project (short alias)
```

#### list and delete projects
```bash
tm list-projects                 # show all available projects
tm lp                            # short alias
tm delete-project old-project    # delete a project and all its tasks
tm dp old-project                # short alias
```

### maintenance

#### version and updates
```bash
tm version                       # show current version
tm v                             # short alias
tm update                        # update to latest version
```

#### clean removal
```bash
tm uninstall                     # remove tm and all data (with confirmation)
tm uninstall --yes               # skip confirmation prompt
tm uninstall -y                  # short form
```

## data storage

- tasks are stored in your system's data directory
- projects are separate data files allowing complete isolation
- on macos: `~/Library/Application Support/tm/`
- on linux: `~/.local/share/tm/`
- on windows: `%APPDATA%/tm/`

## tips

- use short aliases for faster workflow: `tm a`, `tm l`, `tm c`, etc.
- organize work with projects to keep different contexts separate
- nested tasks can go as deep as you need
- completed subtasks automatically mark parent tasks as partially complete
- moving tasks preserves their subtask hierarchy
