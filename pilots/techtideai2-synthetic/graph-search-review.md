# Graph search fix review

**Verdict: do not merge or publish.** The GraphQL repair removes the observed scalar-variable failure, but tenant filtering has an unresolved correctness/security boundary. This review was performed in the same session, not by an independent human reviewer.

## High: tenant identifiers are not compared as exact strings

In a disposable Weaviate 1.35.6 instance, `backend/tests/weaviate-search.integration.mjs` indexed chunks belonging to `test-org` and `test org`. A search for `test-org` returned the chunk belonging to `test org`; the regression test failed. Weaviate's text tokenization is a plausible cause, but the precise internal matching behavior and migration path require further verification. The public knowledge route validates organization IDs as GUIDs, so this test does **not** by itself prove the same collision or exploitability for accepted production IDs. Direct repository callers accept arbitrary strings. Do not claim tenant isolation or merge the change until exact matching and existing-index migration are designed and verified with realistic allowed IDs.

## Coverage and execution limits

- The prior live smoke test passed with dissimilar organization IDs and collection filters, and the pilot completed; that does not disprove the new collision case.
- The live integration test is a standalone Node test and is not included in `pnpm run verify` or CI by default. Its latest execution intentionally fails on the collision case. Include a safe, isolated integration gate before relying on it.
- The pilot used local lexical-hash vectors, a separate AI review of draft labels, and isolated config/error-class imports. It is not a production embedding or application-runtime benchmark.
- The frozen install succeeds with pnpm 9.15.9, matching the CI major; the machine-wide pnpm 12 remains unchanged. Use `pnpm dlx pnpm@9.15.9 ...` until the project's toolchain pin is decided.
- `pnpm dlx pnpm@9.15.9 run verify` passed and all four CI TypeScript checks passed. Python 3.11 tests passed in an isolated container (20 tests). The golden model eval was not run: it invokes an agent runtime and specifies `gpt-4o`, so it may require credentials or incur external cost. No secrets or production data were used.
