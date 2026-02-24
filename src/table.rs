// ABOUTME: Formats port data into a column-aligned plain text table.
// ABOUTME: Produces human-readable output for terminal display.

pub struct Row {
    pub pid: u32,
    pub port: u16,
    pub process: String,
    pub command: String,
}

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
