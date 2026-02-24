// ABOUTME: Formats port data into a column-aligned plain text table.
// ABOUTME: Supports ANSI colour output for terminal display.

const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub struct Row {
    pub port: u16,
    pub application: String,
    pub pid: u32,
}

pub fn format_table(rows: &[Row], use_colour: bool) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let port_width = rows.iter().map(|r| r.port.to_string().len()).max().unwrap().max(4);
    let app_width = rows.iter().map(|r| r.application.len()).max().unwrap().max(11);
    let pid_width = rows.iter().map(|r| r.pid.to_string().len()).max().unwrap().max(3);

    let mut output = if use_colour {
        format!(
            "{BOLD}{:<port_w$}  {:<app_w$}  {:<pid_w$}{RESET}\n",
            "PORT", "APPLICATION", "PID",
            port_w = port_width, app_w = app_width, pid_w = pid_width,
        )
    } else {
        format!(
            "{:<port_w$}  {:<app_w$}  {:<pid_w$}\n",
            "PORT", "APPLICATION", "PID",
            port_w = port_width, app_w = app_width, pid_w = pid_width,
        )
    };

    for row in rows {
        if use_colour {
            output.push_str(&format!(
                "{CYAN}{:<port_w$}{RESET}  {:<app_w$}  {DIM}{:<pid_w$}{RESET}\n",
                row.port, row.application, row.pid,
                port_w = port_width, app_w = app_width, pid_w = pid_width,
            ));
        } else {
            output.push_str(&format!(
                "{:<port_w$}  {:<app_w$}  {:<pid_w$}\n",
                row.port, row.application, row.pid,
                port_w = port_width, app_w = app_width, pid_w = pid_width,
            ));
        }
    }

    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_rows_as_aligned_table_no_colour() {
        let rows = vec![
            Row { port: 3000, application: "repos/drumbeat".into(), pid: 1234 },
            Row { port: 8080, application: "home-networking".into(), pid: 56789 },
        ];

        let table = format_table(&rows, false);
        let lines: Vec<&str> = table.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("PORT"));
        assert!(lines[0].contains("APPLICATION"));
        assert!(lines[0].contains("PID"));

        assert!(lines[1].contains("3000"));
        assert!(lines[1].contains("repos/drumbeat"));
        assert!(lines[1].contains("1234"));

        assert!(lines[2].contains("8080"));
        assert!(lines[2].contains("home-networking"));
        assert!(lines[2].contains("56789"));
    }

    #[test]
    fn formats_rows_with_colour() {
        let rows = vec![
            Row { port: 3000, application: "repos/drumbeat".into(), pid: 1234 },
        ];

        let table = format_table(&rows, true);

        // Header is bold
        assert!(table.contains(BOLD));
        // Port is cyan
        assert!(table.contains(CYAN));
        // PID is dim
        assert!(table.contains(DIM));
        // All codes are reset
        assert!(table.contains(RESET));
    }

    #[test]
    fn no_colour_has_no_ansi_codes() {
        let rows = vec![
            Row { port: 3000, application: "repos/drumbeat".into(), pid: 1234 },
        ];

        let table = format_table(&rows, false);
        assert!(!table.contains('\x1b'));
    }

    #[test]
    fn column_order_is_port_application_pid() {
        let rows = vec![
            Row { port: 3000, application: "myapp".into(), pid: 999 },
        ];

        let table = format_table(&rows, false);
        let header = table.lines().next().unwrap();
        let port_pos = header.find("PORT").unwrap();
        let app_pos = header.find("APPLICATION").unwrap();
        let pid_pos = header.find("PID").unwrap();
        assert!(port_pos < app_pos);
        assert!(app_pos < pid_pos);
    }

    #[test]
    fn empty_rows_returns_empty_string() {
        assert_eq!(format_table(&[], false), "");
        assert_eq!(format_table(&[], true), "");
    }
}
