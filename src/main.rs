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
