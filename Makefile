.PHONY: book clippy doc fmt install push-report serve test test-book

PAGES ?= ../ava-pages
REPORT ?= reports/report.html

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

push-report:
	cp $(REPORT) $(PAGES)/index.html
	git -C $(PAGES) commit -am "report $$(date -u +%F)"
	git -C $(PAGES) push

serve: install
	ava image
	ava serve

test:
	cargo test --workspace

test-book:
	cargo install mdbook --version 0.5.4 --locked
	mdbook test book
