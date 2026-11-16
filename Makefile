.PHONY: all check fmt test run

all: check fmt test

check:
	cargo check --workspace

fmt:
	cargo fmt

test: check fmt
	cargo test --workspace

run: check fmt
	cargo run

