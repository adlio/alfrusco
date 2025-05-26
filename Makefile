workflow: release
	cargo build --all-targets --release && \
	cp target/release/examples/random_user workflow/ && \
	cp target/release/examples/sleep workflow/ && \
	cp target/release/examples/url_items workflow/ && \
	cp target/release/examples/static_output workflow/

build:
	cargo build --all-targets --all-features --examples

test:
	cargo test --all-targets --all-features --examples

release:
	cargo build --all-targets --all-features --examples --release

clean:
	cargo clean
	cargo llvm-cov clean

coverage: check-llvm-cov
	cargo llvm-cov --all-features --examples --tests --show-missing-lines

coverage-html: check-llvm-cov
	cargo llvm-cov --all-features --examples --tests --html --open

coverage-ci: check-llvm-cov
	cargo llvm-cov --all-features --examples --lcov --output-path lcov.info

check-llvm-cov:
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "cargo-llvm-cov is not installed. Install it with: cargo install cargo-llvm-cov"; exit 1; }

fmt:
	cargo +nightly fmt --all

fmt-check:
	cargo +nightly fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features --examples -- -D warnings

clippy-fix:
	cargo clippy --all-targets --all-features --examples --fix -- -D warnings

lint: fmt clippy-fix

.PHONY: all test clean coverage coverage-html coverage-ci check-llvm-cov release fmt fmt-check clippy clippy-fix lint