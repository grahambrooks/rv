# Developer targets. `make check` is the pre-push gate and runs exactly what the
# CI `check` job runs (.github/workflows/ci.yml); `make hooks` wires it into git.
.DEFAULT_GOAL := help

.PHONY: help check fmt fmt-check lint test hooks

help: ## List targets
	@grep -hE '^[a-zA-Z0-9_-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{ printf "  %-16s %s\n", $$1, $$2 }'

check: fmt-check lint test ## The pre-push gate: the CI check job (fmt, clippy, test)

fmt: ## Format the code
	cargo fmt --all

fmt-check: ## Fail if the code is not formatted
	cargo fmt --all --check

lint: ## Clippy with warnings denied, as CI runs it
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

test: ## Tests, as CI runs them
	cargo test --workspace --all-features --locked

hooks: ## Install the prek git hooks (pre-commit and pre-push)
	prek install

include release.mk
