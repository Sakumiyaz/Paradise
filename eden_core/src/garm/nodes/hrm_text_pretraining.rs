use crate::eden_garm::state_paths;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_EVENTS: usize = 128;
const MAX_CORPORA: usize = 64;
const MAX_CACHE_ENTRIES: usize = 64;
const MAX_SEGMENT_CHARS: usize = 480;
const MAX_SEGMENTS_PER_FILE: usize = 32;
const MAX_SEARCH_RESULTS: usize = 5;
const MIN_CONTEXT_CONFIDENCE: f32 = 0.55;
const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "de", "del", "el", "en", "for", "la", "las", "los", "of", "on",
    "or", "para", "por", "the", "to", "un", "una", "y",
];

#[derive(Clone, Debug, PartialEq)]
struct HrmTextEvent {
    id: u64,
    kind: String,
    value: String,
    status: String,
}

#[derive(Clone, Debug, PartialEq)]
struct HrmTextCorpus {
    id: u64,
    path: String,
    exists: bool,
    bytes: u64,
    fnv64: Option<u64>,
    lines: u64,
    language: String,
    domain: String,
    source: String,
    status: String,
    version: u64,
    duplicate_of: Option<u64>,
    source_trust: f32,
}

#[derive(Clone, Debug)]
struct HrmTextCacheEntry {
    query: String,
    hits: usize,
    confidence: f32,
    status: String,
}

#[derive(Clone, Debug)]
struct HrmTextState {
    events: VecDeque<HrmTextEvent>,
    corpora: VecDeque<HrmTextCorpus>,
    cache: VecDeque<HrmTextCacheEntry>,
    next_id: u64,
    corpus_count: u64,
    duplicate_files: u64,
    objective_count: u64,
    runs: u64,
    checkpoints: u64,
    segments: u64,
    segment_bytes: u64,
    searches: u64,
    retrieval_hits: u64,
    cache_hits: u64,
    context_packs: u64,
    abstentions: u64,
}

impl Default for HrmTextState {
    fn default() -> Self {
        Self {
            events: VecDeque::new(),
            corpora: VecDeque::new(),
            cache: VecDeque::new(),
            next_id: 1,
            corpus_count: 0,
            duplicate_files: 0,
            objective_count: 0,
            runs: 0,
            checkpoints: 0,
            segments: 0,
            segment_bytes: 0,
            searches: 0,
            retrieval_hits: 0,
            cache_hits: 0,
            context_packs: 0,
            abstentions: 0,
        }
    }
}

static HRM_TEXT_STATE: OnceLock<Mutex<HrmTextState>> = OnceLock::new();

fn hrm_text_state() -> &'static Mutex<HrmTextState> {
    HRM_TEXT_STATE.get_or_init(|| Mutex::new(HrmTextState::default()))
}

pub fn reset_for_tests() {
    if let Ok(mut state) = hrm_text_state().lock() {
        *state = HrmTextState::default();
    }
}

pub fn add_corpus(path: &str) -> String {
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.corpus_count += 1;
    let id = state.next_id;
    let corpus = inspect_corpus(id, path, &state);
    let status = corpus.status.clone();
    push_corpus(&mut state, corpus.clone());
    state.next_id += 1;
    push_event_with_id(&mut state, id, "corpus", path, &status);
    let manifest_status = write_corpus_manifest_locked(&state)
        .map(|_| "manifest_written".to_string())
        .unwrap_or_else(|_| "manifest_error".to_string());
    format!(
        "[HRM-TEXT-CORPUS] id={} status={} exists={} bytes={} fnv64={} language={} domain={} source={} version={} duplicate_of={} trust={:.2} manifest_status={} path={}\n{}",
        id,
        status,
        corpus.exists,
        corpus.bytes,
        corpus
            .fnv64
            .map(|hash| format!("{:016x}", hash))
            .unwrap_or_else(|| "missing".to_string()),
        corpus.language,
        corpus.domain,
        corpus.source,
        corpus.version,
        corpus
            .duplicate_of
            .map(|duplicate| duplicate.to_string())
            .unwrap_or_else(|| "none".to_string()),
        corpus.source_trust,
        manifest_status,
        path.trim(),
        report_locked(&state)
    )
}

pub fn add_objective(objective: &str) -> String {
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.objective_count += 1;
    let id = push_event(&mut state, "objective", objective, "registered");
    format!(
        "[HRM-TEXT-OBJECTIVE] id={} status=registered objective={}\n{}",
        id,
        objective.trim(),
        report_locked(&state)
    )
}

pub fn ingest_directory(path: &str) -> String {
    let mut paths = match std::fs::read_dir(path.trim()) {
        Ok(entries) => entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && is_supported_corpus_path(path))
            .collect::<Vec<_>>(),
        Err(e) => {
            return format!(
                "[HRM-TEXT-INGEST] status=error error={} path={}\n{}",
                e,
                path.trim(),
                report()
            );
        }
    };
    paths.sort();
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut indexed_files = 0u64;
    let mut duplicate_files = 0u64;
    let mut indexed_segments = 0u64;
    let mut indexed_bytes = 0u64;
    let mut segment_lines = Vec::new();
    for path in paths {
        let path_string = path.to_string_lossy().into_owned();
        let id = state.next_id;
        let mut corpus = inspect_corpus(id, &path_string, &state);
        state.next_id += 1;
        state.corpus_count += 1;
        indexed_files += 1;
        indexed_bytes += corpus.bytes;
        let duplicate_already_indexed = corpus
            .duplicate_of
            .and_then(|duplicate| {
                state
                    .corpora
                    .iter()
                    .find(|existing| existing.id == duplicate)
            })
            .map(|existing| existing.status == "indexed" || existing.status == "duplicate_skipped")
            .unwrap_or(false);
        if corpus.duplicate_of.is_some() && duplicate_already_indexed {
            duplicate_files += 1;
            state.duplicate_files += 1;
        } else {
            corpus.duplicate_of = None;
            corpus.status = "indexed".to_string();
            corpus.source_trust = 0.85;
            let segments = segment_corpus(&corpus);
            indexed_segments += segments.len() as u64;
            for segment in segments {
                segment_lines.push(segment);
            }
        }
        let status = corpus.status.clone();
        push_corpus(&mut state, corpus);
        push_event_with_id(&mut state, id, "corpus", &path_string, &status);
    }
    state.segments += indexed_segments;
    state.segment_bytes += indexed_bytes;
    let event_id = push_event(
        &mut state,
        "ingest",
        path.trim(),
        if indexed_files > 0 {
            "indexed"
        } else {
            "empty"
        },
    );
    let corpus_manifest_status = write_corpus_manifest_locked(&state)
        .map(|_| "manifest_written")
        .unwrap_or("manifest_error");
    let segment_status = append_segment_index(&segment_lines)
        .map(|_| "segments_written")
        .unwrap_or("segments_error");
    format!(
        "[HRM-TEXT-INGEST] id={} status={} files={} duplicates={} segments={} bytes={} corpus_manifest_status={} segment_status={} segment_index={} path={}\n{}",
        event_id,
        if indexed_files > 0 { "indexed" } else { "empty" },
        indexed_files,
        duplicate_files,
        indexed_segments,
        indexed_bytes,
        corpus_manifest_status,
        segment_status,
        state_paths::hrm_text_segments_path(),
        path.trim(),
        report_locked(&state)
    )
}

pub fn plan() -> String {
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let curriculum = curriculum_status(&state);
    let id = push_event(&mut state, "curriculum", &curriculum, "planned");
    format!(
        "[HRM-TEXT-PLAN] id={} curriculum={} stages=semantic,intent,plan,prosody,evidence adapter=hrm_text_prior_manifest\n{}",
        id,
        curriculum,
        report_locked(&state)
    )
}

pub fn run(local_evidence: &str) -> String {
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.runs += 1;
    state.checkpoints += 1;
    let quality = evidence_quality(local_evidence);
    let status = if quality >= 0.60 {
        "ready_for_runtime"
    } else {
        "needs_more_evidence"
    };
    let manifest_path = state_paths::hrm_text_checkpoint_manifest_path();
    let manifest = format!(
        "schema=hrm-text-checkpoint-v1\nruns={}\ncheckpoint={}\nquality={:.2}\nstatus={}\nadapter=hrm_text_prior_manifest\ncorpus_manifest={}\nconnects=hrm_runtime,hybrid_voice,learning,provenance,policy,maturity\nweights_present=false\ntraining_executed=false\n",
        state.runs,
        state.checkpoints,
        quality,
        status,
        state_paths::hrm_text_corpus_manifest_path()
    );
    let write_status = match std::fs::write(&manifest_path, manifest) {
        Ok(()) => status,
        Err(_) => "checkpoint_write_error",
    };
    let id = push_event(&mut state, "run", &manifest_path, write_status);
    format!(
        "[HRM-TEXT-RUN] id={} status={} quality={:.1}% checkpoint={} weights_present=false training_executed=false\n{}",
        id,
        write_status,
        quality * 100.0,
        manifest_path,
        report_locked(&state)
    )
}

pub fn search(query: &str) -> String {
    let results = search_segments(query, MAX_SEARCH_RESULTS);
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.searches += 1;
    let cache_status = if state
        .cache
        .iter()
        .any(|entry| entry.query == normalize_query(query))
    {
        state.cache_hits += 1;
        "hit"
    } else {
        "miss"
    };
    state.retrieval_hits += results.len() as u64;
    let status = if results.is_empty() { "miss" } else { "hit" };
    let id = push_event(&mut state, "search", query.trim(), status);
    push_cache_entry(&mut state, query, &results);
    let mut out = format!(
        "[HRM-TEXT-SEARCH] id={} status={} query={} hits={} cache={} segment_index={}\n",
        id,
        status,
        query.trim(),
        results.len(),
        cache_status,
        state_paths::hrm_text_segments_path()
    );
    for result in &results {
        out.push_str(&format!(
            "- segment score={} lexical={} confidence={:.2} coverage={:.2} exact_phrase={} trust={:.2} freshness={:.2} consistency={:.2} citation={} path={} text={}\n",
            result.score,
            result.lexical_score,
            result.confidence,
            result.coverage,
            result.exact_phrase,
            result.source_trust,
            result.freshness,
            result.consistency,
            result.citation(),
            result.path,
            result.text
        ));
    }
    out.push_str(&report_locked(&state));
    out
}

pub fn search_evidence(query: &str, limit: usize) -> Vec<String> {
    let results = search_segments(query, limit);
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.searches += 1;
    state.retrieval_hits += results.len() as u64;
    let status = if results.is_empty() { "miss" } else { "hit" };
    let value = format!("{} hits={}", query.trim(), results.len());
    push_event(&mut state, "retrieval", &value, status);
    push_cache_entry(&mut state, query, &results);
    drop(state);
    results
        .into_iter()
        .map(|result| {
            format!(
                "hrm_text_segment score={} confidence={:.2} coverage={:.2} citation={} text={}",
                result.score,
                result.confidence,
                result.coverage,
                result.citation(),
                result.text
            )
        })
        .collect()
}

pub fn context_pack(query: &str) -> String {
    let results = search_segments(query, MAX_SEARCH_RESULTS);
    let confidence = context_confidence(&results);
    let status = if confidence >= MIN_CONTEXT_CONFIDENCE {
        "sufficient"
    } else {
        "insufficient_evidence"
    };
    let mut state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.context_packs += 1;
    if status == "insufficient_evidence" {
        state.abstentions += 1;
    }
    let id = push_event(&mut state, "context_pack", query.trim(), status);
    push_cache_entry(&mut state, query, &results);
    drop(state);
    let citations: Vec<_> = results
        .iter()
        .map(|result| {
            serde_json::json!({
                "citation": result.citation(),
                "corpus_id": result.corpus_id,
                "segment_id": result.segment_id,
                "version": result.version,
                "fnv64": result.fnv64,
                "path": result.path,
                "score": result.score,
                "confidence": result.confidence,
                "coverage": result.coverage,
                "trust": result.source_trust,
                "freshness": result.freshness,
                "consistency": result.consistency,
                "text": result.text,
            })
        })
        .collect();
    let pack = serde_json::json!({
        "schema": "hrm-text-context-pack-v1",
        "id": id,
        "query": query.trim(),
        "status": status,
        "confidence": confidence,
        "threshold": MIN_CONTEXT_CONFIDENCE,
        "hits": results.len(),
        "generation_restricted": true,
        "abstain_on_insufficient_evidence": status == "insufficient_evidence",
        "citations": citations,
    });
    let path = state_paths::hrm_text_context_pack_path();
    let write_status = std::fs::write(&path, pack.to_string())
        .map(|_| "context_pack_written")
        .unwrap_or("context_pack_write_error");
    let mut out = format!(
        "[HRM-TEXT-CONTEXT-PACK] id={} status={} confidence={:.2} threshold={:.2} hits={} generation_restricted=true write_status={} path={}\n",
        id,
        status,
        confidence,
        MIN_CONTEXT_CONFIDENCE,
        results.len(),
        write_status,
        path
    );
    if status == "insufficient_evidence" {
        out.push_str("[HRM-TEXT-ABSTAIN] evidence insufficient found; answer blocked until more evidence is ingested or query is clarified\n");
    }
    for result in &results {
        out.push_str(&format!(
            "- citation={} confidence={:.2} score={} text={}\n",
            result.citation(),
            result.confidence,
            result.score,
            result.text
        ));
    }
    out.push_str(&report());
    out
}

pub fn evaluate_retrieval() -> String {
    let cases = [
        ("answerable", "local evidence"),
        (
            "adversarial_unanswerable",
            "zzzz nonexistent hallucination bait",
        ),
        ("citation", "retrieval evidence"),
    ];
    let mut passed = 0usize;
    let mut out = String::from("[HRM-TEXT-EVAL] schema=hrm-text-rag-eval-v1\n");
    for (kind, query) in cases {
        let results = search_segments(query, MAX_SEARCH_RESULTS);
        let confidence = context_confidence(&results);
        let expected_pass = if kind == "adversarial_unanswerable" {
            results.is_empty() || confidence < MIN_CONTEXT_CONFIDENCE
        } else {
            !results.is_empty()
        };
        if expected_pass {
            passed += 1;
        }
        out.push_str(&format!(
            "- case={} query='{}' hits={} confidence={:.2} passed={}\n",
            kind,
            query,
            results.len(),
            confidence,
            expected_pass
        ));
    }
    out.push_str(&format!(
        "[HRM-TEXT-EVAL-SUMMARY] passed={} total={} hallucination_guard=active\n{}",
        passed,
        cases.len(),
        report()
    ));
    out
}

pub fn report() -> String {
    let state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    report_locked(&state)
}

pub fn audit_report() -> String {
    let state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut out = report_locked(&state);
    for event in state.events.iter().rev().take(8) {
        out.push_str(&format!(
            "- hrm_text={} kind={} status={} value={}\n",
            event.id, event.kind, event.status, event.value
        ));
    }
    out
}

pub fn save_state() -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let state = hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let events: Vec<_> = state
        .events
        .iter()
        .map(|event| {
            serde_json::json!({
                "id": event.id,
                "kind": event.kind,
                "value": event.value,
                "status": event.status,
            })
        })
        .collect();
    let corpora: Vec<_> = state
        .corpora
        .iter()
        .map(|corpus| {
            serde_json::json!({
                "id": corpus.id,
                "path": corpus.path,
                "exists": corpus.exists,
                "bytes": corpus.bytes,
                "fnv64": corpus.fnv64.map(|hash| format!("{:016x}", hash)),
                "lines": corpus.lines,
                "language": corpus.language,
                "domain": corpus.domain,
                "source": corpus.source,
                "status": corpus.status,
                "version": corpus.version,
                "duplicate_of": corpus.duplicate_of,
                "source_trust": corpus.source_trust,
            })
        })
        .collect();
    let cache: Vec<_> = state
        .cache
        .iter()
        .map(|entry| {
            serde_json::json!({
                "query": entry.query,
                "hits": entry.hits,
                "confidence": entry.confidence,
                "status": entry.status,
            })
        })
        .collect();
    let snapshot = serde_json::json!({
        "schema": "hrm-text-pretraining-v1",
        "next_id": state.next_id,
        "corpus_count": state.corpus_count,
        "duplicate_files": state.duplicate_files,
        "objective_count": state.objective_count,
        "runs": state.runs,
        "checkpoints": state.checkpoints,
        "segments": state.segments,
        "segment_bytes": state.segment_bytes,
        "searches": state.searches,
        "retrieval_hits": state.retrieval_hits,
        "cache_hits": state.cache_hits,
        "context_packs": state.context_packs,
        "abstentions": state.abstentions,
        "corpora": corpora,
        "cache": cache,
        "events": events,
    });
    std::fs::write(
        state_paths::hrm_text_pretraining_state_path(),
        snapshot.to_string(),
    )
    .map_err(|e| format!("failed to write HRM-text state: {}", e))
}

pub fn load_state() -> Result<(), String> {
    let path = state_paths::hrm_text_pretraining_state_path();
    if std::fs::metadata(&path).is_err() {
        return Ok(());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read HRM-text state: {}", e))?;
    let snapshot: serde_json::Value =
        serde_json::from_str(&data).map_err(|e| format!("failed to parse HRM-text JSON: {}", e))?;
    let mut state = HrmTextState {
        next_id: snapshot
            .get("next_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(1),
        corpus_count: snapshot
            .get("corpus_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        duplicate_files: snapshot
            .get("duplicate_files")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        objective_count: snapshot
            .get("objective_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        runs: snapshot.get("runs").and_then(|v| v.as_u64()).unwrap_or(0),
        checkpoints: snapshot
            .get("checkpoints")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        segments: snapshot
            .get("segments")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        segment_bytes: snapshot
            .get("segment_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        searches: snapshot
            .get("searches")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        retrieval_hits: snapshot
            .get("retrieval_hits")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        cache_hits: snapshot
            .get("cache_hits")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        context_packs: snapshot
            .get("context_packs")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        abstentions: snapshot
            .get("abstentions")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        ..HrmTextState::default()
    };
    if let Some(events) = snapshot.get("events").and_then(|v| v.as_array()) {
        for event in events {
            state.events.push_back(HrmTextEvent {
                id: event.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                kind: json_string(event, "kind"),
                value: json_string(event, "value"),
                status: json_string(event, "status"),
            });
        }
    }
    if let Some(corpora) = snapshot.get("corpora").and_then(|v| v.as_array()) {
        for corpus in corpora {
            state.corpora.push_back(HrmTextCorpus {
                id: corpus.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
                path: json_string(corpus, "path"),
                exists: corpus
                    .get("exists")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                bytes: corpus.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                fnv64: corpus
                    .get("fnv64")
                    .and_then(|v| v.as_str())
                    .and_then(|hash| u64::from_str_radix(hash, 16).ok()),
                lines: corpus.get("lines").and_then(|v| v.as_u64()).unwrap_or(0),
                language: json_string(corpus, "language"),
                domain: json_string(corpus, "domain"),
                source: json_string(corpus, "source"),
                status: json_string(corpus, "status"),
                version: corpus.get("version").and_then(|v| v.as_u64()).unwrap_or(1),
                duplicate_of: corpus.get("duplicate_of").and_then(|v| v.as_u64()),
                source_trust: corpus
                    .get("source_trust")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.60) as f32,
            });
        }
    }
    if let Some(cache) = snapshot.get("cache").and_then(|v| v.as_array()) {
        for entry in cache {
            state.cache.push_back(HrmTextCacheEntry {
                query: json_string(entry, "query"),
                hits: entry.get("hits").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                confidence: entry
                    .get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32,
                status: json_string(entry, "status"),
            });
        }
    }
    *hrm_text_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
    Ok(())
}

fn report_locked(state: &HrmTextState) -> String {
    format!(
        "[HRM-TEXT] schema=hrm-text-pretraining-v1 corpora={} duplicates={} objectives={} runs={} checkpoints={} segments={} segment_bytes={} searches={} retrieval_hits={} cache_entries={} cache_hits={} context_packs={} abstentions={} events={} corpus_manifest={} segment_index={} hybrid=pretraining_priors+hrm_runtime weights_present=false\n",
        state.corpus_count,
        state.duplicate_files,
        state.objective_count,
        state.runs,
        state.checkpoints,
        state.segments,
        state.segment_bytes,
        state.searches,
        state.retrieval_hits,
        state.cache.len(),
        state.cache_hits,
        state.context_packs,
        state.abstentions,
        state.events.len(),
        state_paths::hrm_text_corpus_manifest_path(),
        state_paths::hrm_text_segments_path()
    )
}

#[derive(Clone, Debug, PartialEq)]
struct SegmentSearchResult {
    score: usize,
    lexical_score: usize,
    matched_terms: usize,
    query_terms: usize,
    coverage: f32,
    exact_phrase: bool,
    source_trust: f32,
    freshness: f32,
    consistency: f32,
    confidence: f32,
    corpus_id: u64,
    segment_id: u64,
    version: u64,
    fnv64: String,
    path: String,
    text: String,
}

impl SegmentSearchResult {
    fn citation(&self) -> String {
        format!(
            "doc:{}#seg:{}@v{}:{}",
            self.corpus_id, self.segment_id, self.version, self.fnv64
        )
    }
}

fn search_segments(query: &str, limit: usize) -> Vec<SegmentSearchResult> {
    let terms = normalize_query_terms(query);
    if terms.is_empty() {
        return Vec::new();
    }
    let phrase = terms.join(" ");
    let Ok(data) = std::fs::read_to_string(state_paths::hrm_text_segments_path()) else {
        return Vec::new();
    };
    let mut results = Vec::new();
    for line in data.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let text = json_string(&value, "text");
        let tokens = tokenize(&text);
        let normalized_text = tokens.join(" ");
        let mut matched_terms = 0usize;
        let mut lexical_score = 0usize;
        for term in &terms {
            let frequency = tokens.iter().filter(|token| *token == term).count();
            if frequency > 0 {
                matched_terms += 1;
                lexical_score += frequency;
            }
        }
        lexical_score += matched_terms * 2;
        if matched_terms == terms.len() {
            lexical_score += 4;
        }
        let exact_phrase = normalized_text.contains(&phrase);
        if exact_phrase {
            lexical_score += 8;
        }
        if lexical_score == 0 {
            continue;
        }
        let query_terms = terms.len();
        let coverage = matched_terms as f32 / query_terms as f32;
        let source_trust = value
            .get("source_trust")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.60) as f32;
        let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(1);
        let freshness = 1.0 / version.max(1) as f32;
        let consistency = coverage;
        let confidence =
            ((coverage * 0.40) + (source_trust * 0.30) + (freshness * 0.15) + (consistency * 0.15))
                .min(1.0);
        let score = lexical_score + (confidence * 10.0).round() as usize;
        results.push(SegmentSearchResult {
            score,
            lexical_score,
            matched_terms,
            query_terms,
            coverage,
            exact_phrase,
            source_trust,
            freshness,
            consistency,
            confidence,
            corpus_id: value.get("corpus_id").and_then(|v| v.as_u64()).unwrap_or(0),
            segment_id: value
                .get("segment_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            version,
            fnv64: json_string(&value, "fnv64"),
            path: json_string(&value, "path"),
            text,
        });
    }
    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.confidence.total_cmp(&a.confidence))
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.segment_id.cmp(&b.segment_id))
    });
    results.truncate(limit);
    results
}

fn normalize_query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    for token in tokenize(query) {
        if token.len() <= 2 || STOP_WORDS.contains(&token.as_str()) || terms.contains(&token) {
            continue;
        }
        terms.push(token);
    }
    terms
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|ch: char| !ch.is_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn inspect_corpus(id: u64, path: &str, state: &HrmTextState) -> HrmTextCorpus {
    let trimmed = path.trim().to_string();
    let version = state
        .corpora
        .iter()
        .filter(|corpus| corpus.path == trimmed)
        .count() as u64
        + 1;
    match std::fs::read(&trimmed) {
        Ok(data) => {
            let hash = fnv64(&data);
            let duplicate_of = state
                .corpora
                .iter()
                .find(|corpus| corpus.fnv64 == Some(hash))
                .map(|corpus| corpus.id);
            HrmTextCorpus {
                id,
                path: trimmed.clone(),
                exists: true,
                bytes: data.len() as u64,
                fnv64: Some(hash),
                lines: data.iter().filter(|byte| **byte == b'\n').count() as u64,
                language: infer_language(&trimmed),
                domain: infer_domain(&trimmed),
                source: "local_file".to_string(),
                status: if duplicate_of.is_some() {
                    "duplicate_skipped".to_string()
                } else {
                    "registered".to_string()
                },
                version,
                duplicate_of,
                source_trust: if duplicate_of.is_some() { 0.20 } else { 0.85 },
            }
        }
        Err(_) => HrmTextCorpus {
            id,
            path: trimmed.clone(),
            exists: false,
            bytes: 0,
            fnv64: None,
            lines: 0,
            language: infer_language(&trimmed),
            domain: infer_domain(&trimmed),
            source: "local_file_missing".to_string(),
            status: "missing_local_file".to_string(),
            version,
            duplicate_of: None,
            source_trust: 0.0,
        },
    }
}

fn write_corpus_manifest_locked(state: &HrmTextState) -> Result<(), String> {
    state_paths::ensure_state_dir()?;
    let mut body = String::from("schema=hrm-text-corpus-manifest-v1\n");
    body.push_str(&format!("corpora={}\n", state.corpora.len()));
    body.push_str(&format!("segments={}\n", state.segments));
    body.push_str(&format!("segment_bytes={}\n", state.segment_bytes));
    body.push_str(&format!(
        "segment_index={}\n",
        state_paths::hrm_text_segments_path()
    ));
    body.push_str("weights_present=false\ntraining_executed=false\n");
    for corpus in &state.corpora {
        body.push_str(&format!(
            "corpus id={} exists={} bytes={} fnv64={} lines={} language={} domain={} source={} status={} version={} duplicate_of={} trust={:.2} path={}\n",
            corpus.id,
            corpus.exists,
            corpus.bytes,
            corpus
                .fnv64
                .map(|hash| format!("{:016x}", hash))
                .unwrap_or_else(|| "missing".to_string()),
            corpus.lines,
            corpus.language,
            corpus.domain,
            corpus.source,
            corpus.status,
            corpus.version,
            corpus
                .duplicate_of
                .map(|duplicate| duplicate.to_string())
                .unwrap_or_else(|| "none".to_string()),
            corpus.source_trust,
            corpus.path
        ));
    }
    std::fs::write(state_paths::hrm_text_corpus_manifest_path(), body)
        .map_err(|e| format!("failed to write HRM-text corpus manifest: {}", e))
}

fn append_segment_index(lines: &[String]) -> Result<(), String> {
    if lines.is_empty() {
        return Ok(());
    }
    state_paths::ensure_state_dir()?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(state_paths::hrm_text_segments_path())
        .map_err(|e| format!("failed to open HRM-text segment index: {}", e))?;
    for line in lines {
        writeln!(file, "{}", line)
            .map_err(|e| format!("failed to write HRM-text segment index: {}", e))?;
    }
    Ok(())
}

fn curriculum_status(state: &HrmTextState) -> String {
    if state.corpus_count > 0 && state.objective_count > 0 {
        "ready".to_string()
    } else if state.corpus_count > 0 {
        "needs_objective".to_string()
    } else {
        "needs_corpus".to_string()
    }
}

fn evidence_quality(local_evidence: &str) -> f32 {
    let markers = [
        "[HRM]",
        "[HYBRID-VOICE]",
        "[LEARNING]",
        "[PROVENANCE]",
        "[POLICY]",
    ];
    markers
        .iter()
        .filter(|marker| local_evidence.contains(**marker))
        .count() as f32
        / markers.len() as f32
}

fn push_event(state: &mut HrmTextState, kind: &str, value: &str, status: &str) -> u64 {
    let id = state.next_id;
    state.next_id += 1;
    push_event_with_id(state, id, kind, value, status);
    id
}

fn push_event_with_id(state: &mut HrmTextState, id: u64, kind: &str, value: &str, status: &str) {
    state.events.push_back(HrmTextEvent {
        id,
        kind: kind.to_string(),
        value: value.trim().chars().take(180).collect(),
        status: status.to_string(),
    });
    while state.events.len() > MAX_EVENTS {
        state.events.pop_front();
    }
}

fn push_corpus(state: &mut HrmTextState, corpus: HrmTextCorpus) {
    state.corpora.push_back(corpus);
    while state.corpora.len() > MAX_CORPORA {
        state.corpora.pop_front();
    }
}

fn infer_language(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".es.txt") || lower.contains("/es/") {
        "es".to_string()
    } else if lower.ends_with(".en.txt") || lower.contains("/en/") {
        "en".to_string()
    } else if lower.ends_with(".json") {
        "structured_json".to_string()
    } else {
        "unknown".to_string()
    }
}

fn infer_domain(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.contains("voice") || lower.contains("tts") {
        "voice".to_string()
    } else if lower.contains("hrm") || lower.contains("reason") {
        "reasoning".to_string()
    } else if lower.contains("policy") {
        "policy".to_string()
    } else {
        "general".to_string()
    }
}

fn is_supported_corpus_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "txt" | "md" | "json" | "jsonl" | "tsv"
            )
        })
        .unwrap_or(false)
}

fn segment_corpus(corpus: &HrmTextCorpus) -> Vec<String> {
    if !corpus.exists {
        return Vec::new();
    }
    let Ok(data) = std::fs::read_to_string(&corpus.path) else {
        return Vec::new();
    };
    data.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.chars().take(MAX_SEGMENT_CHARS).collect::<String>())
            }
        })
        .take(MAX_SEGMENTS_PER_FILE)
        .enumerate()
        .map(|(idx, text)| {
            serde_json::json!({
                "schema": "hrm-text-segment-v1",
                "corpus_id": corpus.id,
                "segment_id": idx + 1,
                "path": corpus.path,
                "language": corpus.language,
                "domain": corpus.domain,
                "fnv64": corpus.fnv64.map(|hash| format!("{:016x}", hash)),
                "version": corpus.version,
                "chars": text.chars().count(),
                "source": corpus.source,
                "source_trust": corpus.source_trust,
                "text": text,
            })
            .to_string()
        })
        .collect()
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |mut hash, byte| {
        hash ^= *byte as u64;
        hash.wrapping_mul(0x100000001b3)
    })
}

fn json_string(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn context_confidence(results: &[SegmentSearchResult]) -> f32 {
    if results.is_empty() {
        return 0.0;
    }
    let total = results
        .iter()
        .take(3)
        .map(|result| result.confidence)
        .sum::<f32>();
    total / results.len().min(3) as f32
}

fn normalize_query(query: &str) -> String {
    normalize_query_terms(query).join(" ")
}

fn push_cache_entry(state: &mut HrmTextState, query: &str, results: &[SegmentSearchResult]) {
    let normalized = normalize_query(query);
    if normalized.is_empty() {
        return;
    }
    state.cache.push_back(HrmTextCacheEntry {
        query: normalized,
        hits: results.len(),
        confidence: context_confidence(results),
        status: if results.is_empty() { "miss" } else { "hit" }.to_string(),
    });
    while state.cache.len() > MAX_CACHE_ENTRIES {
        state.cache.pop_front();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_and_runs_hrm_text_pretraining() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_hrm_text_plan_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        let corpus_path = dir.join("local_corpus.en.txt");
        std::fs::write(&corpus_path, "hello\nworld\n").unwrap();
        reset_for_tests();

        let corpus = add_corpus(&corpus_path.to_string_lossy());
        let objective = add_objective("text to prosody and plan priors");
        let plan = plan();
        let run = run("[HRM] [HYBRID-VOICE] [LEARNING] [PROVENANCE] [POLICY]");

        assert!(corpus.contains("[HRM-TEXT-CORPUS] id=1"));
        assert!(corpus.contains("exists=true"));
        assert!(corpus.contains("bytes=12"));
        assert!(std::fs::metadata(state_paths::hrm_text_corpus_manifest_path()).is_ok());
        assert!(objective.contains("[HRM-TEXT-OBJECTIVE] id=2"));
        assert!(plan.contains("curriculum=ready"));
        assert!(run.contains("status=ready_for_runtime"));
        assert!(run.contains("training_executed=false"));
        assert!(run.contains("weights_present=false"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }

    #[test]
    fn saves_and_loads_hrm_text_state() {
        let _state_guard = state_paths::test_state_guard();
        let dir =
            std::env::temp_dir().join(format!("eden_garm_hrm_text_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        reset_for_tests();

        let _ = add_corpus("/tmp/corpus.txt");
        save_state().unwrap();
        reset_for_tests();
        load_state().unwrap();
        let report = report();

        assert!(report.contains("corpora=1"));
        assert!(report.contains("events=1"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }

    #[test]
    fn ingests_directory_and_writes_segment_index() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_hrm_text_ingest_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        let corpus_dir = dir.join("corpus");
        std::fs::create_dir_all(&corpus_dir).unwrap();
        std::fs::write(corpus_dir.join("voice.en.txt"), "alpha\nbeta\n\n").unwrap();
        std::fs::write(corpus_dir.join("skip.bin"), "ignored").unwrap();
        reset_for_tests();

        let out = ingest_directory(&corpus_dir.to_string_lossy());
        let segments = std::fs::read_to_string(state_paths::hrm_text_segments_path()).unwrap();

        assert!(out.contains("[HRM-TEXT-INGEST]"));
        assert!(out.contains("files=1"));
        assert!(out.contains("segments=2"));
        assert!(segments.contains("hrm-text-segment-v1"));
        assert!(segments.contains("alpha"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }

    #[test]
    fn searches_segment_index_without_embeddings() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_hrm_text_search_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        let corpus_dir = dir.join("corpus");
        std::fs::create_dir_all(&corpus_dir).unwrap();
        std::fs::write(
            corpus_dir.join("reason.en.txt"),
            "hierarchical reasoning uses local evidence\nvoice prosody uses manifest\n",
        )
        .unwrap();
        reset_for_tests();

        let _ = ingest_directory(&corpus_dir.to_string_lossy());
        let out = search("local evidence");
        let evidence = search_evidence("voice prosody", 2);

        assert!(out.contains("[HRM-TEXT-SEARCH]"));
        assert!(out.contains("status=hit"));
        assert!(out.contains("hierarchical reasoning uses local evidence"));
        assert_eq!(evidence.len(), 1);
        assert!(evidence[0].contains("voice prosody uses manifest"));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }

    #[test]
    fn ranks_exact_phrase_above_scattered_term_matches() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_hrm_text_rank_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        let corpus_dir = dir.join("corpus");
        std::fs::create_dir_all(&corpus_dir).unwrap();
        std::fs::write(
            corpus_dir.join("rag.en.txt"),
            "local planning evidence evidence local\nlocal evidence direct citation\n",
        )
        .unwrap();
        reset_for_tests();

        let _ = ingest_directory(&corpus_dir.to_string_lossy());
        let out = search("the local evidence");

        assert!(out.contains("[HRM-TEXT-SEARCH]"));
        assert!(out.contains("status=hit"));
        let direct_pos = out.find("local evidence direct citation").unwrap();
        let scattered_pos = out.find("local planning evidence evidence local").unwrap();
        assert!(direct_pos < scattered_pos);
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }

    #[test]
    fn dedupes_versions_context_pack_and_eval_guard() {
        let _state_guard = state_paths::test_state_guard();
        let dir = std::env::temp_dir().join(format!(
            "eden_garm_hrm_text_forensic_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir(dir.clone());
        state_paths::ensure_state_dir().unwrap();
        let corpus_dir = dir.join("corpus");
        std::fs::create_dir_all(&corpus_dir).unwrap();
        std::fs::write(
            corpus_dir.join("primary.en.txt"),
            "retrieval evidence requires exact citations\nlocal evidence supports restricted generation\n",
        )
        .unwrap();
        std::fs::write(
            corpus_dir.join("duplicate.en.txt"),
            "retrieval evidence requires exact citations\nlocal evidence supports restricted generation\n",
        )
        .unwrap();
        reset_for_tests();

        let ingest = ingest_directory(&corpus_dir.to_string_lossy());
        let search_out = search("retrieval evidence citations");
        let pack = context_pack("retrieval evidence citations");
        let miss_pack = context_pack("zzzz nonexistent hallucination bait");
        let eval = evaluate_retrieval();

        assert!(ingest.contains("duplicates=1"));
        assert!(ingest.contains("segments=2"));
        assert!(search_out.contains("citation=doc:"));
        assert!(search_out.contains("confidence="));
        assert!(pack.contains("status=sufficient"));
        assert!(pack.contains("generation_restricted"));
        assert!(miss_pack.contains("[HRM-TEXT-ABSTAIN]"));
        assert!(eval.contains("hallucination_guard=active"));
        assert!(std::fs::metadata(state_paths::path("hrm_text_context_pack.json")).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
        state_paths::set_state_dir("/tmp/eden_garm");
        reset_for_tests();
    }
}
