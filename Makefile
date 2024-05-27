.PHONY: test coverage

test:
	cargo test

coverage:
	cargo tarpaulin --exclude-files tests/* --out Html