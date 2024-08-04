.PHONY: build build-examples test coverage

test:
	cargo test

build:
	cargo build --all

static_output_example: build
	./target/debug/examples/static_output

target/debug/examples/url_items: build
url_items_example: target/debug/examples/url_items
	./target/debug/examples/url_items

coverage:
	cargo tarpaulin --exclude-files tests/* --out Html