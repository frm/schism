# schism

A terminal diff reviewer with inline commenting, folding, and structured export. Think delta meets code review.

## Why

Existing diff pagers (delta, diff-so-fancy) make diffs pretty but don't let you _do_ anything with them. schism lets you review a diff, leave inline comments, and pipe those comments to an AI, a markdown file, or your clipboard — without leaving the terminal.

## Usage

```bash
# Interactive review (default)
git diff | schism

# Pretty-print, no TUI (like delta)
git diff | schism --no-pager

# Review and pipe comments to an AI
git diff | schism | claude

# Review and save to file
git diff | schism > /dev/null  # comments go to stdout
git diff | schism --no-pager   # just read the diff
```

### Modes

**Interactive (default):** Full TUI with syntax-highlighted viewport, file tree sidebar, inline commenting, and folding. Renders to `/dev/tty` so stdout stays free for piping.

**Pretty-print (`--no-pager`):** Delta-style colored output straight to stdout. No interactivity. Works as `core.pager`:

```gitconfig
[core]
    pager = schism --no-pager
```

## Keybindings

### Navigation

| Key | Action |
|---|---|
| `j`/`k`, `↑`/`↓` | Move cursor |
| `J`/`K` | Jump to next/prev file |
| `n`/`N` | Jump to next/prev hunk (or search match) |
| `Space`, `PgDn`/`PgUp` | Page scroll |
| `Ctrl+P` | Fuzzy file finder |
| `/` | Search in diff |

### Folding

| Key | Action |
|---|---|
| `z` | Toggle fold current hunk |
| `Z` | Toggle fold current file |
| `Tab` | Toggle fold all hunks in file |
| `Shift+Tab` | Toggle fold all files |

### Review

| Key | Action |
|---|---|
| `c` | Add/edit comment on current line |
| `dd` | Delete comment |
| `e` | Export comments to markdown file |
| `y` | Copy comments to clipboard |
| `t` | Toggle file tree sidebar |

### Exit

| Key | Action |
|---|---|
| `Enter` | Exit — output comments to stdout (silent if none) |
| `q`/`Esc` | Exit silently |
| `?` | Help overlay |

## Export Formats

### Stdout (on `Enter`)

Pipe-friendly. Only outputs if you have comments:

```
src/auth.rs:42
+ if claims.expired() {
This should handle expired tokens explicitly

src/db/queries.rs:12
- pub fn find_uncached(id: UserId) -> Option<User> {
Why was this removed?
```

### Markdown file (`e`)

Writes `.schism/review-YYYY-MM-DD-HHMMSS.md`:

````markdown
# Code Review

## `src/auth.rs`

### L42 (added)
```rust
if claims.expired() {
```
> This should handle expired tokens explicitly
````

### Clipboard (`y`)

Dense, AI-friendly:

```
src/auth.rs:42 (added) `if claims.expired() {`
— This should handle expired tokens explicitly
```

## Install

```bash
cargo install --path .
```

## Suggested aliases

```bash
alias gd="git diff | schism --no-pager"   # quick read
alias gr="git diff | schism"               # review mode
```

## Built with

- [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) — TUI
- [syntect](https://github.com/trishume/syntect) — syntax highlighting
- [nucleo](https://github.com/helix-editor/nucleo) — fuzzy matching
- [arboard](https://github.com/1Password/arboard) — clipboard

## License

MIT
