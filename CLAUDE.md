# spider

Multi-screen info panel (Over ESP32).

<!-- TODO — three or four lines on shape: the entry point, the two or three
     modules that matter, and where a typical change lands. Claude reads this
     before every task, so a wrong description costs more than an empty one.
     Delete this comment when written. -->

## Rules

Standalone rules live in `.claude/rules/` and are imported here — only files
reachable from `CLAUDE.md` get loaded, so a new rule needs a line below.

@.claude/rules/propose-before-writing.md
@.claude/rules/no-repo-mutation.md
@.claude/rules/no-hanging-commands.md
@.claude/rules/prevent-looping-fail.md


@.claude/rules/rust.md


@.claude/rules/nix.md


@.claude/rules/release.md


## Commands


`just` is the entry point, not the raw toolchain. `just --list` for the rest.

- `just check` — fast type-check, no binary
- `just test` — test suite
- `just fmt` — format sources in place
- `just lint` — lint with warnings denied
- `just ci` — the full local gate; run this before pushing


<!-- TODO — anything above that is wrong here, and any command that exists only
     in this repo: seeding, fixtures, a dev server, a hardware target. -->

## Constraints


- `typos` runs over the source. Add a real term to `typos.toml` rather than
  rewording around it.



- Commit messages must be Conventional Commits; `committed` checks them in CI.



- Links in Markdown are checked in CI. A placeholder URL fails the build.


## Verifying a change


`just ci-light` is the gate. Green means green.


<!-- TODO — what the gate *cannot* catch here, and how it gets checked instead.
     Be specific. "The binary is a full-screen TUI so it can't be driven
     headlessly — UI changes get verified by reading, by `cargo check`, and by
     asking me to run it" is the useful kind. Never claim a change works
     without saying how it was checked. -->
