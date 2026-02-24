// ABOUTME: Entry point for port-authority CLI.
// ABOUTME: Shows listening TCP ports used by development servers.

mod cli;
mod lsof;
mod process;
mod table;

use clap::Parser;

use cli::Cli;
use table::Row;

fn main() {
    let cli = Cli::parse();

    let output = std::process::Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-nP"])
        .output();

    let output = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(e) => {
            eprintln!("Failed to run lsof: {e}");
            std::process::exit(1);
        }
    };

    let mut ports = lsof::parse_lsof_output(&output);
    ports.retain(|p| cli.includes_port(p.port));
    ports.sort_by_key(|p| p.port);

    let pids: Vec<u32> = ports.iter().map(|p| p.pid).collect();
    let cwds = process::get_working_dirs(&pids);

    let home = std::env::var("HOME").unwrap_or_default();

    let rows: Vec<Row> = ports
        .iter()
        .filter(|p| {
            if cli.all {
                return true;
            }
            cwds.get(&p.pid)
                .map(|cwd| process::is_dev_project(cwd, &home))
                .unwrap_or(false)
        })
        .map(|p| {
            let application = cwds
                .get(&p.pid)
                .map(|cwd| process::application_name(cwd, &home))
                .unwrap_or_else(|| "\u{2013}".to_string());
            Row {
                port: p.port,
                application,
                pid: p.pid,
            }
        })
        .collect();

    if rows.is_empty() {
        eprintln!("No listening ports found.");
        return;
    }

    let use_colour = std::io::IsTerminal::is_terminal(&std::io::stdout());
    println!("{}", table::format_table(&rows, use_colour));
}
