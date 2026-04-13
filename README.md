# schism

<img src="assets/logo.png" alt="schism" width="300" />

_For people who know the pieces fit._ A terminal tool for capturing structured
code review notes — locally or on GitHub PRs — and piping them into AI.

## What it does

Most diff pagers help you read a diff. schism helps you do something with it.

You pipe in a diff, navigate it in a TUI, leave inline comments on lines or
whole files, write a review summary — then press Enter. Your notes come out as
structured text or JSON on stdout, ready to pipe into Claude, feed to a script,
or drop into a PR.

Or point it at a GitHub PR and submit a full review — with inline comments,
a review body, and approve/request changes — without leaving the terminal.

It's one-shot and composable by design. No persistence, no database, no account.
Just stdin → review → stdout. Or `--pr` → review → submit.

<video src="https://github.com/user-attachments/assets/41f5c3f0-a12a-4162-ac67-65d52ef73800" controls width="100%"></video>

## Install

```bash
cargo install --git https://github.com/frm/schism
```

## Usage

```bash
git diff | schism            # interactive review
git diff | schism --no-pager # pretty-print, no TUI
git diff | schism --json     # structured JSON output
git diff | schism --tree     # open with file tree visible
```

### PR mode

Review and submit GitHub PR reviews directly. Requires [`gh`](https://cli.github.com).

```bash
schism --pr owner/repo#123
schism --pr https://github.com/owner/repo/pull/123
schism --pr owner/repo#123 --debug   # print payload instead of submitting
```

In PR mode:
- The diff is fetched from GitHub via `gh`
- A status bar shows PR info and the current review action
- `D` opens the PR description with markdown rendering
- `C` browses commits and shows per-commit diffs
- `f`/`F` view full file contents from the PR's head/base refs
- `Enter` submits the review (with confirmation)
- `b` opens the review body editor; `Tab` cycles between comment/approve/request changes

## Git config

Use schism as your default pager so `git diff`, `git show`, and `git log -p`
open in it:

```gitconfig
[core]
    pager = schism
[interactive]
    diffFilter = schism --no-pager
```

## Using with AI

```bash
greview() {
  local comments
  comments=$(git diff "$@" | schism) || return
  [[ -z "$comments" ]] && return
  echo "$comments" | claude "These are my rough notes from a code review. Clean them up into clear, concise review comments."
}

greview          # review working changes
greview HEAD~1   # review last commit
greview main     # review diff against main
```

## Keybindings

### Navigation

| Key | Action |
|---|---|
| `j`/`k`, `↑`/`↓` | Move cursor |
| `J`/`K` | Jump to next/prev file |
| `n`/`N` | Jump to next/prev hunk (or search match) |
| `gg` / `G` | Top / bottom |
| `Ctrl+D`/`U` | Half page down/up |
| `Ctrl+F`/`B` | Full page down/up |
| `Ctrl+P` | Fuzzy file finder |
| `/` | Search in diff |

### Folding

| Key | Action |
|---|---|
| `z` / `Space` | Toggle fold hunk |
| `Z` | Toggle fold file |
| `Tab` | Toggle fold all hunks in file |
| `Shift+Tab` | Toggle fold all files |

### Commenting

| Key | Action |
|---|---|
| `c` | Add/edit comment on current line or file header |
| `dd` | Delete comment |
| `b` | Edit review body |

### File viewer

| Key | Action |
|---|---|
| `f` | Open full file (new version) / close |
| `F` | Open full file (old version) |
| `m` | Toggle old/new in file viewer |
| `J`/`K` | Next/prev file in file viewer |

### PR mode

| Key | Action |
|---|---|
| `D` | Show PR description |
| `C` | Browse commits |
| `Tab` (in body) | Cycle: comment / approve / request changes |
| `Enter` | Submit review (with confirmation) |

### Tools

| Key | Action |
|---|---|
| `t` | Toggle file tree sidebar |
| `h`/`l` | Switch focus between tree and diff |
| `?` | Help overlay |

### Exit

| Key | Action |
|---|---|
| `Enter` | Exit — output comments to stdout (silent if none) |
| `q`/`Esc` | Exit silently, no output |

## Output formats

### Stdout (on `Enter`)

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
    { "path": "src/auth.rs", "line": 42, "change": "+", "text": "Handle expired tokens" },
    { "path": "src/auth.rs", "line": 0,  "change": null, "text": "Needs security review" }
  ]
}
```

## Built with

- [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) — TUI
- [syntect](https://github.com/trishume/syntect) + [two-face](https://github.com/CosmicHorrorDev/two-face) — syntax highlighting
- [nucleo](https://github.com/helix-editor/nucleo) — fuzzy matching
- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) — markdown rendering

## License

MIT
