# Alfrusco Build System
# Requires: cargo-nextest, cargo-llvm-cov (auto-installed if missing)

.DEFAULT_GOAL := help

.PHONY: help test test-without-nextest build release workflow clean coverage coverage-html coverage-ci lint fmt fmt-check clippy clippy-fix doc doc-check check all ci ensure-tools

# Tool installation helpers
CARGO_NEXTEST := $(shell command -v cargo-nextest 2>/dev/null)
CARGO_LLVM_COV := $(shell command -v cargo-llvm-cov 2>/dev/null)

ensure-tools: ## Install required cargo tools if missing
ifndef CARGO_NEXTEST
	@echo "Installing cargo-nextest..."
	@cargo install cargo-nextest --locked
endif
ifndef CARGO_LLVM_COV
	@echo "Installing cargo-llvm-cov..."
	@cargo install cargo-llvm-cov --locked
endif

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

test: ensure-tools ## Run tests with nextest
	@# Clipboard tests require --test-threads 1 (shared resource)
	cargo nextest run --all-targets --all-features --examples --test-threads 1

test-without-nextest: ## Run tests with cargo test (fallback)
	@# Clipboard tests require --test-threads 1 (shared resource)
	cargo test --all-targets --all-features --examples -- --test-threads 1

build: ## Build debug
	cargo build --all-targets --all-features --examples

release: ## Build release
	cargo build --all-targets --all-features --examples --release

workflow: release ## Build and copy examples to workflow/
	cp target/release/examples/random_user workflow/
	cp target/release/examples/sleep workflow/
	cp target/release/examples/url_items workflow/
	cp target/release/examples/static_output workflow/

run-example-%: ## Run example (e.g., make run-example-sleep)
	cargo run --example $*

check: ## Run cargo check
	cargo check --all-targets --all-features --examples

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting
	cargo fmt --all -- --check

clippy: ## Run clippy
	cargo clippy --all-targets --all-features --examples -- -D warnings

clippy-fix: ## Run clippy with auto-fix
	cargo clippy --all-targets --all-features --examples --fix -- -D warnings

lint: fmt clippy ## Format and lint

clean: ## Clean build artifacts
	cargo clean
	cargo llvm-cov clean

doc: ## Generate docs
	cargo doc --no-deps --open

doc-check: ## Check docs build without warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

coverage: ensure-tools ## Show coverage summary in console
	@# Clipboard tests require --test-threads 1 (shared resource)
	cargo llvm-cov --all-features --examples --tests --show-missing-lines -- --test-threads 1

coverage-html: ensure-tools ## Generate HTML coverage report and open
	@# Clipboard tests require --test-threads 1 (shared resource)
	cargo llvm-cov --all-features --examples --tests --html --open -- --test-threads 1

coverage-ci: ensure-tools ## Generate LCOV coverage for CI
	@# Clipboard tests require --test-threads 1 (shared resource)
	cargo llvm-cov --all-features --examples --lcov --output-path lcov.info -- --test-threads 1

all: ensure-tools fmt lint build test ## Format, lint, build, and test

ci: ensure-tools fmt-check lint build doc-check test ## Full CI pipeline (for hooks)
