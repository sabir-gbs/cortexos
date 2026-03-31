.PHONY: build test lint fmt dev setup check-deps migrate

build:
	cargo build --workspace
	pnpm build

test:
	cargo test --workspace
	pnpm test

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	pnpm lint

fmt:
	cargo fmt --all --
	pnpm format

fmt-check:
	cargo fmt --all -- --check
	pnpm format:check

typecheck:
	pnpm typecheck

dev:
	bash scripts/dev.sh

setup:
	bash scripts/setup.sh

check-deps:
	cargo run --bin check-deps

migrate:
	bash scripts/migrate.sh

ci: fmt-check lint typecheck build test check-deps
