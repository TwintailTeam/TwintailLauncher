use rpcdiscord::{DiscordIpc,DiscordIpcClient,activity};
use tauri::AppHandle;
use crate::utils::models::{GameManifest, LauncherInstall};

const CLIENT_ID: &str = "1472729008675491872";

pub fn init(app: &AppHandle, install: LauncherInstall, gm: GameManifest) -> Option<DiscordIpcClient> {
    let mut client = DiscordIpcClient::new(CLIENT_ID).ok()?;
    if client.connect().is_err() { return None; }

    let icon_key = match gm.biz.as_str() {
        "hk4e_global" => "hk4e_icon",
        "hkrpg_global" => "hkrpg_icon",
        "nap_global" => "nap_icon",
        "bh3_global" => "bh3_icon",
        "wuwa_global" => "aki_icon",
        "pgr_global" => "pgr_icon",
        "aethergazer_global" => "agzr_icon",
        "endfield_global" => "endf_icon",
        "stellasora_global" => "stso_icon",
        "dna_global" => "dna_icon",
        "rev1999_global" => "rev1999_icon",
        "sdsgc_global" => "sdsgc_icon",
        &_ => "tl_512"
    };
    let start = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let ver = app.config().version.clone().unwrap_or_default();
    let small_txt = format!("TwintailLauncher v{ver}");
    let details = format!("Playing {}", install.name);

    let payload = activity::Activity::new().details(&details).state("In Game").timestamps(activity::Timestamps::new().start(start)).assets(activity::Assets::new().large_image(icon_key).large_text(&install.name).small_image("tl_512").small_text(&small_txt));
    let _ = client.set_activity(payload);
    log::info!("Discord RPC initialized for {} (ID: {})!", install.name, install.id);
    Some(client)
}

pub fn terminate(client: &mut DiscordIpcClient) {
    let _ = client.clear_activity();
    let _ = client.close();
    log::info!("Discord RPC terminated!");
}
