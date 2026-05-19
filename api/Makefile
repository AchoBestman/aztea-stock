# ─── Aztea Stock API — Dev Commands ─────────────────────────────────────────

## Start API with hot-reload (watches src/ for changes)
dev:
	cargo watch -x run

## Run all tests
test:
	cargo test

## Apply all migrations + seeds (fresh setup)
setup:
	cargo run --bin db setup

## Run pending migrations only
migrate:
	cargo run --bin db migrate

## Rollback last migration
rollback:
	cargo run --bin db rollback

## Seed database
seed:
	cargo run --bin db seed

## Generate a new timestamped migration file pair (usage: make migration NAME=add_foo_to_bar)
migration:
	@ts=$$(date +%Y%m%d%H%M%S); \
	echo "-- Up migration" > migrations/$${ts}_$(NAME).sql; \
	echo "-- Down migration" > migrations/$${ts}_$(NAME).down.sql; \
	echo "✅ Created: migrations/$${ts}_$(NAME).sql  &  migrations/$${ts}_$(NAME).down.sql"

## Build release binary
build:
	cargo build --release

## Check code (no build artifacts)
check:
	cargo check --tests

.PHONY: dev test setup migrate rollback seed migration build check
