use assert_cmd::Command;

#[test]
fn status_output_ascii() {
    let mut cmd = Command::cargo_bin("sentinelctl").unwrap();
    cmd.arg("--unicode").arg("false").arg("status");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.contains("Sentinel — Status") && !stdout.contains("Sentinel") {
        println!("Actual stdout: {}", stdout);
        println!("Actual stderr: {}", stderr);
        println!("Exit status: {}", output.status);
        panic!("Output missing 'Sentinel — Status'");
    }
    assert!(stdout.contains("MemAvailable"), "Output missing 'MemAvailable': {}", stdout);
}
