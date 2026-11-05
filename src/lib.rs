#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    version: u32,
    k: usize,
    query_ids: Vec<String>,
    queries: Vec<Query>,
    faithfulness: Option<Faithfulness>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    judgment_status: JudgmentStatus,
    judgments: Vec<Judgment>,
    ranking: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum JudgmentStatus {
    Complete,
    Incomplete,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Judgment {
    id: String,
    grade: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Faithfulness {
    answer_ids: Vec<String>,
    answers: Vec<Answer>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Answer {
    id: String,
    complete: bool,
    #[serde(default)]
    context: Vec<String>,
    claims: Vec<Claim>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    id: String,
    label: Label,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Label {
    Supported,
    Contradicted,
    Insufficient,
    Partial,
    NotCheckable,
}

#[derive(Serialize)]
pub struct Report {
    pub version: u32,
    pub k: usize,
    pub ndcg_gain: &'static str,
    pub queries: Vec<QueryReport>,
    pub mean_reciprocal_rank: Option<f64>,
    pub mrr_eligible_queries: usize,
    pub faithfulness: Option<FaithfulnessReport>,
}

#[derive(Serialize)]
pub struct QueryReport {
    pub id: String,
    pub recall: Option<f64>,
    pub precision: f64,
    pub hit: Option<f64>,
    pub reciprocal_rank: Option<f64>,
    pub ndcg: Option<f64>,
    pub judged_at_k: Option<f64>,
    pub returned_total: usize,
    pub returned_at_k: usize,
    pub judged_positives: usize,
}

#[derive(Serialize)]
pub struct FaithfulnessReport {
    pub support_rate: Option<f64>,
    pub supported: usize,
    pub checkable: usize,
    pub not_checkable: usize,
    pub answers: usize,
    pub lexical_checked: usize,
    pub lexical_supported: usize,
    pub lexical_support_rate: Option<f64>,
    pub label_conflicts: usize,
}

fn insert_id<'a>(ids: &mut BTreeSet<&'a str>, id: &'a str) -> Result<(), String> {
    if id.trim().is_empty() || !ids.insert(id) {
        return Err("blank or duplicate identifier".into());
    }
    Ok(())
}

fn declared_ids(ids: &[String]) -> Result<BTreeSet<&str>, String> {
    let mut set = BTreeSet::new();
    for id in ids {
        insert_id(&mut set, id)?;
    }
    if set.is_empty() {
        return Err("empty declared universe".into());
    }
    Ok(set)
}

/// Evaluate a single versioned JSON input; malformed or incomplete inputs fail closed.
pub fn evaluate_json(json: &str) -> Result<Report, String> {
    let input: Input = serde_json::from_str(json)
        .map_err(|e| format!("invalid input at line {}, column {}", e.line(), e.column()))?;
    if input.version != 2 || !(1..=10_000).contains(&input.k) || input.queries.is_empty() {
        return Err("unsupported version, invalid cutoff, or empty query set".into());
    }
    let mut query_ids = BTreeSet::new();
    let declared_queries = declared_ids(&input.query_ids)?;
    let mut reports = Vec::new();
    for q in &input.queries {
        insert_id(&mut query_ids, &q.id)?;
        if matches!(q.judgment_status, JudgmentStatus::Incomplete) {
            return Err("incomplete judgments".into());
        }
        let mut grades = BTreeMap::new();
        for j in &q.judgments {
            if j.id.trim().is_empty()
                || j.grade > 2
                || grades.insert(j.id.as_str(), j.grade).is_some()
            {
                return Err("invalid or duplicate judgment".into());
            }
        }
        let mut ranking_ids = BTreeSet::new();
        for id in &q.ranking {
            insert_id(&mut ranking_ids, id)?;
        }
        let relevant = grades.values().filter(|&&grade| grade > 0).count();
        let mut hits = 0;
        let mut first = None;
        let mut judged = 0;
        let mut dcg = 0.0;
        for (index, id) in q.ranking.iter().take(input.k).enumerate() {
            if let Some(grade) = grades.get(id.as_str()) {
                judged += 1;
                if *grade > 0 {
                    hits += 1;
                    if first.is_none() {
                        first = Some(index + 1);
                    }
                }
                dcg += f64::from(*grade) / ((index + 2) as f64).log2();
            }
        }
        let mut ideal: Vec<u8> = grades.values().copied().collect();
        ideal.sort_unstable_by(|a, b| b.cmp(a));
        let idcg: f64 = ideal
            .iter()
            .take(input.k)
            .enumerate()
            .map(|(i, g)| f64::from(*g) / ((i + 2) as f64).log2())
            .sum();
        let returned = q.ranking.len().min(input.k);
        reports.push(QueryReport {
            id: q.id.clone(),
            recall: (relevant > 0).then(|| hits as f64 / relevant as f64),
            precision: hits as f64 / input.k as f64,
            hit: (relevant > 0).then_some(if hits > 0 { 1.0 } else { 0.0 }),
            reciprocal_rank: (relevant > 0).then(|| first.map_or(0.0, |r| 1.0 / r as f64)),
            ndcg: (idcg > 0.0).then(|| dcg / idcg),
            judged_at_k: (returned > 0).then(|| judged as f64 / returned as f64),
            returned_total: q.ranking.len(),
            returned_at_k: returned,
            judged_positives: relevant,
        });
    }
    if query_ids != declared_queries {
        return Err("query universe does not match records".into());
    }
    let faithfulness = input
        .faithfulness
        .map(|f| -> Result<FaithfulnessReport, String> {
            if f.answers.is_empty() {
                return Err("empty answer set".into());
            }
            let declared_answers = declared_ids(&f.answer_ids)?;
            let mut answer_ids = BTreeSet::new();
            let (mut supported, mut checkable, mut not_checkable) = (0, 0, 0);
            let (mut lex_checked, mut lex_supported, mut conflicts) = (0, 0, 0);
            for answer in &f.answers {
                insert_id(&mut answer_ids, &answer.id)?;
                if !answer.complete {
                    return Err("incomplete annotation".into());
                }
                let mut claims = BTreeSet::new();
                for claim in &answer.claims {
                    insert_id(&mut claims, &claim.id)?;
                    if let Some(text) = claim.text.as_deref() {
                        if !answer.context.is_empty() && !matches!(claim.label, Label::NotCheckable)
                        {
                            if let Some(ok) = lexical_supported(text, &answer.context) {
                                lex_checked += 1;
                                lex_supported += usize::from(ok);
                                conflicts +=
                                    usize::from(!ok && matches!(claim.label, Label::Supported));
                            }
                        }
                    }
                    match claim.label {
                        Label::Supported => {
                            supported += 1;
                            checkable += 1;
                        }
                        Label::NotCheckable => not_checkable += 1,
                        Label::Contradicted | Label::Insufficient | Label::Partial => {
                            checkable += 1;
                        }
                    }
                }
            }
            if answer_ids != declared_answers {
                return Err("answer universe does not match records".into());
            }
            Ok(FaithfulnessReport {
                support_rate: (checkable > 0).then(|| supported as f64 / checkable as f64),
                supported,
                checkable,
                not_checkable,
                answers: f.answers.len(),
                lexical_checked: lex_checked,
                lexical_supported: lex_supported,
                lexical_support_rate: (lex_checked > 0)
                    .then(|| lex_supported as f64 / lex_checked as f64),
                label_conflicts: conflicts,
            })
        })
        .transpose()?;
    let eligible: Vec<f64> = reports.iter().filter_map(|q| q.reciprocal_rank).collect();
    let mean_reciprocal_rank =
        (!eligible.is_empty()).then(|| eligible.iter().sum::<f64>() / eligible.len() as f64);
    Ok(Report {
        version: 2,
        k: input.k,
        ndcg_gain: "linear",
        queries: reports,
        mean_reciprocal_rank,
        mrr_eligible_queries: eligible.len(),
        faithfulness,
    })
}

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "are", "was", "were", "with", "that", "this", "from", "has", "have",
    "had", "its", "not", "but", "can", "will", "all", "any", "into", "than", "then", "also",
];
const LEXICAL_THRESHOLD: f64 = 0.8;

fn content_tokens(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|t| t.chars().count() >= 3 && !STOPWORDS.contains(&t.as_str()))
        .collect()
}

/// Deterministic support check: at least 80% of the claim's content tokens must occur in a
/// single context chunk. This is a lexical overlap screen, not entailment verification.
/// Returns `None` when the claim has no content tokens.
fn lexical_supported(claim: &str, context: &[String]) -> Option<bool> {
    let claim_tokens = content_tokens(claim);
    if claim_tokens.is_empty() {
        return None;
    }
    let best = context
        .iter()
        .map(|chunk| {
            let chunk_tokens = content_tokens(chunk);
            claim_tokens
                .iter()
                .filter(|t| chunk_tokens.contains(*t))
                .count()
        })
        .max()
        .unwrap_or(0);
    Some(best as f64 / claim_tokens.len() as f64 >= LEXICAL_THRESHOLD)
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    pub min_recall: Option<f64>,
    pub min_recall_per_query: Option<f64>,
    pub min_precision: Option<f64>,
    pub min_mrr: Option<f64>,
    pub min_ndcg: Option<f64>,
    pub min_support_rate: Option<f64>,
    pub min_lexical_support_rate: Option<f64>,
    pub max_label_conflicts: Option<usize>,
}

#[derive(Serialize)]
pub struct Check {
    pub metric: &'static str,
    pub threshold: f64,
    pub observed: Option<f64>,
    pub passed: bool,
}

#[derive(Serialize)]
pub struct Verdict {
    pub passed: bool,
    pub checks: Vec<Check>,
    pub report: Report,
}

fn macro_mean(values: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    let defined: Vec<f64> = values.flatten().collect();
    (!defined.is_empty()).then(|| defined.iter().sum::<f64>() / defined.len() as f64)
}

/// Gate a report against per-metric floors. Fail closed: a threshold on a metric that is
/// undefined (null) fails, and a threshold set with no thresholds is rejected.
pub fn gate(report: Report, t: &Thresholds) -> Result<Verdict, String> {
    for v in [
        t.min_recall,
        t.min_recall_per_query,
        t.min_precision,
        t.min_mrr,
        t.min_ndcg,
        t.min_support_rate,
        t.min_lexical_support_rate,
    ]
    .into_iter()
    .flatten()
    {
        if !(0.0..=1.0).contains(&v) {
            return Err("threshold outside 0..=1".into());
        }
    }
    let mut checks = Vec::new();
    let mut floor = |metric: &'static str, threshold: Option<f64>, observed: Option<f64>| {
        if let Some(threshold) = threshold {
            let passed = observed.is_some_and(|o| o >= threshold);
            checks.push(Check {
                metric,
                threshold,
                observed,
                passed,
            });
        }
    };
    floor(
        "recall_macro",
        t.min_recall,
        macro_mean(report.queries.iter().map(|q| q.recall)),
    );
    let worst = report
        .queries
        .iter()
        .map(|q| q.recall)
        .collect::<Option<Vec<f64>>>()
        .and_then(|v| v.into_iter().reduce(f64::min));
    floor("recall_worst_query", t.min_recall_per_query, worst);
    floor(
        "precision_macro",
        t.min_precision,
        macro_mean(report.queries.iter().map(|q| Some(q.precision))),
    );
    floor("mrr", t.min_mrr, report.mean_reciprocal_rank);
    floor(
        "ndcg_macro",
        t.min_ndcg,
        macro_mean(report.queries.iter().map(|q| q.ndcg)),
    );
    let f = report.faithfulness.as_ref();
    floor(
        "support_rate",
        t.min_support_rate,
        f.and_then(|f| f.support_rate),
    );
    floor(
        "lexical_support_rate",
        t.min_lexical_support_rate,
        f.and_then(|f| f.lexical_support_rate),
    );
    if let Some(max) = t.max_label_conflicts {
        let observed = f.map(|f| f.label_conflicts);
        checks.push(Check {
            metric: "label_conflicts_max",
            threshold: max as f64,
            observed: observed.map(|o| o as f64),
            passed: observed.is_some_and(|o| o <= max),
        });
    }
    if checks.is_empty() {
        return Err("no thresholds supplied".into());
    }
    Ok(Verdict {
        passed: checks.iter().all(|c| c.passed),
        checks,
        report,
    })
}

pub fn gate_json(input_json: &str, thresholds_json: &str) -> Result<Verdict, String> {
    let t: Thresholds = serde_json::from_str(thresholds_json).map_err(|e| {
        format!(
            "invalid thresholds at line {}, column {}",
            e.line(),
            e.column()
        )
    })?;
    gate(evaluate_json(input_json)?, &t)
}
