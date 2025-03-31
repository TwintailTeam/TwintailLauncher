import {listen} from "@tauri-apps/api/event";

export default function moveTracker(install: string) {
   listen<string>('move_complete', () => {
        let launchbtn = document.getElementById("launch_game_btn");
        let isb = document.getElementById("install_settings_btn");
        let pb = document.getElementById("progress_bar");
        let pbn = document.getElementById("progress_name");
        let pbv = document.getElementById("progress_value");

        if (launchbtn !== null && isb !== null && pb !== null && pbn !== null && pbv !== null) {
            launchbtn.removeAttribute("disabled");
            isb.removeAttribute("disabled");
            pbn.innerText = "Installation move complete!";
            setTimeout(() => {
                pb.classList.add("hidden");
            }, 500);
        }
    });

    listen<any>('move_progress', (event) => {
        let launchbtn = document.getElementById(`launch_game_btn`);
        let isb = document.getElementById(`install_settings_btn`);
        let pb = document.getElementById("progress_bar");
        let pbn = document.getElementById("progress_name");
        let pbv = document.getElementById("progress_value");

        if (launchbtn !== null && isb !== null && pb !== null && pbn !== null && pbv !== null) {
            if (event.payload.install_id === install) {
                launchbtn.setAttribute("disabled", "");
                isb.setAttribute("disabled", "");
                pb.classList.remove("hidden");
                pbn.innerText = `Moving "${event.payload.file}"`;
                setTimeout(() => {
                    for (let i = 1; i < 100; i++) {
                        setTimeout(() => {
                            pbv.style.width = `${i}%`;
                        }, 500);
                    }
                }, 300);
            }
        }
    });
}