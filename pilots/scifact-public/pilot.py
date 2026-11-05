import csv
import hashlib
import json
import os
import re
import sqlite3
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

root = Path(__file__).resolve().parent
source = root / 'source' / 'scifact'
output = root / 'results'
output.mkdir(exist_ok=False)
exe = Path(r'C:\Users\Admin\TechTide\Tools\tide-rill\target\debug\tide-rill.exe')

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def jsonl(path):
    with path.open(encoding='utf-8') as handle:
        return [json.loads(line) for line in handle if line.strip()]

corpus = jsonl(source / 'corpus.jsonl')
query_rows = jsonl(source / 'queries.jsonl')
queries = {row['_id']: row for row in query_rows}
assert len(corpus) == 5183 and len(queries) == len(query_rows) == 1109
ids = {row['_id'] for row in corpus}
assert len(ids) == len(corpus)
with (source / 'qrels' / 'test.tsv').open(encoding='utf-8', newline='') as handle:
    rows = list(csv.DictReader(handle, delimiter='\t'))
qrels = defaultdict(list)
seen = set()
for row in rows:
    qid, doc, score = row['query-id'], row['corpus-id'], int(row['score'])
    assert qid in queries and doc in ids and score in (0, 1, 2) and (qid, doc) not in seen
    seen.add((qid, doc))
    qrels[qid].append({'id':doc, 'grade':score})
assert len(rows) == 339 and len(qrels) == 300

# Only test-set query IDs select queries; judgments never affect the ranking.
db = sqlite3.connect(':memory:')
db.execute("CREATE VIRTUAL TABLE docs USING fts5(id UNINDEXED, title, body, tokenize='unicode61')")
db.executemany('INSERT INTO docs(id,title,body) VALUES(?,?,?)',
               ((row['_id'], row['title'], row['text']) for row in corpus))
rankings = {}
for qid in sorted(qrels):
    terms = list(dict.fromkeys(re.findall(r'[a-z0-9]+', queries[qid]['text'].lower())))
    assert terms, f'no searchable terms for {qid}'
    query = ' OR '.join('"' + term + '"' for term in terms)
    results = db.execute('SELECT id, bm25(docs) AS score FROM docs WHERE docs MATCH ? ORDER BY score, id LIMIT 10', (query,)).fetchall()
    assert results, f'no results for {qid}; do not silently score a failed search'
    rankings[qid] = [{'id':doc, 'score':score} for doc, score in results]
db.close()
rank_file = output / 'rankings.json'
rank_file.write_text(json.dumps(rankings, sort_keys=True, indent=2)+'\n', encoding='utf-8')

cases = []
evidence = []
for qid in sorted(qrels):
    cases.append({'id':qid, 'judgment_status':'complete', 'judgments':qrels[qid], 'ranking':[x['id'] for x in rankings[qid]]})
    for judgment in qrels[qid]:
        metadata = queries[qid].get('metadata', {}).get(judgment['id'])
        evidence.append({'query_id':qid, 'document_id':judgment['id'], 'grade':judgment['grade'], 'rationales':metadata})
inp = {'version':2, 'k':10, 'query_ids':sorted(qrels), 'queries':cases}
input_file = output / 'input.json'
input_file.write_text(json.dumps(inp, sort_keys=True, indent=2)+'\n', encoding='utf-8')
(output / 'evidence-map.json').write_text(json.dumps(evidence, sort_keys=True, indent=2)+'\n', encoding='utf-8')
run = subprocess.run([str(exe),'eval',str(input_file)], check=True, capture_output=True, text=True, encoding='utf-8')
report = json.loads(run.stdout)
assert len(report['queries']) == 300 and report['mrr_eligible_queries'] == 300 and report['faithfulness'] is None
(output / 'report.json').write_text(json.dumps(report, sort_keys=True, indent=2)+'\n', encoding='utf-8')
(output / 'qrels.trec').write_text(''.join(f"{qid} 0 {j['id']} {j['grade']}\n" for qid in sorted(qrels) for j in qrels[qid]), encoding='utf-8')
(output / 'run.trec').write_text(''.join(f"{qid} Q0 {r['id']} {i+1} {10-i} scifact_fts5\n" for qid in sorted(rankings) for i,r in enumerate(rankings[qid])), encoding='utf-8')
summary = {'query_count':len(qrels),'document_count':len(corpus),'qrels_count':len(rows),
           'qrels_with_metadata':sum(bool(e['rationales']) for e in evidence),
           'mean_recall_at_10':sum(q['recall'] for q in report['queries'])/300,
           'mean_precision_at_10':sum(q['precision'] for q in report['queries'])/300,
           'mean_ndcg_at_10':sum(q['ndcg'] for q in report['queries'])/300,
           'mrr_at_10':report['mean_reciprocal_rank'],
           'mean_judged_at_10':sum(q['judged_at_k'] for q in report['queries'])/300,
           'faithfulness':None,
           'python_version':sys.version,'sqlite_version':sqlite3.sqlite_version,
           'sha256':{p.name:digest(p) for p in [root/'scifact.zip', source/'corpus.jsonl', source/'queries.jsonl', source/'qrels'/'test.tsv', Path(__file__), exe, rank_file, input_file, output/'report.json', output/'evidence-map.json']}}
(output / 'summary.json').write_text(json.dumps(summary, sort_keys=True, indent=2)+'\n', encoding='utf-8')
print(json.dumps(summary, indent=2))
