.PHONY: book clippy doc fmt install push-report serve test test-book

PAGES ?= ../ava-pages
SITE ?= reports

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
	rsync -a --delete --exclude .git --exclude .nojekyll $(SITE)/ $(PAGES)/
	git -C $(PAGES) add -A
	git -C $(PAGES) commit -m "report $$(date -u +%F)"
	git -C $(PAGES) push

serve: install
	ava image
	ava serve

test:
	cargo test --workspace

test-book:
	cargo install mdbook --version 0.5.4 --locked
	mdbook test book
