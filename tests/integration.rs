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
