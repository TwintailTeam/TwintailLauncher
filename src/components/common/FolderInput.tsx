import React from 'react'
import {open} from "@tauri-apps/plugin-dialog"
import TextInputPart from "./TextInputPart.tsx";
import {invoke} from "@tauri-apps/api/core";
import {POPUPS} from "../popups/POPUPS.ts";
import moveTracker from "../../utils.ts";

// Thanks Cultivation FUCK NO im not making this myself
// Yes I can not be assed to make inputs, I stole Cultivation's and modified them to fit the theme

interface IProps {
    value?: string
    clearable?: boolean
    onChange?: (value: string) => void
    extensions?: string[]
    readonly?: boolean
    placeholder?: string
    folder?: boolean
    customClearBehaviour?: () => void
    openFolder?: string,
    name?: string,
    id?: string,
    install?: string,
    fetchSettings?: () => void,
    fetchInstallSettings?: (id: string) => void,
    setOpenPopup?: (popup: POPUPS) => void,
}

interface IState {
    value: string
    placeholder: string
    folder: boolean
}

export default class FolderInput extends React.Component<IProps, IState> {
    constructor(props: IProps) {
        super(props)

        this.state = {
            value: props.value || '',
            placeholder: this.props.placeholder || 'Select file or folder...',
            folder: this.props.folder || false,
        }

        this.handleIconClick = this.handleIconClick.bind(this)
    }

    static getDerivedStateFromProps(props: IProps, state: IState) {
        const newState = state

        if (props.value && state.value === '') {
            newState.value = props.value || ''
        }

        if (props.placeholder) {
            newState.placeholder = props.placeholder
        }

        return newState
    }

    async componentDidMount() {
        if (!this.props.placeholder) {
            this.setState({placeholder: this.props.folder ? "Select folder..." : 'Select file(s)...'})
        }
    }

    async handleIconClick() {
        let path;

        if (this.state.folder) {
            path = await open({directory: true})
        } else {
            path = await open({filters: [{ name: 'Files', extensions: this.props.extensions || ['*'] }], defaultPath: this.props.openFolder})
        }

        if (Array.isArray(path)) path = path[0]
        if (!path) return

        this.setState({value: path})
        this.updateSetting(path);

        if (this.props.onChange) this.props.onChange(path)
    }

    updateSetting(path: string) {
        switch (this.props.id) {
            case 'default_game_path': {
                if (this.props.fetchSettings !== undefined) {
                    invoke("update_settings_default_game_path", {path: path}).then(() => {});
                    this.props.fetchSettings();
                }
            }
            break;
            case 'default_xxmi_path': {
                if (this.props.fetchSettings !== undefined) {
                    invoke("update_settings_default_xxmi_path", {path: path}).then(() => {});
                    this.props.fetchSettings();
                }
            }
            break;
            case 'default_fps_unlock_path': {
                if (this.props.fetchSettings !== undefined) {
                    invoke("update_settings_default_fps_unlock_path", {path: path}).then(() => {});
                    this.props.fetchSettings();
                }
            }
            break;
            case 'default_jadeite_path': {
                if (this.props.fetchSettings !== undefined) {
                    invoke("update_settings_default_jadeite_path", {path: path}).then(() => {});
                    this.props.fetchSettings();
                }
            }
            break;
            case 'default_prefix_path': {
                if (this.props.fetchSettings !== undefined) {
                    invoke("update_settings_default_prefix_path", {path: path}).then(() => {});
                    this.props.fetchSettings();
                }
            }
            break;
            case "install_game_path": {
            }
            break;
            case "install_game_path2": {
                if (this.props.fetchInstallSettings !== undefined) {
                    invoke("update_install_game_path", {path: path, id: this.props.install}).then(() => {});
                    this.props.fetchInstallSettings(this.props.install as string);

                    if (this.props.setOpenPopup !== undefined) {
                        this.props.setOpenPopup(POPUPS.NONE);
                        moveTracker();
                    }
                }
            }
            break;
            case "install_runner_path": {
                if (this.props.fetchInstallSettings !== undefined) {
                    invoke("update_install_runner_path", {path: path, id: this.props.install}).then(() => {});
                    this.props.fetchInstallSettings(this.props.install as string);

                    if (this.props.setOpenPopup !== undefined) {
                        this.props.setOpenPopup(POPUPS.NONE);
                        moveTracker();
                    }
                }
            }
            break;
            case "install_dxvk_path": {
                if (this.props.fetchInstallSettings !== undefined) {
                    invoke("update_install_dxvk_path", {path: path, id: this.props.install}).then(() => {});
                    this.props.fetchInstallSettings(this.props.install as string);

                    if (this.props.setOpenPopup !== undefined) {
                        this.props.setOpenPopup(POPUPS.NONE);
                        moveTracker();
                    }
                }
            }
            break;
            case "install_prefix_path": {
            }
            break;
            case "install_prefix_path2": {
                if (this.props.fetchInstallSettings !== undefined) {
                    invoke("update_install_prefix_path", {path: path, id: this.props.install}).then(() => {});
                    this.props.fetchInstallSettings(this.props.install as string);

                    if (this.props.setOpenPopup !== undefined) {
                        this.props.setOpenPopup(POPUPS.NONE);
                        moveTracker();
                    }
                }
            }
            break;
        }
    }

    render() {
        return (
            <div className="flex flex-row items-center justify-between w-full h-6">
                <span className="text-white text-sm">{this.props.name}</span>
                <div className="overflow-ellipsis inline-flex flex-row items-center justify-center">
                    <TextInputPart value={this.state.value}
                                   id={this.props.id}
                                   isPicker={true}
                                   onClick={this.handleIconClick}
                                   placeholder={this.state.placeholder}
                                   clearable={this.props.clearable !== undefined ? this.props.clearable : true}
                                   readOnly={this.props.readonly !== undefined ? this.props.readonly : true}
                                   onChange={(text: string) => {
                                   this.setState({ value: text })
                                   if (this.props.onChange) this.props.onChange(text)
                                   this.forceUpdate();
                                   this.updateSetting(text);
                               }} customClearBehaviour={this.props.customClearBehaviour}/>
                </div>
            </div>
        )
    }
}
