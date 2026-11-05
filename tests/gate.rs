use std::process::Command;
use tide_rill::{evaluate_json, gate_json};

const INPUT: &str = r#"{"version":2,"k":2,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"complete","judgments":[{"id":"A","grade":2},{"id":"B","grade":1}],"ranking":["X","B","A"]}],
"faithfulness":{"answer_ids":["a"],"answers":[{"id":"a","complete":true,
"context":["Rust forbids unsafe code in the crate","The gate fails closed on missing metrics"],
"claims":[
 {"id":"c1","label":"supported","text":"The crate forbids unsafe code"},
 {"id":"c2","label":"supported","text":"Benchmarks show a forty percent speedup"},
 {"id":"c3","label":"insufficient","text":"The gate fails closed on missing metrics"}]}]}}"#;

#[test]
fn lexical_faithfulness_flags_label_conflicts() {
    let f = evaluate_json(INPUT)
        .expect("valid")
        .faithfulness
        .expect("present");
    assert_eq!(f.lexical_checked, 3);
    assert_eq!(f.lexical_supported, 2);
    assert_eq!(f.label_conflicts, 1);
}

#[test]
fn gate_passes_and_fails_per_metric() {
    let pass = gate_json(INPUT, r#"{"min_recall":0.5,"min_mrr":0.5}"#).expect("ok");
    assert!(pass.passed);
    let fail = gate_json(INPUT, r#"{"min_recall":0.5,"max_label_conflicts":0}"#).expect("ok");
    assert!(!fail.passed);
    assert!(fail
        .checks
        .iter()
        .any(|c| c.metric == "recall_macro" && c.passed));
}

#[test]
fn gate_fails_closed_on_undefined_metric_and_bad_thresholds() {
    let no_faith = r#"{"version":2,"k":1,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"complete","judgments":[{"id":"A","grade":1}],"ranking":["A"]}]}"#;
    assert!(
        !gate_json(no_faith, r#"{"min_support_rate":0.1}"#)
            .expect("ok")
            .passed
    );
    assert!(gate_json(no_faith, "{}").is_err());
    assert!(gate_json(no_faith, r#"{"min_mrr":1.5}"#).is_err());
    assert!(gate_json(no_faith, r#"{"unknown":1}"#).is_err());
}

#[test]
fn cli_gate_exit_codes() {
    let exe = env!("CARGO_BIN_EXE_tide-rill");
    let run = |t: &str| {
        Command::new(exe)
            .args(["gate", "examples/synthetic.json", t])
            .output()
            .expect("run CLI")
            .status
            .code()
    };
    assert_eq!(run("examples/thresholds-pass.json"), Some(0));
    assert_eq!(run("examples/thresholds-fail.json"), Some(1));
    assert_eq!(run("examples/missing.json"), Some(2));
}
