# tm - minimal task manager cli

i hate productivity apps, so i made this.

a simple command-line task manager with nested subtasks written in rust.

## installation

```bash
# install from source
cargo install --path .

# or build manually
cargo build --release
sudo cp target/release/tm /usr/local/bin/
```

## usage

### add a task

```bash
tm add "buy groceries"
tm add "walk the dog"
```

### add a subtask

```bash
tm add 0 "get milk"
tm add 0 "get bread"
```

### list tasks

```bash
tm list
```

output format: `[status] index: text` with nested indentation

### complete an item or subtask

```bash
tm check 0        # complete main item
tm check 0 1      # complete subtask 1 of item 0
tm check 0 1 2    # complete deeply nested item
```

### delete an item or subtask

```bash
tm delete 1       # delete main item 1
tm delete 0 1     # delete subtask 1 of item 0
```

### clear completed items

```bash
tm clear
```

removes all completed items and subtasks recursively

### clear all items

```bash
tm clear-all
```
