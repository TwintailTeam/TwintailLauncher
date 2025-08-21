import {DownloadCloudIcon, HardDriveDownloadIcon, X} from "lucide-react";
import {POPUPS} from "./POPUPS.ts";
import FolderInput from "../common/FolderInput.tsx";
import CheckBox from "../common/CheckBox.tsx";
import TextDisplay from "../common/TextDisplay.tsx";
import SelectMenu from "../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import {emit} from "@tauri-apps/api/event";
import {useState, useEffect} from "react";



interface IProps {
    disk: any;
    setOpenPopup: any;
    displayName: string;
    settings: any;
    biz: string;
    versions: any[];
    background: string;
    icon: string;
    pushInstalls: () => void;
    runnerVersions: any[];
    dxvkVersions: any[];
    setCurrentInstall: (installId: string) => void;
    setBackground: (background: string) => void;
    fetchDownloadSizes: (biz: any, version: any, lang: any, path: any, callback: (data: any) => void) => void;
    openAsExisting?: boolean;
}
export default function DownloadGame({disk, setOpenPopup, displayName, settings, biz, versions, background, icon, pushInstalls, runnerVersions, dxvkVersions, setCurrentInstall, setBackground, fetchDownloadSizes, openAsExisting}: IProps) {
    const [skipGameDownload] = useState<boolean>(!!openAsExisting);
    const [selectedGameVersion, setSelectedGameVersion] = useState(versions?.[0]?.value || "");
    const [selectedAudioLang, setSelectedAudioLang] = useState("en-us");
    const [selectedRunnerVersion, setSelectedRunnerVersion] = useState(runnerVersions?.[0]?.value || "");
    const [selectedDxvkVersion, setSelectedDxvkVersion] = useState(dxvkVersions?.[0]?.value || "");

    // Animation state
    const [isClosing, setIsClosing] = useState(false);

    // Update button state when skipGameDownload changes
    useEffect(() => {
        const btn = document.getElementById("game_dl_btn");
        const freedisk = document.getElementById("game_disk_free");

        if (btn && freedisk) {
            if (skipGameDownload) {
                btn.removeAttribute("disabled");
                freedisk.classList.remove("text-red-600");
                freedisk.classList.add("text-white");
                freedisk.classList.remove("font-bold");
            } else {
                if (disk && disk.game_decompressed_size_raw > disk.free_disk_space_raw) {
                    btn.setAttribute("disabled", "");
                    freedisk.classList.add("text-red-600");
                    freedisk.classList.remove("text-white");
                    freedisk.classList.add("font-bold");
                }
            }
        }
    }, [skipGameDownload, disk]);

    // Handle close animation and delayed unmount
    const handleClose = () => {
        setIsClosing(true);
        setTimeout(() => {
            setOpenPopup(POPUPS.NONE);
    }, 220); // match animation duration in tailwind.config.js
    };

    function formatDir() {
        // @ts-ignore
        return document.getElementById("install_game_path").value;
    }

    return (
        <div
            className={`rounded-xl h-auto w-full max-w-2xl mx-auto bg-gradient-to-br from-black/80 via-black/70 to-black/60 backdrop-blur-xl border border-white/30 shadow-2xl ${skipGameDownload ? 'shadow-green-500/25' : 'shadow-purple-500/20'} flex flex-col p-6 overflow-hidden ${isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'} duration-100 ease-out`}
        >
            <div className="flex flex-row items-center justify-between mb-6">
                <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-purple-200 bg-clip-text text-transparent">{skipGameDownload ? "Add" : "Install"} {displayName}</h1>
                <X className="text-white/70 hover:text-white hover:bg-white/10 rounded-lg p-3 w-10 h-10 transition-all duration-200 cursor-pointer" onClick={handleClose}/>
            </div>
            <div className="w-full flex-1 overflow-y-auto overflow-scroll scrollbar-none pr-4 -mr-4">
                <form className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-sm border border-white/20 rounded-xl p-6 flex flex-col gap-4 w-full max-w-xl mx-auto shadow-inner">
                    {/* @ts-ignore */}
                    <div className="w-full"><FolderInput name={"Install location"} clearable={true} value={`${settings.default_game_path}/${biz}`} folder={true} id={"install_game_path"} biz={biz} fetchDownloadSizes={fetchDownloadSizes} version={() => selectedGameVersion} lang={() => selectedAudioLang} helpText={"Location where to download game files."} skipGameDownload={skipGameDownload}/></div>
                    {/* Existing install toggle is now internal; removed from UI */}
                    <div className="w-full"><CheckBox enabled={false} name={"Skip version update check"} id={"skip_version_updates"} helpText={"Skip checking for game updates."}/></div>
                    <div className="w-full"><CheckBox enabled={false} name={"Skip hash validation"} id={"skip_hash_validation"} helpText={"Skip validating files during game repair process, this will speed up the repair process significantly."}/></div>
                    <div className="w-full"><TextDisplay id={"game_disk_free"} name={"Available disk space"} value={`${disk.free_disk_space}`} style={"text-white px-3 w-full"}/></div>
                    <div className="w-full"><TextDisplay id={"game_disk_need"} name={"Required disk space (unpacked)"} value={`${disk.game_decompressed_size}`} style={"text-white px-3 w-full"}/></div>
                    <div className="w-full"><SelectMenu id={"game_version"} name={"Game version"} options={versions} multiple={false} selected={selectedGameVersion} biz={biz} dir={formatDir} fetchDownloadSizes={fetchDownloadSizes} lang={() => selectedAudioLang} helpText={"Version of the game to install."} setOpenPopup={setOpenPopup} skipGameDownload={skipGameDownload} onSelect={setSelectedGameVersion}/></div>
                    <div className="w-full"><SelectMenu id={"game_audio_langs"} name={"Voice pack"} options={[{name: "English (US)", value: "en-us"}, {name: "Japanese", value: "ja-jp"}, {name: "Korean", value: "ko-kr"}, {name: "Chinese", value: "zh-cn"}]} multiple={false} selected={selectedAudioLang} biz={biz} fetchDownloadSizes={fetchDownloadSizes} dir={formatDir} version={() => selectedGameVersion} helpText={"What audio package to install for the game."} setOpenPopup={setOpenPopup} skipGameDownload={skipGameDownload} onSelect={setSelectedAudioLang}/></div>
                    {(window.navigator.platform.includes("Linux")) ? <div className="w-full"><SelectMenu id={"runner_version"} name={"Runner version"} multiple={false} options={runnerVersions} selected={selectedRunnerVersion} helpText={"Wine/Proton version to use for this installation."} setOpenPopup={setOpenPopup} onSelect={setSelectedRunnerVersion}/></div> : null}
                    {(window.navigator.platform.includes("Linux")) ? <div className="w-full"><SelectMenu id={"dxvk_version"} name={"DXVK version"} multiple={false} options={dxvkVersions} selected={selectedDxvkVersion} helpText={"What DXVK version to use for this installation."} setOpenPopup={setOpenPopup} onSelect={setSelectedDxvkVersion}/></div> : null}
                    {(window.navigator.platform.includes("Linux")) ? <div className="w-full"><FolderInput name={"Runner prefix location"} clearable={true} value={`${settings.default_runner_prefix_path}/${biz}`} folder={true} id={"install_prefix_path"} helpText={"Location where to store Wine/Proton prefix."}/></div>: null}
                </form>
            </div>
            <div className="flex justify-center pt-6 mt-4 border-t border-white/10">
                <button className={`flex flex-row gap-3 items-center py-3 px-8 rounded-xl disabled:bg-gray-500 disabled:from-gray-500 disabled:to-gray-600 shadow-lg transition-all duration-200 transform hover:scale-105 font-semibold text-white bg-gradient-to-r focus:outline-none focus-visible:ring-2 ${skipGameDownload ? 'from-green-600 to-green-700 hover:from-green-500 hover:to-green-600 shadow-green-500/40 hover:shadow-green-500/60 focus-visible:ring-green-400' : 'from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 shadow-purple-500/40 hover:shadow-purple-500/60 focus-visible:ring-purple-300'}`} id={"game_dl_btn"} onClick={() => {
                    setIsClosing(true);
                    setTimeout(() => {
                        // ...existing code...
                        // @ts-ignore
                        let hash_skip = document.getElementById("skip_hash_validation").checked;
                        // @ts-ignore
                        let skip_version = document.getElementById("skip_version_updates").checked;
                        // @ts-ignore
                        let install_path = document.getElementById("install_game_path").value;
                        let gvv = selectedGameVersion;
                        let vpp = selectedAudioLang;
                        let rvv = selectedRunnerVersion || "none";
                        let dvv = selectedDxvkVersion || "none";
                        let rp = document.getElementById("install_prefix_path");
                        let rpp = "none";
                        if (rp !== null) {
                            // @ts-ignore
                            rpp = rp.value;
                        }
                        let skipdl = skipGameDownload;
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
                        setOpenPopup(POPUPS.NONE);
                    }, 420);
                }}> {skipGameDownload ? <HardDriveDownloadIcon/> : <DownloadCloudIcon/>} <span>{skipGameDownload ? "Add existing installation" : "Start installation"}</span></button>
            </div>
        </div>
    );
}
