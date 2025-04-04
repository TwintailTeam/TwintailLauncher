import {listen} from "@tauri-apps/api/event";
import {isPermissionGranted, requestPermission, sendNotification} from "@tauri-apps/plugin-notification";

var installName = "?";
var installType = "?";
var shouldNotify = false;

export function moveTracker(install: string) {
   listen<string>('move_complete', async (event: any) => {
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
       sendNotify("KeqingLauncher", `Moving of ${event.payload.install_name}'s ${event.payload.install_type} files complete. You can now again launch all installed games.`, "dialog-information").then(() => {});
   }).then(() => {});

    listen<any>('move_progress', async (event) => {
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
                installName = event.payload.install_name;
                installType = event.payload.install_type;
                shouldNotify = true;
            }
        }
    }).then(() => {
        if (shouldNotify) {
            sendNotify("KeqingLauncher", `Moving of ${installName}'s ${installType} files started. You can not launch any game until move is completed.`, "dialog-information").then(() => {});
            shouldNotify = false;
        }
    });
}

async function checkPermission() {
    if (!(await isPermissionGranted())) {
        return (await requestPermission()) === 'granted'
    }
    return true
}

export async function sendNotify(title: string, content: string, icon: string) {
    if (!(await checkPermission())) {
        return
    }
    sendNotification({title: title, body: content, autoCancel: true, icon: icon});
}