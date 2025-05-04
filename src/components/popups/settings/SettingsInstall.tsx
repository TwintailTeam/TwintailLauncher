import {Trash2Icon, WrenchIcon, X} from "lucide-react";
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
            <div className="rounded-lg h-full w-3/4 flex flex-col p-4 gap-8 overflow-scroll">
                <div className="flex flex-row items-center justify-between">
                    <h1 className="text-white font-bold text-2xl">{this.props.installSettings.name}</h1>
                    <X className="text-white cursor-pointer" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                    }}/>
                </div>
                <div className="flex flex-row-reverse">
                    <button className="flex flex-row gap-1 items-center p-2 bg-red-600 rounded-lg" onClick={() => {
                        this.props.setOpenPopup(POPUPS.INSTALLDELETECONFIRMATION);
                    }}><Trash2Icon/><span className="font-semibold translate-y-px">Uninstall</span>
                    </button>
                    <button className="flex flex-row gap-1 me-2 items-center p-2 bg-blue-600 rounded-lg" onClick={() => {
                        this.props.setOpenPopup(POPUPS.NONE);
                        // @ts-ignore
                        document.getElementById(this.props.installSettings.id).focus();
                        emit("start_game_repair", {install: this.props.installSettings.id, biz: this.props.installSettings.manifest_id}).then(() => {});
                    }}><WrenchIcon/>
                        <span className="font-semibold translate-y-px">Repair install</span>
                    </button>
                </div>
                <div className={`w-full transition-all duration-500 overflow-hidden bg-neutral-700 gap-4 flex flex-col items-center justify-between px-4 p-4 rounded-b-lg rounded-t-lg`} style={{maxHeight: (20 * 64) + "px"}}>
                    <FolderInput name={"Install location"} clearable={true} value={`${this.props.installSettings.directory}`} folder={true} id={"install_game_path2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup}/>
                    <CheckBox enabled={this.props.installSettings.ignore_updates} name={"Skip version update check"} id={"skip_version_updates2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/>
                    <CheckBox enabled={this.props.installSettings.skip_hash_check} name={"Skip hash validation"} id={"skip_hash_validation2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/>
                    {(window.navigator.platform.includes("Linux") && this.state.gameSwitches.jadeite) ? <CheckBox enabled={this.props.installSettings.use_jadeite} name={"Inject Jadeite"} id={"tweak_jadeite"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/> : null}
                    {(this.state.gameSwitches.xxmi) ? <CheckBox enabled={this.props.installSettings.use_xxmi} name={"Inject XXMI"} id={"tweak_xxmi"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/> : null}
                    {(this.state.gameSwitches.fps_unlocker) ? <CheckBox enabled={this.props.installSettings.use_fps_unlock} name={"Inject FPS Unlocker"} id={"tweak_fps_unlock"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/> : null}
                    {(this.state.gameSwitches.fps_unlocker) ? <SelectMenu id={"install_fps_value"} name={"FPS value"} multiple={false} options={this.state.gameFps} selected={`${this.props.installSettings.fps_value}`} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/> : null}
                    <TextInput name={"Environment variables"} value={this.props.installSettings.env_vars} readOnly={false} id={"install_env_vars"} placeholder={"DXVK_HUD=fps;DXVK_LOG=none;"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/>
                    <TextInput name={"Pre launch command"} value={this.props.installSettings.pre_launch_command} readOnly={false} id={"install_pre_launch_cmd"} placeholder={"%command%"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/>
                    <TextInput name={"Launch command"} value={this.props.installSettings.launch_command} readOnly={false} id={"install_launch_cmd"} placeholder={"%command%"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/>
                    <TextInput name={"Launch arguments"} value={this.props.installSettings.launch_args} readOnly={false} id={"install_launch_args"} placeholder={"-dx11 -whatever -thisonetoo"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id}/>
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_runner_version"} name={"Runner version"} multiple={false} options={this.props.runnerVersions} selected={(this.props.installSettings.runner_version === "none" || this.props.installSettings.runner_version === "") ? this.props.runnerVersions[0].value : this.props.installSettings.runner_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner path"} clearable={true} value={`${this.props.installSettings.runner_path}`} folder={true} id={"install_runner_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"Runner prefix path"} clearable={true} value={`${this.props.installSettings.runner_prefix}`} folder={true} id={"install_prefix_path2"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <SelectMenu id={"install_dxvk_version"} name={"DXVK version"} multiple={false} options={this.props.dxvkVersions} selected={(this.props.installSettings.dxvk_version === "none" || this.props.installSettings.dxvk_version === "") ? this.props.dxvkVersions[0].value : this.props.installSettings.dxvk_version} install={this.props.installSettings.id} fetchInstallSettings={this.props.fetchInstallSettings}/> : null}
                    {(window.navigator.platform.includes("Linux")) ? <FolderInput name={"DXVK path"} clearable={true} value={`${this.props.installSettings.dxvk_path}`} folder={true} id={"install_dxvk_path"} fetchInstallSettings={this.props.fetchInstallSettings} install={this.props.installSettings.id} setOpenPopup={this.props.setOpenPopup}/> : null}
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