# ── CI ────────────────────────────────────────────────────────

# act's default image ships no Rust and no sqlite3; full-latest mirrors the
# real GitHub runner, so a local run predicts the hosted one. act also leaves
# USER unset, which trips scripts running under `set -u`; the image runs as
# uid 1001 (runner), which is what the hosted runner reports too.
act_flags := '-W .github/workflows/ci.yml -P ubuntu-latest=catthehacker/ubuntu:full-latest --env USER=runner --env GIT_PAGER=cat --env PAGER=cat'

# Mirror the full CI pipeline locally. Run before pushing.
[group('ci')]
ci-light:
    @echo "▶ format"
    cargo fmt --check
    @echo "▶ clippy"
    cargo clippy --all-targets -- -D warnings
    @echo "▶ test"
    cargo nextest run
    @echo "▶ doc"
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
    @echo "▶ typos"
    typos
    @echo "▶ unused deps"
    cargo machete
    @echo "▶ deny"
    cargo deny check
    @echo "▶ audit"
    cargo audit
    @echo "▶ toml format"
    taplo fmt --check --diff
    @echo "▶ file format"
    editorconfig-checker
    @echo "▶ commits"
    committed -vv HEAD
    @echo "▶ lychee"
    lychee --verbose --no-progress '*.md'
    @echo "▶ nix lint"
    statix check -i templates .
    deadnix --fail --exclude templates .
    @echo "▶ flake check"
    nix flake check --print-build-logs --all-systems
    @echo "✅ CI mirror passed"

# Run the full CI pipeline locally. Run before pushing.
[group('ci')]
ci-deep:
    act {{ act_flags }} -s GITHUB_TOKEN="$(gh auth token)"

# Run a single CI job locally. Usage: just ci-job msrv
[group('ci')]
ci-job JOB:
    act {{ act_flags }} -j {{ JOB }} -s GITHUB_TOKEN="$(gh auth token)"

# List the CI job IDs
[group('ci')]
ci-jobs:
    act -l -W .github/workflows/ci.yml
