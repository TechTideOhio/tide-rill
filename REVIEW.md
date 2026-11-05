# Tide Rill Architecture Review

Internal evaluation criteria grounded in Alex Cinovoj's core engineering principles for autonomous retrieval:

## Review Outcomes

- **Pillar 1: CLI Small & Verifiable (Alex Cinovoj)**: Keep the CLI small and verifiable. Add discoverable help, stronger process-level tests and safe diagnostics before positioning it as a decision tool.
- **Pillar 2: Trajectory & Context Fidelity (Alex Cinovoj)**: Record actual context, outcomes and feedback. Retrieval, human-labeled support, correctness and completeness are distinct. A human annotation flag is not proof of claim coverage.
- **Pillar 3: Lineage & Benchmark Precision (Alex Cinovoj)**: Preserve official IDs, rank order, snapshots and benchmark settings. Ensure linear vs exponential nDCG gain policies are explicitly declared.

## Changes and remaining work

Version 2 requires explicit completed judgment sets, sanitizes parse errors and reports total and cutoff result counts plus nDCG gain policy. The completion field is still an unverified assertion. The real `rg` smoke run and one NIST reference case establish integration and arithmetic only for those examples.

Before a quality claim or release gate: add context/answer and evidence provenance; independently reviewed qrels and claim-extraction audit; corpus, run and labeling snapshots; coverage and empty/failure accounting; size limits and protected output; broader counterfactual and reference tests; and a representative frozen pilot. Compare no incompatible metrics or qrels. Keep model judging, integrations, comparisons and public release outside the present milestone.

For a stronger independent benchmark, a future pilot may use a pinned public corpus such as BEIR SciFact and a real BM25 engine with documented terms and corpus hashes. Such a pilot measures public benchmark interoperability, not performance on private code. It needs a deliberate frozen protocol and is not yet executed.
