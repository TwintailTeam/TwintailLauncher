import "./App.css";
import React from "react";
import RepoManager from "./components/popups/repomanager/RepoManager.tsx";
import {POPUPS} from "./components/popups/POPUPS.ts";
import AddRepo from "./components/popups/AddRepo.tsx";
import SidebarIconManifest from "./components/SidebarIconManifest.tsx";
import {invoke} from "@tauri-apps/api/core";
import SidebarRepos from "./components/SidebarRepos.tsx";
import {DownloadIcon, HardDriveDownloadIcon, Rocket, Settings} from "lucide-react";
import SidebarSettings from "./components/SidebarSettings.tsx";
import SettingsGlobal from "./components/popups/settings/SettingsGlobal.tsx";
import SidebarIconInstall from "./components/SidebarIconInstall.tsx";
import DownloadGame from "./components/popups/DownloadGame.tsx";
import SettingsInstall from "./components/popups/settings/SettingsInstall.tsx";

export default class App extends React.Component<any, any> {
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
            dxvkVersions: []
        }
    }

    render() {
        return (
            <main className="w-full h-screen flex flex-row bg-transparent">
                <img className="w-full h-screen object-cover object-center absolute top-0 left-0 right-0 bottom-0 -z-10" alt={"?"} src={this.state.gameBackground} loading="lazy" decoding="async" srcSet={undefined}/>
                <div className="h-full w-16 p-2 bg-black/50 flex flex-col gap-4 items-center fixed-backdrop-blur-md justify-between">
                    <div className="flex flex-col gap-4 flex-shrink overflow-scroll scrollbar-none">
                        {this.state.currentGame != "" && this.state.gamesinfo.map((game: { manifest_enabled: boolean; assets: any; filename: string; icon: string; display_name: string; biz: string; }) => {
                            return (
                                <SidebarIconManifest key={game.biz} popup={this.state.openPopup} icon={game.assets.game_icon} background={game.assets.game_background} name={game.display_name} enabled={game.manifest_enabled} id={game.biz} setCurrentGame={this.setCurrentGame} setOpenPopup={this.setOpenPopup} setDisplayName={this.setDisplayName} setBackground={this.setBackground} setCurrentInstall={this.setCurrentInstall} setGameIcon={this.setGameIcon} />
                            )
                        })}
                        <hr className="text-white/20 bg-white/20" style={{borderColor: "rgb(255 255 255 / 0.2)"}}/>
                        {this.state.installs.map((install: { game_background: string; game_icon: string; manifest_id: string; name: string; id: string; }) => {
                            return (
                                <SidebarIconInstall key={install.id} popup={this.state.openPopup} icon={install.game_icon} background={install.game_background} name={install.name} enabled={true} id={install.id} manifest_id={install.manifest_id} setCurrentInstall={this.setCurrentInstall} setOpenPopup={this.setOpenPopup} setDisplayName={this.setDisplayName} setBackground={this.setBackground} setPreloadAvailable={this.setPreloadAvailable} setGameIcon={this.setGameIcon} />
                            )
                        })}
                    </div>
                    <div className="flex flex-col gap-4 flex-shrink overflow-scroll scrollbar-none">
                        <hr className="text-white/20 bg-white/20" style={{borderColor: "rgb(255 255 255 / 0.2)"}}/>
                        <SidebarRepos popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                        <SidebarSettings popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                    </div>
                </div>
                <div className="flex flex-row absolute bottom-8 right-16 gap-4">
                    {(this.state.currentInstall !== "" && this.state.preloadAvailable) ? <button onClick={() => {
                        console.log("preload...")
                    }}>
                        <DownloadIcon className="text-blue-500 w-8 h-8" />
                    </button> : null}
                    {(this.state.currentInstall !== "") ? <button id={"install_settings_btn"} onClick={() => {
                        this.fetchInstallSettings(this.state.currentInstall);
                        this.fetchCompatibilityVersions();
                        // Delay for very unnoticeable time to prevent popup opening before state is synced
                        setTimeout(() => {
                            this.setState({openPopup: POPUPS.INSTALLSETTINGS});
                        }, 20);
                    }}><Settings className="text-white w-8 h-8" />
                    </button> : null}
                    {(this.state.currentInstall !== "") ? <button id={"launch_game_btn"} className="flex flex-row gap-2 items-center py-2 px-4 bg-blue-600 rounded-lg" onClick={() => {
                        setTimeout(() => {
                            invoke("game_launch", {id: this.state.currentInstall}).then((r: any) => {
                                if (r) {
                                    //TODO: what to do with launcher window???, disable launch button if launcher is not "close on launch"
                                } else {
                                    console.error("Launch error!");
                                }
                            })
                        }, 20);
                    }}><Rocket/><span className="font-semibold translate-y-px">Launch!</span>
                    </button> : <button id={"game_download_btn"} className="flex flex-row gap-2 items-center py-2 px-4 bg-blue-600 rounded-lg" onClick={() => {
                        this.fetchGameVersions(this.state.currentGame);
                        this.fetchCompatibilityVersions();
                        setTimeout(() => {
                            this.setState({openPopup: POPUPS.DOWNLOADGAME});
                        }, 20);
                    }}><HardDriveDownloadIcon/><span className="font-semibold translate-y-px">Download</span>
                    </button>}
                </div>

                <div className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-20 ${this.state.openPopup == POPUPS.NONE ? "hidden" : "flex fixed-backdrop-blur-lg bg-white/10"}`}>
                    {this.state.openPopup == POPUPS.REPOMANAGER && <RepoManager repos={this.state.reposList} setOpenPopup={this.setOpenPopup} fetchRepositories={this.fetchRepositories}/>}
                    {this.state.openPopup == POPUPS.ADDREPO && <AddRepo setOpenPopup={this.setOpenPopup}/>}
                    {this.state.openPopup == POPUPS.SETTINGS && <SettingsGlobal fetchSettings={this.fetchSettings} settings={this.state.globalSettings} setOpenPopup={this.setOpenPopup} />}
                    {this.state.openPopup == POPUPS.DOWNLOADGAME && <DownloadGame runnerVersions={this.state.runnerVersions} dxvkVersions={this.state.dxvkVersions} versions={this.state.gameVersions} icon={this.state.gameIcon} background={this.state.gameBackground} biz={this.state.currentGame} displayName={this.state.displayName} settings={this.state.globalSettings} setOpenPopup={this.setOpenPopup} pushInstalls={this.pushInstalls}/>}
                    {this.state.openPopup == POPUPS.INSTALLSETTINGS && <SettingsInstall games={this.state.gamesinfo} runnerVersions={this.state.runnerVersions} dxvkVersions={this.state.dxvkVersions} installSettings={this.state.installSettings} setOpenPopup={this.setOpenPopup} pushInstalls={this.pushInstalls} setCurrentInstall={this.setCurrentInstall} setCurrentGame={this.setCurrentGame} setBackground={this.setBackground} fetchInstallSettings={this.fetchInstallSettings}/>}
                </div>
            </main>
        )
    }

    componentDidMount() {
        this.fetchSettings();
        this.fetchRepositories();
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
                    if (games.length > 0 && this.state.currentGame == "") {
                        this.setCurrentGame(games[0].filename.replace(".json", ""));
                        this.setDisplayName(games[0].display_name);
                        this.setBackground(gi[0].assets.game_background);
                        this.setGameIcon(gi[0].assets.game_icon);
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
        invoke("get_install_by_id", {id: install}).then(data => {
            if (data === null) {
                console.error("Install database table contains nothing, some serious fuck up happened!")
            } else {
                this.setState(() => ({installSettings: JSON.parse(data as string)}));
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

    setOpenPopup(state: POPUPS) {
        this.setState({openPopup: state});
    }

    setCurrentGame(game: string) {
        this.setState({currentGame: game});
    }

    setDisplayName(name: string) {
        this.setState({displayName: name});
    }

    setBackground(file: string) {
        this.setState({gameBackground: file});
    }

    setGameIcon(file: string) {
        this.setState({gameIcon: file});
    }

    setReposList(reposList: any) {
        this.setState({reposList: reposList});
    }

    setCurrentInstall(game: string) {
        this.setState({currentInstall: game});
    }

    setPreloadAvailable(game: string) {
        invoke("get_game_manifest_by_manifest_id", {id: game}).then(r => {
            if (r === null) {
                console.error("Failed to get game manifest by manifest id!");
            } else {
                let rr = JSON.parse(r as string);
                if (rr.extra.preload?.metadata !== null) {
                    this.setState({preloadAvailable: true});
                }
            }
        }).catch(e => {
            console.error("Error while querying preload information: " + e)
        })
    }
}