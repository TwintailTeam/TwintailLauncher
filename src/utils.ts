import {listen} from "@tauri-apps/api/event";
import {message} from "@tauri-apps/plugin-dialog";

export default function moveTracker() {
    listen<string>('move_complete', async () => {
        await message("Move completed, you can now launch and edit installation settings.", {
            title: "Installation move complete",
            kind: "info"
        });
        let launchbtn = document.getElementById("launch_game_btn");
        let isb = document.getElementById("install_settings_btn");

        if (launchbtn !== null && isb !== null) {
            launchbtn.removeAttribute("disabled");
            isb.removeAttribute("disabled");
        }
    }).then(() => {});

    listen<string>('move_progress', (_event) => {
    }).then(async () => {
        await message("Moving started, you wont be able to launch or edit options of the installation until completed.", {
            title: "Installation moving...",
            kind: "info"
        });
        let launchbtn = document.getElementById("launch_game_btn");
        let isb = document.getElementById("install_settings_btn");

        if (launchbtn !== null && isb !== null) {
            launchbtn.setAttribute("disabled", "");
            isb.setAttribute("disabled", "");
        }
    });
}