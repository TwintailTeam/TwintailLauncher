import "./App.css";
import React from "react";
import RepoManager from "./components/popups/repomanager/RepoManager.tsx";
import {POPUPS} from "./components/popups/POPUPS.ts";
import AddRepo from "./components/popups/repomanager/AddRepo.tsx";
import SidebarIconManifest from "./components/SidebarIconManifest.tsx";
import {invoke} from "@tauri-apps/api/core";
import SidebarRepos from "./components/SidebarRepos.tsx";
import {ChevronDown, DownloadIcon, Settings} from "lucide-react";
import SidebarSettings from "./components/SidebarSettings.tsx";
import SettingsGlobal from "./components/popups/settings/SettingsGlobal.tsx";
import SidebarIconInstall from "./components/SidebarIconInstall.tsx";
import DownloadGame from "./components/popups/DownloadGame.tsx";
import SettingsInstall from "./components/popups/settings/SettingsInstall.tsx";
import ProgressBar from "./components/common/ProgressBar.tsx";
import InstallDeleteConfirm from "./components/popups/settings/InstallDeleteConfirm.tsx";
import GameButton from "./components/GameButton.tsx";
import PreloadButton from "./components/common/PreloadButton.tsx";
import CollapsableTooltip from "./components/common/CollapsableTooltip.tsx";
import {emit, listen} from "@tauri-apps/api/event";

const EVENTS = [
    'download_progress',
    'download_complete',
    'update_progress',
    'update_complete',
    'repair_progress',
    'repair_complete',
    'preload_progress',
    'preload_complete',
    'move_progress',
    'move_complete'
];

export default class App extends React.Component<any, any> {
    unlistenFns: (() => void)[] = [];
    constructor(props: any) {
        super(props);

        this.setCurrentGame = this.setCurrentGame.bind(this);
        this.setDisplayName = this.setDisplayName.bind(this);
        this.setBackground = this.setBackground.bind(this);
        this.setGameIcon = this.setGameIcon.bind(this);
        this.setReposList = this.setReposList.bind(this);
        this.setOpenPopup = this.setOpenPopup.bind(this);
        this.setCurrentInstall = this.setCurrentInstall.bind(this);

        this.pushGames = this.pushGames.bind(this);
        this.pushGamesInfo = this.pushGamesInfo.bind(this);
        this.pushInstalls = this.pushInstalls.bind(this);
        this.fetchSettings = this.fetchSettings.bind(this);
        this.fetchRepositories = this.fetchRepositories.bind(this);
        this.fetchInstallSettings = this.fetchInstallSettings.bind(this);
        this.fetchDownloadSizes = this.fetchDownloadSizes.bind(this);
        this.fetchGameVersions = this.fetchGameVersions.bind(this);
        this.fetchCompatibilityVersions = this.fetchCompatibilityVersions.bind(this);
        this.refreshDownloadButtonInfo = this.refreshDownloadButtonInfo.bind(this);

        this.state = {
            openPopup: POPUPS.NONE,
            currentGame: "",
            currentInstall: "",
            displayName: "",
            gameBackground: "",
            gameIcon: "",
            gamesinfo: [],
            reposList: [],
            installs: [],
            globalSettings: {},
            preloadAvailable: false,
            gameVersions: [],
            installSettings: {},
            runnerVersions: [],
            dxvkVersions: [],
            downloadSizes: {},
            downloadDir: "",
            downloadVersion: "",
            gameManifest: {},
            disableRun: false,
            disableUpdate: false,
            disableDownload: false,
            disableInstallEdit: false,
            hideProgressBar: true,
            progressName: "?",
            progressVal: 0,
            progressPercent: "0%",
        }
    }

    render() {
        let buttonType = this.determineButtonType();
        return (
            <main className="w-full h-screen flex flex-row bg-transparent">
                <img className="w-full h-screen object-cover object-center absolute top-0 left-0 right-0 bottom-0 -z-10" alt={"?"} src={this.state.gameBackground} loading="lazy" decoding="async" srcSet={undefined}/>
                <div className="h-full w-16 p-2 bg-black/50 flex flex-col items-center fixed-backdrop-blur-md justify-between">
                    <div className="flex flex-col pb-2 gap-2 flex-shrink overflow-scroll scrollbar-none">
                        <CollapsableTooltip text={this.state.globalSettings.hide_manifests ? "Show manifests" : "Hide manifests"} icon={<ChevronDown color="white" onClick={() => {
                            invoke("update_settings_manifests_hide", {enabled: !this.state.globalSettings.hide_manifests}).then(() => {});
                            this.setState((prevState: any) => ({
                                globalSettings: {
                                    ...prevState.globalSettings,
                                    hide_manifests: !prevState.globalSettings.hide_manifests
                                }
                            }))
                        }} className={`h-5 w-14 align-middle border-transparent transition cursor-pointer duration-500 pb-0 mb-0 ${this.state.globalSettings.hide_manifests ? "rotate-00" : "rotate-180"}`}/>}/>
                        <div className={"w-full transition-all duration-500 overflow-scroll scrollbar-none gap-3 flex flex-col flex-shrink items-center"} style={{maxHeight: this.state.globalSettings.hide_manifests ? "0px" : (this.state.gamesinfo.length * 120) + "px"}}>
                            {this.state.currentGame != "" && this.state.gamesinfo.map((game: { manifest_enabled: boolean; assets: any; filename: string; icon: string; display_name: string; biz: string; }) => {
                                return (
                                    <SidebarIconManifest key={game.biz} popup={this.state.openPopup} icon={game.assets.game_icon} background={game.assets.game_background} name={game.display_name} enabled={game.manifest_enabled} id={game.biz} setCurrentGame={this.setCurrentGame} setOpenPopup={this.setOpenPopup} setDisplayName={this.setDisplayName} setBackground={this.setBackground} setCurrentInstall={this.setCurrentInstall} setGameIcon={this.setGameIcon} />
                                )
                            })}
                        </div>
                        <hr className="text-white/20 bg-white/20 p-0" style={{borderColor: "rgb(255 255 255 / 0.2)"}}/>
                        <div className={"gap-3 flex flex-col items-center scrollbar-none overflow-scroll"}>
                            {this.state.installs.map((install: { game_background: string; game_icon: string; manifest_id: string; name: string; id: string; }) => {
                                return (
                                    <SidebarIconInstall key={install.id} popup={this.state.openPopup} icon={install.game_icon} background={install.game_background} name={install.name} enabled={true} id={install.id} setCurrentInstall={this.setCurrentInstall} setOpenPopup={this.setOpenPopup} setDisplayName={this.setDisplayName} setBackground={this.setBackground} setGameIcon={this.setGameIcon} />
                                )
                            })}
                        </div>
                    </div>
                    <div className="flex flex-col gap-4 flex-shrink overflow-scroll scrollbar-none">
                        <hr className="text-white/20 bg-white/20 p-0" style={{borderColor: "rgb(255 255 255 / 0.2)"}}/>
                        <SidebarRepos popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                        <SidebarSettings popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                    </div>
                </div>
                <div className="flex flex-row absolute bottom-8 right-16 gap-4">
                    {(this.state.currentInstall !== "" && this.state.preloadAvailable) ? (<button onClick={() => {
                        emit("start_game_preload", {install: this.state.currentInstall, biz: "", lang: ""}).then(() => {});
                    }}><PreloadButton text={"Predownload update"} icon={<DownloadIcon className="text-green-600 hover:text-green-700 w-8 h-8"/>}/>
                    </button>): null}
                    {(this.state.currentInstall !== "") ? <button id={`install_settings_btn`} disabled={this.state.disableInstallEdit} onClick={() => {
                        // Delay for very unnoticeable time to prevent popup opening before state is synced
                        setTimeout(() => {this.setState({openPopup: POPUPS.INSTALLSETTINGS});}, 20);
                    }}><Settings className="text-white hover:text-white/55 w-8 h-8"/>
                    </button> : null}
                    <GameButton disableDownload={this.state.disableDownload} disableRun={this.state.disableRun} disableUpdate={this.state.disableUpdate} currentInstall={this.state.currentInstall} globalSettings={this.state.globalSettings} refreshDownloadButtonInfo={this.refreshDownloadButtonInfo} buttonType={buttonType}/>
                </div>
                <div className={`absolute items-center justify-center bottom-0 left-96 right-72 p-8 z-20 [top:82%] ${this.state.hideProgressBar ? "hidden" : ""}`} id={"progress_bar"}>
                        <h4 className={"pl-4 pb-1 text-white text-stroke inline"} id={"progress_name"}>{this.state.progressName}</h4><h4 className={"pl-4 pb-1 text-white text-stroke inline"}>(<span id={"progress_percent"}>{this.state.progressPercent}</span>)</h4>
                        <ProgressBar id={"progress_value"} progress={this.state.progressVal} className={"transition-all duration-500 ease-out"}/>
                </div>
                <div className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-20 ${this.state.openPopup == POPUPS.NONE ? "hidden" : "flex fixed-backdrop-blur-lg bg-white/10"}`}>
                    {this.state.openPopup == POPUPS.REPOMANAGER && <RepoManager repos={this.state.reposList} setOpenPopup={this.setOpenPopup} fetchRepositories={this.fetchRepositories}/>}
                    {this.state.openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={this.setOpenPopup}/>}
                    {this.state.openPopup == POPUPS.SETTINGS && <SettingsGlobal fetchSettings={this.fetchSettings} settings={this.state.globalSettings} setOpenPopup={this.setOpenPopup} />}
                    {this.state.openPopup == POPUPS.DOWNLOADGAME && <DownloadGame fetchDownloadSizes={this.fetchDownloadSizes} disk={this.state.downloadSizes} runnerVersions={this.state.runnerVersions} dxvkVersions={this.state.dxvkVersions} versions={this.state.gameVersions} icon={this.state.gameIcon} background={this.state.gameBackground} biz={this.state.currentGame} displayName={this.state.displayName} settings={this.state.globalSettings} setOpenPopup={this.setOpenPopup} pushInstalls={this.pushInstalls} setBackground={this.setBackground} setCurrentInstall={this.setCurrentInstall}/>}
                    {this.state.openPopup == POPUPS.INSTALLSETTINGS && <SettingsInstall games={this.state.gamesinfo} runnerVersions={this.state.runnerVersions} dxvkVersions={this.state.dxvkVersions} installSettings={this.state.installSettings} setOpenPopup={this.setOpenPopup} pushInstalls={this.pushInstalls} setCurrentInstall={this.setCurrentInstall} setCurrentGame={this.setCurrentGame} setBackground={this.setBackground} fetchInstallSettings={this.fetchInstallSettings}/>}
                    {this.state.openPopup == POPUPS.INSTALLDELETECONFIRMATION && <InstallDeleteConfirm installs={this.state.installs} games={this.state.gamesinfo} install={this.state.installSettings} setOpenPopup={this.setOpenPopup} pushInstalls={this.pushInstalls} setCurrentInstall={this.setCurrentInstall} setCurrentGame={this.setCurrentGame} setBackground={this.setBackground}/>}
                </div>
            </main>
        )
    }

    componentDidMount() {
        this.fetchSettings();
        this.fetchRepositories();
        setTimeout(async () => {
            for (const eventType of EVENTS) {
                const unlisten = await listen<string>(eventType, (event) => {
                    const newState = registerEvents(eventType, event);
                    if (newState !== undefined) this.setState(() => ({...newState}));
                });
                this.unlistenFns.push(unlisten);
            }
        }, 20);
    }

    componentWillUnmount() {
        this.unlistenFns.forEach((fn) => fn());
    }

    componentDidUpdate(_prevProps: any, prevState: any) {
        if (this.state.currentInstall && this.state.currentInstall !== prevState.currentInstall) {
            this.fetchInstallSettings(this.state.currentInstall);
            this.fetchCompatibilityVersions();
        }
    }

    fetchRepositories() {
        invoke("list_repositories").then(r => {
            if (r === null) {
                console.error("Repository database table contains nothing, some serious fuck up happened!")
            } else {
                let rr = JSON.parse(r as string);
                this.pushGames(rr);
                this.pushInstalls();
            }
        }).catch(e => {
            console.error("Error while listing database repositories information: " + e)
        });
    }

    pushGames(repos: { id: string; github_id: any; }[]) {
        repos.forEach((r: { id: string; github_id: any; }) => {
            invoke("list_manifests_by_repository_id", { repositoryId: r.id }).then(m => {
                if (m === null) {
                    console.error("Manifest database table contains nothing, some serious fuck up happened!")
                } else {
                    let g = JSON.parse(m as string);
                        this.pushGamesInfo(g);
                        let entries: any[] = [];
                        g.forEach((e: any) => entries.push(e));
                        // @ts-ignore
                        r["manifests"] = entries;
                        this.setReposList(repos);
                }
            }).catch(e => {
                console.error("Error while listing database manifest information: " + e)
            })
        });
    }

    pushGamesInfo(games: { filename: any; display_name: string; id: string; enabled: boolean; }[]) {
        invoke("list_game_manifests").then(m => {
            if (m === null) {
                console.error("GameManifest repository fetch issue, some serious fuck up happened!")
            } else {
                let gi = JSON.parse(m as string);
                // Hacky way to pass some values from DB manifest data onto the list of games we use to render SideBarIcon components
                gi.forEach((e: any) => {
                  let g = games.find(g => g.filename.replace(".json", "") === e.biz);
                  // @ts-ignore
                    e["manifest_id"] = g.id;
                  // @ts-ignore
                    e["manifest_enabled"] = g.enabled;
                  // @ts-ignore
                    e["manifest_file"] = g.filename;
                });

                this.setState(() => ({gamesinfo: gi}), () => {
                    if (this.state.installs.length === 0) {
                        if (games.length > 0 && this.state.currentGame == "") {
                            this.setCurrentGame(games[0].filename.replace(".json", ""));
                            this.setDisplayName(games[0].display_name);
                            this.setBackground(gi[0].assets.game_background);
                            this.setGameIcon(gi[0].assets.game_icon);
                            setTimeout(() => {
                                // @ts-ignore
                                document.getElementById(gi[0].biz).focus();
                            }, 20);
                        }
                    } else {
                        this.setCurrentGame(games[0].filename.replace(".json", ""));
                        this.setDisplayName(this.state.installs[0].name);
                        this.setBackground(this.state.installs[0].game_background);
                        this.setGameIcon(this.state.installs[0].game_icon);
                        this.setCurrentInstall(this.state.installs[0].id);
                        setTimeout(() => {
                            // @ts-ignore
                            document.getElementById(`${this.state.installs[0].id}`).focus();
                        }, 20);
                    }
                });
            }
        }).catch(e => {
            console.error("Error while listing game manifest information: " + e)
        })
    }

    pushInstalls() {
        invoke("list_installs").then(m => {
            if (m === null) {
                console.error("Installs fetch issue, some serious fuck up happened!")
            } else {
                let gi = JSON.parse(m as string);
                this.setState(() => ({installs: gi}));
            }
        }).catch(e => {
            console.error("Error while listing installs information: " + e)
        })
    }

    fetchSettings() {
        invoke("list_settings").then(data => {
            if (data === null) {
                console.error("Settings database table contains nothing, some serious fuck up happened!")
            } else {
                this.setState(() => ({globalSettings: JSON.parse(data as string)}));
            }
        });
    }

    fetchInstallSettings(install: any) {
        invoke("get_install_by_id", {id: install}).then(async data => {
            if (data === null) {
                console.error("Failed to fetch install settings!");
                this.setState(() => ({installSettings: null, gameManifest: null, preloadAvailable: false}));
            } else {
                let parsed = JSON.parse(data as string);
                let md = await this.fetchManifestById(parsed.manifest_id);
                // @ts-ignore
                let isPreload = md.extra.preload['metadata'] !== null;
                this.setState(() => ({installSettings: parsed, gameManifest: md, preloadAvailable: isPreload}));
            }
        });
    }

    fetchGameVersions(biz: string) {
        let game = this.state.gamesinfo.filter((g: any) => g.biz == biz)[0];
        let tmp: { value: any; name: any; }[] = [];
        game.game_versions.forEach((g: any) => {
            tmp.push({value: g.metadata.version, name: (game.latest_version === g.metadata.version) ? `Latest (${g.metadata.version})` : g.metadata.version});
        });
        this.setState({gameVersions: tmp});
    }

    fetchCompatibilityVersions() {
        invoke("list_compatibility_manifests").then(data => {
            if (data === null) {
                console.error("Failed to get compatibility versions.");
            } else {
                let r = JSON.parse(data as string);
                let dxvks: any[] = [];
                let wines: any[] = [];
                // Bad but will work for now... DO NOT EVER FILTER LIKE THIS...
                r.filter((e: any) => e.display_name.toLowerCase().includes("dxvk")).forEach((e: any) => {
                    e.versions.forEach((v: any) => dxvks.push({value: v.version, name: v.version}));
                });
                r.filter((e: any) => !e.display_name.toLowerCase().includes("dxvk")).forEach((e: any) => {
                    e.versions.forEach((v: any) => wines.push({value: v.version, name: v.version}));
                });
                this.setState({runnerVersions: wines, dxvkVersions: dxvks});
            }
        })
    }

    fetchDownloadSizes(biz: any, version: any, lang: any, path: any, callback: (data: any) => void) {
        invoke("get_download_sizes", {biz: biz, version: version, path: path, lang: lang}).then(data => {
            if (data === null) {
                console.error("Could not get download sizes!");
            } else {
                callback(JSON.parse(data as string));
                this.setState({downloadSizes: JSON.parse(data as string)});
            }
        });
    }

    async fetchManifestById(install: any) {
        let rslt: {latest_version: null, extra: {preload: {metadata: null}}};
        let data = await invoke("get_game_manifest_by_manifest_id", {id: install});
        if (data === null) {
            console.error("Failed to fetch game manifest info!");
            rslt = {latest_version: null, extra: {preload: {metadata: null}}};
        } else {
            rslt = JSON.parse(data as string);
        }
        return rslt;
    }

    refreshDownloadButtonInfo() {
        this.fetchGameVersions(this.state.currentGame);
        this.fetchCompatibilityVersions();
        setTimeout(() => {
            this.fetchDownloadSizes(this.state.currentGame, this.state.gameVersions[0].value, "en-us", `${this.state.globalSettings.default_game_path}/${this.state.currentGame}`, () => {});
            this.setState({openPopup: POPUPS.DOWNLOADGAME});
        }, 20);
    }

    determineButtonType() {
        let buttonType: "download" | "update" | "launch";

        if (!this.state.currentInstall || this.state.currentInstall === "") {
            buttonType = "download";
        } else if (this.state.installSettings.version !== this.state.gameManifest.latest_version && !this.state.preloadAvailable && !this.state.installSettings.ignore_updates) {
            if (this.state.gameManifest.latest_version !== null) {
                buttonType = "update";
            } else {
                buttonType = "launch";
            }
        } else {
            buttonType = "launch";
        }
        return buttonType;
    }

    setOpenPopup(state: POPUPS) {this.setState({openPopup: state});}
    setCurrentGame(game: string) {this.setState({currentGame: game});}
    setDisplayName(name: string) {this.setState({displayName: name});}
    setBackground(file: string) {this.setState({gameBackground: file});}
    setGameIcon(file: string) {this.setState({gameIcon: file});}
    setReposList(reposList: any) {this.setState({reposList: reposList});}
    setCurrentInstall(game: string) {this.setState({currentInstall: game});}
}

// === UTILITY ===
function registerEvents(eventType: string, event: any) {
    switch (eventType) {
        case "move_complete":
        case 'download_complete':
        case 'update_complete':
        case 'repair_complete':
        case 'preload_complete': {
            return {
                hideProgressBar: true,
                disableInstallEdit: false,
                disableRun: false,
                disableUpdate: false,
                disableDownload: false,
                progressName: `?`,
                progressVal: 0,
                progressPercent: `0%`
            };
        }
        case 'move_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                progressName: `Moving "${event.payload.file}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`
            };
        }
        case 'download_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                progressName: `Downloading "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`
            };
        }
        case 'update_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                progressName: `Updating "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`
            };
        }
        case 'repair_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                progressName: `Repairing "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`
            };
        }
        case 'preload_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                progressName: `Predownloading "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`
            };
        }
    }
}

function toPercent(number: any, total: any) { return (parseInt(number) / parseInt(total)) * 100; }