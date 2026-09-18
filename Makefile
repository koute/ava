.PHONY: book clippy doc fmt install report serve test test-book


book: test-book
	mdbook serve book --open

clippy:
	cargo clippy --all-targets -- -D warnings

doc:
	cargo doc --no-deps --workspace

fmt:
	cargo fmt --all

install:
	cargo install --path crates/ava --locked --force

report: install
	ava report -p max -n tcc-parity-max -n chess-parity-max -n r2wars-parity-max -p high -n tcc-parity -n chess-parity -n r2wars-parity

serve: install
	ava image
	ava serve

test:
	cargo test --workspace

test-book:
	cargo install mdbook --version 0.5.4 --locked
	mdbook test book
