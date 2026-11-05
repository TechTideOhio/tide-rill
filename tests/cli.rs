use std::process::Command;

#[test]
fn example_works_and_missing_file_fails_without_success_output() {
    let exe = env!("CARGO_BIN_EXE_tide-rill");
    let good = Command::new(exe)
        .args(["eval", "examples/synthetic.json"])
        .output()
        .expect("run CLI");
    assert!(good.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&good.stdout).expect("JSON output");
    assert_eq!(parsed["mrr_eligible_queries"], 1);
    let bad = Command::new(exe)
        .args(["eval", "missing-file.json"])
        .output()
        .expect("run CLI");
    assert_eq!(bad.status.code(), Some(2));
    assert!(bad.stdout.is_empty());
}
