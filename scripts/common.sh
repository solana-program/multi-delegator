#!/usr/bin/env bash
# Shared configuration and functions for validator scripts

# Configuration (can be overridden via environment)
KEYPAIR_FILE="${KEYPAIR_FILE:-keys/multi_delegator-keypair.json}"
PROGRAM_SO="${PROGRAM_SO:-target/deploy/multi_delegator.so}"
LEDGER_DIR="${LEDGER_DIR:-.validator-ledger}"
RPC_PORT="${RPC_PORT:-8899}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

check_keypair() {
  if [ ! -f "$KEYPAIR_FILE" ]; then
    echo -e "${RED}Error: Keypair file not found: $KEYPAIR_FILE${NC}"
    exit 1
  fi
}

check_program_so() {
  local build_if_missing="${1:-false}"
  if [ ! -f "$PROGRAM_SO" ]; then
    if [ "$build_if_missing" = true ]; then
      echo -e "${YELLOW}Program SO file not found. Building...${NC}"
      SCRIPT_DIR_COMMON="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
      PROJECT_ROOT_COMMON="$( cd "$SCRIPT_DIR_COMMON/.." && pwd )"
      cd "$PROJECT_ROOT_COMMON/programs/multi_delegator" && cargo build-sbf
      cd "$PROJECT_ROOT_COMMON"
    else
      echo -e "${YELLOW}Warning: Program SO file not found: $PROGRAM_SO${NC}"
      echo -e "${YELLOW}Run 'just build-program' first to build the program.${NC}"
      exit 1
    fi
  fi
}

get_program_id() {
  solana-keygen pubkey "$KEYPAIR_FILE"
}

is_validator_running() {
  curl -s -X POST http://127.0.0.1:$RPC_PORT \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' 2>/dev/null | grep -q '"result":"ok"'
}

build_surfpool_args() {
  SURFPOOL_ARGS=(
    --no-tui
    --port "$RPC_PORT"
    --offline
  )
}

start_validator_foreground() {
  local program_id="$1"

  build_surfpool_args

  echo -e "${GREEN}Starting surfpool...${NC}"
  echo -e "  - Program ID: $program_id"
  echo -e "  - Program SO: $PROGRAM_SO"
  echo -e "  - RPC Port: $RPC_PORT"
  echo ""

  surfpool start "${SURFPOOL_ARGS[@]}"
}

start_validator_background() {
  local program_id="$1"

  build_surfpool_args

  surfpool start "${SURFPOOL_ARGS[@]}" > /tmp/validator.log 2>&1 &
  VALIDATOR_PID=$!
  echo "  Validator PID: $VALIDATOR_PID"
}

# Waits for RPC to respond to getHealth. Returns 1 on timeout.
wait_for_validator() {
  local timeout="${1:-30}"

  echo -e "${YELLOW}  Waiting for validator to start...${NC}"
  for i in $(seq 1 $timeout); do
    if is_validator_running; then
      echo -e "  ${GREEN}Validator is ready!${NC}"
      return 0
    fi
    if [ $i -eq $timeout ]; then
      echo -e "  ${RED}Validator failed to start within ${timeout}s${NC}"
      echo "  Check /tmp/validator.log for details"
      cat /tmp/validator.log 2>/dev/null || true
      return 1
    fi
    sleep 1
  done
}

# Polls until program account is executable. Returns 1 on timeout.
wait_for_program() {
  local program_id="$1"
  local timeout="${2:-30}"

  echo -e "${YELLOW}  Waiting for program deployment...${NC}"
  for i in $(seq 1 $timeout); do
    if curl -s -X POST http://127.0.0.1:$RPC_PORT \
      -H "Content-Type: application/json" \
      -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"$program_id\",{\"encoding\":\"base64\"}]}" \
      | grep -q '"executable":true'; then
      echo -e "  ${GREEN}Program deployed successfully${NC}"
      return 0
    fi
    if [ $i -eq $timeout ]; then
      echo -e "  ${RED}Program not found after ${timeout}s${NC}"
      echo "  Check /tmp/validator.log for details"
      cat /tmp/validator.log 2>/dev/null | tail -20 || true
      return 1
    fi
    sleep 1
  done
}

# Polls a local HTTP endpoint until it responds. Returns 1 on timeout.
wait_for_http() {
  local url="$1"
  local label="$2"
  local timeout="${3:-15}"
  local match="${4:-}"

  echo -e "${YELLOW}  Waiting for ${label}...${NC}"
  for i in $(seq 1 $timeout); do
    if [ -n "$match" ]; then
      curl -s "$url" 2>/dev/null | grep -q "$match" && break
    else
      curl -sf "$url" >/dev/null 2>&1 && break
    fi
    if [ $i -eq $timeout ]; then
      echo -e "  ${RED}${label} failed to start within ${timeout}s${NC}"
      return 1
    fi
    sleep 1
  done
  echo -e "  ${GREEN}${label} is ready!${NC}"
  return 0
}
