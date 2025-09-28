import {FolderOpenIcon, MinusCircleIcon, PlusCircleIcon, Trash2Icon, WrenchIcon, X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import FolderInput from "../../common/FolderInput.tsx";
import CheckBox from "../../common/CheckBox.tsx";
import TextInput from "../../common/TextInput.tsx";
import SelectMenu from "../../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import React from "react";
import {emit} from "@tauri-apps/api/event";
import SubMenu from "../../common/SubMenu.tsx";

interface IProps {
    games: any,
    runnerVersions: any,
    dxvkVersions: any,
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
                        {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={this.props.installSettings.use_gamemode} name={"Launch with Gamemode"} id={"tweak_gamemode"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Launch game using gamemode by FeralInteractive. You need it installed on your system for this to work!"}/> : null}
                        {(this.state.gameSwitches.xxmi) ? <CheckBox enabled={this.props.installSettings.use_xxmi} name={"Inject XXMI"} id={"tweak_xxmi"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Enable and inject XXMI modding tool."}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <SubMenu name={"MangoHUD settings"} page={POPUPS.MANGOHUDSETTINGS} setOpenPopup={this.props.setOpenPopup}/> : null}
                        {(this.state.gameSwitches.fps_unlocker) ? <SubMenu name={"FPS Unlocker settings"} page={POPUPS.FPSUNLOCKERSETTINGS} setOpenPopup={this.props.setOpenPopup}/> : null }
                        <TextInput name={"Environment variables"} value={this.props.installSettings.env_vars} readOnly={false} id={"install_env_vars"} placeholder={`DXVK_HUD="fps,devinfo";DXVK_LOG=none;`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={`Pass extra variables to Wine/Proton. Each entry is divided and list must end with ";"`}/>
                        <TextInput name={"Pre launch command"} value={this.props.installSettings.pre_launch_command} readOnly={false} id={"install_pre_launch_cmd"} placeholder={`${window.navigator.platform.includes("Linux") ? '"/long path/linuxapp"' : "taskmgr.exe"}`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Command that will be ran before game launches. Running stuff under Wine/Proton requires %runner% variable."}/>
                        <TextInput name={"Launch command"} value={this.props.installSettings.launch_command} readOnly={false} id={"install_launch_cmd"} placeholder={`${window.navigator.platform.includes("Linux") ? '%runner% [run] "/path long/thing.exe"' : '"/path long/thing.exe"'}`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Custom command to launch the game. On linux this will run whatever you enter here inside Wine/Proton."}/>
                        <TextInput name={"Launch arguments"} value={this.props.installSettings.launch_args} readOnly={false} id={"install_launch_args"} placeholder={"-dx11 -whatever -thisonetoo"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Additional arguments to pass to the game. Each entry is separated with space."}/>
                        {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_runner_version"} name={"Runner version"} multiple={false} options={this.props.runnerVersions} selected={(this.props.installSettings.runner_version === "none" || this.props.installSettings.runner_version === "") ? this.props.runnerVersions[0].value : this.props.installSettings.runner_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings} helpText={"Wine/Proton version used by this installation."} setOpenPopup={this.props.setOpenPopup}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner location"} clearable={true} value={`${this.props.installSettings.runner_path}`} folder={true} id={"install_runner_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location of the Wine/Proton runner. Usually points to directory containing "bin" or "files" directory.`}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner prefix location"} clearable={true} value={`${this.props.installSettings.runner_prefix}`} folder={true} id={"install_prefix_path2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location where Wine/Proton prefix is stored. Should point to directory where "system.reg" is stored.`}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_dxvk_version"} name={"DXVK version"} multiple={false} options={this.props.dxvkVersions} selected={(this.props.installSettings.dxvk_version === "none" || this.props.installSettings.dxvk_version === "") ? this.props.dxvkVersions[0].value : this.props.installSettings.dxvk_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings} helpText={"DXVK version used by this installation."} setOpenPopup={this.props.setOpenPopup}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"DXVK location"} clearable={true} value={`${this.props.installSettings.dxvk_path}`} folder={true} id={"install_dxvk_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location from which folder to pull DXVK for this installation. Should point to directory with "x32" and "x64" directories.`}/> : null}
                    </div>
                </div>
                <div className="flex justify-center gap-3 pt-5 mt-4 border-t border-white/10 flex-wrap">
                    {this.props.installSettings.shortcut_path === "" && (
                        <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-500 hover:to-blue-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                            this.props.setOpenPopup(POPUPS.NONE);
                            // @ts-ignore
                            document.getElementById(this.props.installSettings.id).focus();
                            invoke("add_shortcut", {installId: this.props.installSettings.id, shortcutType: "desktop"}).then(() => {});
                        }}><PlusCircleIcon/><span>Create shortcut</span>
                        </button>
                    )}
                    {this.props.installSettings.shortcut_path !== "" && (
                        <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                                this.props.setOpenPopup(POPUPS.NONE);
                                // @ts-ignore
                                document.getElementById(this.props.installSettings.id).focus();
                                invoke("remove_shortcut", {installId: this.props.installSettings.id, shortcutType: "desktop"}).then(() => {});
                            }}><MinusCircleIcon/><span>Delete shortcut</span>
                        </button>
                    )}
                    <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "install"}).then(() => {});
                    }}><FolderOpenIcon/><span>Open game folder</span>
                    </button>
                    {this.props.installSettings.use_xxmi ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "mods"}).then(() => {});
                    }}><FolderOpenIcon/><span>Open mods folder</span>
                    </button>: null}
                    {(window.navigator.platform.includes("Linux")) ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "runner"}).then(() => {});
                    }}><FolderOpenIcon/><span>Open runner folder</span>
                    </button>: null}
                    {(window.navigator.platform.includes("Linux")) ? <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-purple-600 to-purple-700 hover:from-purple-500 hover:to-purple-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "runner_prefix"}).then(() => {});
                    }}><FolderOpenIcon/><span>Open prefix folder</span>
                    </button>: null}
                    <button className="flex flex-row gap-3 items-center py-3 px-6 bg-gradient-to-r from-orange-600 to-orange-700 hover:from-orange-500 hover:to-orange-600 rounded-xl transition-all duration-200 transform hover:scale-105 font-semibold text-white" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        emit("start_game_repair", {install: this.props.installSettings.id, biz: this.props.installSettings.manifest_id, lang: "en-us"}).then(() => {});
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
