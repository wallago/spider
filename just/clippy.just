# Lint with clippy, treating warnings as errors
[group('quality')]
lint:
    cargo clippy --all-targets -- -D warnings
