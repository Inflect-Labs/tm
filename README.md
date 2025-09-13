# td - minimal todo cli

i hate productivity apps, so i made this.

a simple command-line todo list manager with nested subtasks written in rust.

## installation

```bash
# install from source
cargo install --path .

# or build manually
cargo build --release
sudo cp target/release/td /usr/local/bin/
```

## usage

### add a todo

```bash
td add "buy groceries"
td add "walk the dog"
```

### add a subtask

```bash
td add 0 "get milk"
td add 0 "get bread"
```

### list todos

```bash
td list
```

output format: `[status] index: text` with nested indentation

### complete an item or subtask

```bash
td check 0        # complete main item
td check 0 1      # complete subtask 1 of item 0
td check 0 1 2    # complete deeply nested item
```

### delete an item or subtask

```bash
td delete 1       # delete main item 1
td delete 0 1     # delete subtask 1 of item 0
```

### clear completed items

```bash
td clear
```

removes all completed items and subtasks recursively

### clear all items

```bash
td clear-all
```
