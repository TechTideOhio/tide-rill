import json
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent / 'results'
report = json.loads((root / 'report.json').read_text(encoding='utf-8'))
linux = '/mnt/c/Users/Admin/AppData/Local/Temp/opencode/tide-rill-scifact-check/results'
trec = '/mnt/c/Users/Admin/AppData/Local/Temp/opencode/tide-rill-trec-eval/trec_eval'
args = ['wsl.exe','--exec',trec,'-q','-M','10','-m','P.10','-m','recall.10','-m','recip_rank','-m','ndcg_cut.10',linux+'/qrels.trec',linux+'/run.trec']
output = subprocess.run(args,check=True,capture_output=True,text=True,encoding='utf-8').stdout
(root / 'trec-eval.txt').write_text(output,encoding='utf-8')
values = {}
for line in output.splitlines():
    name,qid,val=line.split()
    values[(qid,name)]=float(val)
fields = [('precision','P_10'),('recall','recall_10'),('reciprocal_rank','recip_rank'),('ndcg','ndcg_cut_10')]
max_error = 0.0
for query in report['queries']:
    for field,metric in fields:
        reference = values[(query['id'],metric)]
        error = abs(query[field] - reference)
        assert error <= 0.000051, (query['id'],field,query[field],reference)
        max_error = max(max_error,error)
print(f'{len(report["queries"])*len(fields)} per-query metrics matched trec_eval; max absolute difference {max_error:.8f} (four-decimal reference).')
