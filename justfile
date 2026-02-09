# Default: run checks
default: check

# Build the library
build:
    cargo build

# Build with all features
build-all:
    cargo build --all-features

# Run all tests
test:
    cargo test

# Run tests with all features enabled
test-all:
    cargo test --all-features

# Run clippy lints
clippy:
    cargo clippy --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Full check: fmt, clippy, test
check: fmt-check clippy test-all

# Generate and open docs
doc:
    cargo doc --all-features --open

# Run the basic message example
example-message:
    cargo run --example message

# Run the streaming example
example-streaming:
    cargo run --example streaming

# Run the tool use example
example-tools:
    cargo run --example tools

# Clean build artifacts
clean:
    cargo clean
