# port-authority Design

A CLI tool that shows listening TCP ports used by development servers, helping
identify port clashes.

## Core Behaviour

Run `port-authority` to see a table of listening ports:

| PID | Port | Process | Command |
|-----|------|---------|---------|
| 1234 | 3000 | node | next dev |
| 5678 | 8080 | python | uvicorn main:app |

- Default: show ports 1024-65535, sorted by port number
- Filters out system services while capturing dev server ports

## CLI Flags

- `port-authority` — default, ports 1024-65535
- `port-authority --all` / `-a` — all listening ports
- `port-authority --port 3000` / `-p 3000` — specific port
- `port-authority --range 3000-4000` / `-r 3000-4000` — port range

No config files, no persistent state.

## Implementation

**Platform:** Rust, macOS-focused.

**Port discovery:** Shell out to `lsof -iTCP -sTCP:LISTEN -nP` and parse the
columnar output. This gives us PID, process name, and listen address/port
without requiring elevated privileges for the user's own processes.

**Command line retrieval:** Shell out to `ps -p {pid} -o args=` for each PID
to get the full command line (macOS has no `/proc` filesystem).

**Dependencies:** `clap` for argument parsing. No async, no serde, no colour
libraries.

**Output:** Column-aligned plain text table with a header row.

**Error handling:**
- `lsof` not found or fails: clear error message, exit 1
- PID disappeared between `lsof` and `ps`: show "–" for the command

## Testing

**Unit tests:** Parse `lsof` and `ps` output from fixture strings. Test pure
parsing functions with known inputs.

**Integration test:** Bind a `TcpListener` on a known port, run the binary,
assert the port appears in output. No mocks — real ports, real processes.
