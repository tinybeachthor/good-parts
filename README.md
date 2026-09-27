# good-parts

Find the complex parts of a code repository — the functions and files that are
the best candidates for refactoring.

`good-parts` walks a repository, measures every function with
[arborist-metrics](https://crates.io/crates/arborist-metrics) (cognitive
complexity, cyclomatic complexity, and source lines of code), and ranks the
hotspots.

## Installation

```sh
cargo install --path .
```

This installs two binaries that behave identically:

- `good-parts` — standalone CLI
- `cargo-good-parts` — lets you run it as `cargo good-parts`

## Usage

```sh
good-parts                      # analyze the current directory
cargo good-parts path/to/repo   # same, as a cargo subcommand
good-parts src/a.rs src/b.py    # only specific files
good-parts --files              # rank files instead of functions
good-parts --format json        # machine-readable output
good-parts --help               # all options, examples and exit codes
```

Example output:

```
SCORE  COG  CYC  SLOC  LOCATION            FUNCTION
41.1   31!  14   62    src/parser.rs:118   parse_expression
18.5   12   8    50    src/eval.rs:40      eval_block

48 files, 612 functions, 1 over cognitive threshold 15
```

`!` marks functions whose cognitive complexity exceeds `--threshold`.

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `-n, --top <N>` | `20` | Number of candidates to show |
| `--sort <KEY>` | `score` | Rank by `score`, `cognitive`, `cyclomatic`, or `sloc` |
| `--threshold <N>` | `15` | Cognitive complexity above which a function is flagged |
| `--min-cognitive <N>` | `1` | Hide functions below this cognitive complexity |
| `--fail-over-threshold` | | Exit 1 if any function exceeds `--threshold` |
| `--files` | | Rank files instead of functions |
| `--format <FMT>` | `table` | `table` or `json` (versioned, for tools and agents) |
| `--exclude <GLOB>` | | Extra patterns to skip (repeatable) |
| `--no-methods` | | Analyze free functions only |
| `--agent-instructions` | | Print instructions for AI coding agents |

`PATH` can be repeated and may be files or directories (default: `.`).
Reported paths are relative to the current directory.

Files ignored by `.gitignore` are skipped automatically (inside a git
repository); use `--exclude` for anything else, e.g. `--exclude 'vendor/**'`.

### Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | `--fail-over-threshold` was given and a function exceeds `--threshold` |
| `2` | Error (invalid arguments, unreadable path, ...) |

## Use with AI agents

`good-parts` is built to be driven by AI coding assistants such as Claude Code:
`--format json` output is stable and versioned (`schema_version`), locations
are ready to open, and `--fail-over-threshold` lets an agent check its own
refactors through the exit code.

`good-parts --agent-instructions` prints a Markdown guide for agents: when to
use the tool, the JSON fields, and a find → refactor → verify workflow. Point
your agent at it from `AGENTS.md` or `CLAUDE.md`:

```markdown
To find refactoring candidates, run `good-parts --agent-instructions` and follow it.
```

Or paste the guide in directly:

```sh
good-parts --agent-instructions >> AGENTS.md
```

## How candidates are ranked

Each function gets a score:

```
score = cognitive + 0.5 × cyclomatic + sloc / 20
```

Cognitive complexity dominates because it best reflects how hard code is to
understand; cyclomatic complexity and length act as tie-breakers. A file's
score is the sum of its functions' scores, so files with many hard functions
rise to the top.

## Supported languages

Rust, Python, JavaScript, TypeScript, Java, and Go. Other files are skipped.

## Development

```sh
cargo test
cargo clippy --all-targets
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
