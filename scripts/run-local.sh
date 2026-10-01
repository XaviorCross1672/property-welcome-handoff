#!/bin/sh
set -eu

cargo run --bin property_onboarding &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true' EXIT
sleep 2

curl --fail-with-body -X POST http://127.0.0.1:3000/tenant-onboarding \
  -H 'Content-Type: application/json' \
  -d '{"request_id":"lease-2048-welcome","user_id":"tenant-2048","tenant_name":"Morgan Lee","email":"chenhua@changba.com","phone":"","signup_channel":"email","maintenance_requests":["kitchen faucet"],"tenant_documents":["lease","move-in checklist"],"inspection_reminders":["2026-10-15"]}'
