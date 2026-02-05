#!/usr/bin/env bash
set -e

# Usage: ./start-webapp.sh [--reset] [--skip-init]
#   --reset     Wipe ledger and start fresh (default: retain state)
#   --skip-init Skip test environment initialization

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$PROJECT_ROOT"

source "$SCRIPT_DIR/common.sh"

VALIDATOR_PID=""
API_PID=""
WEBAPP_PID=""

cleanup() {
  echo ""
  echo -e "${YELLOW}Cleaning up...${NC}"
  [ -n "$WEBAPP_PID" ] && kill $WEBAPP_PID 2>/dev/null || true
  [ -n "$API_PID" ] && kill $API_PID 2>/dev/null || true
  [ -n "$VALIDATOR_PID" ] && kill $VALIDATOR_PID 2>/dev/null || true
  echo -e "${GREEN}Cleanup complete.${NC}"
}
trap cleanup EXIT INT TERM

RESET_LEDGER=false
SKIP_INIT=false
for arg in "$@"; do
  case $arg in
    --reset|--clean) RESET_LEDGER=true ;;
    --skip-init) SKIP_INIT=true ;;
  esac
done

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Multi-Delegator Full Stack Launcher${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "  Reset ledger: ${YELLOW}$RESET_LEDGER${NC}"
echo -e "  Skip init:    ${YELLOW}$SKIP_INIT${NC}"
echo ""

echo -e "${BLUE}[1/6] Checking prerequisites...${NC}"
check_keypair
check_program_so true
PROGRAM_ID=$(get_program_id)
echo -e "  Program ID: ${GREEN}$PROGRAM_ID${NC}"

handle_ledger "$RESET_LEDGER" true

echo ""
echo -e "${BLUE}[2/6] Starting Solana validator...${NC}"
start_validator_background "$PROGRAM_ID" "$RESET_LEDGER"
wait_for_validator 30 || exit 1

echo ""
echo -e "${BLUE}[3/6] Verifying program deployment...${NC}"
verify_program_deployed "$PROGRAM_ID" || exit 1

echo ""
echo -e "${BLUE}[4/6] Initializing test environment...${NC}"
if [ "$SKIP_INIT" = true ]; then
  echo -e "  ${GREEN}Skipping init (reusing existing ledger)${NC}"
else
  cd webapp/scripts && bun install --silent && bun init-test-environment.ts
  cd "$PROJECT_ROOT"
  echo -e "  ${GREEN}Test environment initialized${NC}"
fi

echo ""
echo -e "${BLUE}[5/6] Starting API server...${NC}"
cd webapp/api && bun install --silent
bun run dev > /tmp/api.log 2>&1 &
API_PID=$!
cd "$PROJECT_ROOT"
echo "  API PID: $API_PID"

echo -e "${YELLOW}  Waiting for API server...${NC}"
for i in {1..15}; do
  if curl -s http://localhost:3001/api/health 2>/dev/null | grep -q '"status"'; then
    echo -e "  ${GREEN}API server is ready!${NC}"
    break
  fi
  [ $i -eq 15 ] && echo -e "  ${YELLOW}API server may not be ready yet (continuing anyway)${NC}"
  sleep 1
done

echo ""
echo -e "${BLUE}[6/6] Starting webapp...${NC}"
cd webapp && npm run dev > /tmp/webapp.log 2>&1 &
WEBAPP_PID=$!
cd "$PROJECT_ROOT"
echo "  Webapp PID: $WEBAPP_PID"

echo -e "${YELLOW}  Waiting for webapp...${NC}"
for i in {1..15}; do
  if curl -s http://localhost:5173 2>/dev/null | grep -q 'html'; then
    echo -e "  ${GREEN}Webapp is ready!${NC}"
    break
  fi
  [ $i -eq 15 ] && echo -e "  ${YELLOW}Webapp may not be ready yet (continuing anyway)${NC}"
  sleep 1
done

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  All services started successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "  ${BLUE}Validator:${NC}   http://localhost:$RPC_PORT"
echo -e "  ${BLUE}API Server:${NC}  http://localhost:3001"
echo -e "  ${BLUE}Webapp:${NC}      http://localhost:5173"
echo ""
echo -e "  ${BLUE}Program ID:${NC}  $PROGRAM_ID"
echo ""
echo -e "  ${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

wait
