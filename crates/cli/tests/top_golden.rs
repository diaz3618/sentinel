use assert_cmd::Command;

#[test]
fn top_output_ascii() {
    let mut cmd = Command::cargo_bin("sentinelctl").unwrap();
    cmd.arg("--unicode").arg("false").arg("top").arg("--limit").arg("5");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PID"));
    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("RSS"));
}
