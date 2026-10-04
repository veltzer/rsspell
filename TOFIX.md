# TOFIX

Findings from a code scan on 2026-10-04.

## High

- `src/main.rs:247-277` - `rsspell scan` always exits 0, even when it reports typos (verified: a file with four typos returns rc=0). `check_markdown`/`check_svg` (lines 279-308) throw away the `Vec<String>` the `find_*_typos` helpers return. Total the typos and return an error (non-zero exit) when any are found, so the tool can gate CI.
- `src/main.rs:280,304` - `fs::read_to_string(path).expect("Could not read file")` panics (rc=101) on the first non-UTF-8 or unreadable `.md`/`.svg` under the tree, which ends the whole scan. Return a `Result` with the path in its context (or report and skip the file) instead of panicking.
- `README.md:39`, `docs/src/usage.md:28` - documented `rsspell scan --ignore word1 word2` does not work: `ignore` (`src/main.rs:30-31`) takes one value per flag, so `word2` is parsed as the positional path (and a trailing path then fails with "unexpected argument"). Add `num_args = 1..` / `value_delimiter = ','` to the arg, or document `-i word1 -i word2`.

## Medium

- `src/main.rs:295-299` - every `Event::Text` is checked, including text inside fenced and indented code blocks, so identifiers in code are reported as typos (verified: `foo_barz qwxz` inside a code fence is flagged). Track `Start(Tag::CodeBlock)`/`End(TagEnd::CodeBlock)` and skip text in between.
- `README.md:50`, `docs/src/usage.md:22,64` - the examples `rsspell dicts install de-DE` and `--lang de-DE` name a dictionary that does not exist upstream: wooorm/dictionaries has `de`, `de-AT`, `de-CH`, but no `de-DE`, so the documented command fails with a 404. Use `de` in the examples.
- `src/main.rs:266` - `WalkDir::new(root_path)` descends into `.git/`, `target/`, `node_modules/`, `.venv/`, so a default `rsspell scan` in a repo spell-checks vendored markdown; `filter_map(|e| e.ok())` also hides permission errors. Skip hidden and well-known build dirs (or honor `.gitignore` via the `ignore` crate) and report walk errors.
- `src/main.rs:329` - typo lines print only the word, no line number, and the file name is on a separate header line; output is not `path:line: word`, so editors and CI annotations cannot jump to it. Carry the byte offset (pulldown-cmark `into_offset_iter`) and print `path:line:col`.

## Low

- `src/main.rs:252` - `.rsspellignore` is read from the current directory, not from the scanned root, so `rsspell scan some/dir` ignores `some/dir/.rsspellignore`; `docs/src/usage.md:33` says "in the root of your project". Resolve it relative to the scan root, or document the CWD behavior.
- `src/main.rs:249` - the `[a-zA-Z]+` word regex splits on apostrophes and drops all non-ASCII letters, so it cannot work for the `de`/`fr` dictionaries `dicts install` offers (e.g. "Größe" becomes "Gr" + "e"). Use a Unicode word pattern (`\p{L}+(?:['’]\p{L}+)*`).
- `src/main.rs:342-395` - the embedded-dictionary setup is copied into four tests; extract a `fn test_dict() -> (Dictionary, Regex)` helper (or reuse `load_dictionary("en-US")`).
- `dictionaries/en_US.aff`, `dictionaries/en_US.dic` - embedded third-party Hunspell data (SCOWL-derived) with no provenance or license note, inside an MIT crate published to crates.io. Add a `dictionaries/README.md` naming the source and its license.
