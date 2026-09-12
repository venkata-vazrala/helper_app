.PHONY: run test fmt clippy ci app install

run:
	cargo run --release

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

ci:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo test --locked

# Build dist/Helper.app — double-click to run, no cargo needed after this.
app:
	./scripts/package.sh

# Optional: copy the app into /Applications.
install: app
	./scripts/install.sh
