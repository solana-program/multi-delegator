#!/usr/bin/env bash
set -e

# Usage: ./start-validator.sh

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$PROJECT_ROOT"

source "$SCRIPT_DIR/common.sh"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Surfpool Validator Launcher${NC}"
echo -e "${GREEN}========================================${NC}"

check_keypair
check_program_so false
PROGRAM_ID=$(get_program_id)
echo -e "${GREEN}Program ID: $PROGRAM_ID${NC}"

if is_validator_running; then
  echo -e "${YELLOW}Validator is already running on port $RPC_PORT${NC}"
  exit 0
fi

start_validator_foreground "$PROGRAM_ID"
