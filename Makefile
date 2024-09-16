.PHONY: build build-examples test coverage workflow

test:
	cargo test --all-targets --all-features

workflow:
	cargo build --all-targets --release && \
	cp target/release/examples/sleep workflow/ && \
	cp target/release/examples/url_items workflow/ && \
	cp target/release/examples/static_output workflow/

build:
	cargo build --all-targets --all-features --examples

release:
	cargo build --all-targets --all-features --examples --release

static_output_example: build
	./target/debug/examples/static_output

target/debug/examples/url_items: build
url_items_example: target/debug/examples/url_items	
	alfred_workflow_data=./test_workflow/workflow_data && \
	alfred_workflow_cache=./test_workflow/workflow_cache && \
	./target/debug/examples/url_items example | jq

coverage:
	cargo tarpaulin --all-targets --all-features --examples --exclude-files tests/* --out Html