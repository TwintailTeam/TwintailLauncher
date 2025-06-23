import {listen} from "@tauri-apps/api/event";

export function moveTracker(_install: string) {
   listen<string>('move_complete', async () => {
       let launchbtn = await waitForElement("launch_game_btn");
       let isb = await waitForElement("install_settings_btn");
       let updatebtn = await waitForElement(`update_game_btn`);
       let pb = await waitForElement("progress_bar");

       isb.removeAttribute("disabled");
       pb.classList.add("hidden");
       if (launchbtn) launchbtn.removeAttribute("disabled");
       if (updatebtn) updatebtn.removeAttribute("disabled");
   }).then(() => {});

    listen<any>('move_progress', async (event) => {
        let launchbtn = await waitForElement(`launch_game_btn`);
        let isb = await waitForElement(`install_settings_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        isb.setAttribute("disabled", "");
        pb.classList.remove("hidden");
        pbn.textContent = `Moving "${event.payload.file}"`;
        pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
        progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
        if (launchbtn) launchbtn.setAttribute("disabled", "");
        if (updatebtn) updatebtn.setAttribute("disabled", "");
    }).then(async () => {});
}

export function generalEventsHandler() {
    // Download events
    listen<string>('download_complete', async () => {
        let launchbtn = await waitForElement("launch_game_btn");
        let isb = await waitForElement("install_settings_btn");
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = await waitForElement("progress_bar");

        isb.removeAttribute("disabled");
        pb.classList.add("hidden");
        if (launchbtn) launchbtn.removeAttribute("disabled");
        if (updatebtn) updatebtn.removeAttribute("disabled");
    }).then(() => {});

    listen<any>('download_progress', async (event) => {
        let launchbtn = await waitForElement(`launch_game_btn`);
        let isb = await waitForElement(`install_settings_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        isb.setAttribute("disabled", "");
        pb.classList.remove("hidden");
        pbn.textContent = `Downloading "${event.payload.name}"`;
        pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
        progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
        if (launchbtn) launchbtn.setAttribute("disabled", "");
        if (updatebtn) updatebtn.setAttribute("disabled", "");
    }).then(() => {});

    // Update events
    listen<string>('update_complete', async () => {
        let launchbtn = await waitForElement("launch_game_btn");
        let updatebtn = await waitForElement(`update_game_btn`);
        let isb = await waitForElement("install_settings_btn");
        let pb = await waitForElement("progress_bar");

        isb.removeAttribute("disabled");
        pb.classList.add("hidden");
        if (launchbtn) launchbtn.removeAttribute("disabled");
        if (updatebtn) updatebtn.removeAttribute("disabled");
    }).then(() => {});

    listen<any>('update_progress', async (event) => {
        let launchbtn = await waitForElement(`launch_game_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let isb = await waitForElement(`install_settings_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        isb.setAttribute("disabled", "");
        pb.classList.remove("hidden");
        pbn.textContent = `Updating "${event.payload.name}"`;
        pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
        progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
        if (launchbtn) launchbtn.setAttribute("disabled", "");
        if (updatebtn) updatebtn.setAttribute("disabled", "");
    }).then(() => {});

    // Repair events
    listen<string>('repair_complete', async () => {
        let launchbtn = await waitForElement("launch_game_btn");
        let isb = await waitForElement("install_settings_btn");
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = await waitForElement("progress_bar");

        isb.removeAttribute("disabled");
        pb.classList.add("hidden");
        if (launchbtn) launchbtn.removeAttribute("disabled");
        if (updatebtn) updatebtn.removeAttribute("disabled");
    }).then(() => {});

    listen<any>('repair_progress', async (event) => {
        let launchbtn = await waitForElement(`launch_game_btn`);
        let isb = await waitForElement(`install_settings_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        isb.setAttribute("disabled", "");
        pb.classList.remove("hidden");
        pbn.textContent = `Repairing "${event.payload.name}"`;
        pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
        progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
        if (launchbtn) launchbtn.setAttribute("disabled", "");
        if (updatebtn) updatebtn.setAttribute("disabled", "");
    }).then(() => {});

    // Preload events
    listen<string>('preload_complete', async () => {
        let launchbtn = await waitForElement("launch_game_btn");
        let updatebtn = await waitForElement(`update_game_btn`);
        let isb = await waitForElement("install_settings_btn");
        let pb = await waitForElement("progress_bar");

        isb.removeAttribute("disabled");
        pb.classList.add("hidden");
        if (launchbtn) launchbtn.removeAttribute("disabled");
        if (updatebtn) updatebtn.removeAttribute("disabled");
    }).then(() => {});

    listen<any>('preload_progress', async (event) => {
        let launchbtn = await waitForElement(`launch_game_btn`);
        let updatebtn = await waitForElement(`update_game_btn`);
        let isb = await waitForElement(`install_settings_btn`);
        let pb = await waitForElement("progress_bar");
        let pbn = await waitForElement("progress_name");
        let pbv = await waitForElement("progress_value");
        let progressPercent = await waitForElement("progress_percent");

        isb.setAttribute("disabled", "");
        pb.classList.remove("hidden");
        pbn.textContent = `Predownloading "${event.payload.name}"`;
        pbv.style.width = `${Math.round(toPercent(event.payload.progress, event.payload.total))}%`;
        progressPercent.textContent = `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`;
        if (launchbtn) launchbtn.setAttribute("disabled", "");
        if (updatebtn) updatebtn.setAttribute("disabled", "");
    }).then(() => {});
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
            const el = document.getElementById(id);
            if (el) {
                clearInterval(interval);
                resolve(el);
            }
        }, timeout);
    });
}

function toPercent(number: any, total: any) { return (parseInt(number) / parseInt(total)) * 100; }