# SciFact public-data interoperability check

## Result

Tide Rill processed the BEIR SciFact test qrels for **300 queries and 5,183 documents**. On a local SQLite FTS5 BM25 baseline at k=10, the observed mean recall was **0.7989**, MRR **0.6354**, linear nDCG **0.6695**, and precision **0.0887**. These are properties of this particular retriever and dataset, not scores for Tide Rill itself or TechTide's application.

The output matched pinned NIST `trec_eval` on **1,200 per-query metric values** (precision, recall, reciprocal rank and linear nDCG for 300 queries) within 0.000051, the four-decimal reference's rounding precision. Faithfulness was **not evaluated**: the report contains `null`.

## Data and provenance

- Source: [BEIR SciFact](https://github.com/beir-cellar/beir/wiki/Datasets-available), downloaded from `https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/scifact.zip`. The downloaded archive MD5 matched BEIR's published `5f7d1de60b170fc8027bb7898e2efca1`; its SHA-256 is `536e14446a0ba56ed1398ab1055f39fe852686ecad24a6306c80c490fa8e0165`.
- Original scientific-claim dataset: [Wadden et al., 2020](https://aclanthology.org/2020.emnlp-main.609/). The paper describes expert and life-science annotators verifying claims they did not write; a subset was independently re-annotated. This is an existing research judgment set, not a new independent human review of every pair in this pilot.
- The BEIR test split contains 339 judged query-document pairs, all grade 1. `results/evidence-map.json` maps each pair to the corresponding query metadata when present. **Only 209 of 339 pairs had rationale metadata in the BEIR query file**; the remaining 130 have `null` rationale metadata. Source files and artifact SHA-256 hashes are in `results/summary.json`. Do not treat this as complete evidence-span provenance.
- The original SciFact repository [license](https://github.com/allenai/scifact/blob/master/LICENSE.md) lists claims and evidence as CC BY 4.0 and abstracts as ODC-By 1.0; some third-party dataset cards report noncommercial terms. This is a local research check, not permission to redistribute or use commercially. Verify applicable terms before either.

## Method and limitations

`pilot.py` loaded the 300 test query IDs from the qrels but did not use relevance judgments when ranking. It indexed title and abstract with SQLite FTS5 `unicode61`; queries used distinct lowercased ASCII alphanumeric terms joined with OR. Results were sorted by FTS5's `bm25` score and document ID for ties and capped at ten. Every query returned ten results. SQLite version and hashes are recorded in `results/summary.json`.

The `judgment_status: complete` input value means the provided BEIR qrel set was imported; it **does not mean all corpus documents were judged**. Unjudged retrieved documents receive zero numerical gain. Mean judged-at-10 was **0.0887**, so precision and nDCG in particular should not be interpreted as definitive judgments on the other results. SciFact is biomedical research, not representative of TechTide's code-retrieval workload. No held-out application dataset, provenance-complete answer/claim judgments, or automated faithfulness verifier was evaluated.

The adapter and reference scripts are retained as snapshots with their outputs. They use paths for the original Windows/WSL scratch environment under `C:/Users/Admin/AppData/Local/Temp/opencode/tide-rill-scifact-check/` and deliberately refuse to overwrite an existing results directory. Reproduction elsewhere requires adjusting those paths, restoring the exact source archive, and creating a fresh output directory. The original corpus and query text are not copied here. The input, ranked IDs, evidence map, scores and oracle output are in `results/`; treat them as derived third-party research data.
