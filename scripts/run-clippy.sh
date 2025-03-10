#!/bin/bash

# Standard Clippy check without auto-fixing
echo "Running Clippy checks..."
cargo clippy -- -D warnings -A clippy::uninlined_format_args -A clippy::redundant_closure_for_method_calls

# Optional: Run with specific lint allowing
# echo "Running Clippy checks with specific lint allowing..."
# cargo clippy -- -A clippy::uninlined_format_args -A clippy::redundant_closure_for_method_calls -D warnings

# For auto-fixing, but excluding certain lints
echo "Running Clippy auto-fix..."
cargo fix --allow-dirty --allow-staged

echo "Clippy process completed." 