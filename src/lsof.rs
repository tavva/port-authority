// ABOUTME: Parses lsof output to extract listening TCP port information.
// ABOUTME: Handles the columnar format from `lsof -iTCP -sTCP:LISTEN -nP`.

#[derive(Debug, PartialEq)]
pub struct ListeningPort {
    pub pid: u32,
    pub port: u16,
    pub process: String,
}

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

        // NAME field is last, format: "addr:port" or "*:port"
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
