# Multi Delegator - Solana program build automation
# https://github.com/casey/just

# Use bash for all recipes
set shell := ["bash", "-uc"]

# Variables
program_dir := "programs/multi_delegator"
ts_client_dir := "clients/typescript"
webapp_dir := "webapp"
deploy_key := "keys/multi_delegator-keypair.json"
target_deploy_key := "target/deploy/multi_delegator-keypair.json"
idl_file := program_dir / "idl/multi_delegator.json"
program_id := "3PuMsYqaLY4Sy1DR8np3aAiHravZXCeyMYDUECLqfswY"

# List available recipes
default:
    @just --list

# ============================================
# Setup and initialization
# ============================================

# Install dependencies and configure git hooks
setup: setup-hooks
    #!/usr/bin/env bash
    set -euo pipefail

    commands=(pnpm cargo solana-keygen surfpool)
    for cmd in "${commands[@]}"; do
        if ! command -v "$cmd" &>/dev/null; then
            echo "Error: $cmd is required but not installed"
            exit 1
        fi
    done

    pnpm install
    echo "✓ Setup complete"

# Configure git hooks path
setup-hooks:
    git config core.hooksPath .githooks
    @echo "✓ Git hooks configured"

# Copy deployment keypair to build directory
prepare-deploy-keys:
    #!/usr/bin/env bash
    set -euo pipefail

    mkdir -p "target/deploy"

    if [[ -f "{{deploy_key}}" ]]; then
        cp "{{deploy_key}}" "{{target_deploy_key}}"
        echo "✓ Deploy key copied"
    else
        echo "Error: {{deploy_key}} not found"
        exit 1
    fi

# ============================================
# Build recipes
# ============================================

# Check if rebuild is needed (exits 0 if rebuild needed, 1 if up-to-date)
[private]
needs-rebuild target source:
    #!/usr/bin/env bash
    if [[ -f "{{target}}" ]] && [[ "{{source}}" -ot "{{target}}" ]]; then
        echo "✓ {{target}} is up-to-date"
        exit 1
    fi

# Build everything (program + clients + webapp)
build: prepare-deploy-keys build-program build-client build-webapp

# Compile Solana program to .so
build-program: prepare-deploy-keys
    cd {{program_dir}} && cargo build-sbf
    @echo "✓ Program built"

# Generate IDL from Rust source
generate-idl:
    #!/usr/bin/env bash
    set -euo pipefail

    if just needs-rebuild "{{idl_file}}" "{{program_dir}}/src/instructions/mod.rs" 2>/dev/null; then
        cd {{program_dir}}
        cargo build
        echo "✓ IDL generated"
    fi

# Generate TypeScript and Rust clients from IDL
generate-client: generate-idl
    #!/usr/bin/env bash
    set -euo pipefail

    if just needs-rebuild "clients/typescript/src/generated/index.ts" "{{idl_file}}" 2>/dev/null; then
        pnpm run generate
        echo "✓ Clients generated"
    fi

# Build TypeScript client
build-client: generate-client
    #!/usr/bin/env bash
    set -euo pipefail

    if just needs-rebuild "{{ts_client_dir}}/dist/index.js" "clients/typescript/src/generated/index.ts" 2>/dev/null; then
        cd {{ts_client_dir}}
        pnpm run build
        echo "✓ TypeScript client built"
    fi

# Build webapp
build-webapp: generate-idl
    cd {{webapp_dir}} && npm install && npm run build
    @echo "✓ Webapp built"

# ============================================
# Test recipes
# ============================================

# Run all tests
test: test-program test-client

# Run Rust program tests
test-program:
    cd {{program_dir}} && cargo test-sbf

# Run tests with compute unit benchmark report
test-and-benchmark:
    cd {{program_dir}} && CU_REPORT=1 cargo test-sbf

# Run TypeScript client integration tests
test-client: build-program generate-client ensure-surfpool
    cd {{ts_client_dir}} && pnpm run test

# ============================================
# Validator management
# ============================================

# Start surfpool validator if not already running
ensure-surfpool:
    #!/usr/bin/env bash
    set -euo pipefail

    # Check if validator is already running
    if curl -sf -X POST http://localhost:8899 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' &>/dev/null; then
        echo "✓ Validator already running"
        exit 0
    fi

    echo "Starting surfpool validator..."
    mkdir -p .surfpool
    nohup surfpool start --ci --no-tui --block-production-mode transaction \
        > /tmp/surfpool.log 2>&1 &
    echo $! > .surfpool/pid.txt

    # Wait for program deployment
    for i in {1..7}; do
        if curl -sf -X POST http://localhost:8899 \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["{{program_id}}",{"encoding":"base64"}]}' \
            | grep -q '"executable":true'; then
            echo "✓ Program deployed successfully"
            exit 0
        fi
        echo "Waiting for program deployment... ($i/7)"
        sleep 1
    done

    echo "Error: Program deployment failed"
    echo "Surfpool logs:"
    cat /tmp/surfpool.log
    just kill-surfpool
    exit 1

# Stop surfpool validator
kill-surfpool:
    #!/usr/bin/env bash
    set -euo pipefail

    if [[ ! -f .surfpool/pid.txt ]]; then
        echo "No surfpool validator running"
        exit 0
    fi

    pid=$(cat .surfpool/pid.txt)
    if kill -9 "$pid" 2>/dev/null; then
        echo "✓ Killed surfpool validator (PID: $pid)"
        rm -f .surfpool/pid.txt
    else
        echo "Warning: Could not kill process $pid (may already be stopped)"
        rm -f .surfpool/pid.txt
    fi

# Stop all validators
kill-validator: kill-surfpool
    @killall -9 solana-test-validator 2>/dev/null || true
    @killall -9 surfpool 2>/dev/null || true
    @rm -f .surfpool/pid.txt 2>/dev/null || true
    @rm -rf .validator-ledger 2>/dev/null || true
    @echo "✓ All validators stopped"

# ============================================
# Webapp recipes
# ============================================

# Start webapp development stack with optional flags
webapp reset="false" skip_init="false": build-program
    #!/usr/bin/env bash
    set -euo pipefail

    args=()
    [[ "{{reset}}" == "true" ]] && args+=(--reset)
    [[ "{{skip_init}}" == "true" ]] && args+=(--skip-init)

    ./scripts/start-webapp.sh "${args[@]}"

# ============================================
# Clean recipes
# ============================================

# Clean all build artifacts
clean: clean-program clean-client

# Clean Rust build artifacts and IDL
clean-program:
    cargo clean
    rm -f {{idl_file}}
    @echo "✓ Program cleaned"

# Clean TypeScript client artifacts
clean-client:
    cd {{ts_client_dir}} && pnpm run clean
    @echo "✓ Client cleaned"

# Clean webapp and validator files
clean-webapp:
    @echo "Cleaning webapp..."
    @pkill -f "solana-test-validator" 2>/dev/null || true
    @pkill -f "surfpool" 2>/dev/null || true
    @rm -rf {{webapp_dir}}/{node_modules,dist,api/node_modules,scripts/node_modules}
    @rm -rf .{validator-ledger,surfpool}
    @rm -f /tmp/surfpool.log
    @echo "✓ Webapp cleaned"

# ============================================
# Format and lint recipes
# ============================================

# Check formatting without fixing
fmt-check:
    @echo "Checking Rust formatting..."
    @cargo fmt -p multi-delegator --check
    @echo "Checking TypeScript formatting..."
    @cd {{ts_client_dir}} && pnpm run format:check
    @echo "✓ Format check passed"

# Auto-format all code
fmt:
    @echo "Formatting Rust..."
    @cargo fmt -p multi-delegator
    @echo "Formatting TypeScript..."
    @cd {{ts_client_dir}} && pnpm run format
    @echo "✓ Code formatted"

# Lint with auto-fix
lint:
    @echo "Linting Rust..."
    @cargo clippy --workspace --exclude multidelegator-client --all-targets --no-deps --fix -- -D warnings
    @echo "Linting TypeScript..."
    @cd {{ts_client_dir}} && pnpm run lint
    @echo "✓ Code linted"

# Check linting without fixing
lint-check:
    @echo "Checking Rust lint..."
    @cargo clippy --workspace --exclude multidelegator-client --all-targets --no-deps -- -D warnings
    @echo "Checking TypeScript lint..."
    @cd {{ts_client_dir}} && pnpm run lint:check
    @echo "✓ Lint check passed"

# Run all code quality checks
check: fmt-check lint-check
