.PHONY: build build-program build-client test test-program test-and-benchmark test-client clean clean-program clean-client ensure-surfpool generate-client generate-idl kill-validator setup prepare-deploy-keys fmt-check fmt lint fmt-check lint-check

# Setup target to check prerequisites and install dependencies
setup: setup-hooks
	@command -v bun >/dev/null 2>&1 || { echo >&2 "bun is required but not installed. Aborting."; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo >&2 "cargo is required but not installed. Aborting."; exit 1; }
	@command -v shank >/dev/null 2>&1 || { echo >&2 "shank is required but not installed. Aborting."; exit 1; }
	@command -v surfpool >/dev/null 2>&1 || { echo >&2 "surfpool is required but not installed. Aborting."; exit 1; }
	cd client && bun install

setup-hooks:
	@git config core.hooksPath .githooks
	@echo "Git hooks configured."

prepare-deploy-keys:
	@mkdir -p programs/multi_delegator/target/deploy
	@if [ -f keys/multi_delegator-keypair.json ]; then \
		cp keys/multi_delegator-keypair.json programs/multi_delegator/target/deploy/multi_delegator-keypair.json; \
		echo "Deploy key copied to programs/multi_delegator/target/deploy/"; \
	else \
		echo "Key file keys/multi_delegator-keypair.json not found"; \
	fi

# Find all Rust source files for dependency tracking
RUST_SOURCES := $(shell find programs/multi_delegator/src -name '*.rs')

# Output files
SO_FILE := programs/multi_delegator/target/deploy/multi_delegator.so
DEPLOY_KEY_FILE := programs/multi_delegator/target/deploy/multi_delegator-keypair.json
IDL_FILE := programs/multi_delegator/idl/multi_delegator.json
GENERATED_CLIENT := client/src/generated/index.ts

# Deploy key setup - only copies if source is newer than destination (Make timestamp comparison)
$(DEPLOY_KEY_FILE): keys/multi_delegator-keypair.json
	@mkdir -p $(@D)  # $(@D) = directory path of the target file ($@), @ silences command output
	cp $< $@        # $< = first prerequisite (source), $@ = target file (destination)

# Build targets with dependencies
build: setup $(DEPLOY_KEY_FILE) build-program build-client

# Program build with file dependencies - rebuilds if ANY .rs file changes
$(SO_FILE): $(RUST_SOURCES)
	cd programs/multi_delegator && cargo build-sbf

build-program: $(DEPLOY_KEY_FILE) $(SO_FILE)

# IDL generation - rebuilds if ANY .rs file changes
$(IDL_FILE): $(RUST_SOURCES)
	cd programs/multi_delegator && shank idl

generate-idl: $(IDL_FILE)

# Client code generation depends on IDL
$(GENERATED_CLIENT): $(IDL_FILE)
	cd client && bun run generate

generate-client: $(GENERATED_CLIENT)

# Build client with all dependencies
build-client: $(GENERATED_CLIENT)
	cd client && bun run build

# Test targets
test: setup test-program test-client

test-program:
	cd programs/multi_delegator && cargo test-sbf

# Run tests and generate CU benchmark report (writes to programs/multi_delegator/cu_report.md)
test-and-benchmark:
	cd programs/multi_delegator && CU_REPORT=1 cargo test-sbf

# Program ID from keypair (used to verify deployment)
PROGRAM_ID := 3PuMsYqaLY4Sy1DR8np3aAiHravZXCeyMYDUECLqfswY

# Ensure surfpool is running (starts if not running, no-op if already running)
ensure-surfpool:
	@if ! curl -s -X POST http://localhost:8899 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getHealth","params":[]}' > /dev/null 2>&1; then \
		echo "Starting surfpool in background..."; \
		mkdir -p .surfpool; \
		nohup surfpool start --watch --no-tui --block-production-mode transaction > /tmp/surfpool.log 2>&1  & \
		echo $$! > .surfpool/pid.txt; \
		echo "Waiting for surfpool to start..."; \
		for i in 1 2 3 4 5 6 7; do \
			if curl -s -X POST http://localhost:8899 -H "Content-Type: application/json" \
				-d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["$(PROGRAM_ID)",{"encoding":"base64"}]}' \
				| grep -q '"executable":true'; then \
				echo "Program deployed successfully"; \
				break; \
			fi; \
			echo "Waiting for program deployment... ($$i/7)"; \
			sleep 1; \
		done; \
		if [ $$i -eq 7 ]; then \
			echo "Program deployment failed"; \
			echo "Surfpool logs:"; \
			cat /tmp/surfpool.log; \
			exit 1; \
		fi; \
	else \
		echo "validator is already running"; \
	fi

kill-validator:
	@if [ -f .surfpool/pid.txt ]; then \
		pid=$$(cat .surfpool/pid.txt); \
		kill -9 $$pid; \
		if [ $$? -eq 0 ]; then \
			echo "Killed surfpool validator with pid $$pid"; \
			rm -f .surfpool/pid.txt; \
		else \
			echo "Failed to kill process $$pid" >&2; \
			exit 1; \
		fi \
	else \
		echo "No pid file found. Surfpool validator is not running or pid file was not created."; \
	fi

# test-client: builds everything needed, ensures surfpool, then runs tests
test-client: $(SO_FILE) $(GENERATED_CLIENT) ensure-surfpool
	cd client && bun run test

# Clean targets
clean: clean-program clean-client

clean-program:
	cd programs/multi_delegator && cargo clean
	rm programs/multi_delegator/idl/multi_delegator.json

clean-client:
	cd client && bun run clean

# Format and Lint targets
fmt-check:
	@echo "Checking Rust formatting..."
	cd programs/multi_delegator && cargo fmt --check
	@echo "Checking TypeScript formatting..."
	cd client && bun run format:check

fmt:
	@echo "Formatting Rust code..."
	cd programs/multi_delegator && cargo fmt
	@echo "Formatting TypeScript code..."
	cd client && bun run format

lint:
	@echo "Linting Rust code..."
	cd programs/multi_delegator && cargo clippy --all-targets --no-deps --fix -- -D warnings
	@echo "Linting TypeScript code..."
	cd client && bun run lint

lint-check:
	@echo "Linting Rust code..."
	cd programs/multi_delegator && cargo clippy --all-targets --no-deps -- -D warnings
	@echo "Linting TypeScript code..."
	cd client && bun run lint:check

check: fmt-check lint-check

# Default target
.DEFAULT_GOAL := build
