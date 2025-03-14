.PHONY: test coverage coverage-html clean test-integration test-unit test-all

# Test all packages
test:
	cargo test --workspace

# Run unit tests only
test-unit:
	cargo test --lib --workspace

# Run integration tests only
test-integration:
	cargo test --test '*' --workspace

# Run all tests with features
test-all:
	cargo test --all-features --workspace

# Run coverage report
coverage:
	cargo tarpaulin --workspace

# Run coverage with HTML report and 75% threshold
coverage-html:
	cargo tarpaulin --all-features --workspace --timeout 120 --out Html --fail-under 75

# Clean build artifacts
clean:
	cargo clean
	rm -f tarpaulin-report.html
