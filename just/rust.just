# Run the app. Usage: just run -- statement.csv bagels.db
[group('dev')]
run *ARGS:
    cargo run -- --config config.toml {{ ARGS }}

# Build the optimized release binary
[group('dev')]
build:
    cargo build --release

# Type-check the whole workspace (fast feedback, no binary)
[group('dev')]
check:
    cargo check --all-targets

# Format Rust + Nix sources in place
[group('dev')]
fmt:
    cargo fmt
    nixfmt flake.nix package.nix module.nix

# Run the test suite. Usage: just test [pattern]
[group('dev')]
test *ARGS:
    cargo nextest run {{ ARGS }}
