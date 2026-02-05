#!/usr/bin/env bash
set -e

# Usage: ./start-validator.sh [--reset]
#   --reset   Wipe ledger and start fresh (default: retain state)

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$PROJECT_ROOT"

source "$SCRIPT_DIR/common.sh"

RESET_LEDGER=false
for arg in "$@"; do
  case $arg in
    --reset|--clean) RESET_LEDGER=true ;;
  esac
done

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Solana Test Validator Launcher${NC}"
echo -e "${GREEN}========================================${NC}"

check_keypair
check_program_so false
PROGRAM_ID=$(get_program_id)
echo -e "${GREEN}Program ID: $PROGRAM_ID${NC}"

if is_validator_running; then
  echo -e "${YELLOW}Validator is already running on port $RPC_PORT${NC}"
  exit 0
fi

handle_ledger "$RESET_LEDGER"
start_validator_foreground "$PROGRAM_ID" "$RESET_LEDGER"
