# Tide Rill validation: synthetic arithmetic and CLI behavior

**Result: partial validation.** The checks below support using this internal prototype for exploratory synthetic evaluation. They do not validate it as a benchmark, an automated faithfulness judge, or a production release gate.

A later public-data interoperability run on BEIR SciFact is documented in [pilots/scifact-public/README.md](pilots/scifact-public/README.md). It does not establish in-domain representativeness or complete evidence provenance.

## Checks performed

- `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked --all-targets` passed on Windows. The test command reported seven tests passing across the integration suites.
- `cargo +1.82.0 test --locked --all-targets` also passed on Windows with the declared minimum Rust version.
- The actual CLI processed `examples/synthetic.json` and reported recall 0.5, precision 0.5, reciprocal rank 0.5, linear nDCG 0.23981246656813146, and human-label support aggregation 0.5. These are synthetic labels, not an independently validated faithfulness assessment.
- Compared 24 generated synthetic queries at k=1, 3, and 5 against NIST `trec_eval` commit `d95ca64e14a47d763ae349fb65e6d8cde4141dbd`, executed in WSL. All 288 per-query precision, recall, reciprocal-rank, and linear nDCG comparisons matched within 0.000051, accommodating the oracle's four-decimal output. The reference was run with `-M k` to align its reciprocal-rank cutoff; grades above zero count as relevant. These generated cases had at least one positive judgment and nonempty rankings.
- Fifteen invalid CLI inputs, including missing file, malformed JSON, duplicate IDs, invalid grade/cutoff, incomplete judgments, and incomplete or inconsistent answer annotations, each returned exit code 2, empty stdout, and no synthetic private sentinel in stderr.
- Separate synthetic behavior checks covered no-positive nulls and MRR exclusion, missed-positive zero, judged-result accounting, improved ranking, all five support labels, and null support when all claims are not checkable.

## Boundaries

- The reference comparison tests metric arithmetic on generated examples; it does not establish corpus representativeness, human label quality, ingestion correctness, broad oracle parity, or real-world retrieval performance.
- Support scoring aggregates supplied labels only. It does not extract claims, inspect evidence, verify retrieved context or provenance, or establish answer correctness. The TechTideAI2 synthetic pilot has no faithfulness evaluation.
- Input and output can reveal query identifiers. Input sizes are not bounded and network isolation, alternate operating systems, and production-scale behavior were not tested in this run.
- A release gate still requires an independently judged, frozen representative dataset and documented evidence provenance, plus broader robustness, privacy, and platform testing.

## Evidence and reproduction

Validation was run on 2026-11-01 local time (2026-11-02 UTC), using Rust/Cargo 1.98.1 on `x86_64-pc-windows-gnu`. The debug binary SHA-256 was `e661e9f146b1506cd34bbe6b1c4731acbc5df2223c0526ed6334548610ef734c`; `src/lib.rs` SHA-256 was `67adbf272b6c48dcac2c12c5ca4f37622a7c675bae19d2e6f8d11383fe369f58`.

Temporary scripts `tide-rill-oracle-check.cjs`, `tide-rill-negative-check.cjs`, and `tide-rill-behavior-check.cjs`, along with `tide-rill-oracle-data/` and the pinned `tide-rill-trec-eval/` checkout, are under `C:/Users/Admin/AppData/Local/Temp/opencode/`. These temporary artifacts are not durable or a published benchmark. Gate logs are under `Tools/agent-harness/logs/2026-11-02T02-51-54-cargo-*.log`.
