#![allow(clippy::float_cmp, clippy::unreadable_literal)]

use tide_rill::evaluate_json;

#[test]
fn judgment_completion_and_sanitized_errors_are_required() {
    let missing = r#"{"version":1,"k":1,"query_ids":["q"],"queries":[{"id":"q","judgments":[],"ranking":[]}]}"#;
    assert!(evaluate_json(missing).is_err());
    let incomplete = r#"{"version":2,"k":1,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"incomplete","judgments":[],"ranking":[]}]}"#;
    assert!(evaluate_json(incomplete).is_err());
    let secret = r#"{"SENTINEL_PRIVATE_VALUE":123}"#;
    assert!(!evaluate_json(secret)
        .err()
        .expect("invalid input")
        .contains("SENTINEL_PRIVATE_VALUE"));
}

#[test]
fn distinguishes_total_results_from_cutoff_and_records_gain_policy() {
    let input = r#"{"version":2,"k":1,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"complete","judgments":[{"id":"a","grade":2}],"ranking":["a","b"]}]}"#;
    let report = evaluate_json(input).expect("valid input");
    let json = serde_json::to_value(report).expect("serializable");
    assert_eq!(json["queries"][0]["returned_total"], 2);
    assert_eq!(json["queries"][0]["returned_at_k"], 1);
    assert_eq!(json["ndcg_gain"], "linear");
}

#[test]
fn computes_known_rank_metrics_and_manual_support() {
    let input = r#"{"version":2,"k":2,"query_ids":["q1"],"queries":[{"id":"q1","judgment_status":"complete","judgments":[{"id":"A","grade":2},{"id":"B","grade":1}],"ranking":["X","B","A"]}],"faithfulness":{"answer_ids":["a"],"answers":[{"id":"a","complete":true,"claims":[{"id":"c1","label":"supported"},{"id":"c2","label":"insufficient"}]}]}}"#;
    let report = evaluate_json(input).expect("valid evaluation");
    let q = &report.queries[0];
    assert_eq!(q.recall, Some(0.5));
    assert_eq!(q.precision, 0.5);
    assert_eq!(q.reciprocal_rank, Some(0.5));
    assert_eq!(report.mean_reciprocal_rank, Some(0.5));
    assert!((q.ndcg.expect("defined") - 0.23981246656813146).abs() < 1e-12);
    assert_eq!(
        report.faithfulness.expect("present").support_rate,
        Some(0.5)
    );
}

#[test]
fn rejects_duplicate_rankings_and_incomplete_annotations() {
    let duplicate = r#"{"version":2,"k":1,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"complete","judgments":[],"ranking":["a","a"]}]}"#;
    assert!(evaluate_json(duplicate).is_err());
    let incomplete = r#"{"version":2,"k":1,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"complete","judgments":[],"ranking":[]}],"faithfulness":{"answer_ids":["a"],"answers":[{"id":"a","complete":false,"claims":[]}]}}"#;
    assert!(evaluate_json(incomplete).is_err());
}

#[test]
fn rejects_missing_declared_queries_and_answers() {
    let missing_query = r#"{"version":2,"k":1,"query_ids":["q","missing"],"queries":[{"id":"q","judgment_status":"complete","judgments":[],"ranking":[]}]}"#;
    assert!(evaluate_json(missing_query).is_err());
    let missing_answer = r#"{"version":2,"k":1,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"complete","judgments":[],"ranking":[]}],"faithfulness":{"answer_ids":["a","missing"],"answers":[{"id":"a","complete":true,"claims":[]}]}}"#;
    assert!(evaluate_json(missing_answer).is_err());
}

#[test]
fn empty_rankings_and_no_positive_queries_have_explicit_nulls() {
    let input = r#"{"version":2,"k":2,"query_ids":["q"],"queries":[{"id":"q","judgment_status":"complete","judgments":[],"ranking":[]}]}"#;
    let report = evaluate_json(input).expect("valid empty ranking");
    let q = &report.queries[0];
    assert_eq!(q.recall, None);
    assert_eq!(q.ndcg, None);
    assert_eq!(q.judged_at_k, None);
    assert_eq!(q.precision, 0.0);
}
