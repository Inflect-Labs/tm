# td - minimal todo cli

i hate productivity apps, so i made this.

a simple command-line todo list manager written in rust.

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

### list todos
```bash
td list
```
output format: `[status] index: text`

### complete an item
```bash
td check 0
```

### delete an item
```bash
td delete 1
```

### clear completed items
```bash
td clear
```

### clear all items
```bash
td clear-all
```
