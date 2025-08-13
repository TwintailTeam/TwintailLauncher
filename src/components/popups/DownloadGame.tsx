import {DownloadCloudIcon, X} from "lucide-react";
import {POPUPS} from "./POPUPS.ts";
import FolderInput from "../common/FolderInput.tsx";
import CheckBox from "../common/CheckBox.tsx";
import TextDisplay from "../common/TextDisplay.tsx";
import SelectMenu from "../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import {emit} from "@tauri-apps/api/event";
import {useState, useEffect} from "react";

interface IProps {
    icon: string,
    background: string,
    versions: any,
    settings: any,
    biz: any,
    displayName: string,
    runnerVersions: any,
    dxvkVersions: any,
    setOpenPopup: (popup: POPUPS) => void,
    pushInstalls: () => void,
    setCurrentInstall: (id: string) => void,
    setBackground: (id: string) => void,
    fetchDownloadSizes: (biz: any, version: any, lang: any, dir: any, callback: (data: any) => void) => void,
    disk: any
}

export default function DownloadGame({disk, setOpenPopup, displayName, settings, biz, versions, background, icon, pushInstalls, runnerVersions, dxvkVersions, setCurrentInstall, setBackground, fetchDownloadSizes}: IProps) {
    const [skipGameDownload, setSkipGameDownload] = useState(false);

    // Update button state when skipGameDownload changes
    useEffect(() => {
        const btn = document.getElementById("game_dl_btn");
        const freedisk = document.getElementById("game_disk_free");

        if (btn && freedisk) {
            if (skipGameDownload) {
                // Enable button and reset disk space styling when skipping download
                btn.removeAttribute("disabled");
                freedisk.classList.remove("text-red-600");
                freedisk.classList.add("text-white");
                freedisk.classList.remove("font-bold");
            } else {
                // Re-check disk space when not skipping download
                if (disk && disk.game_decompressed_size_raw > disk.free_disk_space_raw) {
                    btn.setAttribute("disabled", "");
                    freedisk.classList.add("text-red-600");
                    freedisk.classList.remove("text-white");
                    freedisk.classList.add("font-bold");
                }
            }
        }
    }, [skipGameDownload, disk]);

    return (
        <div className="rounded-lg h-auto w-3/5 bg-black/70 border border-white/20 flex flex-col p-4 gap-8 overflow-scroll scrollbar-none">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">Download {displayName}</h1>
                <X className="text-white hover:text-gray-400 cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex flex-row-reverse">
                <button className="flex flex-row gap-2 items-center py-2 px-4 bg-purple-600 hover:bg-purple-700 rounded-lg disabled:bg-gray-500 me-5" id={"game_dl_btn"} onClick={() => {
                    setOpenPopup(POPUPS.NONE);
                    // @ts-ignore
                    let hash_skip = document.getElementById("skip_hash_validation").checked;
                    // @ts-ignore
                    let skip_version = document.getElementById("skip_version_updates").checked;
                    // @ts-ignore
                    let install_path = document.getElementById("install_game_path").value;
                    // @ts-ignore
                    let gv = document.getElementById("game_version");
                    // @ts-ignore
                    let gvv = gv.options[gv.selectedIndex].value;

                    // @ts-ignore
                    let vp = document.getElementById("game_audio_langs");
                    // @ts-ignore
                    let vpp = vp.options[vp.selectedIndex].value;

                    let rv = document.getElementById("runner_version");
                    let rvv = "none";
                    if (rv !== null) {
                        // @ts-ignore
                        rvv = rv.options[rv.selectedIndex].value;
                    }

                    let dv = document.getElementById("dxvk_version");
                    let dvv = "none";
                    if (dv !== null) {
                        // @ts-ignore
                        dvv = dv.options[dv.selectedIndex].value;
                    }

                    let rp = document.getElementById("install_prefix_path");
                    let rpp = "none";
                    if (rp !== null) {
                        // @ts-ignore
                        rpp = rp.value;
                    }

                    // @ts-ignore
                    let skipdl = document.getElementById("skip_game_dl").checked;

                    invoke("add_install", {
                        manifestId: biz,
                        version: gvv,
                        audioLang: vpp,
                        name: displayName,
                        directory: install_path,
                        runnerPath: "none",
                        dxvkPath: "none",
                        runnerVersion: rvv,
                        dxvkVersion: dvv,
                        gameIcon: icon,
                        gameBackground: background,
                        ignoreUpdates: skip_version,
                        skipHashCheck: hash_skip,
                        useJadeite: false,
                        useXxmi: false,
                        useFpsUnlock: false,
                        envVars: "",
                        preLaunchCommand: "",
                        launchCommand: "",
                        fpsValue: "60",
                        runnerPrefix: rpp,
                        launchArgs: "",
                        skipGameDl: skipdl
                    }).then((r: any) => {
                        if (r.success) {
                            pushInstalls();
                            setCurrentInstall(r.install_id as string);
                            setBackground(r.background as string);
                            setTimeout(() => {
                                let installui = document.getElementById(r.install_id);
                                if (installui) installui.focus();
                                if (!skipdl) {
                                    emit("start_game_download", {install: r.install_id, biz: biz, lang: vpp}).then(() => {});
                                }
                            }, 20);
                        } else {
                            console.error("Download error!");
                        }
                    });
                }}><DownloadCloudIcon/><span className="font-semibold">{skipGameDownload ? "Add existing installation" : "Start download"}</span></button>
            </div>
            <div className="w-full overflow-y-auto overflow-scroll scrollbar-none pr-4 -mr-4">
                <div className="bg-black/70 border border-white/10 rounded-lg p-4 flex flex-col gap-4">
                    {/* @ts-ignore */}
                    <FolderInput name={"Install location"} clearable={true} value={`${settings.default_game_path}/${biz}`} folder={true} id={"install_game_path"} biz={biz} fetchDownloadSizes={fetchDownloadSizes} version={getVersion} lang={getAudio} helpText={"Location where to download game files."} skipGameDownload={skipGameDownload}/>
                    <CheckBox enabled={false} name={"Skip game download (Existing install)"} id={"skip_game_dl"} helpText={"This will skip downloading game files, useful if you already have game installed and just want to use that installation."} onToggle={setSkipGameDownload}/>
                    <CheckBox enabled={false} name={"Skip version update check"} id={"skip_version_updates"} helpText={"Skip checking for game updates."}/>
                    <CheckBox enabled={false} name={"Skip hash validation"} id={"skip_hash_validation"} helpText={"Skip validating files during game repair process, this will speed up the repair process significantly."}/>
                    <TextDisplay id={"game_disk_free"} name={"Available disk space"} value={`${disk.free_disk_space}`} style={"text-white px-3"}/>
                    <TextDisplay id={"game_disk_need"} name={"Required disk space (unpacked)"} value={`${disk.game_decompressed_size}`} style={"text-white px-3"}/>
                    <SelectMenu id={"game_version"} name={"Game version"} options={versions} multiple={false} selected={""} biz={biz} dir={formatDir} fetchDownloadSizes={fetchDownloadSizes} lang={getAudio} helpText={"Version of the game to install."} setOpenPopup={setOpenPopup} skipGameDownload={skipGameDownload}/>
                    <SelectMenu id={"game_audio_langs"} name={"Voice pack"} options={[{name: "English (US)", value: "en-us"}, {name: "Japanese", value: "ja-jp"}, {name: "Korean", value: "ko-kr"}, {name: "Chinese", value: "zh-cn"}]} multiple={false} selected={""} biz={biz} fetchDownloadSizes={fetchDownloadSizes} dir={formatDir} version={getVersion} helpText={"What audio package to install for the game."} setOpenPopup={setOpenPopup} skipGameDownload={skipGameDownload}/>
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"runner_version"} name={"Runner version"} multiple={false} options={runnerVersions} selected={runnerVersions[0].value} helpText={"Wine/Proton version to use for this installation."} setOpenPopup={setOpenPopup}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"dxvk_version"} name={"DXVK version"} multiple={false} options={dxvkVersions} selected={dxvkVersions[0].value} helpText={"What DXVK version to use for this installation."} setOpenPopup={setOpenPopup}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner prefix location"} clearable={true} value={`${settings.default_runner_prefix_path}/${biz}`} folder={true} id={"install_prefix_path"} helpText={"Location where to store Wine/Proton prefix."}/>: null}
                </div>
            </div>
        </div>
    )
}

function formatDir() {
    // @ts-ignore
    return document.getElementById("install_game_path").value;
}

function getVersion() {
    // @ts-ignore
    let gv = document.getElementById("game_version");
    // @ts-ignore
    return gv.options[gv.selectedIndex].value;
}

function getAudio() {
    // @ts-ignore
    let gv = document.getElementById("game_audio_langs");
    // @ts-ignore
    return gv.options[gv.selectedIndex].value;
}
