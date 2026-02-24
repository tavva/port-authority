# Port Authority

A fast, zero-config CLI that shows which dev servers are running on your machine.

```
$ pa
PORT  APPLICATION                      PID
3000  repos/my-app/apps/dashboard    90596
8080  repos/api-server               12041
5173  repos/frontend                 45302
```

Most developers run multiple projects and forget what's listening where. `pa` cuts through the noise — it automatically filters out system processes and shows only your development servers.

## Install

```bash
cargo install port-authority
```

This installs the `pa` binary to `~/.cargo/bin/`.

### From source

```bash
git clone https://github.com/tavva/port-authority.git
cd port-authority
cargo install --path .
```

## Usage

Run `pa` with no arguments to see your dev servers:

```bash
pa
```

### Flags

| Flag | Description |
|------|-------------|
| `-a, --all` | Show all listening ports (skip dev project filtering) |
| `-p, --port <PORT>` | Show only a specific port |
| `-r, --range <RANGE>` | Show ports in a range, e.g. `3000-4000` |

### Examples

Check if something is running on port 3000:

```bash
pa -p 3000
```

See all ports in the typical dev range:

```bash
pa -r 3000-9000
```

See everything listening on your machine:

```bash
pa --all
```

## How it works

`pa` uses `lsof` to find listening TCP ports, then looks up each process's working directory to determine if it's a development project. A process counts as a dev project if its working directory is under `$HOME` and isn't a system path like `Library/` or a dotfile directory.

Output is colour-coded in terminals and plain text when piped, so it works well with other tools:

```bash
pa | grep 3000
```

## Requirements

- macOS (uses `lsof` for port and process discovery)
- Rust 1.85+ (uses the 2024 edition)

## Licence

[MIT](LICENCE)
