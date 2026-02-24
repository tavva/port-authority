// ABOUTME: Defines CLI argument structure using clap derive macros.
// ABOUTME: Supports filtering by port, range, or showing all ports.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "port-authority", about = "Show listening TCP ports for development servers")]
pub struct Cli {
    /// Skip dev-project filtering, show all listening ports
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
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("expected format: START-END (e.g. 3000-4000)".to_string());
    }
    let start_str = parts[0];
    let end_str = parts[1];
    let start: u16 = start_str.parse().map_err(|_| format!("invalid start port: {start_str}"))?;
    let end: u16 = end_str.parse().map_err(|_| format!("invalid end port: {end_str}"))?;
    if start > end {
        return Err(format!("start port {start} is greater than end port {end}"));
    }
    Ok((start, end))
}

impl Cli {
    /// Returns true if the given port should be included in output.
    pub fn includes_port(&self, port: u16) -> bool {
        if let Some(p) = self.port {
            return port == p;
        }
        if let Some((start, end)) = self.range {
            return port >= start && port <= end;
        }
        if self.all {
            return port >= 1024;
        }
        true
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
    fn includes_port_default_includes_all() {
        let cli = Cli { all: false, port: None, range: None };
        assert!(cli.includes_port(22));
        assert!(cli.includes_port(80));
        assert!(cli.includes_port(1024));
        assert!(cli.includes_port(3000));
        assert!(cli.includes_port(65535));
    }

    #[test]
    fn includes_port_all_flag_filters_low_ports() {
        let cli = Cli { all: true, port: None, range: None };
        assert!(!cli.includes_port(22));
        assert!(!cli.includes_port(80));
        assert!(!cli.includes_port(1023));
        assert!(cli.includes_port(1024));
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
