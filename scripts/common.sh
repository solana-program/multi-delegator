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
      just build-program
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

handle_ledger() {
  local reset_ledger="$1"
  local set_skip_init="${2:-false}"

  if [ "$reset_ledger" = true ]; then
    if [ -d "$LEDGER_DIR" ]; then
      echo -e "${YELLOW}Cleaning old ledger directory (--reset flag)...${NC}"
      rm -rf "$LEDGER_DIR"
    fi
  elif [ -d "$LEDGER_DIR" ]; then
    echo -e "${GREEN}Reusing existing ledger (use --reset to start fresh)${NC}"
    if [ "$set_skip_init" = true ]; then
      SKIP_INIT=true
    fi
  fi
}

build_validator_args() {
  local program_id="$1"
  local reset_ledger="$2"

  VALIDATOR_ARGS=(
    --bpf-program "$program_id" "$PROGRAM_SO"
    --clone TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
    --clone TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
    --clone ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL
    --url https://api.mainnet-beta.solana.com
    --ledger "$LEDGER_DIR"
    --rpc-port "$RPC_PORT"
    --account-index program-id
    --account-index spl-token-owner
    --account-index spl-token-mint
  )

  if [ "$reset_ledger" = true ]; then
    VALIDATOR_ARGS+=(--reset)
  fi
}

start_validator_foreground() {
  local program_id="$1"
  local reset_ledger="$2"

  build_validator_args "$program_id" "$reset_ledger"

  echo -e "${GREEN}Starting solana-test-validator...${NC}"
  echo -e "  - Program ID: $program_id"
  echo -e "  - Program SO: $PROGRAM_SO"
  echo -e "  - Ledger: $LEDGER_DIR"
  echo -e "  - RPC Port: $RPC_PORT"
  echo -e "  - Fresh start: $reset_ledger"
  echo ""

  solana-test-validator "${VALIDATOR_ARGS[@]}"
}

start_validator_background() {
  local program_id="$1"
  local reset_ledger="$2"

  build_validator_args "$program_id" "$reset_ledger"

  solana-test-validator "${VALIDATOR_ARGS[@]}" > /tmp/validator.log 2>&1 &
  VALIDATOR_PID=$!
  echo "  Validator PID: $VALIDATOR_PID"
}

wait_for_validator() {
  local timeout="${1:-30}"

  echo -e "${YELLOW}  Waiting for validator to start...${NC}"
  for i in $(seq 1 $timeout); do
    if is_validator_running; then
      echo -e "  ${GREEN}Validator is ready!${NC}"
      return 0
    fi
    if [ $i -eq $timeout ]; then
      echo -e "  ${RED}Validator failed to start within $timeout seconds${NC}"
      echo "  Check /tmp/validator.log for details"
      cat /tmp/validator.log
      return 1
    fi
    sleep 1
  done
}

verify_program_deployed() {
  local program_id="$1"

  if curl -s -X POST http://127.0.0.1:$RPC_PORT \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"$program_id\",{\"encoding\":\"base64\"}]}" \
    | grep -q '"executable":true'; then
    echo -e "  ${GREEN}Program deployed successfully${NC}"
    return 0
  else
    echo -e "  ${RED}Program not found on validator${NC}"
    return 1
  fi
}
