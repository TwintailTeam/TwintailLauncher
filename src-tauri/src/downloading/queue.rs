use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::downloading::QueueJobPayload;
use crate::utils::db_manager::{get_install_info_by_id,get_manifest_info_by_id};
use crate::utils::repo_manager::get_manifest;

static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueJobKind {
    GameDownload,
    GameUpdate,
    GamePreload,
    GameRepair,
    RunnerDownload,
    SteamrtDownload,
    Steamrt4Download,
    XxmiDownload,
    ExtrasDownload,
}

#[derive(Debug)]
pub struct QueueJob {
    pub id: String,
    pub kind: QueueJobKind,
    pub payload: QueueJobPayload,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueJobView {
    pub id: String,
    pub kind: QueueJobKind,
    pub install_id: String,
    pub name: String,
    pub status: QueueJobStatus,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStatePayload {
    pub max_concurrent: usize,
    pub paused: bool,
    pub auto_paused: bool,
    pub running: Vec<QueueJobView>,
    pub queued: Vec<QueueJobView>,
    pub completed: Vec<QueueJobView>,
    pub paused_jobs: Vec<QueueJobView>,
    pub pausing_installs: Vec<String>,
}

#[derive(Clone)]
pub struct DownloadQueueHandle {
    tx: std::sync::mpsc::Sender<QueueCommand>,
}

impl DownloadQueueHandle {
    pub fn enqueue(&self, kind: QueueJobKind, payload: QueueJobPayload) -> String {
        let job_id = format!("job_{}", JOB_COUNTER.fetch_add(1, Ordering::Relaxed));
        let _ = self.tx.send(QueueCommand::Enqueue(QueueJob {
            id: job_id.clone(),
            kind,
            payload,
        }));
        job_id
    }

    pub fn move_up(&self, job_id: String) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::MoveUp(job_id, tx));
        rx.recv().unwrap_or(false)
    }

    pub fn move_down(&self, job_id: String) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::MoveDown(job_id, tx));
        rx.recv().unwrap_or(false)
    }

    pub fn remove(&self, job_id: String) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::Remove(job_id, tx));
        rx.recv().unwrap_or(false)
    }

    pub fn remove_by_install_id(&self, install_id: String) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::RemoveByInstallId(install_id, tx));
        rx.recv().unwrap_or(false)
    }

    pub fn set_paused(&self, paused: bool) {
        let _ = self.tx.send(QueueCommand::SetPaused(paused));
    }

    pub fn activate_job(&self, job_id: String) -> Option<String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::ActivateJob(job_id, tx));
        rx.recv().unwrap_or(None)
    }

    pub fn reorder(&self, job_id: String, new_position: usize) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::Reorder(job_id, new_position, tx));
        rx.recv().unwrap_or(false)
    }

    pub fn get_state(&self) -> Option<QueueStatePayload> {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::GetState(tx));
        rx.recv().ok()
    }

    pub fn resume_job(&self, install_id: String) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::ResumeJob(install_id, tx));
        rx.recv().unwrap_or(false)
    }

    pub fn set_pausing(&self, install_id: String, is_pausing: bool) {
        let _ = self.tx.send(QueueCommand::SetPausing(install_id, is_pausing));
    }

    pub fn clear_completed(&self) {
        let _ = self.tx.send(QueueCommand::ClearCompleted);
    }

    pub fn auto_pause(&self) {
        let _ = self.tx.send(QueueCommand::AutoPause);
    }

    pub fn auto_resume(&self) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::AutoResume(tx));
        rx.recv().unwrap_or(false)
    }

    pub fn is_auto_paused(&self) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::IsAutoPaused(tx));
        rx.recv().unwrap_or(false)
    }

    pub fn has_job_for_id(&self, install_id: String) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = self.tx.send(QueueCommand::HasJobForId(install_id, tx));
        rx.recv().unwrap_or(false)
    }
}

#[allow(unused)]
pub enum QueueCommand {
    Enqueue(QueueJob),
    SetMaxConcurrent(usize),
    SetPaused(bool),
    SetPausing(String, bool),
    MoveUp(String, std::sync::mpsc::Sender<bool>),
    MoveDown(String, std::sync::mpsc::Sender<bool>),
    Remove(String, std::sync::mpsc::Sender<bool>),
    RemoveByInstallId(String, std::sync::mpsc::Sender<bool>),
    ActivateJob(String, std::sync::mpsc::Sender<Option<String>>),
    Reorder(String, usize, std::sync::mpsc::Sender<bool>),
    GetState(std::sync::mpsc::Sender<QueueStatePayload>),
    ResumeJob(String, std::sync::mpsc::Sender<bool>),
    ClearCompleted,
    AutoPause,
    AutoResume(std::sync::mpsc::Sender<bool>),
    IsAutoPaused(std::sync::mpsc::Sender<bool>),
    HasJobForId(String, std::sync::mpsc::Sender<bool>),
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum QueueJobOutcome {
    Completed,
    Failed,
    Cancelled,
}

fn emit_queue_state(app: &AppHandle, max_concurrent: usize, paused: bool, auto_paused: bool, active: &HashMap<String, QueueJobView>, queued: &VecDeque<QueueJobView>, completed: &VecDeque<QueueJobView>, paused_jobs: &HashMap<String, QueueJobView>, pausing_installs: &HashSet<String>) {
    let payload = QueueStatePayload {
        max_concurrent,
        paused,
        auto_paused,
        running: active.values().cloned().collect(),
        queued: queued.iter().cloned().collect(),
        completed: completed.iter().cloned().collect(),
        paused_jobs: paused_jobs.values().cloned().collect(),
        pausing_installs: pausing_installs.iter().cloned().collect(),
    };
    let _ = app.emit("download_queue_state", payload);
}

pub fn start_download_queue_worker(app: AppHandle, initial_max_concurrent: usize, run_job: fn(AppHandle, QueueJob) -> QueueJobOutcome) -> DownloadQueueHandle {
    let (tx, rx) = std::sync::mpsc::channel::<QueueCommand>();
    let (done_tx, done_rx) = std::sync::mpsc::channel::<(String, QueueJobOutcome)>();

    std::thread::spawn(move || {
        let mut max_concurrent = initial_max_concurrent.max(1);
        let mut paused = false;
        let mut auto_paused = false; // True if paused due to connection loss (not manual)
        let mut activating = false; // Flag to prevent auto-pause during job activation
        let mut queued: VecDeque<QueueJob> = VecDeque::new();
        let mut queued_views: VecDeque<QueueJobView> = VecDeque::new();
        let mut active: HashMap<String, QueueJobView> = HashMap::new();
        let mut active_jobs: HashMap<String, QueueJob> = HashMap::new(); // Keep job data for potential requeueing
        let mut completed_views: VecDeque<QueueJobView> = VecDeque::new();
        let mut paused_jobs: HashMap<String, QueueJobView> = HashMap::new(); // Jobs paused by user (keyed by install_id)
        let mut paused_jobs_data: HashMap<String, QueueJob> = HashMap::new(); // Job data for paused jobs
        let mut pausing_installs: HashSet<String> = HashSet::new(); // Installs currently transitioning to paused

        loop {
            while let Ok((job_id, outcome)) = done_rx.try_recv() {
                if let Some(mut view) = active.remove(&job_id) {
                    let removed_job = active_jobs.remove(&job_id);
                    let install_id = view.install_id.clone();

                    match outcome {
                        QueueJobOutcome::Completed => {
                            view.status = QueueJobStatus::Completed;
                            completed_views.push_front(view);
                            while completed_views.len() > 25 {
                                completed_views.pop_back();
                            }
                        }
                        QueueJobOutcome::Failed => {
                            view.status = QueueJobStatus::Failed;
                            completed_views.push_front(view);
                            while completed_views.len() > 25 {
                                completed_views.pop_back();
                            }
                        }
                        QueueJobOutcome::Cancelled => {
                            // When cancelled during activation, put the job back in queue
                            if activating {
                                if let Some(job) = removed_job {
                                    // Put the cancelled job back at the front of the queue (after the activating job)
                                    view.status = QueueJobStatus::Queued;
                                    queued.insert(1.min(queued.len()), job);
                                    queued_views.insert(1.min(queued_views.len()), view);
                                }
                            } else {
                                // Normal pause - move to paused_jobs for later resume
                                paused = true;
                                view.status = QueueJobStatus::Paused;
                                // Clear the "pausing" state since we're now fully paused
                                pausing_installs.remove(&install_id);
                                paused_jobs.insert(install_id.clone(), view);
                                if let Some(job) = removed_job {
                                    paused_jobs_data.insert(install_id, job);
                                }
                            }
                        }
                    };
                }
                emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
            }

            // Only auto-start next job if not paused
            if !paused {
                while active.len() < max_concurrent {
                    let Some(job) = queued.pop_front() else { break; };
                    let Some(mut view) = queued_views.pop_front() else { break; };

                    view.status = QueueJobStatus::Running;
                    let job_id = job.id.clone();
                    active.insert(job_id.clone(), view);
                    active_jobs.insert(job_id.clone(), QueueJob { id: job.id.clone(), kind: job.kind, payload: job.payload.clone() });

                    // Clear the activating flag since the new job is now starting
                    activating = false;

                    emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);

                    let app2 = app.clone();
                    let done_tx2 = done_tx.clone();
                    let runner = run_job;

                    // run_job is blocking-heavy (downloads + extraction), so we keep it in a dedicated OS thread.
                    std::thread::spawn(move || {
                        let outcome = runner(app2, job);
                        let _ = done_tx2.send((job_id, outcome));
                    });
                }
            }

            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(cmd) => match cmd {
                    QueueCommand::Enqueue(job) => {
                        let install_id = job.payload.get_id();
                        let name = if let (QueueJobKind::GamePreload, QueueJobPayload::Game(p)) = (&job.kind, &job.payload) { get_install_info_by_id(&app, p.install.clone()).map(|install| { let fallback = install.name.clone(); let ver = install.version.clone(); get_manifest_info_by_id(&app, install.manifest_id.clone()).and_then(|gid| get_manifest(&app, gid.filename)).and_then(|gm| gm.extra.preload).and_then(|pl| pl.metadata).map(|pmd| fallback.replace(ver.as_str(), pmd.version.as_str())).unwrap_or(fallback) }) } else if let QueueJobPayload::Game(ref p) = job.payload { get_install_info_by_id(&app, p.install.clone()).map(|i| i.name) } else { None }.unwrap_or_else(|| job.payload.get_name());
                        queued_views.push_back(QueueJobView {
                            id: job.id.clone(),
                            kind: job.kind,
                            install_id,
                            name,
                            status: QueueJobStatus::Queued,
                        });
                        queued.push_back(job);
                        emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                    }
                    QueueCommand::SetMaxConcurrent(n) => {
                        max_concurrent = n.max(1);
                        emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                    }
                    QueueCommand::SetPaused(p) => {
                        paused = p;
                        // Clear auto_paused when user manually changes pause state
                        if !p { auto_paused = false; }
                        emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                    }
                    QueueCommand::SetPausing(install_id, is_pausing) => {
                        if is_pausing { pausing_installs.insert(install_id); } else { pausing_installs.remove(&install_id); }
                        emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                    }
                    QueueCommand::MoveUp(job_id, reply) => {
                        let mut success = false;
                        if let Some(idx) = queued.iter().position(|j| j.id == job_id) {
                            if idx > 0 {
                                queued.swap(idx, idx - 1);
                                queued_views.swap(idx, idx - 1);
                                success = true;
                                emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                            }
                        }
                        let _ = reply.send(success);
                    }
                    QueueCommand::MoveDown(job_id, reply) => {
                        let mut success = false;
                        if let Some(idx) = queued.iter().position(|j| j.id == job_id) {
                            if idx < queued.len().saturating_sub(1) {
                                queued.swap(idx, idx + 1);
                                queued_views.swap(idx, idx + 1);
                                success = true;
                                emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                            }
                        }
                        let _ = reply.send(success);
                    }
                    QueueCommand::Remove(job_id, reply) => {
                        let mut success = false;
                        if let Some(idx) = queued.iter().position(|j| j.id == job_id) {
                            queued.remove(idx);
                            queued_views.remove(idx);
                            success = true;
                            emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                        }
                        let _ = reply.send(success);
                    }
                    QueueCommand::RemoveByInstallId(install_id, reply) => {
                        let mut removed_any = false;
                        let mut removed_job_ids = Vec::new();

                        // Remove all queued jobs matching this install_id
                        let mut i = 0;
                        while i < queued.len() {
                            if queued[i].payload.get_id() == install_id {
                                removed_job_ids.push(queued[i].id.clone());
                                queued.remove(i);
                                queued_views.remove(i);
                                removed_any = true;
                            } else { i += 1; }
                        }

                        // Also remove from paused jobs
                        if let Some(view) = paused_jobs.remove(&install_id) {
                            removed_job_ids.push(view.id);
                            paused_jobs_data.remove(&install_id);
                            removed_any = true;
                        }

                        // Also remove from completed history
                        let mut j = 0;
                        while j < completed_views.len() {
                            if completed_views[j].install_id == install_id {
                                removed_job_ids.push(completed_views[j].id.clone());
                                completed_views.remove(j);
                                removed_any = true;
                            } else { j += 1; }
                        }

                        // Remove from active immediately
                        let mut active_to_remove = Vec::new();
                        for (jid, v) in active.iter() {
                            if v.install_id == install_id { active_to_remove.push(jid.clone()); }
                        }
                        for jid in active_to_remove {
                            active.remove(&jid);
                            active_jobs.remove(&jid);
                            removed_job_ids.push(jid);
                            removed_any = true;
                        }

                        if removed_any {
                            for jid in removed_job_ids { let _ = app.emit("download_removed", jid); }
                            emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                        }
                        let _ = reply.send(removed_any);
                    }
                    QueueCommand::Reorder(job_id, new_position, reply) => {
                        let mut success = false;
                        if let Some(idx) = queued.iter().position(|j| j.id == job_id) {
                            let job = queued.remove(idx).unwrap();
                            let view = queued_views.remove(idx).unwrap();
                            let insert_pos = new_position.min(queued.len());
                            queued.insert(insert_pos, job);
                            queued_views.insert(insert_pos, view);
                            success = true;
                            emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                        }
                        let _ = reply.send(success);
                    }
                    QueueCommand::ActivateJob(job_id, reply) => {
                        // Find the job in queue and move it to front, then unpause
                        let mut install_id = None;
                        if let Some(idx) = queued.iter().position(|j| j.id == job_id) {
                            // Move any paused jobs back to the queue (at the end) before activating new job
                            for (paused_install_id, mut view) in paused_jobs.drain() {
                                if let Some(job) = paused_jobs_data.remove(&paused_install_id) {
                                    view.status = QueueJobStatus::Queued;
                                    queued.push_back(job);
                                    queued_views.push_back(view);
                                }
                            }

                            let job = queued.remove(idx).unwrap();
                            let view = queued_views.remove(idx).unwrap();
                            install_id = Some(view.install_id.clone());
                            queued.push_front(job);
                            queued_views.push_front(view);
                            activating = true; // Prevent auto-pause when current job is cancelled
                            paused = false; // Unpause to start this job
                            emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                        }
                        let _ = reply.send(install_id);
                    }
                    QueueCommand::GetState(reply) => {
                        // Return current queue state for initial sync
                        let payload = QueueStatePayload {
                            max_concurrent,
                            paused,
                            auto_paused,
                            running: active.values().cloned().collect(),
                            queued: queued_views.iter().cloned().collect(),
                            completed: completed_views.iter().cloned().collect(),
                            paused_jobs: paused_jobs.values().cloned().collect(),
                            pausing_installs: pausing_installs.iter().cloned().collect(),
                        };
                        let _ = reply.send(payload);
                    }
                    QueueCommand::ResumeJob(install_id, reply) => {
                        // Resume a paused job - move it from paused to front of queue and unpause
                        let mut success = false;
                        if let Some(mut view) = paused_jobs.remove(&install_id) {
                            if let Some(job) = paused_jobs_data.remove(&install_id) {
                                view.status = QueueJobStatus::Queued;
                                queued.push_front(job);
                                queued_views.push_front(view);
                                paused = false; // Unpause to start this job
                                success = true;
                                emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                            }
                        }
                        let _ = reply.send(success);
                    }
                    QueueCommand::ClearCompleted => {
                        completed_views.clear();
                        emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                    }
                    QueueCommand::AutoPause => {
                        // Auto-pause due to connection loss - only if not already paused
                        if !paused {
                            paused = true;
                            auto_paused = true;
                            emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                        }
                    }
                    QueueCommand::AutoResume(reply) => {
                        // Only resume if we auto-paused (not manually paused)
                        let success = if auto_paused {
                            auto_paused = false;
                            paused = false;
                            emit_queue_state(&app, max_concurrent, paused, auto_paused, &active, &queued_views, &completed_views, &paused_jobs, &pausing_installs);
                            true
                        } else { false };
                        let _ = reply.send(success);
                    }
                    QueueCommand::IsAutoPaused(reply) => {
                        let _ = reply.send(auto_paused);
                    }
                    QueueCommand::HasJobForId(install_id, reply) => {
                        let found = queued_views.iter().any(|v| v.install_id == install_id) || active.values().any(|v| v.install_id == install_id) || paused_jobs.contains_key(&install_id);
                        let _ = reply.send(found);
                    }
                    QueueCommand::Shutdown => break,
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    DownloadQueueHandle { tx }
}
