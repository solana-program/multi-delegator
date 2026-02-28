#!/usr/bin/env bash
set -e

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
  pkill -f "surfpool" 2>/dev/null || true
  echo -e "${GREEN}Cleanup complete.${NC}"
}
trap cleanup EXIT INT TERM

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Multi-Delegator Full Stack Launcher${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

echo -e "${BLUE}[1/9] Preparing deploy keys...${NC}"
mkdir -p "target/deploy"
if [[ -f "$KEYPAIR_FILE" ]]; then
  cp "$KEYPAIR_FILE" "target/deploy/multi_delegator-keypair.json"
  echo -e "  ${GREEN}Deploy key copied${NC}"
else
  echo -e "  ${RED}Error: $KEYPAIR_FILE not found${NC}"
  exit 1
fi

echo ""
echo -e "${BLUE}[2/9] Checking prerequisites...${NC}"
check_keypair
check_program_so true
PROGRAM_ID=$(get_program_id)
echo -e "  Program ID: ${GREEN}$PROGRAM_ID${NC}"

echo ""
echo -e "${BLUE}[3/9] Building client library...${NC}"
cd clients/typescript && pnpm install --silent && pnpm run build
cd "$PROJECT_ROOT"
echo -e "  ${GREEN}Client built${NC}"

echo ""
echo -e "${BLUE}[4/9] Installing webapp dependencies...${NC}"
cd webapp && pnpm install --silent
cd "$PROJECT_ROOT"

echo ""
echo -e "${BLUE}[5/9] Building webapp...${NC}"
cd webapp && pnpm run build
cd "$PROJECT_ROOT"
echo -e "  ${GREEN}Webapp built${NC}"

echo ""
echo -e "${BLUE}[6/9] Starting surfpool...${NC}"
if is_validator_running; then
  echo -e "  ${GREEN}Surfpool already running on port $RPC_PORT${NC}"
else
  pkill -f "surfpool" 2>/dev/null || true
  pkill -f "solana-test-validator" 2>/dev/null || true
  sleep 1
  start_validator_background "$PROGRAM_ID"
  wait_for_validator 30 || exit 1
fi

echo ""
echo -e "${BLUE}[7/9] Verifying program deployment...${NC}"
wait_for_program "$PROGRAM_ID" 30 || exit 1

echo ""
echo -e "${BLUE}[8/9] Initializing test environment & starting API server...${NC}"
cd webapp/scripts && pnpm install --silent && pnpm run init
cd "$PROJECT_ROOT"
echo -e "  ${GREEN}Test environment initialized${NC}"

# Always restart API to pick up fresh config
pkill -f "tsx.*server.ts" 2>/dev/null || true
sleep 1
cd webapp/api && pnpm install --silent
pnpm run dev > /tmp/api.log 2>&1 &
API_PID=$!
cd "$PROJECT_ROOT"
echo "  API PID: $API_PID"
wait_for_http "http://localhost:3001/api/health" "API server" 15 '"status"' || exit 1

echo ""
echo -e "${BLUE}[9/9] Starting webapp...${NC}"
if curl -s http://localhost:5173 2>/dev/null | grep -q 'html'; then
  echo -e "  ${GREEN}Webapp already running on port 5173${NC}"
else
  cd webapp && pnpm run dev > /tmp/webapp.log 2>&1 &
  WEBAPP_PID=$!
  cd "$PROJECT_ROOT"
  echo "  Webapp PID: $WEBAPP_PID"
  wait_for_http "http://localhost:5173" "Webapp" 15 'html' || exit 1
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  All services started successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "  ${BLUE}Surfpool:${NC}    http://localhost:$RPC_PORT"
echo -e "  ${BLUE}API Server:${NC}  http://localhost:3001"
echo -e "  ${BLUE}Webapp:${NC}      http://localhost:5173"
echo ""
echo -e "  ${BLUE}Program ID:${NC}  $PROGRAM_ID"
echo ""
echo -e "  ${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

wait
