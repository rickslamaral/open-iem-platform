.DEFAULT_GOAL := help
SHELL := /usr/bin/env bash

PROJECT := open-iem-platform
SERVER_MANIFEST := server/Cargo.toml

.PHONY: help install run run-local up down logs status lint fmt test test-unit test-integration test-audio build package diagnostics docs validate clean coverage

help:
	@printf '%s\n' 'Open IEM Platform developer targets:'
	@printf '%s\n' '  make install          validate required local tools'
	@printf '%s\n' '  make run              run API server natively (requires config)'
	@printf '%s\n' '  make run-local        run API server with local development configuration'
	@printf '%s\n' '  make up/down/logs     manage project-owned compose services (up rebuilds images)'
	@printf '%s\n' '  make status           report repository and build state'
	@printf '%s\n' '  make lint/fmt         lint Rust and typecheck/format supported frontends'
	@printf '%s\n' '  make test             run Rust and frontend tests, including simulated audio harness'
	@printf '%s\n' '  make test-audio       run deterministic SIMULATED audio integration tests'
	@printf '%s\n' '  make build            build Rust and frontend artifacts'
	@printf '%s\n' '  make package          report package outputs (none until release packaging)'
	@printf '%s\n' '  make diagnostics      report environment validation state'
	@printf '%s\n' '  make docs/validate     validate documentation and project skills'
	@printf '%s\n' '  make clean            remove generated build outputs only'
	@printf '%s\n' '  make coverage         generate Rust line/function coverage report (requires cargo-llvm-cov)'

install:
	@command -v cargo >/dev/null || { echo 'MISSING: cargo'; exit 3; }
	@command -v npm >/dev/null || { echo 'MISSING: npm'; exit 3; }
	@command -v node >/dev/null || { echo 'MISSING: node'; exit 3; }
	@echo 'OK: cargo, node and npm available'

run:
	@echo 'Native server requires OPENIEM_* configuration; use: cargo run --manifest-path $(SERVER_MANIFEST) --bin api-server'
	@cargo run --manifest-path $(SERVER_MANIFEST) --bin api-server

run-local:
	@echo 'Starting API server with current local development environment; audio remains SIMULATED.'
	@cargo run --manifest-path $(SERVER_MANIFEST) --bin api-server

up:
	@docker compose up -d --build

down:
	@docker compose down

logs:
	@docker compose logs --tail=200

status:
	@git status --short
	@printf 'Rust: '; cargo --version
	@printf 'Node: '; node --version
	@printf 'Frontend outputs: '; test -d web/musician/dist && test -d web/engineer/dist && echo 'present' || echo 'not built'

fmt:
	@cargo fmt --manifest-path $(SERVER_MANIFEST) --all

lint:
	@cargo fmt --manifest-path $(SERVER_MANIFEST) --all -- --check
	@cargo clippy --manifest-path $(SERVER_MANIFEST) --all-targets -- -D warnings
	@npm run typecheck --prefix web/musician
	@npm run typecheck --prefix web/engineer

test: test-unit test-integration test-audio
	@python3 -m pytest -q tests/test_validate_version.py
	@npm test --prefix web/musician -- --run
	@npm test --prefix web/engineer -- --run

test-unit:
	@cargo test --manifest-path $(SERVER_MANIFEST) --workspace --lib

test-integration:
	@cargo test --manifest-path $(SERVER_MANIFEST) --package api-server --test integration

test-audio:
	@cargo test --manifest-path $(SERVER_MANIFEST) --package audio-engine --test deterministic_harness

build:
	@cargo build --manifest-path $(SERVER_MANIFEST) --workspace
	@npm run build --prefix web/musician
	@npm run build --prefix web/engineer

package:
	@echo 'No release package produced: release packaging remains gated by CI and release workflow.'
	@echo 'See .github/workflows/release.yml.'

diagnostics:
	@bash scripts/validate-environment.sh

docs:
	@bash scripts/validate-docs.sh
	@bash scripts/validate-pdf.sh

validate: lint test docs
	@bash scripts/validate-skills.sh
	@git diff --check

clean:
	@cargo clean --manifest-path $(SERVER_MANIFEST)
	@rm -rf web/musician/dist web/engineer/dist
	@echo 'Generated build outputs removed; source and user configuration preserved.'

coverage:
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { echo 'MISSING: cargo-llvm-cov. Install with: cargo install cargo-llvm-cov --locked'; exit 3; }
	@echo 'Running Rust coverage (cargo-llvm-cov)...'
	@cargo llvm-cov --manifest-path $(SERVER_MANIFEST) --all --summary-only
	@cargo llvm-cov --manifest-path $(SERVER_MANIFEST) --all --lcov --output-path coverage/lcov.info
	@test -s coverage/lcov.info
	@echo 'Coverage report written to coverage/lcov.info'
