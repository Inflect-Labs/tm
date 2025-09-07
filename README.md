# td - Minimal Todo CLI

A simple command-line todo list manager written in Rust. Stores todos in JSON format with cross-platform file storage.

## Installation

```bash
# Install from source
cargo install --path .

# Or build manually
cargo build --release
sudo cp target/release/td /usr/local/bin/
```

## Usage

### Add a todo
```bash
td add "Buy groceries"
td add "Walk the dog"
```

### List todos
```bash
td list
```
Output format: `[status] index: text`

### Complete a todo
```bash
td complete 0
```

### Delete a todo
```bash
td delete 1
```

### Clear completed todos
```bash
td clear
```

### Clear all todos
```bash
td clear-all
```

## Data Storage

Todos are stored in JSON format at:
- Linux: `~/.local/share/td/todos.json`
- macOS: `~/Library/Application Support/td/todos.json`
- Windows: `C:\Users\{username}\AppData\Roaming\td\todos.json`

## Todo Format

Each todo contains:
- `text`: The todo description
- `completed`: Boolean completion status
- `created_at`: Creation timestamp
- `completed_at`: Completion timestamp (if completed)

## Commands

- `add <text>` - Add a new todo
- `list` - Show all todos with indices
- `complete <index>` - Mark todo as completed
- `delete <index>` - Remove todo
- `clear` - Remove all completed todos
- `clear-all` - Remove all todos
- `help` - Show help information