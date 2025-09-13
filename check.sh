#!/bin/bash

echo "Running pre-commit checks..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Cargo not found, skipping Rust checks"
    exit 0
fi

# Run clippy
echo "Running clippy..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy failed! Fix the warnings before committing."
    exit 1
fi

# Run tests
echo "Running tests..."
if ! cargo test; then
    echo "❌ Tests failed! Fix the failing tests before committing."
    exit 1
fi

# Check formatting
echo "Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting check failed! Run 'cargo fmt' to fix formatting."
    exit 1
fi

echo "✅ All checks passed! Ready to commit."
exit 0
