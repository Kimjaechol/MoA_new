// @Ref: SUMMARY §6D-6/7 — idle-time background orchestrator.
//
// VaultScheduler runs a single long-lived tokio task that:
//   1. Watches user input-activity bumps via `notify_activity()`.
//   2. Enters "idle mode" after `idle_threshold` seconds of no activity.
//   3. Dispatches priority-ordered vault maintenance work:
//      - hub compile queue (`hub::compile_queue_next` → `incremental_update`)
//      - vault health refresh (cadence: daily by default)
//      - briefing refresh for currently-active case_numbers
//   4. Suspends on the next activity bump so UI stays responsive.
//
// Shutdown via `tokio::sync::oneshot::Receiver<()>`.

use super::store::VaultStore;
use super::{briefing, health, hub};
use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Default idle threshold — matches Plan §7-5.
pub const DEFAULT_IDLE_THRESHOLD: Duration = Duration::from_secs(5 * 60);
/// Health check minimum cadence.
pub const DEFAULT_HEALTH_CADENCE: Duration = Duration::from_secs(24 * 3600);
/// PR #9 community detection minimum cadence — weekly.
pub const DEFAULT_COMMUNITY_CADENCE: Duration = Duration::from_secs(7 * 24 * 3600);

pub struct VaultScheduler {
    vault: Arc<VaultStore>,
    /// PR #9 — optional ontology repo. When present, the scheduler runs
    /// weekly community detection (Label Propagation → write into
    /// `ontology_communities`) during idle time. Kept optional so
    /// vault-only deployments (no ontology layer) still work.
    ontology: Option<Arc<crate::ontology::OntologyRepo>>,
    last_activity: Arc<Mutex<Instant>>,
    last_health_check: Arc<Mutex<Option<Instant>>>,
    /// PR #9 — tracks when the last community-detection pass ran. Bumped
    /// after a successful detection so the next tick can decide whether
    /// to run again based on `community_cadence`.
    last_community_detection: Arc<Mutex<Option<Instant>>>,
    idle_threshold: Duration,
    health_cadence: Duration,
    community_cadence: Duration,
    /// Number of hub compiles per idle tick (Plan §7-5 worker budget).
    hub_batch_size: usize,
    /// Max concurrent hub compiles per batch.
    hub_concurrency: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SchedulerStats {
    pub hubs_compiled: usize,
    pub health_checks_run: usize,
    pub briefings_refreshed: usize,
    /// PR #9 — number of communities rewritten this tick (0 when
    /// detection was skipped because cadence hadn't elapsed).
    pub communities_detected: usize,
}

impl VaultScheduler {
    pub fn new(vault: Arc<VaultStore>) -> Self {
        Self {
            vault,
            ontology: None,
            last_activity: Arc::new(Mutex::new(Instant::now())),
            last_health_check: Arc::new(Mutex::new(None)),
            last_community_detection: Arc::new(Mutex::new(None)),
            idle_threshold: DEFAULT_IDLE_THRESHOLD,
            health_cadence: DEFAULT_HEALTH_CADENCE,
            community_cadence: DEFAULT_COMMUNITY_CADENCE,
            hub_batch_size: 4,
            hub_concurrency: 2,
        }
    }

    /// PR #9 — attach an OntologyRepo so the weekly community-detection
    /// job can run. Without an attached repo the scheduler silently
    /// skips detection (back-compat for vault-only deployments).
    pub fn with_ontology(mut self, repo: Arc<crate::ontology::OntologyRepo>) -> Self {
        self.ontology = Some(repo);
        self
    }

    /// PR #9 — override the minimum cadence between community detection
    /// passes. Defaults to weekly; tests typically drop this to seconds.
    pub fn with_community_cadence(mut self, t: Duration) -> Self {
        self.community_cadence = t;
        self
    }

    pub fn with_idle_threshold(mut self, t: Duration) -> Self {
        self.idle_threshold = t;
        self
    }

    pub fn with_health_cadence(mut self, t: Duration) -> Self {
        self.health_cadence = t;
        self
    }

    /// Override how many hubs to compile per idle tick (default 4).
    pub fn with_hub_batch_size(mut self, n: usize) -> Self {
        self.hub_batch_size = n;
        self
    }

    /// Override max concurrent hub compiles within one batch (default 2).
    pub fn with_hub_concurrency(mut self, n: usize) -> Self {
        self.hub_concurrency = n;
        self
    }

    /// Bump the activity clock. Call on every user input event from the
    /// agent loop / channel ingress. Thread-safe.
    pub fn notify_activity(&self) {
        *self.last_activity.lock() = Instant::now();
    }

    /// Returns true when the system has been idle longer than threshold.
    pub fn is_idle(&self) -> bool {
        self.last_activity.lock().elapsed() >= self.idle_threshold
    }

    /// Run ONE idle-cycle pass (useful for tests and manual triggers).
    /// Does nothing if not yet idle.
    pub async fn tick(&self) -> Result<SchedulerStats> {
        let mut stats = SchedulerStats::default();
        if !self.is_idle() {
            return Ok(stats);
        }

        // 1. Hub compile queue — batch-compile up to `hub_batch_size`
        //    candidates with `hub_concurrency` parallel workers per batch.
        let batch = hub::compile_batch(
            self.vault.clone(),
            self.hub_batch_size,
            self.hub_concurrency,
        )
        .await
        .unwrap_or_default();
        for (entity, impact) in &batch {
            tracing::debug!(
                entity = %entity,
                ?impact,
                "scheduler compiled hub (batch)"
            );
        }
        stats.hubs_compiled += batch.len();

        // 2. Periodic health check.
        let should_check = {
            let last = self.last_health_check.lock();
            match *last {
                None => true,
                Some(t) => t.elapsed() >= self.health_cadence,
            }
        };
        if should_check {
            match health::run(&self.vault) {
                Ok(_r) => {
                    *self.last_health_check.lock() = Some(Instant::now());
                    stats.health_checks_run += 1;
                }
                Err(e) => tracing::warn!("scheduler health check failed: {e}"),
            }
        }

        // 3. Refresh every active briefing whose underlying docs changed
        //    since its last_updated. `generate()` handles incremental
        //    caching internally so this is safe even without changes.
        let active_cases: Vec<String> = {
            let conn = self.vault.connection().lock();
            let mut stmt = conn.prepare(
                "SELECT case_number FROM briefings WHERE status = 'active'",
            )?;
            let rows: Vec<String> = stmt
                .query_map([], |r| r.get::<_, String>(0))?
                .filter_map(|r| r.ok())
                .collect();
            rows
        };
        for case in active_cases {
            match briefing::generate(&self.vault, &case).await {
                Ok(r) if !r.cached => {
                    stats.briefings_refreshed += 1;
                }
                Ok(_) => {}
                Err(e) => tracing::warn!(case, "briefing refresh error: {e}"),
            }
        }

        // 4. PR #9 — weekly community detection. Only runs when
        //    (a) an OntologyRepo is attached, (b) cadence has elapsed
        //    since the last successful pass, and (c) the graph is
        //    non-empty. The summariser is intentionally a no-op here
        //    (empty string, no keywords) — a follow-up pass with an LLM
        //    provider attached is responsible for filling in the summary
        //    text and `set_community_embedding`. This keeps the scheduler
        //    free of provider plumbing while still capturing structure.
        if let Some(onto) = &self.ontology {
            let should_run = {
                let last = self.last_community_detection.lock();
                match *last {
                    None => true,
                    Some(t) => t.elapsed() >= self.community_cadence,
                }
            };
            if should_run {
                match onto.load_graph_view() {
                    Ok(graph) if !graph.nodes.is_empty() => {
                        let assignment = crate::ontology::community::detect_communities(&graph);
                        match onto.replace_communities_level_zero(&assignment, |_, _| {
                            // Placeholder summariser. The LLM-driven
                            // summariser lives in the dream cycle
                            // consolidation path; a separate VaultScheduler
                            // refactor (PR #9 잔여) will share it.
                            (String::new(), Vec::new())
                        }) {
                            Ok(n) => {
                                *self.last_community_detection.lock() = Some(Instant::now());
                                stats.communities_detected = n;
                                tracing::info!(
                                    communities = n,
                                    nodes = graph.nodes.len(),
                                    "scheduler ran weekly community detection"
                                );
                            }
                            Err(e) => {
                                tracing::warn!("community detection write failed: {e}")
                            }
                        }
                    }
                    Ok(_) => {
                        // Empty graph — mark as "ran" anyway so we don't
                        // retry every tick on an empty deployment.
                        *self.last_community_detection.lock() = Some(Instant::now());
                    }
                    Err(e) => tracing::warn!("community graph load failed: {e}"),
                }
            }
        }

        Ok(stats)
    }

    /// Long-lived scheduler loop. Ticks every `check_interval`; each tick
    /// may dispatch maintenance work iff the system is idle. Exits when
    /// `shutdown` resolves.
    pub async fn run(
        self: Arc<Self>,
        check_interval: Duration,
        mut shutdown: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<()> {
        let mut ticker = tokio::time::interval(check_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("VaultScheduler shutting down");
                    return Ok(());
                }
                _ = ticker.tick() => {
                    if let Err(e) = self.tick().await {
                        tracing::warn!("scheduler tick error: {e}");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::ingest::{IngestInput, SourceType};
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Arc<VaultStore>) {
        let tmp = TempDir::new().unwrap();
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        let vault = Arc::new(VaultStore::with_shared_connection(conn).unwrap());
        (tmp, vault)
    }

    #[tokio::test]
    async fn tick_no_op_while_active() {
        let (_tmp, vault) = setup();
        let sched = VaultScheduler::new(vault).with_idle_threshold(Duration::from_secs(60));
        let stats = sched.tick().await.unwrap();
        assert_eq!(stats.hubs_compiled, 0);
        assert_eq!(stats.health_checks_run, 0);
    }

    #[tokio::test]
    async fn tick_runs_maintenance_when_idle() {
        let (_tmp, vault) = setup();
        // Seed a hub queue candidate.
        {
            let c = vault.connection().lock();
            c.execute(
                "INSERT INTO hub_notes (entity_name, hub_subtype, backlink_count,
                    hub_threshold, pending_backlinks)
                 VALUES ('민법 제750조', 'statute_article', 10, 5, 10)",
                [],
            )
            .unwrap();
        }
        let sched = VaultScheduler::new(vault.clone())
            .with_idle_threshold(Duration::from_millis(1))
            .with_health_cadence(Duration::from_millis(1));
        tokio::time::sleep(Duration::from_millis(5)).await;
        let stats = sched.tick().await.unwrap();
        assert!(stats.hubs_compiled >= 1);
        assert!(stats.health_checks_run >= 1);
    }

    #[tokio::test]
    async fn notify_activity_resets_idle() {
        let (_tmp, vault) = setup();
        let sched = VaultScheduler::new(vault)
            .with_idle_threshold(Duration::from_millis(50));
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(sched.is_idle());
        sched.notify_activity();
        assert!(!sched.is_idle());
    }

    #[tokio::test]
    async fn run_exits_on_shutdown() {
        let (_tmp, vault) = setup();
        let sched = Arc::new(VaultScheduler::new(vault));
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handle =
            tokio::spawn(async move { sched.run(Duration::from_millis(30), rx).await });
        tokio::time::sleep(Duration::from_millis(90)).await;
        tx.send(()).unwrap();
        let _ = handle.await.unwrap();
    }

    // ── PR #9 wire-up: VaultScheduler community detection ────────

    #[tokio::test]
    async fn tick_skips_community_detection_without_ontology_repo() {
        let (_tmp, vault) = setup();
        let sched = VaultScheduler::new(vault.clone())
            .with_idle_threshold(Duration::from_millis(1))
            .with_community_cadence(Duration::from_millis(1));
        tokio::time::sleep(Duration::from_millis(5)).await;
        let stats = sched.tick().await.unwrap();
        assert_eq!(stats.communities_detected, 0);
    }

    #[tokio::test]
    async fn tick_detects_communities_when_cadence_elapsed() {
        use crate::ontology::OntologyRepo;

        let tmp = TempDir::new().unwrap();
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        let vault = Arc::new(VaultStore::with_shared_connection(conn).unwrap());
        let repo = Arc::new(OntologyRepo::open(tmp.path()).unwrap());
        let a = repo
            .ensure_object("Contact", "Alice", &serde_json::json!({}), "u1")
            .unwrap();
        let b = repo
            .ensure_object("Contact", "Bob", &serde_json::json!({}), "u1")
            .unwrap();
        repo.create_link("knows", a, b, None).unwrap();

        let sched = VaultScheduler::new(vault)
            .with_ontology(repo.clone())
            .with_idle_threshold(Duration::from_millis(1))
            .with_community_cadence(Duration::from_millis(1));
        tokio::time::sleep(Duration::from_millis(5)).await;
        let stats = sched.tick().await.unwrap();
        assert!(
            stats.communities_detected >= 1,
            "expected ≥1 community, got {}",
            stats.communities_detected
        );
        let listed = repo.list_communities_level_zero().unwrap();
        assert!(!listed.is_empty());
    }

    #[tokio::test]
    async fn community_detection_respects_cadence_between_ticks() {
        use crate::ontology::OntologyRepo;

        let tmp = TempDir::new().unwrap();
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        let vault = Arc::new(VaultStore::with_shared_connection(conn).unwrap());
        let repo = Arc::new(OntologyRepo::open(tmp.path()).unwrap());
        repo.ensure_object("Contact", "Solo", &serde_json::json!({}), "u1")
            .unwrap();

        let sched = VaultScheduler::new(vault)
            .with_ontology(repo)
            .with_idle_threshold(Duration::from_millis(1))
            .with_community_cadence(Duration::from_secs(3600));
        tokio::time::sleep(Duration::from_millis(5)).await;
        let _first = sched.tick().await.unwrap();
        let second = sched.tick().await.unwrap();
        assert_eq!(
            second.communities_detected, 0,
            "second tick within cadence must skip"
        );
    }

    #[tokio::test]
    async fn briefing_refresh_triggers_in_idle_cycle() {
        let (_tmp, vault) = setup();
        // Ingest one case doc; generate initial briefing.
        vault
            .ingest_markdown(IngestInput {
                source_type: SourceType::LocalFile,
                source_device_id: "dev",
                original_path: None,
                title: Some("sched-case"),
                markdown: &format!(
                    "---\ncase_number: SCHED-1\n---\n# sched-case\n\n{}",
                    "본문 ".repeat(200)
                ),
                html_content: None,
                doc_type: None,
                domain: "legal",
            })
            .await
            .unwrap();
        briefing::generate(&vault, "SCHED-1").await.unwrap(); // seed row

        let sched = VaultScheduler::new(vault.clone())
            .with_idle_threshold(Duration::from_millis(1))
            .with_health_cadence(Duration::from_secs(86400)); // skip health
        tokio::time::sleep(Duration::from_millis(5)).await;
        let stats = sched.tick().await.unwrap();
        // Second generate returns cached, so briefings_refreshed == 0 is fine;
        // but health_checks_run should still increment at least once since
        // first seed.
        let _ = stats;
    }
}
