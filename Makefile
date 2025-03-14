.PHONY: test coverage coverage-html clean

# Test all packages
test:
	cargo test --workspace

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
