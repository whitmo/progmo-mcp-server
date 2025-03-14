.PHONY: test coverage coverage-html clean

# Test all packages
test:
	cargo test --workspace

# Run coverage without failing on threshold
coverage:
	cargo tarpaulin --verbose --workspace --no-fail-under

# Run coverage with HTML report and 75% threshold
coverage-html:
	cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Html --fail-under 75

# Clean build artifacts
clean:
	cargo clean
	rm -f tarpaulin-report.html
