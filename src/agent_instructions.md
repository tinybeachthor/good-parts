# good-parts: instructions for AI coding agents

`good-parts` finds the functions and files in a repository that are hardest to
understand, so you know where refactoring pays off most. It measures cognitive
complexity (how hard code is to follow), cyclomatic complexity (number of
independent paths) and source lines of code (SLOC).

## When to use it

- The user asks what to refactor, where the code is messy, or how to reduce
  complexity or technical debt.
- Before refactoring: to choose targets backed by numbers instead of guesses.
- After refactoring: to confirm the code you changed actually got simpler.

## Commands

Always use `--format json`; it is a stable, versioned format.

```sh
good-parts --format json -n 10                   # top 10 functions in the current directory
good-parts --format json --files -n 10           # top 10 files
good-parts --format json path/to/a.rs path/to/b.py   # only these files
good-parts --format json --exclude 'vendor/**'   # skip paths (repeatable)
good-parts --fail-over-threshold --threshold 15  # exit 1 if any function is over 15
```

Other options: `--sort score|cognitive|cyclomatic|sloc`, `--min-cognitive N`
(hide functions below N, default 1), `--no-methods`. Run `good-parts --help`
for the full list.

## JSON output (schema_version 1)

```json
{
  "schema_version": 1,
  "kind": "functions",
  "summary": { "files": 48, "functions": 612, "over_threshold": 3, "threshold": 15 },
  "hotspots": [
    {
      "file": "src/parser.rs",
      "language": "Rust",
      "location": "src/parser.rs:118",
      "name": "parse_expression",
      "start_line": 118,
      "end_line": 190,
      "cognitive": 31,
      "cyclomatic": 14,
      "sloc": 62,
      "score": 41.1,
      "exceeds_threshold": true
    }
  ]
}
```

- `hotspots` is sorted best candidate first and limited by `-n` (default 20).
- `file` and `location` are relative to the current directory; open them directly.
- `start_line`/`end_line` are 1-based and inclusive.
- `score = cognitive + 0.5 * cyclomatic + sloc / 20`; higher means a better
  refactoring candidate.
- `summary` counts every analyzed function, not only the ones in `hotspots`.
- With `--files`, `kind` is `"files"` and each hotspot has `file`, `language`,
  `functions`, `over_threshold`, `total_cognitive`, `max_cognitive`,
  `cyclomatic`, `sloc` and `score` (the sum of its functions' scores).

## Workflow

1. Run `good-parts --format json -n 10` and pick candidates, starting from the top.
   Prefer functions with `exceeds_threshold: true`.
2. Read the function at `location` (lines `start_line` to `end_line`) and its callers.
3. Refactor without changing behavior. Run the project's tests before and after.
4. Re-run `good-parts --format json` on the files you edited and confirm the
   score of the refactored code dropped and no new function exceeds the threshold.
5. Report what you changed with before/after numbers.

If the user wants a quality gate, run `good-parts --fail-over-threshold` and
treat exit code 1 as "complexity too high".

## Refactoring guidance

- Cognitive complexity grows fastest with nesting. Flatten it: return early,
  use guard clauses, and invert conditions.
- Extract well-named helper functions for distinct steps or deeply nested blocks.
- Replace long `if`/`else if` chains or flag parameters with lookup tables,
  polymorphism or pattern matching where the language supports it.
- Do not split code mechanically just to lower the numbers. The goal is code
  that is easier to read; a helper that needs many parameters or hides the
  control flow is worse.
- Ask the user before large, cross-cutting refactors.

## Exit codes

- `0`: success
- `1`: `--fail-over-threshold` was given and a function exceeds `--threshold`
- `2`: error (invalid arguments, unreadable path); the message is on stderr

## Limitations

- Supported languages: Rust, Python, JavaScript, TypeScript, Java, Go. Other
  files are skipped (explicitly named ones produce a warning on stderr).
- `.gitignore` is only honored inside git repositories; use `--exclude` elsewhere.
- Metrics measure structure, not correctness or design. Use them to prioritize,
  then use judgment.
