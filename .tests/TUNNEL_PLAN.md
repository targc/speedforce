# Test Plan: HTTP-over-TCP Tunnel System

## Services to Check
- [ ] tunnel-server health (HTTP:8080, TCP:7000)
- [ ] tunnel-client connection status
- [ ] local HTTP service (port 3000)

Scripts: TUNNEL_SCRIPTS/check_services.sh

## API/Forwarding Tests
- [ ] Basic GET request forwarding
- [ ] Path preservation (/foo/bar)
- [ ] Query string preservation (?x=1&y=2)
- [ ] POST with JSON body
- [ ] Custom headers preservation
- [ ] Binary data (POST with binary body)

Scripts: TUNNEL_SCRIPTS/test_forwarding.sh

## Error Handling Tests
- [ ] No client connected → 503 Service Unavailable
- [ ] Local service down → 502 Bad Gateway
- [ ] Invalid responses → 502

Scripts: TUNNEL_SCRIPTS/test_errors.sh

## Reconnection Tests
- [ ] Client auto-reconnect after disconnect
- [ ] Exponential backoff behavior
- [ ] Server restart recovery
- [ ] Multiple clients (last one wins)

Scripts: TUNNEL_SCRIPTS/test_reconnection.sh

## Logs
- [ ] No errors in tunnel-server logs
- [ ] No errors in tunnel-client logs
- [ ] Proper connection/disconnection messages

Scripts: TUNNEL_SCRIPTS/scan_logs.sh

## Acceptance Criteria Validation
- [ ] All 8 acceptance criteria from CLAUDE.md verified
