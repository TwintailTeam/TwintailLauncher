import {FolderOpenIcon, MonitorIcon, Trash2Icon, WrenchIcon, X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import FolderInput from "../../common/FolderInput.tsx";
import CheckBox from "../../common/CheckBox.tsx";
import TextInput from "../../common/TextInput.tsx";
import SelectMenu from "../../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import React from "react";
import {emit} from "@tauri-apps/api/event";
import SubMenu from "../../common/SubMenu.tsx";

// Steam icon component - official Steam logo
const SteamIcon = ({ className }: { className?: string }) => (
    <svg className={className} viewBox="0 0 32 32" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
        <path d="M18.102 12.129c0-0 0-0 0-0.001 0-1.564 1.268-2.831 2.831-2.831s2.831 1.268 2.831 2.831c0 1.564-1.267 2.831-2.831 2.831-0 0-0 0-0.001 0h0c-0 0-0 0-0.001 0-1.563 0-2.83-1.267-2.83-2.83 0-0 0-0 0-0.001v0zM24.691 12.135c0-2.081-1.687-3.768-3.768-3.768s-3.768 1.687-3.768 3.768c0 2.081 1.687 3.768 3.768 3.768v0c2.080-0.003 3.765-1.688 3.768-3.767v-0zM10.427 23.76l-1.841-0.762c0.524 1.078 1.611 1.808 2.868 1.808 1.317 0 2.448-0.801 2.93-1.943l0.008-0.021c0.155-0.362 0.246-0.784 0.246-1.226 0-1.757-1.424-3.181-3.181-3.181-0.405 0-0.792 0.076-1.148 0.213l0.022-0.007 1.903 0.787c0.852 0.364 1.439 1.196 1.439 2.164 0 1.296-1.051 2.347-2.347 2.347-0.324 0-0.632-0.066-0.913-0.184l0.015 0.006zM15.974 1.004c-7.857 0.001-14.301 6.046-14.938 13.738l-0.004 0.054 8.038 3.322c0.668-0.462 1.495-0.737 2.387-0.737 0.001 0 0.002 0 0.002 0h-0c0.079 0 0.156 0.005 0.235 0.008l3.575-5.176v-0.074c0.003-3.12 2.533-5.648 5.653-5.648 3.122 0 5.653 2.531 5.653 5.653s-2.531 5.653-5.653 5.653h-0.131l-5.094 3.638c0 0.065 0.005 0.131 0.005 0.199 0 0.001 0 0.002 0 0.003 0 2.342-1.899 4.241-4.241 4.241-2.047 0-3.756-1.451-4.153-3.38l-0.005-0.027-5.755-2.383c1.841 6.345 7.601 10.905 14.425 10.905 8.281 0 14.994-6.713 14.994-14.994s-6.713-14.994-14.994-14.994c-0 0-0.001 0-0.001 0h0z"/>
    </svg>
);

interface IProps {
    games: any,
    installedRunners: any,
    installSettings: any,
    setOpenPopup: (popup: POPUPS) => void,
    pushInstalls: () => void,
    setCurrentInstall: (id: string) => void,
    setCurrentGame: (id: string) => void,
    setBackground: (id: string) => void,
    fetchInstallSettings: (id: string) => void,
    // Prefetched to avoid pop-in
    prefetchedSwitches: any,
    prefetchedFps: any
}

interface IState {
    gameSwitches: any,
    gameFps: any,
    isClosing: boolean
}

export default class SettingsInstall extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props);
        this.state = {
            // Initialize from prefetched props to render immediately
            gameSwitches: props.prefetchedSwitches || {},
            gameFps: props.prefetchedFps || [],
            isClosing: false
        }
    }

    render() {
        return (
            <div className={`rounded-xl w-[90vw] max-w-4xl max-h-[85vh] bg-zinc-900 border border-white/20 flex flex-col p-6 overflow-hidden ${this.state.isClosing ? 'animate-bg-fade-out' : 'animate-bg-fade-in'}`}>
                <div className="flex flex-row items-center justify-between mb-2">
                    <h1 className="text-white font-bold text-3xl bg-gradient-to-r from-white to-cyan-200 bg-clip-text text-transparent">{this.props.installSettings.name}</h1>
                    <X className="text-white/70 hover:text-white hover:bg-white/10 rounded-lg p-3 w-12 h-12 transition-all duration-200 cursor-pointer" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                    }}/>
                </div>
                <div className="w-full overflow-y-auto overflow-x-hidden hover-scrollbar pr-4 -mr-4 flex-1">
                    <div className="p-6 flex flex-col gap-2">
                        <FolderInput name={"Install location"} clearable={true} value={`${this.props.installSettings.directory}`} folder={true} id={"install_game_path2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={"Location where game is installed. Usually should be set where main game exe is located."}/>
                        <CheckBox enabled={this.props.installSettings.ignore_updates} name={"Skip version update check"} id={"skip_version_updates2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Skip checking for game updates."}/>
                        <CheckBox enabled={this.props.installSettings.skip_hash_check} name={"Skip hash validation"} id={"skip_hash_validation2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Skip validating files during game repair process, this will speed up the repair process significantly."}/>
                        {(window.navigator.platform.includes("Linux") && this.state.gameSwitches.jadeite) ? <CheckBox enabled={this.props.installSettings.use_jadeite} name={"Launch with Jadeite"} id={"tweak_jadeite"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Launch game using Jadeite patch."}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={this.props.installSettings.use_gamemode} name={"Enable Feral Gamemode"} id={"tweak_gamemode"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Launch game using gamemode by FeralInteractive. You need it installed on your system for this to work!"}/> : null}
                        {(this.state.gameSwitches.xxmi) ? <SubMenu name={"XXMI settings"} page={POPUPS.XXMISETTINGS} setOpenPopup={this.props.setOpenPopup}/> : null }
                        {(window.navigator.platform.includes("Linux")) ? <SubMenu name={"MangoHUD settings"} page={POPUPS.MANGOHUDSETTINGS} setOpenPopup={this.props.setOpenPopup}/> : null}
                        {(this.state.gameSwitches.fps_unlocker) ? <SubMenu name={"FPS Unlocker settings"} page={POPUPS.FPSUNLOCKERSETTINGS} setOpenPopup={this.props.setOpenPopup}/> : null }
                        <TextInput name={"Environment variables"} value={this.props.installSettings.env_vars} readOnly={false} id={"install_env_vars"} placeholder={`DXVK_HUD="fps,devinfo";DXVK_LOG=none;`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={`Pass extra variables to Proton.\nExamples:\n- DXVK_HUD=fps;\n-DXVK_HUD=fps,devinfo;PROTON_LOG=1;\n- DXVK_HUD="fps,devinfo";SOMEVAR="/path/to/something";`}/>
                        <TextInput name={"Pre launch command"} value={this.props.installSettings.pre_launch_command} readOnly={false} id={"install_pre_launch_cmd"} placeholder={`${window.navigator.platform.includes("Linux") ? '"/long path/linuxapp"' : "taskmgr.exe"}`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={`Command that will be ran before game launches. You can use quotes around paths if needed.\nAvailable variables:\n- %steamrt% = SteamLinuxRuntime binary (Usage: %steamrt% --verb=waitforexitandrun -- %reaper%)\n- %reaper% = Process reaper binary (Usage: %reaper% SteamLaunch AppId=0 -- %runner%)\n- %runner% = Call proton binary\n- %game_exe% = Points to game executable\n- %runner_dir% = Path of current runner (not a binary you can append any binary from this folder)\n- %prefix% = Path to root of runner prefix location field\n- %install_dir% = Path to game install location field\n- %steamrt_path% = Path to SteamLinuxRuntime folder (you can append other binaries from the folder)`}/>
                        <TextInput name={"Launch command"} value={this.props.installSettings.launch_command} readOnly={false} id={"install_launch_cmd"} placeholder={`${window.navigator.platform.includes("Linux") ? '%runner% [run] "/path long/thing.exe"' : '"/path long/thing.exe"'}`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={`Custom command to launch the game. You can use quotes around paths if needed.\nAvailable variables:\n- %steamrt% = SteamLinuxRuntime binary (Usage: %steamrt% --verb=waitforexitandrun -- %reaper%)\n- %reaper% = Process reaper binary (Usage: %reaper% SteamLaunch AppId=0 -- %runner%)\n- %runner% = Call proton binary\n- %game_exe% = Points to game executable\n- %runner_dir% = Path of current runner (not a binary you can append any binary from this folder)\n- %prefix% = Path to root of runner prefix location field\n- %install_dir% = Path to game install location field\n- %steamrt_path% = Path to SteamLinuxRuntime folder (you can append other binaries from the folder)`}/>
                        <TextInput name={"Launch arguments"} value={this.props.installSettings.launch_args} readOnly={false} id={"install_launch_args"} placeholder={"-dx11 -whatever -thisonetoo"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Additional arguments to pass to the game. Each entry is separated with space."}/>
                        {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_runner_version"} name={"Runner version"} multiple={false} options={this.props.installedRunners} selected={(this.props.installSettings.runner_version === "none" || this.props.installSettings.runner_version === "") ? this.props.installedRunners[0].value : this.props.installSettings.runner_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings} helpText={"Wine/Proton version used by this installation."} setOpenPopup={this.props.setOpenPopup}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner location"} clearable={true} value={`${this.props.installSettings.runner_path}`} folder={true} id={"install_runner_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location of the Wine/Proton runner. Usually points to directory containing "bin" or "files" directory.`}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner prefix location"} clearable={true} value={`${this.props.installSettings.runner_prefix}`} folder={true} id={"install_prefix_path2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location where Wine/Proton prefix is stored. Should point to directory where "system.reg" is stored.`}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? null/*<SelectMenu id={"install_dxvk_version"} name={"DXVK version"} multiple={false} options={this.props.dxvkVersions} selected={(this.props.installSettings.dxvk_version === "none" || this.props.installSettings.dxvk_version === "") ? this.props.dxvkVersions[0].value : this.props.installSettings.dxvk_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings} helpText={"DXVK version used by this installation."} setOpenPopup={this.props.setOpenPopup}/>*/ : null}
                        {(window.navigator.platform.includes("Linux")) ? null/*<FolderInput name={"DXVK location"} clearable={true} value={`${this.props.installSettings.dxvk_path}`} folder={true} id={"install_dxvk_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location from which folder to pull DXVK for this installation. Should point to directory with "x32" and "x64" directories.`}/>*/ : null}
                    </div>
                </div>
                <div className="flex justify-center gap-3 pt-5 mt-4 border-t border-white/10 flex-wrap">
                    <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {runnerVersion: "", manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "install"}).then(() => {});
                    }}><FolderOpenIcon/><span>Game folder</span>
                    </button>
                    {this.props.installSettings.use_xxmi ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {runnerVersion: "", manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "mods"}).then(() => {});
                    }}><FolderOpenIcon/><span>Mods folder</span>
                    </button>: null}
                    {(window.navigator.platform.includes("Linux")) ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {runnerVersion: "", manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "runner"}).then(() => {});
                    }}><FolderOpenIcon/><span>Runner folder</span>
                    </button>: null}
                    {(window.navigator.platform.includes("Linux")) ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {runnerVersion: "", manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "runner_prefix"}).then(() => {});
                    }}><FolderOpenIcon/><span>Prefix folder</span>
                    </button>: null}
                    {this.props.installSettings.shortcut_is_steam ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-sky-600 to-sky-700 hover:from-sky-500 hover:to-sky-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("remove_shortcut", { installId: this.props.installSettings.id, shortcutType: "steam" }).then(() => {});
                    }}><Trash2Icon/><span>Remove from Steam</span>
                    </button> : <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-500 hover:to-blue-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("add_shortcut", { installId: this.props.installSettings.id, shortcutType: "steam" }).then(() => {});
                    }}><SteamIcon className={"w-6 h-6"}/><span>Add to Steam</span>
                    </button>}
                    {this.props.installSettings.shortcut_path !== "" ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-sky-600 to-sky-700 hover:from-sky-500 hover:to-sky-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("remove_shortcut", { installId: this.props.installSettings.id, shortcutType: "desktop" }).then(() => {});
                    }}><Trash2Icon/><span>Remove from Desktop</span>
                    </button> : <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-500 hover:to-blue-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("add_shortcut", { installId: this.props.installSettings.id, shortcutType: "desktop" }).then(() => {});
                    }}><MonitorIcon/><span>Add to Desktop</span>
                    </button>}
                    <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-orange-600 to-orange-700 hover:from-orange-500 hover:to-orange-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        emit("start_game_repair", {install: this.props.installSettings.id, biz: this.props.installSettings.manifest_id, lang: "en-us", region: this.props.installSettings.region_code}).then(() => {});
                    }}><WrenchIcon/><span>Repair</span>
                    </button>
                    <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.INSTALLDELETECONFIRMATION);
                    }}><Trash2Icon/><span>Uninstall</span>
                    </button>
                </div>
            </div>
        )
    }

    componentDidUpdate(prevProps: IProps) {
        // Keep state in sync if the active install changes or prefetched data updates
        if (prevProps.installSettings?.id !== this.props.installSettings?.id ||
            prevProps.prefetchedSwitches !== this.props.prefetchedSwitches ||
            prevProps.prefetchedFps !== this.props.prefetchedFps) {
            this.setState({
                gameSwitches: this.props.prefetchedSwitches || {},
                gameFps: this.props.prefetchedFps || []
            });
        }
    }
}
