use serde::Serialize;
use std::time::{Instant};

#[derive(Serialize, Clone)]
pub struct NetworkStatus {
    pub status: String, // "online", "slow", "offline"
    pub latency_ms: Option<u64>,
    pub message: String,
}

#[tauri::command]
pub async fn check_network_connectivity() -> NetworkStatus {
    let endpoints = ["https://store.steampowered.com", "https://one.one.one.one", "https://twintaillauncher.app"];

    let mut best_latency: Option<u64> = None;
    for endpoint in endpoints {
        let start = Instant::now();
        match fischl::utils::check_network_status(endpoint.to_string()).await {
            Ok(response) => {
                let latency = start.elapsed().as_millis() as u64;
                if response.status().is_success() || response.status().as_u16() == 204 || response.status().as_u16() == 405 {
                    // Return immediately if any endpoint is fast
                    if latency < 5000 {
                        log::debug!("Network check: online ({}ms via {})", latency, endpoint);
                        return NetworkStatus { status: "online".to_string(), latency_ms: Some(latency), message: "Connection is good".to_string() };
                    }
                    // Track best latency across slow endpoints
                    if best_latency.is_none() || latency < best_latency.unwrap() { best_latency = Some(latency); }
                }
            }
            Err(_) => { continue; }
        }
    }

    // If any endpoint responded (but all were slow), report slow with best latency
    if let Some(latency) = best_latency {
        log::warn!("Network check: slow (best {}ms, all endpoints responded slowly)", latency);
        return NetworkStatus { status: "slow".to_string(), latency_ms: Some(latency), message: "Connection is slow".to_string() };
    }

    log::warn!("Network check: offline, all endpoints unreachable");
    NetworkStatus { status: "offline".to_string(), latency_ms: None, message: "Unable to connect to the internet".to_string() }
}
