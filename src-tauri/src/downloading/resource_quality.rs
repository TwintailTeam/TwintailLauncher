// ------------ Resource Quality (Wuthering Waves SD/HD/UHD) ------------
// Kuro ships WuWa as a shared "common" pack plus one texture pack per quality (SD/HD/UHD)
use crate::DownloadState;
use crate::downloading::DownloadGamePayload;
use crate::downloading::QueueJobPayload;
use crate::downloading::queue::{QueueJobKind, QueueJobOutcome};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id};
use crate::utils::models::LauncherInstall;
use crate::utils::repo_manager::get_manifest;
use crate::utils::{run_async_command, show_dialog_with_callback};
use fischl::utils::downloader::AsyncDownloader;
use fischl::utils::{KuroIndex, KuroResource};
use md5::{Digest, Md5};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Listener, Manager, Runtime};

pub const QUALITIES: [&str; 3] = ["sd", "hd", "uhd"];
pub const DEFAULT_QUALITY: &str = "hd";

// Kuro's launcher endpoint
const WUWA_BUNDLE_CONFIG_URL: &str = "https://prod-alicdn-gamestarter.kurogame.com/launcher/game/50004_P7xcUZnEr1AXIGON25E6KjpOgTlVrg6e/G153/official/index.json";
const QUALITY_ROOT: &str = "Client/Content";
const DOWNLOAD_WORKERS: usize = 6;
const VERIFY_WORKERS: usize = 4;
const SPACE_HEADROOM: u64 = 1 << 30;

pub fn supports_quality(biz: &str) -> bool { biz == "wuwa_global" }

pub fn normalize_quality(quality: &str) -> &'static str {
    QUALITIES.iter().copied().find(|q| q.eq_ignore_ascii_case(quality.trim())).unwrap_or(DEFAULT_QUALITY)
}

fn quality_dir(game_dir: &Path, quality: &str) -> PathBuf { game_dir.join(QUALITY_ROOT).join(quality.to_ascii_uppercase()) }

pub fn installed_qualities(game_dir: &Path) -> Vec<String> {
    QUALITIES.iter().filter(|q| std::fs::read_dir(quality_dir(game_dir, q)).is_ok_and(|mut d| d.next().is_some())).map(|q| q.to_string()).collect()
}

/// The quality the game should load: the selected one when its files are on disk, otherwise whatever is installed.
pub fn effective_quality(install: &LauncherInstall) -> &'static str {
    let selected = normalize_quality(&install.resource_quality);
    let on_disk = installed_qualities(Path::new(&install.directory));
    match on_disk.first() {
        Some(first) if !on_disk.iter().any(|q| q == selected) => normalize_quality(first),
        _ => selected,
    }
}

/// True when the install can't be served by the legacy manifest.
pub fn needs_pack_sync(install: &LauncherInstall, quality: &str) -> bool {
    normalize_quality(quality) != DEFAULT_QUALITY || installed_qualities(Path::new(&install.directory)).iter().any(|q| q != DEFAULT_QUALITY)
}

/// Replaces any -krqlv= flag
pub fn with_quality_arg(args: &str, quality: &str) -> String {
    let flag = format!("-krqlv={quality}");
    args.split_whitespace().filter(|a| !a.starts_with("-krqlv=")).chain(std::iter::once(flag.as_str())).collect::<Vec<_>>().join(" ")
}

// ------------ Kuro Bundle Config ------------

fn join_url(base: &str, path: &str) -> String { format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/')) }

fn json_u64(v: &Value) -> Option<u64> {
    match v { Value::Number(n) => n.as_u64(), Value::String(s) => s.trim().parse().ok(), _ => None }
}

fn json_string(v: &Value) -> String {
    match v { Value::String(s) => s.clone(), Value::Number(n) => n.to_string(), _ => String::new() }
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = AsyncDownloader::setup_client(false).await;
    let rsp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !rsp.status().is_success() { return Err(format!("{url} returned {}", rsp.status())); }
    let body = rsp.bytes().await.map_err(|e| e.to_string())?;
    // Kuro's launcher endpoints answer with gzip whether or not it was asked for
    if body.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(&body[..]).read_to_end(&mut out).map_err(|e| e.to_string())?;
        Ok(out)
    } else {
        Ok(body.to_vec())
    }
}

async fn fetch_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    serde_json::from_slice(&fetch_bytes(url).await?).map_err(|e| format!("{url}: {e}"))
}

pub async fn fetch_bundle_config() -> Result<Value, String> {
    let config: Value = fetch_json(WUWA_BUNDLE_CONFIG_URL).await?;
    if !config["bundles"].is_object() || !config["resourcePacks"].is_object() { return Err("Kuro's launcher config has no quality bundles".to_string()); }
    Ok(config)
}

fn bundle_pack_names(config: &Value, quality: &str) -> Result<Vec<String>, String> {
    let bundle = quality.to_ascii_uppercase();
    let packs = config["bundles"][&bundle]["resourcePacks"].as_array().ok_or_else(|| format!("Kuro offers no {bundle} quality"))?.iter().filter_map(|p| p.as_str().map(str::to_string)).collect::<Vec<_>>();
    if packs.is_empty() { return Err(format!("the {bundle} quality lists no resource packs")); }
    Ok(packs)
}

fn cdn_list(config: &Value) -> Vec<String> {
    let mut cdns: Vec<(i64, String)> = Vec::new();
    for c in config["cdnList"].as_array().into_iter().flatten() {
        let Some(url) = c["url"].as_str().map(str::trim).filter(|u| !u.is_empty()) else { continue; };
        if !cdns.iter().any(|(_, seen)| seen == url) { cdns.push((c["P"].as_i64().unwrap_or(0), url.to_string())); }
    }
    cdns.sort_by(|a, b| b.0.cmp(&a.0));
    cdns.into_iter().map(|(_, url)| url).collect()
}

#[derive(Clone, Debug)]
pub struct QualityPlan {
    pub quality: &'static str,
    pub version: String,
    pub cdns: Vec<String>,
    pub base_path: String,
    pub index_files: Vec<String>,
}

pub fn plan_for(config: &Value, quality: &str) -> Result<QualityPlan, String> {
    let quality = normalize_quality(quality);
    let mut base_path: Option<String> = None;
    let mut version = String::new();
    let mut index_files = Vec::new();
    for name in bundle_pack_names(config, quality)? {
        let pack = &config["resourcePacks"][name.as_str()];
        let index_file = pack["indexFile"].as_str().map(str::trim).unwrap_or_default();
        if index_file.is_empty() { return Err(format!("the {name} pack has no file list")); }
        let pack_base = pack["baseUrl"].as_str().unwrap_or_default().to_string();
        match &base_path {
            None => base_path = Some(pack_base),
            Some(base) if *base != pack_base => return Err(format!("the {} packs download from different folders", quality.to_ascii_uppercase())),
            Some(_) => {}
        }
        if version.is_empty() { version = json_string(&pack["version"]); }
        index_files.push(index_file.to_string());
    }
    let cdns = cdn_list(config);
    if cdns.is_empty() { return Err("Kuro's launcher config lists no CDN".to_string()); }
    Ok(QualityPlan { quality, version, cdns, base_path: base_path.unwrap_or_default(), index_files })
}

#[derive(Serialize, Debug, Clone)]
pub struct QualitySize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

pub fn quality_sizes(config: &Value) -> HashMap<String, QualitySize> {
    let mut sizes = HashMap::new();
    for quality in QUALITIES {
        let Ok(packs) = bundle_pack_names(config, quality) else { continue; };
        let mut size = QualitySize { download_bytes: 0, install_bytes: 0 };
        for name in packs {
            let pack = &config["resourcePacks"][name.as_str()];
            size.download_bytes += json_u64(&pack["size"]).unwrap_or(0);
            size.install_bytes += json_u64(&pack["unCompressSize"]).or_else(|| json_u64(&pack["size"])).unwrap_or(0);
        }
        sizes.insert(quality.to_string(), size);
    }
    sizes
}

pub fn config_version(config: &Value) -> String {
    bundle_pack_names(config, DEFAULT_QUALITY).ok().and_then(|packs| packs.first().map(|p| json_string(&config["resourcePacks"][p.as_str()]["version"]))).unwrap_or_default()
}

// ------------ Pack Sync ------------
// Checks every file of common + the chosen quality in place, downloads what is missing or wrong into a
// staging folder, and only moves the new files in once everything is done.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Verify { Size, Hash }

pub enum SyncOutcome {
    Done,
    Cancelled,
    Failed(String),
    NoSpace { needed: u64, available: u64 },
}

fn resource_path(root: &Path, dest: &str) -> Option<PathBuf> {
    let rel = Path::new(dest);
    (rel.components().count() > 0 && rel.components().all(|c| matches!(c, Component::Normal(_)))).then(|| root.join(rel))
}

fn md5_matches(path: &Path, expected: &str) -> bool {
    let expected = expected.trim().trim_start_matches("md5:").to_ascii_lowercase();
    // crc64 entries can't be checked here, the size check already passed
    if expected.is_empty() || expected.starts_with("crc64:") { return true; }
    let Ok(mut file) = std::fs::File::open(path) else { return false; };
    let mut hasher = Md5::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(_) => return false,
        }
    }
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>() == expected
}

fn file_matches(path: &Path, resource: &KuroResource, verify: Verify) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) if meta.is_file() && meta.len() == resource.size => verify == Verify::Size || md5_matches(path, &resource.md5),
        _ => false,
    }
}

async fn fetch_index(cdns: &[String], index_file: &str) -> Result<KuroIndex, String> {
    let mut last_error = String::new();
    for cdn in cdns {
        match fetch_json::<KuroIndex>(&join_url(cdn, index_file)).await {
            Ok(index) => return Ok(index),
            Err(e) => { log::warn!("Quality pack list from {cdn} failed ({e}), trying the next CDN"); last_error = e; }
        }
    }
    Err(last_error)
}

fn verify_files<F>(game_dir: &Path, resources: &[KuroResource], verify: Verify, progress: &F, cancel: &AtomicBool) -> Vec<KuroResource> where F: Fn(u64, u64, u64, u64, u64, u64, u8) + Sync {
    let total: u64 = resources.iter().map(|r| r.size).sum();
    let checked = AtomicU64::new(0);
    let cursor = AtomicUsize::new(0);
    let done = AtomicBool::new(false);
    let bad = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        s.spawn(|| {
            while !done.load(Ordering::Relaxed) {
                let c = checked.load(Ordering::Relaxed);
                progress(c, total, c, total, 0, 0, 4);
                std::thread::sleep(Duration::from_millis(250));
            }
        });
        let workers = (0..VERIFY_WORKERS).map(|_| s.spawn(|| {
            while !cancel.load(Ordering::Relaxed) {
                let Some(resource) = resources.get(cursor.fetch_add(1, Ordering::Relaxed)) else { break; };
                let ok = resource_path(game_dir, &resource.dest).is_some_and(|p| file_matches(&p, resource, verify));
                if !ok { bad.lock().unwrap().push(resource.clone()); }
                checked.fetch_add(resource.size, Ordering::Relaxed);
            }
        })).collect::<Vec<_>>();
        for w in workers { let _ = w.join(); }
        done.store(true, Ordering::Relaxed);
    });
    bad.into_inner().unwrap()
}

struct Counters {
    downloaded: AtomicU64,
    net: AtomicU64,
}

async fn fetch_one(bases: &[String], staged: PathBuf, resource: &KuroResource, counters: &Arc<Counters>, cancel: &Arc<AtomicBool>) -> Result<(), String> {
    // fischl doesn't export its client type, so each file gets its own; negligible next to pack-sized files
    let client = Arc::new(AsyncDownloader::setup_client(false).await);
    // bytes of this file currently counted in `downloaded`, so retries and the final credit stay exact
    let mut credited = 0u64;
    let mut last_error = String::new();
    for attempt in 0..3 {
        if cancel.load(Ordering::Relaxed) { break; }
        let resumed = std::fs::metadata(&staged).map(|m| m.len().min(resource.size)).unwrap_or(0);
        if resumed >= credited { counters.downloaded.fetch_add(resumed - credited, Ordering::Relaxed); } else { counters.downloaded.fetch_sub(credited - resumed, Ordering::Relaxed); }
        credited = resumed;

        let url = join_url(&bases[attempt % bases.len()], &resource.dest);
        let mut dl = match AsyncDownloader::new(client.clone(), &url).await {
            Ok(d) => d.with_cancel_token(Some(cancel.clone())),
            Err(e) => { last_error = e.to_string(); continue; }
        };
        let file_credit = Arc::new(AtomicU64::new(credited));
        let cb = {
            let counters = counters.clone();
            let file_credit = file_credit.clone();
            let mut last = resumed;
            move |current: u64, _: u64, _: u64, _: u64| {
                let diff = current.saturating_sub(last);
                if diff > 0 {
                    last = current;
                    counters.downloaded.fetch_add(diff, Ordering::Relaxed);
                    counters.net.fetch_add(diff, Ordering::Relaxed);
                    file_credit.fetch_add(diff, Ordering::Relaxed);
                }
            }
        };
        let result = dl.download(staged.clone(), cb).await;
        credited = file_credit.load(Ordering::Relaxed);
        if let Err(e) = result { last_error = e.to_string(); continue; }
        if !file_matches(&staged, resource, Verify::Hash) {
            last_error = "checksum mismatch".to_string();
            let _ = std::fs::remove_file(&staged);
            continue;
        }
        if resource.size >= credited { counters.downloaded.fetch_add(resource.size - credited, Ordering::Relaxed); } else { counters.downloaded.fetch_sub(credited - resource.size, Ordering::Relaxed); }
        return Ok(());
    }
    counters.downloaded.fetch_sub(credited, Ordering::Relaxed);
    Err(format!("{}: {last_error}", resource.dest))
}

async fn download_files<F>(bases: Vec<String>, staging: &Path, files: Vec<KuroResource>, progress: Arc<F>, cancel: Arc<AtomicBool>) -> Result<(), String> where F: Fn(u64, u64, u64, u64, u64, u64, u8) + Send + Sync + 'static {
    let total: u64 = files.iter().map(|r| r.size).sum();
    let queue = Arc::new(Mutex::new(VecDeque::from(files)));
    let counters = Arc::new(Counters { downloaded: AtomicU64::new(0), net: AtomicU64::new(0) });
    let failures = Arc::new(Mutex::new(Vec::<String>::new()));
    let bases = Arc::new(bases);

    let reporter = tokio::spawn({
        let counters = counters.clone();
        let progress = progress.clone();
        async move {
            let mut last_net = 0u64;
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let net = counters.net.load(Ordering::Relaxed);
                let speed = net.saturating_sub(last_net) * 2;
                last_net = net;
                let current = counters.downloaded.load(Ordering::Relaxed).min(total);
                progress(current, total, current, total, speed, speed, 2);
            }
        }
    });

    let workers = (0..DOWNLOAD_WORKERS).map(|_| {
        let queue = queue.clone();
        let counters = counters.clone();
        let failures = failures.clone();
        let bases = bases.clone();
        let staging = staging.to_path_buf();
        let cancel = cancel.clone();
        tokio::spawn(async move {
            while !cancel.load(Ordering::Relaxed) {
                let Some(resource) = queue.lock().unwrap().pop_front() else { break; };
                let Some(staged) = resource_path(&staging, &resource.dest) else { failures.lock().unwrap().push(format!("{}: unsafe path", resource.dest)); continue; };
                if let Err(e) = fetch_one(&bases, staged, &resource, &counters, &cancel).await {
                    if !cancel.load(Ordering::Relaxed) { log::warn!("Quality pack file failed: {e}"); failures.lock().unwrap().push(e); }
                }
            }
        })
    }).collect::<Vec<_>>();
    for w in workers { let _ = w.await; }
    reporter.abort();

    let failures = failures.lock().unwrap();
    if failures.is_empty() { Ok(()) } else { Err(format!("{} file(s) failed to download, first: {}", failures.len(), failures[0])) }
}

pub async fn sync_packs<F>(plan: &QualityPlan, game_dir: &Path, staging_name: &str, verify: Verify, progress: F, cancel: Arc<AtomicBool>) -> SyncOutcome where F: Fn(u64, u64, u64, u64, u64, u64, u8) + Send + Sync + 'static {
    let progress = Arc::new(progress);
    let mut resources: Vec<KuroResource> = Vec::new();
    let mut seen = HashSet::new();
    let mut deletes = Vec::new();
    for index_file in &plan.index_files {
        let index = match fetch_index(&plan.cdns, index_file).await { Ok(i) => i, Err(e) => return SyncOutcome::Failed(e) };
        for resource in index.resource { if seen.insert(resource.dest.clone()) { resources.push(resource); } }
        deletes.extend(index.delete_files.unwrap_or_default());
    }
    if resources.is_empty() { return SyncOutcome::Failed("the server returned an empty file list".to_string()); }
    let total: u64 = resources.iter().map(|r| r.size).sum();
    log::info!("{} quality: {} files ({:.2} GB) across {} pack(s)", plan.quality.to_ascii_uppercase(), resources.len(), total as f64 / 1e9, plan.index_files.len());

    let needed = verify_files(game_dir, &resources, verify, progress.as_ref(), &cancel);
    if cancel.load(Ordering::Relaxed) { return SyncOutcome::Cancelled; }

    let staging_root = game_dir.join(staging_name);
    let staging = staging_root.join("staging");
    if !needed.is_empty() {
        let pending: u64 = needed.iter().map(|r| r.size.saturating_sub(resource_path(&staging, &r.dest).and_then(|p| std::fs::metadata(p).ok()).map(|m| m.len()).unwrap_or(0))).sum();
        if let Some(available) = fischl::utils::available(game_dir) {
            if available < pending + SPACE_HEADROOM { return SyncOutcome::NoSpace { needed: pending + SPACE_HEADROOM, available }; }
        }
        log::info!("{} quality: downloading {} file(s), {:.2} GB", plan.quality.to_ascii_uppercase(), needed.len(), pending as f64 / 1e9);
        let bases = plan.cdns.iter().map(|c| join_url(c, &plan.base_path)).collect::<Vec<_>>();
        if let Err(e) = download_files(bases, &staging, needed.clone(), progress.clone(), cancel.clone()).await {
            return if cancel.load(Ordering::Relaxed) { SyncOutcome::Cancelled } else { SyncOutcome::Failed(e) };
        }
        if cancel.load(Ordering::Relaxed) { return SyncOutcome::Cancelled; }

        progress(total, total, total, total, 0, 0, 5);
        for resource in &needed {
            let (Some(from), Some(to)) = (resource_path(&staging, &resource.dest), resource_path(game_dir, &resource.dest)) else { continue; };
            if let Some(parent) = to.parent() { let _ = std::fs::create_dir_all(parent); }
            if let Err(e) = std::fs::rename(&from, &to) { return SyncOutcome::Failed(format!("could not move {} into place: {e}", resource.dest)); }
        }
    }

    for dest in deletes {
        if let Some(path) = resource_path(game_dir, &dest) { if path.is_file() && std::fs::remove_file(&path).is_ok() { log::info!("Removed {dest}, Kuro dropped it from the game"); } }
    }
    let _ = std::fs::remove_dir_all(&staging_root);
    progress(total, total, total, total, 0, 0, 0);
    SyncOutcome::Done
}

pub fn remove_other_qualities(game_dir: &Path, keep: &str) {
    for quality in QUALITIES.iter().filter(|q| **q != keep) {
        let dir = quality_dir(game_dir, quality);
        if !dir.exists() { continue; }
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => log::info!("Removed the {} texture pack", quality.to_ascii_uppercase()),
            Err(e) => log::warn!("Could not remove {}: {e}", dir.display()),
        }
    }
}

pub enum PackSyncResult {
    Done,
    Cancelled,
    Failed,
    Reported,
}

pub fn run_pack_sync<R: Runtime, F>(app: &AppHandle<R>, install: &LauncherInstall, quality: &str, expected_version: &str, staging_name: &str, verify: Verify, progress: F, cancel: Arc<AtomicBool>) -> PackSyncResult where F: Fn(u64, u64, u64, u64, u64, u64, u8) + Send + Sync + 'static {
    let quality = normalize_quality(quality);
    let plan = match run_async_command(async { plan_for(&fetch_bundle_config().await?, quality) }) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Cannot resolve {} quality packs for \"{}\": {e}", quality.to_ascii_uppercase(), install.name);
            show_dialog_with_callback(app, "warning", "TwintailLauncher", "dialogs.quality_config_error", Some(vec!["dialogs.buttons.ok"]), None, Some(HashMap::from([("install_name", install.name.as_str()), ("error", e.as_str())])));
            return PackSyncResult::Reported;
        }
    };
    if plan.version != expected_version {
        log::warn!("Quality packs are for {} but \"{}\" needs {}", plan.version, install.name, expected_version);
        show_dialog_with_callback(app, "warning", "TwintailLauncher", "dialogs.quality_version_mismatch", Some(vec!["dialogs.buttons.ok"]), None, Some(HashMap::from([("install_name", install.name.as_str()), ("version", plan.version.as_str()), ("expected", expected_version)])));
        return PackSyncResult::Reported;
    }

    let game_dir = Path::new(&install.directory);
    if !game_dir.exists() { let _ = std::fs::create_dir_all(game_dir); }
    match run_async_command(sync_packs(&plan, game_dir, staging_name, verify, progress, cancel)) {
        SyncOutcome::Done => { remove_other_qualities(game_dir, quality); PackSyncResult::Done }
        SyncOutcome::Cancelled => PackSyncResult::Cancelled,
        SyncOutcome::NoSpace { needed, available } => {
            let (needed, available) = (fischl::utils::prettify_bytes(needed), fischl::utils::prettify_bytes(available));
            show_dialog_with_callback(app, "warning", "TwintailLauncher", "dialogs.quality_insufficient_space", Some(vec!["dialogs.buttons.ok"]), None, Some(HashMap::from([("install_name", install.name.as_str()), ("needed", needed.as_str()), ("available", available.as_str())])));
            PackSyncResult::Reported
        }
        SyncOutcome::Failed(e) => { log::warn!("{} quality sync failed for \"{}\": {e}", quality.to_ascii_uppercase(), install.name); PackSyncResult::Failed }
    }
}

// ------------ Quality Switch Job ------------

pub fn progress_emitter<R: Runtime>(app: &AppHandle<R>, event: &'static str, job_id: String, name: String) -> impl Fn(u64, u64, u64, u64, u64, u64, u8) + Send + Sync + 'static {
    let app = app.clone();
    move |download_current, download_total, install_current, install_total, net_speed, disk_speed, phase| {
        let mut dlp = HashMap::new();
        dlp.insert("job_id", job_id.clone());
        dlp.insert("name", name.clone());
        dlp.insert("progress", download_current.to_string());
        dlp.insert("total", download_total.to_string());
        dlp.insert("speed", net_speed.to_string());
        dlp.insert("disk", disk_speed.to_string());
        dlp.insert("install_progress", install_current.to_string());
        dlp.insert("install_total", install_total.to_string());
        // Phase: 0=idle, 1=verifying, 2=downloading, 3=installing, 4=validating, 5=moving
        dlp.insert("phase", phase.to_string());
        let _ = app.emit(event, dlp);
    }
}

pub fn register_quality_switch_handler<R: Runtime>(app: &AppHandle<R>) {
    let a = app.clone();
    app.listen("start_quality_switch", move |event| {
        let Ok(payload) = serde_json::from_str::<DownloadGamePayload>(event.payload()) else { return; };
        let state = a.state::<DownloadState>();
        let q = state.queue.lock().unwrap().clone();
        if let Some(queue) = q {
            if queue.has_job_for_id(payload.install.clone()) { show_dialog_with_callback(&a, "warning", "TwintailLauncher", "dialogs.game_already_queued_quality", Some(vec!["dialogs.buttons.ok"]), None, None); return; }
            queue.enqueue(QueueJobKind::GameQualitySwitch, QueueJobPayload::Game(payload));
        } else {
            let h5 = a.clone();
            std::thread::spawn(move || {
                let job_id = format!("direct_quality_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
                let _ = run_quality_switch(h5, payload, job_id);
            });
        }
    });
}

pub fn run_quality_switch<R: Runtime>(h5: AppHandle<R>, payload: DownloadGamePayload, job_id: String) -> QueueJobOutcome {
    let install_id = payload.install.clone();
    let Some(install) = get_install_info_by_id(&h5, install_id.clone()) else { log::warn!("Cannot switch quality: install {install_id} not found"); return QueueJobOutcome::Failed; };
    let Some(gm) = get_manifest_info_by_id(&h5, install.manifest_id.clone()).and_then(|lm| get_manifest(&h5, lm.filename)) else { return QueueJobOutcome::Failed; };
    if !supports_quality(&gm.biz) { return QueueJobOutcome::Failed; }
    let quality = normalize_quality(&install.resource_quality);
    log::info!("Switching \"{}\" to {} quality", install.name, quality.to_ascii_uppercase());

    let emit_progress = progress_emitter(&h5, "repair_progress", job_id.clone(), install.name.clone());
    emit_progress(0, 1000, 0, 1000, 0, 0, 0);

    let cancel_token = Arc::new(AtomicBool::new(false));
    { h5.state::<DownloadState>().tokens.lock().unwrap().insert(install_id.clone(), cancel_token.clone()); }

    #[cfg(target_os = "linux")]
    crate::utils::apply_patch(&h5, install.directory.clone(), "aki".to_string(), "remove".to_string());
    // Common files only get a size check here, a switch isn't a repair
    let result = run_pack_sync(&h5, &install, quality, &install.version, "repairing", Verify::Size, emit_progress, cancel_token.clone());
    #[cfg(target_os = "linux")]
    crate::utils::apply_patch(&h5, install.directory.clone(), "aki".to_string(), "add".to_string());

    { h5.state::<DownloadState>().tokens.lock().unwrap().remove(&install_id); }
    match result {
        PackSyncResult::Done => {
            log::info!("\"{}\" now runs on {} quality", install.name, quality.to_ascii_uppercase());
            let _ = h5.emit("repair_complete", ());
            QueueJobOutcome::Completed
        }
        PackSyncResult::Cancelled => {
            log::info!("Quality switch cancelled for \"{}\"", install.name);
            let _ = h5.emit("repair_paused", HashMap::from([("job_id", job_id), ("name", install.name.clone())]));
            QueueJobOutcome::Cancelled
        }
        PackSyncResult::Failed => {
            show_dialog_with_callback(&h5, "warning", "TwintailLauncher", "dialogs.quality_switch_error", Some(vec!["dialogs.buttons.ok"]), None, Some(HashMap::from([("install_name", install.name.as_str())])));
            let _ = h5.emit("repair_complete", ());
            QueueJobOutcome::Failed
        }
        PackSyncResult::Reported => {
            let _ = h5.emit("repair_complete", ());
            QueueJobOutcome::Failed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn config() -> Value {
        let pack = |name: &str, size: u64| json!({ "version": "3.7.0", "indexFile": format!("g/{name}/indexFile.json"), "baseUrl": "g/zip/", "size": size, "unCompressSize": size });
        json!({
            "cdnList": [{ "P": 10, "url": "https://a/" }, { "P": 0, "url": "https://b/" }, { "P": 90, "url": "https://c/" }],
            "resourcePacks": { "common": pack("common", 40), "sd": pack("sd", 22), "hd": pack("hd", 46), "uhd": pack("uhd", 66) },
            "bundles": {
                "SD": { "resourcePacks": ["common", "sd"] },
                "HD": { "resourcePacks": ["common", "hd"] },
                "UHD": { "resourcePacks": ["common", "uhd"] }
            }
        })
    }

    #[test]
    fn plans_common_plus_the_quality_pack() {
        let plan = plan_for(&config(), "UHD").unwrap();
        assert_eq!(plan.quality, "uhd");
        assert_eq!(plan.version, "3.7.0");
        assert_eq!(plan.index_files, ["g/common/indexFile.json", "g/uhd/indexFile.json"]);
        assert_eq!(plan.cdns, ["https://c/", "https://a/", "https://b/"]);
        assert_eq!(join_url(&plan.cdns[0], &plan.base_path), "https://c/g/zip/");
    }

    #[test]
    fn sizes_add_common_to_each_quality() {
        let sizes = quality_sizes(&config());
        assert_eq!(sizes["sd"].install_bytes, 62);
        assert_eq!(sizes["uhd"].download_bytes, 106);
        assert_eq!(config_version(&config()), "3.7.0");
    }

    #[test]
    fn quality_flag_replaces_the_manifest_default() {
        assert_eq!(with_quality_arg("-dx12 -krqlv=hd", "uhd"), "-dx12 -krqlv=uhd");
        assert_eq!(with_quality_arg("", "sd"), "-krqlv=sd");
        assert_eq!(normalize_quality("UHD"), "uhd");
        assert_eq!(normalize_quality("8k"), "hd");
    }

    #[test]
    fn rejects_paths_escaping_the_game_folder() {
        let root = Path::new("game");
        assert!(resource_path(root, "Client/Content/SD/a.pak").is_some());
        assert!(resource_path(root, "../evil.dll").is_none());
        assert!(resource_path(root, "").is_none());
    }
}
