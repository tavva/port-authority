// ABOUTME: Retrieves working directories for processes and identifies dev projects.
// ABOUTME: Uses `lsof` to batch-fetch cwds and filters to projects under $HOME.

use std::collections::HashMap;
use std::process::Command;

/// Runs `lsof -a -d cwd -p PID1,PID2,... -Fn` and returns a map of PID to cwd.
pub fn get_working_dirs(pids: &[u32]) -> HashMap<u32, String> {
    if pids.is_empty() {
        return HashMap::new();
    }

    let pid_list: String = pids.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
    let output = Command::new("lsof")
        .args(["-a", "-d", "cwd", "-p", &pid_list, "-Fn"])
        .output();

    match output {
        Ok(out) => parse_lsof_cwd_output(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => HashMap::new(),
    }
}

/// Parses the `p{PID}\nfcwd\nn{path}` format from `lsof -Fn`.
pub fn parse_lsof_cwd_output(output: &str) -> HashMap<u32, String> {
    let mut result = HashMap::new();
    let mut current_pid: Option<u32> = None;

    for line in output.lines() {
        if let Some(pid_str) = line.strip_prefix('p') {
            current_pid = pid_str.parse().ok();
        } else if let Some(path) = line.strip_prefix('n') {
            if let Some(pid) = current_pid {
                result.insert(pid, path.to_string());
            }
        }
    }

    result
}

/// Returns true if cwd is a dev project under $HOME (not Library, dotfiles, or bare $HOME).
pub fn is_dev_project(cwd: &str, home: &str) -> bool {
    let Some(relative) = cwd.strip_prefix(home) else {
        return false;
    };
    let Some(relative) = relative.strip_prefix('/') else {
        return false;
    };
    if relative.is_empty() {
        return false;
    }
    if relative.starts_with("Library/") || relative.starts_with('.') {
        return false;
    }
    true
}

/// Strips $HOME/ prefix from cwd to get a relative application name.
pub fn application_name(cwd: &str, home: &str) -> String {
    cwd.strip_prefix(home)
        .and_then(|r| r.strip_prefix('/'))
        .unwrap_or(cwd)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lsof_cwd_output_single_pid() {
        let output = "p12345\nfcwd\nn/Users/ben/repos/drumbeat\n";
        let result = parse_lsof_cwd_output(output);
        assert_eq!(result.get(&12345).unwrap(), "/Users/ben/repos/drumbeat");
    }

    #[test]
    fn parses_lsof_cwd_output_multiple_pids() {
        let output = "\
p12345\nfcwd\nn/Users/ben/repos/drumbeat\n\
p5678\nfcwd\nn/Users/ben/home-networking\n";
        let result = parse_lsof_cwd_output(output);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&12345).unwrap(), "/Users/ben/repos/drumbeat");
        assert_eq!(result.get(&5678).unwrap(), "/Users/ben/home-networking");
    }

    #[test]
    fn parses_lsof_cwd_output_empty() {
        let result = parse_lsof_cwd_output("");
        assert!(result.is_empty());
    }

    #[test]
    fn parses_lsof_cwd_output_skips_invalid_pid() {
        let output = "pabc\nfcwd\nn/some/path\n";
        let result = parse_lsof_cwd_output(output);
        assert!(result.is_empty());
    }

    #[test]
    fn is_dev_project_under_home() {
        assert!(is_dev_project("/Users/ben/repos/drumbeat", "/Users/ben"));
        assert!(is_dev_project("/Users/ben/home-networking", "/Users/ben"));
        assert!(is_dev_project("/Users/ben/repos/deep/nested/project", "/Users/ben"));
    }

    #[test]
    fn is_dev_project_rejects_library() {
        assert!(!is_dev_project("/Users/ben/Library/Caches/foo", "/Users/ben"));
        assert!(!is_dev_project("/Users/ben/Library/Application Support/bar", "/Users/ben"));
    }

    #[test]
    fn is_dev_project_rejects_dotfiles() {
        assert!(!is_dev_project("/Users/ben/.config/something", "/Users/ben"));
        assert!(!is_dev_project("/Users/ben/.nvm/versions/node", "/Users/ben"));
    }

    #[test]
    fn is_dev_project_rejects_bare_home() {
        assert!(!is_dev_project("/Users/ben", "/Users/ben"));
    }

    #[test]
    fn is_dev_project_rejects_outside_home() {
        assert!(!is_dev_project("/usr/local/bin", "/Users/ben"));
        assert!(!is_dev_project("/tmp/project", "/Users/ben"));
    }

    #[test]
    fn application_name_strips_home_prefix() {
        assert_eq!(application_name("/Users/ben/repos/drumbeat", "/Users/ben"), "repos/drumbeat");
        assert_eq!(application_name("/Users/ben/home-networking", "/Users/ben"), "home-networking");
    }

    #[test]
    fn application_name_returns_full_path_when_no_home_prefix() {
        assert_eq!(application_name("/usr/local/bin", "/Users/ben"), "/usr/local/bin");
    }
}
