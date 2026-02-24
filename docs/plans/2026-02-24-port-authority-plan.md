# port-authority Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a CLI tool that shows listening TCP ports for development servers on macOS.

**Architecture:** Shell out to `lsof` to discover listening TCP ports, parse the columnar output, enrich with full command lines from `ps`, and display as a formatted table. `clap` for argument parsing.

**Tech Stack:** Rust, clap (derive), lsof, ps

---

### Task 1: Scaffold the Rust project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Initialise the Cargo project**

Run:
```bash
cargo init /Users/ben/repos/port-authority --name port-authority
```

**Step 2: Add clap dependency**

Run:
```bash
cd /Users/ben/repos/port-authority && cargo add clap --features derive
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Update .gitignore**

Add `/target` to `.gitignore` (Cargo convention).

**Step 5: Add ABOUTME comments to main.rs**

Replace the contents of `src/main.rs` with:

```rust
// ABOUTME: Entry point for port-authority CLI.
// ABOUTME: Shows listening TCP ports used by development servers.

fn main() {
    println!("port-authority");
}
```

**Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs .gitignore
git commit -m "Scaffold Rust project with clap dependency"
```

---

### Task 2: Parse lsof output

This is the core parsing logic. We parse the columnar output of
`lsof -iTCP -sTCP:LISTEN -nP` into structured data.

Real lsof output looks like this:
```
COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
rapportd    639  ben    8u  IPv4  0x20361eda3d95af0      0t0  TCP *:49849 (LISTEN)
node      12345  ben   22u  IPv6 0xed939eee49b8e1a4      0t0  TCP *:3000 (LISTEN)
python     5678  ben   11u  IPv4 0x664ba80d33aeb010      0t0  TCP 127.0.0.1:8080 (LISTEN)
```

The NAME column contains the address:port with "(LISTEN)" suffix.
Addresses can be `*` (all interfaces), `127.0.0.1`, `[::1]`, etc.

**Files:**
- Create: `src/lsof.rs`
- Modify: `src/main.rs` (add module declaration)

**Step 1: Write the failing tests**

Create `src/lsof.rs` with ABOUTME comments, the `ListeningPort` struct, and tests:

```rust
// ABOUTME: Parses lsof output to extract listening TCP port information.
// ABOUTME: Handles the columnar format from `lsof -iTCP -sTCP:LISTEN -nP`.

#[derive(Debug, PartialEq)]
pub struct ListeningPort {
    pub pid: u32,
    pub port: u16,
    pub process: String,
}

pub fn parse_lsof_output(output: &str) -> Vec<ListeningPort> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_lsof_output() {
        let output = "\
COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node      12345  ben   22u  IPv6 0xed939eee49b8e1a4      0t0  TCP *:3000 (LISTEN)
python     5678  ben   11u  IPv4 0x664ba80d33aeb010      0t0  TCP 127.0.0.1:8080 (LISTEN)";

        let ports = parse_lsof_output(output);
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0], ListeningPort { pid: 12345, port: 3000, process: "node".into() });
        assert_eq!(ports[1], ListeningPort { pid: 5678, port: 8080, process: "python".into() });
    }

    #[test]
    fn skips_header_line() {
        let output = "COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME\n";
        let ports = parse_lsof_output(output);
        assert!(ports.is_empty());
    }

    #[test]
    fn handles_empty_output() {
        let ports = parse_lsof_output("");
        assert!(ports.is_empty());
    }

    #[test]
    fn deduplicates_same_pid_and_port() {
        // lsof can show the same port twice (IPv4 and IPv6)
        let output = "\
COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
rapportd    639  ben    8u  IPv4  0x20361eda3d95af0      0t0  TCP *:49849 (LISTEN)
rapportd    639  ben    9u  IPv6 0xed939eee49b8e1a4      0t0  TCP *:49849 (LISTEN)";

        let ports = parse_lsof_output(output);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], ListeningPort { pid: 639, port: 49849, process: "rapportd".into() });
    }

    #[test]
    fn handles_ipv6_localhost() {
        let output = "\
COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node      12345  ben   22u  IPv6 0xed939eee49b8e1a4      0t0  TCP [::1]:3000 (LISTEN)";

        let ports = parse_lsof_output(output);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].port, 3000);
    }
}
```

Add `mod lsof;` to `src/main.rs`.

**Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — `todo!()` panics

**Step 3: Implement parse_lsof_output**

Replace the `todo!()` in `parse_lsof_output` with:

```rust
pub fn parse_lsof_output(output: &str) -> Vec<ListeningPort> {
    let mut seen = std::collections::HashSet::new();
    let mut ports = Vec::new();

    for line in output.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            continue;
        }

        let process = fields[0].to_string();
        let pid: u32 = match fields[1].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        // NAME field is last, format: "addr:port (LISTEN)" or "*:port (LISTEN)"
        let name = fields[8];
        let port: u16 = match name.rsplit(':').next().and_then(|p| p.parse().ok()) {
            Some(p) => p,
            None => continue,
        };

        if seen.insert((pid, port)) {
            ports.push(ListeningPort { pid, port, process });
        }
    }

    ports
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/lsof.rs src/main.rs
git commit -m "Add lsof output parser with deduplication"
```

---

### Task 3: Get full command lines via ps

For each PID from lsof, fetch the full command line using
`ps -p {pid} -o args=`.

**Files:**
- Create: `src/command.rs`
- Modify: `src/main.rs` (add module declaration)

**Step 1: Write the failing tests**

Create `src/command.rs`:

```rust
// ABOUTME: Retrieves full command lines for processes by PID.
// ABOUTME: Uses `ps` to get the command that started each process.

pub fn get_command(pid: u32) -> String {
    todo!()
}

/// Parse the output of `ps -p {pid} -o args=`.
/// Returns the trimmed command string, or "–" if empty/missing.
pub fn parse_ps_output(output: &str) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_ps_output() {
        let output = "node /Users/ben/project/node_modules/.bin/next dev\n";
        assert_eq!(parse_ps_output(output), "node /Users/ben/project/node_modules/.bin/next dev");
    }

    #[test]
    fn returns_dash_for_empty_output() {
        assert_eq!(parse_ps_output(""), "–");
        assert_eq!(parse_ps_output("  \n"), "–");
    }
}
```

Add `mod command;` to `src/main.rs`.

**Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — `todo!()` panics

**Step 3: Implement parse_ps_output and get_command**

```rust
use std::process::Command;

pub fn get_command(pid: u32) -> String {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "args="])
        .output();

    match output {
        Ok(out) => parse_ps_output(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => "–".to_string(),
    }
}

pub fn parse_ps_output(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        "–".to_string()
    } else {
        trimmed.to_string()
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/command.rs src/main.rs
git commit -m "Add command line retrieval via ps"
```

---

### Task 4: Table formatting

Format the collected data into a column-aligned table.

**Files:**
- Create: `src/table.rs`
- Modify: `src/main.rs` (add module declaration)

**Step 1: Write the failing test**

Create `src/table.rs`:

```rust
// ABOUTME: Formats port data into a column-aligned plain text table.
// ABOUTME: Produces human-readable output for terminal display.

pub struct Row {
    pub pid: u32,
    pub port: u16,
    pub process: String,
    pub command: String,
}

pub fn format_table(rows: &[Row]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_rows_as_aligned_table() {
        let rows = vec![
            Row { pid: 1234, port: 3000, process: "node".into(), command: "next dev".into() },
            Row { pid: 56789, port: 8080, process: "python".into(), command: "uvicorn main:app".into() },
        ];

        let table = format_table(&rows);
        let lines: Vec<&str> = table.lines().collect();

        // Header present
        assert!(lines[0].contains("PID"));
        assert!(lines[0].contains("PORT"));
        assert!(lines[0].contains("PROCESS"));
        assert!(lines[0].contains("COMMAND"));

        // Data rows present and aligned
        assert!(lines[1].contains("1234"));
        assert!(lines[1].contains("3000"));
        assert!(lines[1].contains("node"));
        assert!(lines[1].contains("next dev"));

        assert!(lines[2].contains("56789"));
        assert!(lines[2].contains("8080"));
        assert!(lines[2].contains("python"));
        assert!(lines[2].contains("uvicorn main:app"));
    }

    #[test]
    fn empty_rows_returns_empty_string() {
        assert_eq!(format_table(&[]), "");
    }
}
```

Add `mod table;` to `src/main.rs`.

**Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — `todo!()` panics

**Step 3: Implement format_table**

```rust
pub fn format_table(rows: &[Row]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let pid_width = rows.iter().map(|r| r.pid.to_string().len()).max().unwrap().max(3);
    let port_width = rows.iter().map(|r| r.port.to_string().len()).max().unwrap().max(4);
    let proc_width = rows.iter().map(|r| r.process.len()).max().unwrap().max(7);

    let mut output = format!(
        "{:<pid_w$}  {:<port_w$}  {:<proc_w$}  {}\n",
        "PID", "PORT", "PROCESS", "COMMAND",
        pid_w = pid_width, port_w = port_width, proc_w = proc_width,
    );

    for row in rows {
        output.push_str(&format!(
            "{:<pid_w$}  {:<port_w$}  {:<proc_w$}  {}\n",
            row.pid, row.port, row.process, row.command,
            pid_w = pid_width, port_w = port_width, proc_w = proc_width,
        ));
    }

    output.trim_end().to_string()
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/table.rs src/main.rs
git commit -m "Add column-aligned table formatter"
```

---

### Task 5: CLI argument parsing with clap

Wire up `clap` for `--all`, `--port`, and `--range` flags.

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs` (add module declaration)

**Step 1: Write the failing tests**

Create `src/cli.rs`:

```rust
// ABOUTME: Defines CLI argument structure using clap derive macros.
// ABOUTME: Supports filtering by port, range, or showing all ports.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "port-authority", about = "Show listening TCP ports for development servers")]
pub struct Cli {
    /// Show all listening ports, including system services
    #[arg(short, long)]
    pub all: bool,

    /// Show only this specific port
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Show ports in this range (e.g. 3000-4000)
    #[arg(short, long, value_parser = parse_range)]
    pub range: Option<(u16, u16)>,
}

fn parse_range(s: &str) -> Result<(u16, u16), String> {
    todo!()
}

impl Cli {
    /// Returns true if the given port should be included in output.
    pub fn includes_port(&self, port: u16) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_range_valid() {
        assert_eq!(parse_range("3000-4000").unwrap(), (3000, 4000));
    }

    #[test]
    fn parse_range_single_number_is_error() {
        assert!(parse_range("3000").is_err());
    }

    #[test]
    fn parse_range_reversed_is_error() {
        assert!(parse_range("4000-3000").is_err());
    }

    #[test]
    fn parse_range_invalid_number_is_error() {
        assert!(parse_range("abc-def").is_err());
    }

    #[test]
    fn includes_port_default_filters_low_ports() {
        let cli = Cli { all: false, port: None, range: None };
        assert!(!cli.includes_port(22));
        assert!(!cli.includes_port(80));
        assert!(!cli.includes_port(443));
        assert!(!cli.includes_port(1023));
        assert!(cli.includes_port(1024));
        assert!(cli.includes_port(3000));
        assert!(cli.includes_port(8080));
        assert!(cli.includes_port(65535));
    }

    #[test]
    fn includes_port_all_flag() {
        let cli = Cli { all: true, port: None, range: None };
        assert!(cli.includes_port(22));
        assert!(cli.includes_port(80));
        assert!(cli.includes_port(3000));
    }

    #[test]
    fn includes_port_specific_port() {
        let cli = Cli { all: false, port: Some(3000), range: None };
        assert!(cli.includes_port(3000));
        assert!(!cli.includes_port(8080));
        assert!(!cli.includes_port(22));
    }

    #[test]
    fn includes_port_range() {
        let cli = Cli { all: false, port: None, range: Some((3000, 4000)) };
        assert!(cli.includes_port(3000));
        assert!(cli.includes_port(3500));
        assert!(cli.includes_port(4000));
        assert!(!cli.includes_port(2999));
        assert!(!cli.includes_port(4001));
    }
}
```

Add `mod cli;` to `src/main.rs`.

**Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — `todo!()` panics

**Step 3: Implement parse_range and includes_port**

```rust
fn parse_range(s: &str) -> Result<(u16, u16), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("expected format: START-END (e.g. 3000-4000)".to_string());
    }
    let start: u16 = parts[0].parse().map_err(|_| format!("invalid start port: {}", parts[0]))?;
    let end: u16 = parts[1].parse().map_err(|_| format!("invalid end port: {}", parts[1]))?;
    if start > end {
        return Err(format!("start port {} is greater than end port {}", start, end));
    }
    Ok((start, end))
}

impl Cli {
    pub fn includes_port(&self, port: u16) -> bool {
        if let Some(p) = self.port {
            return port == p;
        }
        if let Some((start, end)) = self.range {
            return port >= start && port <= end;
        }
        if self.all {
            return true;
        }
        port >= 1024
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS

**Step 5: Commit**

```bash
git add src/cli.rs src/main.rs
git commit -m "Add CLI argument parsing with clap"
```

---

### Task 6: Wire everything together in main

Connect all the modules: parse args, run lsof, filter, enrich with
command lines, format, and print.

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement main**

```rust
// ABOUTME: Entry point for port-authority CLI.
// ABOUTME: Shows listening TCP ports used by development servers.

mod cli;
mod command;
mod lsof;
mod table;

use std::process;

use clap::Parser;

use cli::Cli;
use table::Row;

fn main() {
    let cli = Cli::parse();

    let output = process::Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-nP"])
        .output();

    let output = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(e) => {
            eprintln!("Failed to run lsof: {e}");
            process::exit(1);
        }
    };

    let mut ports = lsof::parse_lsof_output(&output);
    ports.retain(|p| cli.includes_port(p.port));
    ports.sort_by_key(|p| p.port);

    let rows: Vec<Row> = ports
        .iter()
        .map(|p| Row {
            pid: p.pid,
            port: p.port,
            process: p.process.clone(),
            command: command::get_command(p.pid),
        })
        .collect();

    let table = table::format_table(&rows);
    if !table.is_empty() {
        println!("{table}");
    }
}
```

**Step 2: Verify it compiles and runs**

Run: `cargo run`
Expected: Shows a table of listening ports (or empty if none in range)

Run: `cargo run -- --all`
Expected: Shows all listening ports including system services

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "Wire up main to run lsof, filter, and display ports"
```

---

### Task 7: Integration test

Test the full binary end-to-end by binding a real port and checking
the output.

**Files:**
- Create: `tests/integration.rs`

**Step 1: Write the integration test**

Create `tests/integration.rs`:

```rust
// ABOUTME: End-to-end integration tests for port-authority.
// ABOUTME: Binds real TCP ports and verifies the binary output.

use std::net::TcpListener;
use std::process::Command;

#[test]
fn finds_a_listening_port() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind port");
    let port = listener.local_addr().unwrap().port();

    let output = Command::new(env!("CARGO_BIN_EXE_port-authority"))
        .args(["--port", &port.to_string()])
        .output()
        .expect("Failed to run port-authority");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&port.to_string()),
        "Expected port {port} in output:\n{stdout}"
    );
}

#[test]
fn no_output_for_unused_port() {
    // Bind and immediately drop to ensure port is free
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind port");
        listener.local_addr().unwrap().port()
    };
    // listener dropped, port is free

    let output = Command::new(env!("CARGO_BIN_EXE_port-authority"))
        .args(["--port", &port.to_string()])
        .output()
        .expect("Failed to run port-authority");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty() || !stdout.contains(&port.to_string()),
        "Expected port {port} NOT in output:\n{stdout}"
    );
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --test integration`
Expected: Both tests PASS

**Step 3: Run all tests together**

Run: `cargo test`
Expected: All unit + integration tests PASS

**Step 4: Commit**

```bash
git add tests/integration.rs
git commit -m "Add integration tests with real TCP listeners"
```

---

### Task 8: Final polish

Clean up any warnings and verify everything works end to end.

**Step 1: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

Fix any clippy issues if they arise.

**Step 2: Run all tests one final time**

Run: `cargo test`
Expected: All tests PASS

**Step 3: Test manually**

Run: `cargo run -- --all`
Expected: Shows all listening ports with full command lines

Run: `cargo run -- --range 3000-9000`
Expected: Shows only ports in that range (or empty if none)

**Step 4: Commit any final fixes**

If any changes were needed, commit them.
