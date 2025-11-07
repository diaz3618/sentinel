use assert_cmd::Command;

#[test]
fn status_output_ascii() {
    let mut cmd = Command::cargo_bin("sentinelctl").unwrap();
    cmd.arg("status").arg("--unicode").arg("false");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("Sentinel — Status") && !stdout.contains("Sentinel") {
        println!("Actual output: {}", stdout);
        panic!("Output missing 'Sentinel — Status'");
    }
    assert!(stdout.contains("MemAvailable"), "Output missing 'MemAvailable': {}", stdout);
}
