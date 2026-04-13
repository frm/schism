# schism

<center><img src="assets/logo.png" alt="schism" width="300" /></center>

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

### Features

#### Folds

You can fold hunks, files or everything in one go.

<details>
<summary><strong>Demo</strong></summary>

![folds](assets/folds.gif)

</details>

#### File tree and Ctrl-P

Navigate from file to file with `J`/`K`. There's also a file tree of the changes
you can use to navigate to files quickly. Use Ctrl-P to quickly access a fuzzy
finder.

<details>
<summary><strong>Demo</strong></summary>

![file tree and ctrl-p](assets/tree.gif)

</details>

#### File mode

Diffing is hard without context. You can quickly review the full file contents
and swap between the old and the new revision. `f` opens the new revision, `F`
opens the old and `m` toggles between them.

<details>
<summary><strong>Demo</strong></summary>

![file mode](assets/file.gif)

</details>

#### Inline comments and AI feedback

Add comments inline or in a review body. It will get output in plaintext with
context of the file, line number and line contents of your comment. You can also
output in json with the `--json` flag.

<details>
<summary><strong>Demo</strong></summary>

![inline comments](assets/comments.gif)

</details>

Use this to pipe into an AI. This allows you to quickly review AI changes and
plug them back into it with review comments and context.

<details>
<summary><strong>Demo</strong></summary>

![ai feedback](assets/ai.gif)

</details>

#### Review PRs

You can review PRs from the command line. Just use the `--pr` flag, add inline
comments or a review body. Toggle between accept/comment/request changes and
confirm with the right context before submitting your review.

PR review supports file mode, which allows you to view the full file contents,
in both the old and new revisions without leaving the tool.

You can also read the PR description, browse through comments and diff them
individually.

<details>
<summary><strong>Demo</strong></summary>

![pr review](assets/pr.gif)

</details>

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

### PR mode

| Key             | Action                                     |
| --------------- | ------------------------------------------ |
| `D`             | Show PR description                        |
| `C`             | Browse commits                             |
| `Tab` (in body) | Cycle: comment / approve / request changes |
| `Enter`         | Submit review (with confirmation)          |

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

## Built with

- [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) — TUI
- [syntect](https://github.com/trishume/syntect) + [two-face](https://github.com/CosmicHorrorDev/two-face) — syntax highlighting
- [nucleo](https://github.com/helix-editor/nucleo) — fuzzy matching
- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) — markdown rendering

## License

MIT
