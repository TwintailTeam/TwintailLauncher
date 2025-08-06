import {FolderOpenIcon, Trash2Icon, WrenchIcon, X} from "lucide-react";
import {POPUPS} from "../POPUPS.ts";
import FolderInput from "../../common/FolderInput.tsx";
import CheckBox from "../../common/CheckBox.tsx";
import TextInput from "../../common/TextInput.tsx";
import SelectMenu from "../../common/SelectMenu.tsx";
import {invoke} from "@tauri-apps/api/core";
import React from "react";
import {emit} from "@tauri-apps/api/event";

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
    fetchInstallSettings: (id: string) => void
}

interface IState {
    gameSwitches: any,
    gameFps: any
}

export default class SettingsInstall extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props);
        this.state = {
            gameSwitches: {},
            gameFps: []
        }
    }
    
    render() {
        return (
            <div className="rounded-lg h-full w-1/2 bg-black/50 fixed-backdrop-blur-md border border-white/20 flex flex-col p-6 gap-6 overflow-scroll scrollbar-none">
                <div className="flex flex-row items-center justify-between">
                    <h1 className="text-white font-bold text-2xl">{this.props.installSettings.name}</h1>
                    <X className="text-white hover:text-gray-400 cursor-pointer" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                    }}/>
                </div>
                <div className="flex flex-row-reverse gap-2">
                    <button className="flex flex-row gap-2 items-center py-2 px-4 bg-red-600 hover:bg-red-700 rounded-lg" onClick={() => {
                        this.props.setOpenPopup(POPUPS.INSTALLDELETECONFIRMATION);
                    }}><Trash2Icon/><span className="font-semibold">Uninstall</span>
                    </button>
                    <button className="flex flex-row gap-2 me-2 items-center py-2 px-4 bg-orange-600 hover:bg-orange-700 rounded-lg" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        emit("start_game_repair", {install: this.props.installSettings.id, biz: this.props.installSettings.manifest_id, lang: "en-us"}).then(() => {});
                    }}><WrenchIcon/><span className="font-semibold">Repair install</span>
                    </button>
                    <button className="flex flex-row gap-2 me-2 items-center py-2 px-4 bg-purple-600 hover:bg-purple-700 rounded-lg" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "install"}).then(() => {});
                    }}><FolderOpenIcon/><span className="font-semibold">Open game folder</span>
                    </button>
                    {this.props.installSettings.use_xxmi ? <button className="flex flex-row gap-2 me-2 items-center py-2 px-4 bg-purple-600 hover:bg-purple-700 rounded-lg" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        invoke("open_folder", {manifestId: this.props.installSettings.manifest_id, installId: this.props.installSettings.id, pathType: "mods"}).then(() => {});
                    }}><FolderOpenIcon/><span className="font-semibold">Open mods folder</span>
                    </button>: null}
                </div>
                <div className="w-full overflow-y-auto overflow-scroll scrollbar-none pr-4 -mr-4">
                    <div className="bg-black/20 border border-white/10 rounded-lg p-4 flex flex-col gap-4">
                        <FolderInput name={"Install location"} clearable={true} value={`${this.props.installSettings.directory}`} folder={true} id={"install_game_path2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={"Location where game is installed. Usually should be set where main game exe is located."}/>
                        <CheckBox enabled={this.props.installSettings.ignore_updates} name={"Skip version update check"} id={"skip_version_updates2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Skip checking for game updates."}/>
                        <CheckBox enabled={this.props.installSettings.skip_hash_check} name={"Skip hash validation"} id={"skip_hash_validation2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Skip validating files during game repair process, this will speed up the repair process significantly."}/>
                        {(window.navigator.platform.includes("Linux") && this.state.gameSwitches.jadeite) ? <CheckBox enabled={this.props.installSettings.use_jadeite} name={"Launch with Jadeite"} id={"tweak_jadeite"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Launch game using Jadeite patch."}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={this.props.installSettings.use_gamemode} name={"Launch with Gamemode"} id={"tweak_gamemode"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Launch game using gamemode by FeralInteractive. You need it installed on your system for this to work!"}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <CheckBox enabled={this.props.installSettings.use_mangohud} name={"Show MangoHUD"} id={"tweak_mangohud"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Enable MangoHUD monitor. You need it installed on your system for this to work!"}/> : null}
                        {(this.state.gameSwitches.xxmi) ? <CheckBox enabled={this.props.installSettings.use_xxmi} name={"Inject XXMI"} id={"tweak_xxmi"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Enable and inject XXMI modding tool."}/> : null}
                        {(this.state.gameSwitches.fps_unlocker) ? <CheckBox enabled={this.props.installSettings.use_fps_unlock} name={"Inject FPS Unlocker"} id={"tweak_fps_unlock"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Load and inject fps unlocking into the game. Pick FPS in the menu bellow."}/> : null}
                        {(this.state.gameSwitches.fps_unlocker) ? <SelectMenu id={"install_fps_value"} name={"FPS value"} multiple={false} options={this.state.gameFps} selected={`${this.props.installSettings.fps_value}`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Target FPS to unlock game to."} setOpenPopup={this.props.setOpenPopup}/> : null}
                        <TextInput name={"Environment variables"} value={this.props.installSettings.env_vars} readOnly={false} id={"install_env_vars"} placeholder={"DXVK_HUD=fps;DXVK_LOG=none;"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={`Pass extra variables to Wine/Proton. Each entry is divided and list must end with ";"`}/>
                        <TextInput name={"Pre launch command"} value={this.props.installSettings.pre_launch_command} readOnly={false} id={"install_pre_launch_cmd"} placeholder={"%command%"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Command that will be ran before game launches. Running stuff under Wine/Proton requires you to call runner binary."}/>
                        <TextInput name={"Launch command"} value={this.props.installSettings.launch_command} readOnly={false} id={"install_launch_cmd"} placeholder={"%command%"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Custom command to launch the game. On linux this will run whatever you enter here inside Wine/Proton."}/>
                        <TextInput name={"Launch arguments"} value={this.props.installSettings.launch_args} readOnly={false} id={"install_launch_args"} placeholder={"-dx11 -whatever -thisonetoo"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} helpText={"Additional arguments to pass to the game. Each entry is separated with space."}/>
                        {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_runner_version"} name={"Runner version"} multiple={false} options={this.props.runnerVersions} selected={(this.props.installSettings.runner_version === "none" || this.props.installSettings.runner_version === "") ? this.props.runnerVersions[0].value : this.props.installSettings.runner_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings} helpText={"Wine/Proton version used by this installation."} setOpenPopup={this.props.setOpenPopup}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner path"} clearable={true} value={`${this.props.installSettings.runner_path}`} folder={true} id={"install_runner_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location of the Wine/Proton runner. Usually points to directory containing "bin" or "files" directory.`}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner prefix path"} clearable={true} value={`${this.props.installSettings.runner_prefix}`} folder={true} id={"install_prefix_path2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location where Wine/Proton prefix is stored. Should point to directory where "system.reg" is stored.`}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_dxvk_version"} name={"DXVK version"} multiple={false} options={this.props.dxvkVersions} selected={(this.props.installSettings.dxvk_version === "none" || this.props.installSettings.dxvk_version === "") ? this.props.dxvkVersions[0].value : this.props.installSettings.dxvk_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings} helpText={"DXVK version used by this installation."} setOpenPopup={this.props.setOpenPopup}/> : null}
                        {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"DXVK path"} clearable={true} value={`${this.props.installSettings.dxvk_path}`} folder={true} id={"install_dxvk_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup} helpText={`Location from which folder to pull DXVK for this installation. Should point to directory with "x32" and "x64" directories.`}/> : null}
                    </div>
                </div>
            </div>
        )
    }

    async componentDidMount() {
        let r = await invoke("get_game_manifest_by_manifest_id", {id: this.props.installSettings.manifest_id});
        if (r == null) {
            console.error("Failed to fetch game info for installation settings!");
            this.setState({gameSwitches: {xxmi: true, fps_unlocker: true, jadeite: true}, gameFps: [{value: "60", name: "60"}]});
        } else {
            let rr = JSON.parse(r as string);

            let fpslist: any = [];
            rr.extra.fps_unlock_options.forEach((e: any) => fpslist.push({value: `${e}`, name: `${e}`}));
            this.setState({gameSwitches: rr.extra.switches, gameFps: fpslist});
        }
    }
}
