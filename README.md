# schism

shitty AI-generated logo, idk idc:

<img src="assets/logo.png" alt="schism" width="300" />

A terminal tool for capturing structured code review notes and piping them into
AI. For people who know the pieces fit.

## What it does

Most diff pagers help you read a diff. schism helps you do something with it.

You pipe in a diff, navigate it in a TUI, leave inline comments on lines or
whole files, write a review summary — then press Enter. Your notes come out as
structured text or JSON on stdout, ready to pipe into Claude, feed to a script,
or drop into a PR.

It's one-shot and composable by design. No persistence, no database, no account.
Just stdin → review → stdout.

<video src="assets/demo.mp4" controls></video>

## Install

```bash
cargo install --git https://github.com/frm/schism
```

## Git config

Use schism as your default pager so `git diff`, `git show`, and `git log -p`
open in it:

```gitconfig
[core]
    pager = schism
[interactive]
    diffFilter = schism --no-pager
```

## Usage

```bash
git diff | schism            # interactive review
git diff | schism --no-pager # pretty-print, no TUI
git diff | schism --json     # structured JSON output
git diff | schism --tree     # open with file tree visible
```

## Usage with AI

Build a custom script like:

```bash
# Review, then have Claude turn your rough notes into polished feedback
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

You annotate what matters — bad patterns, questions, nits — and let the AI clean
up the prose. The JSON output (`--json`) works well for more structured prompts:

```bash
git diff | schism --json | claude "Here is a JSON object with my code review notes. Summarize the main concerns and draft a PR comment."
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

## License

MIT
