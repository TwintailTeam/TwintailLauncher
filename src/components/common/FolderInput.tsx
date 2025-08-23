import React, { createRef } from 'react'
import ReactDOM from 'react-dom'
import {open} from "@tauri-apps/plugin-dialog"
import TextInputPart from "./TextInputPart.tsx";
import {invoke} from "@tauri-apps/api/core";
import {POPUPS} from "../popups/POPUPS.ts";
import HelpTooltip from "./HelpTooltip.tsx";

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
    biz?: string,
    version?: () => string,
    lang?: () => string,
    fetchDownloadSizes?: (biz: any, version: any, lang: any, path: any, callback: (data: any) => void) => void,
    helpText: string,
    skipGameDownload?: boolean
}

interface IState {
    value: string
    placeholder: string
    folder: boolean
    showTooltip: boolean
    tooltipPosition: { top: number, left: number }
}

export default class FolderInput extends React.Component<IProps, IState> {
    private containerRef = createRef<HTMLDivElement>();

    constructor(props: IProps) {
        super(props)

        this.state = {
            value: props.value || '',
            placeholder: this.props.placeholder || 'Select file or folder...',
            folder: this.props.folder || false,
            showTooltip: false,
            tooltipPosition: { top: 0, left: 0 }
        }

        this.handleIconClick = this.handleIconClick.bind(this)
        this.handleMouseEnter = this.handleMouseEnter.bind(this)
        this.handleMouseLeave = this.handleMouseLeave.bind(this)
    }

    static getDerivedStateFromProps(props: IProps, state: IState) {
        const newState = {...state}

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
            case 'default_runner_path': {
                if (this.props.fetchSettings !== undefined) {
                    invoke("update_settings_default_runner_path", {path: path}).then(() => {});
                    this.props.fetchSettings();
                }
            }
            break;
            case 'default_dxvk_path': {
                if (this.props.fetchSettings !== undefined) {
                    invoke("update_settings_default_dxvk_path", {path: path}).then(() => {});
                    this.props.fetchSettings();
                }
            }
            break;
            case "install_game_path": {
                if (this.props.fetchDownloadSizes !== undefined && this.props.version !== undefined && this.props.lang !== undefined) {
                    this.props.fetchDownloadSizes(this.props.biz, this.props.version(), this.props.lang(), path, (disk) => {
                        // @ts-ignore
                        let btn = document.getElementById("game_dl_btn");
                        // @ts-ignore
                        let freedisk = document.getElementById("game_disk_free");

                        // Skip space validation if existing installation is selected
                        if (this.props.skipGameDownload || disk.game_decompressed_size_raw <= disk.free_disk_space_raw) {
                            // @ts-ignore
                            btn.removeAttribute("disabled");
                            // @ts-ignore
                            freedisk.classList.remove("text-red-600");
                            // @ts-ignore
                            freedisk.classList.add("text-white");
                            // @ts-ignore
                            freedisk.classList.remove("font-bold");
                        } else {
                            // @ts-ignore
                            btn.setAttribute("disabled", "");
                            // @ts-ignore
                            freedisk.classList.add("text-red-600");
                            // @ts-ignore
                            freedisk.classList.remove("text-white");
                            // @ts-ignore
                            freedisk.classList.add("font-bold");
                        }
                    });
                }
            }
            break;
            case "install_game_path2": {
                if (this.props.fetchInstallSettings !== undefined) {
                    invoke("update_install_game_path", {path: path, id: this.props.install}).then(() => {});
                    this.props.fetchInstallSettings(this.props.install as string);

                    if (this.props.setOpenPopup !== undefined) {
                        this.props.setOpenPopup(POPUPS.NONE);
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
                    }
                }
            }
            break;
        }
    }

    handleMouseEnter() {
        if (!this.state.value) return;

        if (this.containerRef.current) {
            const rect = this.containerRef.current.getBoundingClientRect();
            const windowWidth = window.innerWidth;

            const contentLength = this.state.value.length;
            const estimatedCharWidth = 7;
            const minTooltipWidth = 120;
            const calculatedWidth = Math.max(minTooltipWidth, Math.min(320, contentLength * estimatedCharWidth));
            const isRightSideOverflow = rect.right + calculatedWidth + 8 > windowWidth;

            this.setState({ showTooltip: true, tooltipPosition: {top: rect.top, left: isRightSideOverflow ? Math.max(8, rect.left - calculatedWidth - 8) : rect.right + 8} });
        }
    }

    handleMouseLeave() {
        this.setState({ showTooltip: false });
    }

    render() {
        return (
            <div className="flex w-full items-center gap-4 max-sm:flex-col max-sm:items-stretch">
                <span className="text-white text-sm flex items-center gap-1 w-56 shrink-0 max-sm:w-full">{this.props.name}
                    <HelpTooltip text={this.props.helpText}/>
                </span>
                <div className="overflow-ellipsis inline-flex flex-row items-center justify-end relative ml-auto w-[320px]" ref={this.containerRef} onMouseEnter={this.handleMouseEnter} onMouseLeave={this.handleMouseLeave}>
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
                    {this.state.showTooltip ? ReactDOM.createPortal(
                        <div className="whitespace-pre-wrap break-words bg-black/75 text-white text-xs rounded-lg p-2 max-w-md overflow-auto fixed z-30" style={{top: `${this.state.tooltipPosition.top}px`, left: `${this.state.tooltipPosition.left}px`, maxWidth: '320px'}}>
                            {this.state.value}
                        </div>, document.body) : null}
                </div>
            </div>
        )
    }
}