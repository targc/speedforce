1. Goal

Build a minimal HTTP-over-TCP tunnel in Rust so that:

A public server receives HTTP requests from third parties (webhooks, etc.).

A local client on your dev machine connects out to that server via TCP.

The server forwards any HTTP request it receives to the client over that TCP connection.

The client replays the request to a local HTTP service (e.g. http://127.0.0.1:3000) and sends the response back through the tunnel.

This is for local development and debugging, not production traffic.

2. High-Level Architecture

Two binaries:

Tunnel Server

Public HTTP endpoint (e.g. on 0.0.0.0:8080).

TCP listener for tunnel clients (e.g. on 0.0.0.0:7000).

Stores one active client connection at a time.

Forwards HTTP requests over that connection and returns responses.

Tunnel Client

Runs on developer machine.

Connects to server’s TCP address.

Maintains a single persistent TCP connection.

Forwards requests to a local HTTP service (127.0.0.1:<LOCAL_PORT>) and returns responses.

Data flow:

External caller → Server HTTP → TCP tunnel → Client → Local HTTP → back through tunnel → Server HTTP → External caller

3. Functional Requirements
3.1 HTTP behavior (Server)

Listens on a configurable HTTP address (default: 0.0.0.0:8080).

Accepts:

Any HTTP method (GET, POST, PUT, DELETE, etc.).

Any path and query (e.g. /webhook, /api/v1/foo?x=1).

For every incoming HTTP request:

Extract method, full path + query, headers, and body bytes.

If a client is connected:

Wrap this into a tunnel request message and send it over the TCP connection.

Wait for a corresponding tunnel response message.

Return that response to the original HTTP caller (status, headers, body).

If no client is connected:

Respond with 503 Service Unavailable.

3.2 Tunnel connection & client semantics (Server)

Listens on a configurable TCP address (default: 0.0.0.0:7000).

At any time, there is only one active client connection:

When a new client connects:

If an old client is connected, close the old connection.

Replace it with the new one.

All HTTP traffic is sent through the currently active client.

3.3 HTTP behavior (Client)

Connects to server’s TCP endpoint (configurable address).

Has a configurable local HTTP port (default: 3000).

Once connected:

Continuously reads tunnel request messages from the server.

For each message:

Make an HTTP request to http://127.0.0.1:<LOCAL_PORT><path> using the given method, headers, and body.

Collect the local HTTP response (status, headers, body).

Wrap it into a tunnel response message and send it back over the same TCP connection.

3.4 Reconnect & resilience (Client)

Runs a loop:

Try to connect to the server.

If connection fails or drops:

Log the error.

Wait a short delay (e.g. a couple seconds).

Try again.

No manual restart of the server should be needed when the client goes up/down; reconnect should restore functionality automatically.

3.5 Error semantics

Server side:

If a client is not connected:

Immediately return 503 Service Unavailable for all HTTP requests.

If sending the tunnel request or reading the tunnel response fails:

Log the error.

Return 502 Bad Gateway to the HTTP caller.

Client side:

If the local HTTP request to 127.0.0.1:<LOCAL_PORT> fails:

Log the error.

Return a tunnel response with an error status (e.g. 502) and a small error body.

4. Protocol Requirements

The server and client communicate over the TCP connection using a simple framed protocol:

Each message:

A length prefix (fixed-size integer) indicating payload length.

Followed by a JSON payload of that length.

Two logical message types:

TunnelRequest: from server to client

Fields: HTTP method, full path (including query), headers, body bytes.

TunnelResponse: from client to server

Fields: HTTP status code, headers, body bytes.

Constraints:

Path must preserve the original path + query exactly.

Body must support arbitrary binary data (not just UTF-8).

No need for:

Message IDs.

Multiplexing.

Multiple in-flight requests.
Sequential request/response per connection is acceptable.

5. Non-Functional Requirements
5.1 Tech stack

Language: Rust (2021 or later).

Async runtime required.

Use any reasonable async HTTP server/client libraries; they must support:

Custom methods.

Full control over headers and body bytes.

5.2 Performance

Must handle multiple sequential HTTP requests over a single tunnel without leaking resources.

No throughput or latency targets; “fast enough” for typical dev webhooks.

5.3 Reliability

Server must stay up even if:

No client is connected.

Clients connect/disconnect repeatedly.

Client must be able to:

Recover from temporary network failures via auto-reconnect.

Cleanly handle server restarts (reconnecting when the TCP port is available again).

5.4 Security / Scope limits

Explicitly out of scope for this version:

Encryption/TLS.

Authentication/authorization.

Multi-tenant / multiple client IDs.

Persistent queues or retries beyond what the third-party webhook provider already does.

6. Configuration

At minimum, support environment variables or config values for:

Server:

HTTP listen address (default 0.0.0.0:8080).

TCP listen address for tunnel (default 0.0.0.0:7000).

Client:

Server TCP address (default 127.0.0.1:7000 for local testing).

Local HTTP port (default 3000).

Command-line flags are optional but nice-to-have.

7. Acceptance Scenarios

The implementation is “done” when all of these are true:

Basic forwarding

Local app runs on 127.0.0.1:3000 and returns something visible.

Server is running and listening on HTTP + TCP.

Client is running and connected to server.

Sending an HTTP request to the server (any path, any method) results in:

The same request hitting the local app.

The server returning the local app’s response.

Arbitrary paths & queries

Requests like GET /foo/bar?x=1&y=2 hit the local app as GET /foo/bar?x=1&y=2.

Client down

When the client is not connected:

Any HTTP request to the server gets 503 Service Unavailable.

Client reconnect

After stopping and restarting the client:

Server continues running.

New HTTP requests are again forwarded to the local app without restarting the server.

Two clients

If two clients try to connect at the same time with the same settings:

The second connection becomes the active one.

The first one gets disconnected (no split-brain behavior).

HTTP traffic goes only to the latest active client.
