# schism

A terminal diff reviewer with inline commenting, folding, file tree, and structured export.

## Why

Diff pagers make diffs pretty but don't let you _do_ anything with them. schism lets you review a diff, leave inline comments on lines or whole files, write a review body, and pipe everything to an AI — without leaving the terminal.

## Install

```bash
cargo install --git https://github.com/frm/schism
```

## Usage

```bash
# Interactive review
git diff | schism

# Pretty-print, no TUI
git diff | schism --no-pager

# Output as JSON
git diff | schism --json

# Open with file tree visible
git diff | schism --tree
```

## Git config

Use schism as your default pager so `git diff`, `git show`, and `git log -p` all open in it:

```gitconfig
[core]
    pager = schism
[interactive]
    diffFilter = schism --no-pager
```

## Example usage with Claude

schism outputs comments to stdout when you press `Enter`. Add this to your shell config to review a diff and have Claude polish your notes into a proper review:

```bash
greview() {
  local comments
  comments=$(git diff "$@" | schism) || return
  [[ -z "$comments" ]] && return
  echo "$comments" | claude "These are my rough notes from a code review. Clean them up into clear, concise review comments."
}
```

Then:

```bash
greview          # review working changes
greview HEAD~1   # review last commit
greview main     # review diff against main
```

## Keybindings

### Navigation

| Key              | Action                                   |
| ---------------- | ---------------------------------------- |
| `j`/`k`, `↑`/`↓` | Move cursor                              |
| `J`/`K`          | Jump to next/prev file                   |
| `n`/`N`          | Jump to next/prev hunk (or search match) |
| `gg` / `G`       | Top / bottom                             |
| `Ctrl+D`/`U`     | Half page down/up                        |
| `Ctrl+F`/`B`     | Full page down/up                        |
| `Ctrl+P`         | Fuzzy file finder                        |
| `/`              | Search in diff                           |

### Folding

| Key           | Action                        |
| ------------- | ----------------------------- |
| `z` / `Space` | Toggle fold hunk              |
| `Z`           | Toggle fold file              |
| `Tab`         | Toggle fold all hunks in file |
| `Shift+Tab`   | Toggle fold all files         |

### Commenting

| Key  | Action                                          |
| ---- | ----------------------------------------------- |
| `c`  | Add/edit comment on current line or file header |
| `dd` | Delete comment                                  |
| `b`  | Edit review body                                |

### File viewer

| Key     | Action                               |
| ------- | ------------------------------------ |
| `f`     | Open full file (new version) / close |
| `F`     | Open full file (old version)         |
| `m`     | Toggle old/new in file viewer        |
| `J`/`K` | Next/prev file in file viewer        |

### Tools

| Key     | Action                             |
| ------- | ---------------------------------- |
| `t`     | Toggle file tree sidebar           |
| `h`/`l` | Switch focus between tree and diff |
| `?`     | Help overlay                       |

### Exit

| Key       | Action                                            |
| --------- | ------------------------------------------------- |
| `Enter`   | Exit — output comments to stdout (silent if none) |
| `q`/`Esc` | Exit silently, no output                          |

## Output formats

### Stdout (on `Enter`)

Only outputs if you have comments:

```
src/auth.rs:42
+ if claims.expired() {
Handle expired tokens explicitly

src/auth.rs
Whole file needs a security review
```

### JSON (`--json`)

```json
{
  "body": "Overall looks good, a few nits",
  "comments": [
    {
      "path": "src/auth.rs",
      "line": 42,
      "change": "+",
      "text": "Handle expired tokens"
    },
    {
      "path": "src/auth.rs",
      "line": 0,
      "change": null,
      "text": "Needs security review"
    }
  ]
}
```

## Built with

- [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) — TUI
- [syntect](https://github.com/trishume/syntect) — syntax highlighting
- [nucleo](https://github.com/helix-editor/nucleo) — fuzzy matching

## License

MIT
