// ABOUTME: Retrieves full command lines for processes by PID.
// ABOUTME: Uses `ps` to get the command that started each process.

use std::process::Command;

pub fn get_command(pid: u32) -> String {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "args="])
        .output();

    match output {
        Ok(out) => parse_ps_output(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => "\u{2013}".to_string(),
    }
}

/// Parse the output of `ps -p {pid} -o args=`.
/// Returns the trimmed command string, or "\u{2013}" if empty/missing.
pub fn parse_ps_output(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        "\u{2013}".to_string()
    } else {
        trimmed.to_string()
    }
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
        assert_eq!(parse_ps_output(""), "\u{2013}");
        assert_eq!(parse_ps_output("  \n"), "\u{2013}");
    }
}
