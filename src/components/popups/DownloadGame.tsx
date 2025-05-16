import {DownloadCloudIcon, X} from "lucide-react";
import {POPUPS} from "./POPUPS.ts";
import FolderInput from "../common/FolderInput.tsx";
import CheckBox from "../common/CheckBox.tsx";
import TextDisplay from "../common/TextDisplay.tsx";
import SelectMenu from "../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import {emit} from "@tauri-apps/api/event";

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

    return (
        <div className="rounded-lg h-3/4 w-2/4 flex flex-col p-4 gap-8 overflow-scroll">
            <div className="flex flex-row items-center justify-between">
                <h1 className="text-white font-bold text-2xl">Download {displayName}</h1>
                <X className="text-white cursor-pointer" onClick={() => setOpenPopup(POPUPS.NONE)}/>
            </div>
            <div className="flex flex-row-reverse">
                <button className="flex flex-row gap-1 items-center p-2 bg-blue-600 rounded-lg disabled:bg-gray-500" id={"game_dl_btn"} onClick={() => {
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
                        rpp = rp.value + "/" + gvv;
                    }

                    // @ts-ignore
                    let skipdl = document.getElementById("skip_game_dl").checked;

                    invoke("add_install", {
                        manifestId: biz,
                        version: gvv,
                        audioLang: vpp,
                        name: displayName,
                        directory: install_path + "/" + gvv,
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
                                // @ts-ignore
                                document.getElementById(r.install_id).focus();
                                if (!skipdl) {
                                    emit("start_game_download", {install: r.install_id, biz: biz, lang: vpp}).then(() => {});
                                }
                            }, 20);
                        } else {
                            console.error("Download error!");
                        }
                    });
                }}><DownloadCloudIcon/><span className="font-semibold translate-y-px">Start download</span></button>
            </div>
                <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-700 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`} style={{maxHeight: (20 * 64) + "px"}}>
                    {/* @ts-ignore */}
                    <FolderInput name={"Install location"} clearable={true} value={`${settings.default_game_path}/${biz}`} folder={true} id={"install_game_path"} biz={biz} fetchDownloadSizes={fetchDownloadSizes} version={getVersion} lang={getAudio}/>
                    <CheckBox enabled={false} name={"Skip game download (Existing install)"} id={"skip_game_dl"}/>
                    <CheckBox enabled={false} name={"Skip version update check"} id={"skip_version_updates"}/>
                    <CheckBox enabled={false} name={"Skip hash validation"} id={"skip_hash_validation"}/>
                    <TextDisplay id={"game_disk_free"} name={"Available disk space"} value={`${disk.free_disk_space}`} style={"text-white px-3"}/>
                    <TextDisplay id={"game_disk_need"} name={"Required disk space (unpacked)"} value={`${disk.game_decompressed_size}`} style={"text-white px-3"}/>
                    <SelectMenu id={"game_version"} name={"Game version"} options={versions} multiple={false} selected={""} biz={biz} dir={formatDir} fetchDownloadSizes={fetchDownloadSizes} lang={getAudio}/>
                    <SelectMenu id={"game_audio_langs"} name={"Voice pack"} options={[{name: "English (US)", value: "en-us"}, {name: "Japanese", value: "ja-jp"}, {name: "Korean", value: "ko-kr"}, {name: "Chinese", value: "zh-cn"}]} multiple={false} selected={""} biz={biz} fetchDownloadSizes={fetchDownloadSizes} dir={formatDir} version={getVersion}/>
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"runner_version"} name={"Runner version"} multiple={false} options={runnerVersions} selected={runnerVersions[0].value}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"dxvk_version"} name={"DXVK version"} multiple={false} options={dxvkVersions} selected={dxvkVersions[0].value}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner prefix location"} clearable={true} value={`${settings.default_runner_prefix_path}/${biz}`} folder={true} id={"install_prefix_path"}/>: null}
                </div>
            </div>
    )
}

function formatDir() {
    // @ts-ignore
    let install_path = document.getElementById("install_game_path").value;
    // @ts-ignore
    let gv = document.getElementById("game_version");
    // @ts-ignore
    let gvv = gv.options[gv.selectedIndex].value;
    return install_path + "/" + gvv;
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
