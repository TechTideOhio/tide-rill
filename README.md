# tide-rill: TideOS Eval Kit (Tool 5)

**Evaluate retrieval separately from generation: recall@k, MRR, linear nDCG, faithfulness (human labels plus a deterministic lexical screen), and a fail-closed per-metric gate.**

Offline, pure-Rust (zero-FFI, `forbid(unsafe_code)`) retrieval and faithfulness evaluator. v0.1.0 adds `gate`. The gate is only as trustworthy as the labels fed to it: see limitations. No trademark clearance has been performed.

Synthetic arithmetic and CLI validation results, including remaining limitations, are recorded in [VALIDATION.md](VALIDATION.md).

```text
cargo run --locked -- eval examples/synthetic.json
cargo run --locked -- gate examples/synthetic.json examples/thresholds-pass.json   # exit 0
cargo run --locked -- gate examples/synthetic.json examples/thresholds-fail.json   # exit 1
```

JSON output includes query IDs: treat it as sensitive. Errors go to stderr; exit 0 means input was evaluated, not that quality passed. Exit 2 indicates input or usage error. The tool makes no network requests, but network isolation has not been tested. Do not put private exports in this directory or source control.

## Input v2

`examples/synthetic.json` documents the complete input. The nonempty `query_ids` array declares the query universe; `queries` must contain exactly one record per ID. Every query explicitly supplies `judgment_status: "complete"`, judgments and ranking, even if empty; incomplete or missing status is rejected. Completion is an assertion, not proof of exhaustive labeling. Grades are 0, 1, or 2; positive means grade > 0. Ranking order is authoritative. A failed retrieval must not be represented as an empty ranking; v2 cannot represent failed calls, so omit scoring that run and investigate it. No automated importer is provided yet.

For k 1..10000, recall divides hits by all judged positives. Precision divides by k. Reciprocal rank and hit are null with no judged positives. nDCG uses linear gain, log2 discounts and the ideal order of judged items. Unjudged items have zero numerical gain but are not asserted irrelevant; judged_at_k shows how much of the returned top k has judgments. Empty rankings yield null judged_at_k. MRR is the macro mean over queries with judged positives, with the eligible count. No-positive queries do not prove a task is unanswerable.

Optional `faithfulness` declares `answer_ids` and provides one completed annotation record per answer. Each claim has a unique ID per answer and a label: `supported`, `contradicted`, `insufficient`, `partial`, or `not_checkable`. The support rate is supported / (supported + contradicted + insufficient + partial), null when no claims are checkable. This is a human-label aggregation, not entailment verification. A declared `complete` flag is an assertion, not proof that annotators found every claim or evaluated the actual context. This prototype does not capture or verify context, evidence spans, rubric, annotator, snapshot, corpus, run provenance or independent review. Do not use it to gate faithfulness until those controls are designed and verified.

## Pilot limitations

- `examples/rg-smoke.json` is a manually transcribed real ripgrep run (`rg --files-with-matches --sort path --fixed-strings reciprocal_rank src tests`). It is an exact-match, path-ordered smoke test; its two illustrative labels were assigned after observing results, not independently reviewed, and it is not a representative or frozen evaluation dataset. The source files may change after capture. Run `cargo run --locked -- eval examples/rg-smoke.json` to score it.
- The single graded example was checked against NIST `trec_eval` commit `d95ca64e14a47d763ae349fb65e6d8cde4141dbd` (tag v9.0.8). With `-M 2`, the oracle returned precision, recall, reciprocal rank = 0.5000 and linear nDCG = 0.2398. This is a single cross-check at the oracle's printed precision, not broad parity. The Windows checkout had CRLF: its quicktest differs in line endings, but `diff --strip-trailing-cr` passed.
- NornicDB's exponential-gain nDCG is not numerically comparable to this tool's declared linear gain on graded qrels.
- There are no comparisons, integrations, release automation, snapshot checks, statistical guarantees, completeness auditing, privacy protections for outputs, or production gates. Input identifiers and JSON diagnostics can reveal private data.
- Validate the input contract, labeling workflow, test coverage, privacy controls, cross-platform behavior, and metric semantics before any pilot decision or public release.


## Gate (Tool 5 promotion)

`gate <input.json> <thresholds.json>` scores the input and checks per-metric floors: `min_recall` (macro), `min_recall_per_query` (worst query, so one weak query cannot hide behind an average), `min_precision`, `min_mrr`, `min_ndcg`, `min_support_rate`, `min_lexical_support_rate` (all 0..1) and `max_label_conflicts`. Exit 0 = all pass, 1 = a floor failed, 2 = input or usage error. Fail closed: a floor on an undefined (null) metric fails, empty thresholds and unknown keys are rejected.

## Lexical faithfulness screen

An answer may carry `context` (chunks) and each claim optional `text`. A claim is lexically supported when at least 80% of its content tokens (3+ chars, minus a small stopword list) occur in one context chunk. `label_conflicts` counts claims a human labeled `supported` that fail the screen: review those first. This is overlap, not entailment: paraphrases can fail and lexically similar contradictions can pass. Treat `lexical_support_rate` as a cheap tripwire beside the human labels, never a replacement.
