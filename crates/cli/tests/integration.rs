use assert_cmd::Command;

#[test]
fn test_help_snapshot() {
    // Use the standard environment variable provided by Cargo for tests
    // This avoids the deprecated assert_cmd::cargo::cargo_bin logic
    let bin_path = env!("CARGO_BIN_EXE_vc");
    let mut cmd = Command::new(bin_path);
    cmd.arg("--help");

    // Snapshot the output.
    // Normalized for Windows CI (CRLF -> LF)
    let output = cmd.output().unwrap().stdout;
    let text = String::from_utf8(output).unwrap().replace("\r\n", "\n");
    insta::assert_snapshot!(text);
}

#[test]
fn test_version_snapshot() {
    let bin_path = env!("CARGO_BIN_EXE_vc");
    let mut cmd = Command::new(bin_path);
    cmd.arg("--version");

    // Snapshot the output (e.g. "vanity_cli 0.1.0-beta.4")
    let output = cmd.output().unwrap().stdout;
    let text = String::from_utf8(output).unwrap().replace("\r\n", "\n");
    insta::assert_snapshot!(text);
}

#[test]
fn test_invalid_arg_snapshot() {
    let bin_path = env!("CARGO_BIN_EXE_vc");
    let mut cmd = Command::new(bin_path);
    cmd.arg("--this-flag-does-not-exist");

    // Check stderr for the error message
    let output = cmd.output().unwrap().stderr;
    let text = String::from_utf8(output).unwrap().replace("\r\n", "\n");
    insta::assert_snapshot!(text);
}

#[test]
fn test_headless_execution() {
    let bin_path = env!("CARGO_BIN_EXE_vc");
    let mut cmd = Command::new(bin_path);
    cmd.arg("--prefix")
        .arg("a")
        .arg("--no-tui")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}
