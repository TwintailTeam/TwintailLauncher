import {emit, listen} from "@tauri-apps/api/event";
import {isPermissionGranted, requestPermission, sendNotification} from "@tauri-apps/plugin-notification";

export function moveTracker(install: string) {
   listen<string>('move_complete', async (event: any) => {
       let launchbtn = document.getElementById("launch_game_btn");
       let isb = document.getElementById("install_settings_btn");
       let updatebtn = await waitForElement(`update_game_btn`);
       let pb = document.getElementById("progress_bar");
       let pbn = document.getElementById("progress_name");
       let pbv = document.getElementById("progress_value");

       if (isb !== null && pb !== null && pbn !== null && pbv !== null) {
           if (launchbtn) launchbtn.removeAttribute("disabled");
           if (updatebtn) updatebtn.removeAttribute("disabled");
           isb.removeAttribute("disabled");
           pbn.textContent = "Move complete!";
           setTimeout(() => {pb.classList.add("hidden");}, 500);
       }
       await sendNotify("TwintailLauncher", `Moving of ${event.payload.install_name}'s ${event.payload.install_type} files complete.`, "dialog-information");
       emit("prevent_exit", false).then(() => {});
   }).then(() => {});

    listen<any>('move_progress', async (event) => {
        let launchbtn = document.getElementById(`launch_game_btn`);
        let isb = document.getElementById(`install_settings_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = document.getElementById("progress_bar");
        let pbn = document.getElementById("progress_name");
        let pbv = document.getElementById("progress_value");

        if (isb !== null && pb !== null && pbn !== null && pbv !== null) {
            if (event.payload.install_id === install) {
                if (launchbtn) launchbtn.setAttribute("disabled", "");
                if (updatebtn) updatebtn.setAttribute("disabled", "");
                isb.setAttribute("disabled", "");
                pb.classList.remove("hidden");
                pbn.textContent = `Moving "${event.payload.file}"`;
                await simulateProgress();
                emit("prevent_exit", true).then(() => {});
            }
        }
    }).then(async () => {});
}

export function generalEventsHandler() {
    listen<any>("telemetry_block", (event) => {
        switch (event.payload) {
            case 1: {
                sendNotify("TwintailLauncher", "Successfully blocked telemetry servers.", "dialog-information").then(() => {});
            }
            break;
            case 2: {
                sendNotify("TwintailLauncher", 'Telemetry servers already blocked.', "dialog-information").then(() => {});
            }
            break;
            case 0: {
                sendNotify("TwintailLauncher", 'Failed to block telemetry servers, Please press "Block telemetry" in launcher settings!', "dialog-error").then(() => {});
            }
            break;
        }
    }).then(() => {});

    // Download events
    listen<string>('download_complete', async (event: any) => {
        let launchbtn = await waitForElement("launch_game_btn");
        let isb = await waitForElement("install_settings_btn");
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");

        if (isb !== null && pb !== null && pbn !== null && pbv !== null) {
            pbn.textContent = "Download complete!";
            setTimeout(() => {pb.classList.add("hidden");}, 500);
            if (launchbtn) launchbtn.removeAttribute("disabled");
            if (updatebtn) updatebtn.removeAttribute("disabled");
            isb.removeAttribute("disabled");
        }
        await sendNotify("TwintailLauncher", `Download of ${event.payload} complete.`, "dialog-information");
        emit("prevent_exit", false).then(() => {});
    }).then(() => {});

    listen<any>('download_progress', async (event) => {
        let launchbtn = await waitForElement(`launch_game_btn`);
        let isb = await waitForElement(`install_settings_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        if (isb !== null && pb !== null && pbn !== null && pbv !== null) {
           if (launchbtn) launchbtn.setAttribute("disabled", "");
           if (updatebtn) updatebtn.setAttribute("disabled", "");
           isb.setAttribute("disabled", "");
           pb.classList.remove("hidden");
           pbn.textContent = `Downloading "${event.payload.name}"`;
           pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
           progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
           emit("prevent_exit", true).then(() => {});
        }
    }).then(() => {});

    // Update events
    listen<string>('update_complete', async (event: any) => {
        let launchbtn = await waitForElement("launch_game_btn");
        let updatebtn = await waitForElement(`update_game_btn`);
        let isb = await waitForElement("install_settings_btn");
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");

        if (isb !== null && pb !== null && pbn !== null && pbv !== null && updatebtn !== null) {
            if (launchbtn) launchbtn.removeAttribute("disabled");
            if (updatebtn) updatebtn.removeAttribute("disabled");
            isb.removeAttribute("disabled");
            pbn.textContent = "Updates complete!";
            setTimeout(() => {pb.classList.add("hidden");}, 500);
        }
        await sendNotify("TwintailLauncher", `Updating ${event.payload} complete.`, "dialog-information");
        emit("prevent_exit", false).then(() => {});
    }).then(() => {});

    listen<any>('update_progress', async (event) => {
        let launchbtn = await waitForElement(`launch_game_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let isb = await waitForElement(`install_settings_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        if (isb !== null && pb !== null && pbn !== null && pbv !== null && updatebtn !== null) {
            if (launchbtn) launchbtn.setAttribute("disabled", "");
            if (updatebtn) updatebtn.setAttribute("disabled", "");
            isb.setAttribute("disabled", "");
            pb.classList.remove("hidden");
            pbn.textContent = `Updating "${event.payload.name}"`;
            pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
            progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
            emit("prevent_exit", true).then(() => {});
        }
    }).then(() => {});

    // Repair events
    listen<string>('repair_complete', async (event: any) => {
        let launchbtn = document.getElementById("launch_game_btn");
        let isb = document.getElementById("install_settings_btn");
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = document.getElementById("progress_bar");
        let pbn = document.getElementById("progress_name");
        let pbv = document.getElementById("progress_value");

        if (isb !== null && pb !== null && pbn !== null && pbv !== null) {
            if (launchbtn) launchbtn.removeAttribute("disabled");
            if (updatebtn) updatebtn.removeAttribute("disabled");
            isb.removeAttribute("disabled");
            pbn.textContent = "Repair complete!";
            setTimeout(() => {pb.classList.add("hidden");}, 500);
        }
        await sendNotify("TwintailLauncher", `Repair of ${event.payload} complete.`, "dialog-information");
        emit("prevent_exit", false).then(() => {});
    }).then(() => {});

    listen<any>('repair_progress', async (event) => {
        let launchbtn = document.getElementById(`launch_game_btn`);
        let isb = document.getElementById(`install_settings_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = document.getElementById("progress_bar");
        let pbn = document.getElementById("progress_name");
        let pbv = document.getElementById("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        if (isb !== null && pb !== null && pbn !== null && pbv !== null) {
            if (launchbtn) launchbtn.setAttribute("disabled", "");
            if (updatebtn) updatebtn.setAttribute("disabled", "");
            isb.setAttribute("disabled", "");
            pb.classList.remove("hidden");
            pbn.textContent = `Repairing "${event.payload.name}"`;
            pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
            progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
            emit("prevent_exit", true).then(() => {});
        }
    }).then(() => {});
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

function waitForElement(id: string, timeout = 3000): Promise<HTMLElement> {
    return new Promise((resolve, _reject) => {
        const interval = setInterval(() => {
            const el = document.getElementById(id);
            if (el) {
                clearInterval(interval);
                resolve(el);
            }
        }, 50);
        setTimeout(() => {
            clearInterval(interval);
            // @ts-ignore
            resolve(null)
        }, timeout);
    });
}

let progress = 0;
let barWidth = 5;
async function simulateProgress() {
    let progressBar = await waitForElement("progress_value");
    let progressPercent = await waitForElement("progress_percent");
    if (progress < 100) {
        let base = progress < 60 ? 2 + Math.random() * 2 : progress < 85 ? 1 + Math.random() : 0.2 + Math.random() * 0.5;
        if (Math.random() < 0.07 && progress > 30 && progress < 95) {
            setTimeout(simulateProgress, 800 + Math.random() * 5000);
            return;
        }
        progress = Math.min(progress + base, 100);
        barWidth = 5 + (progress * 0.85);
        progressBar.style.width = `${barWidth}%`;
        progressPercent.textContent = `${Math.floor(progress)}%`;
        setTimeout(simulateProgress, 50 + Math.random() * 220);
    } else {
        progressBar.style.width = '100%';
        progressPercent.textContent = '100%';
    }
}

function toPercent(number: any, total: any) {
    return (parseInt(number) / parseInt(total)) * 100;
}