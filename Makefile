SHELL := /bin/bash
.DEFAULT_GOAL := help

.PHONY: help setup env tracker test test-backend test-cli test-frontend test-tracker check build up down logs clean deploy

help:
	@printf '%s\n' \
	  'make setup          Install frontend and tracker dependencies' \
	  'make env            Generate .env with secure random secrets' \
	  'make test           Run every automated test' \
	  'make check          Format/lint/type-check all code' \
	  'make build          Build backend, CLI, tracker, and frontend' \
	  'make deploy         Back up, deploy, and verify production' \
	  'make up             Start the production-like Docker stack' \
	  'make down           Stop the Docker stack' \
	  'make logs           Follow Docker logs'

setup:
	npm --prefix tracker ci
	npm --prefix frontend ci

env:
	./scripts/generate-env.sh

tracker:
	npm --prefix tracker run build
	cp tracker/dist/slimlytics.js frontend/static/tracker.js
	cp tracker/dist/slimlytics.js frontend/static/s.js
	mkdir -p frontend/src/lib/server/generated
	cp tracker/dist/slimlytics.js frontend/src/lib/server/generated/tracker.iife.txt

test: test-backend test-cli test-tracker test-frontend

test-backend:
	cargo test --manifest-path backend/Cargo.toml --all-targets

test-cli:
	cargo test --manifest-path cli/Cargo.toml --all-targets

test-tracker:
	npm --prefix tracker test -- --run

test-frontend:
	npm --prefix frontend test -- --run

check:
	cargo fmt --manifest-path backend/Cargo.toml --all -- --check
	cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
	cargo fmt --manifest-path cli/Cargo.toml --all -- --check
	cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings
	sh -n scripts/install-cli.sh
	bash -n scripts/deploy-production.sh
	npm --prefix tracker run check
	npm --prefix frontend run check

build: tracker
	cargo build --manifest-path backend/Cargo.toml --release --locked
	cargo build --manifest-path cli/Cargo.toml --release --locked
	npm --prefix frontend run build
	node frontend/scripts/verify-api-docs-build.mjs

deploy:
	./scripts/deploy-production.sh

up:
	@test -f .env || (echo 'Copy .env.example to .env and replace every placeholder first.' >&2; exit 1)
	docker compose up --build -d

down:
	docker compose down

logs:
	docker compose logs -f

clean:
	cargo clean --manifest-path backend/Cargo.toml
	cargo clean --manifest-path cli/Cargo.toml
	rm -rf frontend/.svelte-kit frontend/build tracker/dist frontend/static/tracker.js frontend/static/s.js
