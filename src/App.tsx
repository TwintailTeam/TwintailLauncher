import "./App.css";
import React from "react";
import {POPUPS} from "./components/popups/POPUPS.ts";
import {invoke} from "@tauri-apps/api/core";
import SidebarSettings from "./components/sidebar/SidebarSettings.tsx";
import SidebarIconInstall from "./components/sidebar/SidebarIconInstall.tsx";
import SidebarLink from "./components/sidebar/SidebarLink.tsx";
import { preloadImages } from "./utils/imagePreloader";
import AppLoadingScreen from "./components/AppLoadingScreen";
import SidebarManifests from "./components/sidebar/SidebarManifests.tsx";
import { determineButtonType } from "./utils/determineButtonType";
import BackgroundLayer from "./components/layout/BackgroundLayer";
import ManifestsPanel from "./components/layout/ManifestsPanel";
import ActionBar from "./components/layout/ActionBar";
import DownloadProgress from "./components/layout/DownloadProgress";
import PopupOverlay from "./components/layout/PopupOverlay";
import { startInitialLoad } from "./services/loader";
import SidebarRunners from "./components/sidebar/SidebarRunners.tsx";


export default class App extends React.Component<any, any> {
    loaderController?: { cancel: () => void };
    preloadedBackgrounds: Set<string>;
    // Ref to measure floating manifests panel width to prevent snap during close
    manifestsPanelRef: React.RefObject<HTMLDivElement>;
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
        this.fetchInstallResumeStates = this.fetchInstallResumeStates.bind(this);
        this.fetchDownloadSizes = this.fetchDownloadSizes.bind(this);
        this.fetchGameVersions = this.fetchGameVersions.bind(this);
        this.fetchCompatibilityVersions = this.fetchCompatibilityVersions.bind(this);
        this.refreshDownloadButtonInfo = this.refreshDownloadButtonInfo.bind(this);
        this.fetchInstalledRunners = this.fetchInstalledRunners.bind(this);

    // @ts-ignore
    this.preloadedBackgrounds = new Set();
    this.manifestsPanelRef = React.createRef<HTMLDivElement>();

        this.state = {
            isInitialLoading: true,
            isContentLoaded: false,
            loadingProgress: 0,
            loadingMessage: "Initializing...",
            showLoadingOverlay: true,
            overlayFadingOut: false,
            openPopup: POPUPS.NONE,
            currentGame: "",
            currentInstall: "",
            displayName: "",
            gameBackground: "",
            previousBackground: "",
            transitioningBackground: false,
            bgLoading: false,
            bgVersion: 0,
            gameIcon: "",
            gamesinfo: [],
            reposList: [],
            installs: [],
            globalSettings: {},
            preloadAvailable: false,
            gameVersions: [],
            installSettings: {},
            installGameSwitches: {},
            installGameFps: [],
            manifestsInitialLoading: true,
            manifestsOpenVisual: false,
            manifestsPanelWidth: null,
            runnerVersions: [],
            dxvkVersions: [],
            runners: [],
            installedRunners: [],
            downloadSizes: {},
            downloadDir: "",
            downloadVersion: "",
            gameManifest: {},
            disableRun: false,
            disableUpdate: false,
            disableDownload: false,
            disableInstallEdit: false,
            disablePreload: false,
            disableResume: false,
            hideProgressBar: true,
            progressName: "?",
            progressVal: 0,
            progressPercent: "0%",
            progressPretty: 0,
            progressPrettyTotal: 0,
            resumeStates: {},
            openDownloadAsExisting: false
        }
    }

    render() {
        const buttonType = determineButtonType({
            currentInstall: this.state.currentInstall,
            installSettings: this.state.installSettings,
            gameManifest: this.state.gameManifest,
            preloadAvailable: this.state.preloadAvailable,
            resumeStates: this.state.resumeStates,
        });

        return (
            <>
            <main className={`w-full h-screen flex flex-row bg-transparent overflow-x-hidden transition-opacity duration-500 ${this.state.isContentLoaded ? 'opacity-100' : 'opacity-0'} ${this.state.openPopup != POPUPS.NONE ? "popup-open" : ""}`}>
                <BackgroundLayer
                    currentSrc={this.state.gameBackground}
                    previousSrc={this.state.previousBackground}
                    transitioning={this.state.transitioningBackground}
                    bgVersion={this.state.bgVersion}
                    popupOpen={this.state.openPopup != POPUPS.NONE}
                    bgLoading={this.state.bgLoading}
                    onMainLoad={() => {
                        // Background image has rendered; stop spinner and finish initial reveal if needed
                        this.setState((prev: any) => ({ bgLoading: false, isContentLoaded: prev.isContentLoaded ? prev.isContentLoaded : prev.isContentLoaded }), () => {
                            if (!this.state.isContentLoaded) {
                                setTimeout(() => { this.setState({ isContentLoaded: true }); }, 100);
                            }
                        });
                    }}
                />
                {this.state.openPopup != POPUPS.NONE && (
                    <div className="pointer-events-none absolute top-0 bottom-0 left-16 right-0 z-10 animate-fadeIn">
                        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(10,10,15,0.55)_0%,rgba(5,5,10,0.70)_55%,rgba(0,0,0,0.82)_100%)]"/>
                        <div className="absolute inset-0 backdrop-fallback-grid opacity-[0.04]"/>
                    </div>
                )}
                {/* Top floating manifest panel (slides out from left), toggled by the sidebar chevron */}
                <ManifestsPanel
                    openPopup={this.state.openPopup}
                    manifestsOpenVisual={this.state.manifestsOpenVisual}
                    manifestsInitialLoading={this.state.manifestsInitialLoading}
                    gamesinfo={this.state.currentGame !== "" ? this.state.gamesinfo : []}
                    manifestsPanelRef={this.manifestsPanelRef}
                    setCurrentGame={this.setCurrentGame}
                    setOpenPopup={this.setOpenPopup}
                    setDisplayName={this.setDisplayName}
                    setBackground={this.setBackground}
                    setCurrentInstall={this.setCurrentInstall}
                    setGameIcon={this.setGameIcon}
                    onRequestClose={() => {
                        if (this.state.manifestsOpenVisual && this.state.openPopup === POPUPS.NONE) {
                            this.setState({ manifestsOpenVisual: false });
                        }
                    }}
                />
                <div className="h-full w-16 p-2 bg-black/50 flex flex-col items-center justify-start animate-slideInLeft" style={{ animationDelay: '100ms' }}>
                    {/* Separate, centered section for the download/manifests toggle */}
                    <div className="flex items-center justify-center h-16 animate-slideInLeft" style={{ animationDelay: '150ms' }}>
                        <SidebarManifests
                            isOpen={this.state.manifestsOpenVisual}
                            popup={this.state.openPopup}
                            hasInstalls={(this.state.installs?.length || 0) > 0}
                            onToggle={() => {
                                const nextOpen = !this.state.manifestsOpenVisual;
                                // Instant visual flip for spam-safe reversible transitions
                                this.setState((prev: any) => ({
                                    manifestsOpenVisual: nextOpen,
                                    globalSettings: { ...prev.globalSettings, hide_manifests: !nextOpen }
                                }));
                                // Persist setting asynchronously, no timeouts
                                invoke("update_settings_manifests_hide", { enabled: !nextOpen }).then(() => {});
                            }}
                        />
                    </div>
                    {/* Scrollable section for installs and separators */}
                    <div className="flex flex-col pb-2 gap-2 flex-shrink overflow-scroll scrollbar-none select-none animate-slideInLeft" style={{ animationDelay: '200ms' }}>
                        <div className={"w-full transition-all duration-500 ease-in-out overflow-visible scrollbar-none gap-3 flex flex-col flex-shrink items-center"} style={{
                            maxHeight: "0px",
                            opacity: 0,
                            transform: "translateY(-10px)"
                        }}>
                            {/* Manifests moved to the top bar */}
                        </div>
                        <hr className={`text-white/20 bg-white/20 p-0 transition-all duration-500 ${this.state.manifestsClosing ? 'animate-slideUpToPosition' : ''}`} style={{
                            borderColor: "rgb(255 255 255 / 0.2)",
                            animationDelay: this.state.manifestsClosing ? "100ms" : "0ms",
                            '--target-y': this.state.manifestsClosing ? `-${(this.state.gamesinfo.length * 56) + 12}px` : '0px'
                        } as React.CSSProperties}/>
                        <div className={`gap-3 flex flex-col items-center scrollbar-none overflow-scroll select-none transition-all duration-500 ${this.state.manifestsClosing ? 'animate-slideUpToPosition' : (this.state.globalSettings.hide_manifests ? '' : 'animate-slideDownToPosition')}`} style={{
                            animationDelay: this.state.manifestsClosing ? "100ms" : "0ms",
                            '--target-y': this.state.manifestsClosing ? `-${(this.state.gamesinfo.length * 56) + 12}px` : '0px'
                        } as React.CSSProperties}>
                            {this.state.installs.map((install: any, index: number) => {
                                // Find corresponding game manifest info by manifest_id
                                const game = (this.state.gamesinfo || []).find((g: any) => g.manifest_id === install.manifest_id);
                                const latest = game?.latest_version ?? null;
                                const hasUpdate = !!(latest && install?.version && latest !== install.version && !install?.ignore_updates);
                                return (
                                    <div key={install.id} className="animate-slideInLeft" style={{ animationDelay: `${index * 100 + 600}ms` }}>
                                        <SidebarIconInstall
                                            popup={this.state.openPopup}
                                            icon={install.game_icon}
                                            background={install.game_background}
                                            name={install.name}
                                            enabled={true}
                                            id={install.id}
                                            hasUpdate={hasUpdate}
                                            setCurrentInstall={this.setCurrentInstall}
                                            setOpenPopup={this.setOpenPopup}
                                            setDisplayName={this.setDisplayName}
                                            setBackground={this.setBackground}
                                            setGameIcon={this.setGameIcon}
                                            installSettings={this.state.installSettings}
                                            onOpenInstallSettings={() => {
                                                this.setCurrentInstall(install.id);
                                                this.setDisplayName(install.name);
                                                this.setBackground(install.game_background);
                                                this.setGameIcon(install.game_icon);
                                                this.fetchInstallSettings(install.id);
                                                this.setOpenPopup(POPUPS.INSTALLSETTINGS);
                                            }}
                                            onRefreshSettings={() => {this.fetchInstallSettings(install.id);}}
                                        />
                                    </div>
                                )
                            })}
                        </div>
                    </div>
                    <div className="flex flex-col gap-4 flex-shrink overflow-visible scrollbar-none animate-slideInLeft mt-auto" style={{ animationDelay: '900ms' }}>
                        <hr className="text-white/20 bg-white/20 p-0 animate-slideInLeft" style={{borderColor: "rgb(255 255 255 / 0.2)", animationDelay: '950ms'}}/>
                        {(window.navigator.platform.includes("Linux")) && (
                            <div className="animate-slideInLeft" style={{ animationDelay: '1000ms' }}>
                                <SidebarRunners popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                            </div>
                        )}
                        <div className="animate-slideInLeft" style={{ animationDelay: '1100ms' }}>
                            <SidebarSettings popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                        </div>
                        <div className="animate-slideInLeft" style={{ animationDelay: '1200ms' }}>
                            <SidebarLink popup={this.state.openPopup} title={"Discord"} iconType={"discord"} uri={"https://discord.gg/nDMJDwuj7s"} />
                        </div>
                        <div className="animate-slideInLeft" style={{ animationDelay: '1300ms' }}>
                            <SidebarLink popup={this.state.openPopup} title={"Support the project"} iconType={"donate"} uri={"https://ko-fi.com/twintailteam"} />
                        </div>
                    </div>
                </div>
                <ActionBar
                    currentInstall={this.state.currentInstall}
                    preloadAvailable={this.state.preloadAvailable}
                    disablePreload={this.state.disablePreload}
                    disableInstallEdit={this.state.disableInstallEdit}
                    disableResume={this.state.disableResume}
                    disableDownload={this.state.disableDownload}
                    disableRun={this.state.disableRun}
                    disableUpdate={this.state.disableUpdate}
                    resumeStates={this.state.resumeStates}
                    globalSettings={this.state.globalSettings}
                    installSettings={this.state.installSettings}
                    buttonType={buttonType}
                    refreshDownloadButtonInfo={this.refreshDownloadButtonInfo}
                    onOpenInstallSettings={() => {
                        this.setState({ disableInstallEdit: true }, async () => {
                            await this.fetchInstallSettings(this.state.currentInstall);
                            this.setState({ openPopup: POPUPS.INSTALLSETTINGS, disableInstallEdit: false });
                        });
                    }}
                />
                <DownloadProgress
                    hidden={this.state.hideProgressBar}
                    name={this.state.progressName}
                    percentText={this.state.progressPercent}
                    pretty={this.state.progressPretty}
                    prettyTotal={this.state.progressPrettyTotal}
                    progressVal={this.state.progressVal}
                />
                <PopupOverlay
                    openPopup={this.state.openPopup}
                    setOpenPopup={this.setOpenPopup}
                    reposList={this.state.reposList}
                    fetchRepositories={this.fetchRepositories}
                    fetchSettings={this.fetchSettings}
                    globalSettings={this.state.globalSettings}
                    downloadSizes={this.state.downloadSizes}
                    runnerVersions={this.state.runnerVersions}
                    dxvkVersions={this.state.dxvkVersions}
                    gameVersions={this.state.gameVersions}
                    runners={this.state.runners}
                    installedRunners={this.state.installedRunners}
                    fetchInstalledRunners={this.fetchInstalledRunners}
                    gameIcon={this.state.gameIcon}
                    gameBackground={this.state.gameBackground}
                    currentGame={this.state.currentGame}
                    displayName={this.state.displayName}
                    openDownloadAsExisting={this.state.openDownloadAsExisting}
                    fetchDownloadSizes={this.fetchDownloadSizes}
                    pushInstalls={this.pushInstalls}
                    setBackground={this.setBackground}
                    setCurrentInstall={this.setCurrentInstall}
                    gamesinfo={this.state.gamesinfo}
                    installSettings={this.state.installSettings}
                    setCurrentGame={this.setCurrentGame}
                    fetchInstallSettings={this.fetchInstallSettings}
                    installGameSwitches={this.state.installGameSwitches}
                    installGameFps={this.state.installGameFps}
                    installs={this.state.installs}
                />
            </main>
            {this.state.showLoadingOverlay && (
                <AppLoadingScreen
                    progress={this.state.loadingProgress}
                    message={this.state.loadingMessage}
                    fadingOut={this.state.overlayFadingOut}
                />
            )}
            </>
        )
    }

    async componentDidMount() {
        // Kick off background-style initial loading via service
        this.loaderController = startInitialLoad({
            fetchSettings: this.fetchSettings,
            fetchRepositories: this.fetchRepositories,
            fetchCompatibilityVersions: this.fetchCompatibilityVersions,
            fetchInstalledRunners: this.fetchInstalledRunners,
            getGamesInfo: () => this.state.gamesinfo,
            getInstalls: () => this.state.installs,
            preloadImages: (images, onProgress, preloaded) => preloadImages(images, onProgress, preloaded),
            preloadedBackgrounds: this.preloadedBackgrounds,
            setProgress: (progress, message) => this.setState({ loadingProgress: progress, loadingMessage: message }),
            completeInitialLoading: () => this.completeInitialLoading(),
            pushInstalls: this.pushInstalls,
            applyEventState: (ns) => this.setState(() => ({ ...ns })),
            getCurrentInstall: () => this.state.currentInstall,
            fetchInstallResumeStates: this.fetchInstallResumeStates,
        });
    }

    completeInitialLoading() {
        this.setState({ loadingProgress: 100 });
        // Start cross-fade: reveal main content and fade out overlay
        setTimeout(() => {
            this.setState({ isContentLoaded: true, overlayFadingOut: true });
            // After overlay fade duration, remove it and finalize initial state
            setTimeout(() => {
                this.setState({ showLoadingOverlay: false, isInitialLoading: false });
            }, 520); // match overlay CSS duration (500ms) + small buffer
        }, 150);
    }

    componentWillUnmount() {
        if (this.loaderController) {
            this.loaderController.cancel();
        }
    }

    componentDidUpdate(_prevProps: any, prevState: any) {
        if (this.state.currentInstall && this.state.currentInstall !== prevState.currentInstall) {
            this.fetchInstallSettings(this.state.currentInstall);
            this.fetchInstallResumeStates(this.state.currentInstall);
            this.fetchCompatibilityVersions();
            this.fetchInstalledRunners();
        }
    }

    fetchRepositories() {
        return invoke("list_repositories").then(r => {
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
                    // Reset initial loading state after animations complete
                    if (this.state.manifestsInitialLoading && gi.length > 0) {
                        const maxDelay = (gi.length - 1) * 100 + 400; // Last item delay
                        const animationDuration = 600; // slideInLeft duration
                        setTimeout(() => {this.setState({ manifestsInitialLoading: false });}, maxDelay + animationDuration + 50);
                    }

                    if (this.state.installs.length === 0) {
                        if (games.length > 0 && this.state.currentGame == "") {
                            let bg = (gi[0].assets.game_live_background !== "") ? gi[0].assets.game_background : gi[0].assets.game_background;
                            this.setCurrentGame(games[0].filename.replace(".json", ""));
                            this.setDisplayName(games[0].display_name);
                            this.setBackground(bg);
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
                        this.fetchInstallResumeStates(this.state.installs[0].id);
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
                // No installs left, set installs to empty array
                this.setState(() => ({ installs: [] }));
            } else {
                let gi = JSON.parse(m as string);
                this.setState(() => ({installs: gi}), () => {
                    // Also preload installed-specific assets (older/different versions)
                    try {
                        const backgrounds: string[] = (this.state.installs || [])
                            .map((i: any) => i?.game_background)
                            .filter((s: any) => !!s);
                        const icons: string[] = (this.state.installs || [])
                            .map((i: any) => i?.game_icon)
                            .filter((s: any) => !!s);
                        const images = Array.from(new Set([...(backgrounds as string[]), ...(icons as string[])]));
                        // Only preload ones we haven't already cached
                        const notPreloaded = images.filter((u) => !this.preloadedBackgrounds.has(u));
                        if (notPreloaded.length > 0) {
                            preloadImages(notPreloaded, undefined, this.preloadedBackgrounds).then(() => {});
                        }
                    } catch (e) {
                        console.warn("Install assets preload failed:", e);
                    }
                });
            }
        }).catch(e => {
            console.error("Error while listing installs information: " + e)
        })
    }

    fetchSettings() {
        return invoke("list_settings").then(data => {
            if (data === null) {
                console.error("Settings database table contains nothing, some serious fuck up happened!")
            } else {
                const gs = JSON.parse(data as string);
                this.setState(() => ({
                    globalSettings: gs
                }));
            }
        });
    }

    fetchInstallSettings(install: any) {
        return invoke("get_install_by_id", {id: install}).then(async data => {
            if (data === null) {
                console.error("Failed to fetch install settings!");
                this.setState(() => ({installSettings: null, gameManifest: null, preloadAvailable: false, installGameSwitches: {}, installGameFps: []}));
            } else {
                let parsed = JSON.parse(data as string);
                let md = await this.fetchManifestById(parsed.manifest_id);
                // @ts-ignore
                let isPreload = md.extra.preload['metadata'] !== null;
                // Prepare switches and fps list for SettingsInstall (keep newer fields if present)
                const switches = md?.extra?.switches ?? {};
                const fpsList = Array.isArray(md?.extra?.fps_unlock_options) ? md.extra.fps_unlock_options.map((e: any) => ({ value: `${e}`, name: `${e}` })) : [];
                this.setState(() => ({installSettings: parsed, gameManifest: md, preloadAvailable: isPreload, installGameSwitches: switches, installGameFps: fpsList}));
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
        return invoke("list_compatibility_manifests").then(data => {
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
                r.filter((e: any) => !e.display_name.toLowerCase().includes("dxvk") && !e.display_name.toLowerCase().includes("wine")).forEach((e: any) => {
                    e.versions.forEach((v: any) => wines.push({value: v.version, name: v.version}));
                });
                let d = r.filter((e: any) => !e.display_name.toLowerCase().includes("dxvk") && !e.display_name.toLowerCase().includes("wine"));
                this.setState({runnerVersions: wines, dxvkVersions: dxvks, runners: d});
            }
        })
    }

    fetchInstalledRunners() {
        return invoke("list_installed_runners").then(data => {
            if (data === null) {
                console.error("Failed to get installed runners.");
            } else {
                let r = JSON.parse(data as string);
                let installed: any[] = [];
                r.filter((e: any) => e.is_installed).forEach((e: any) => {installed.push(e);});
                this.setState({installedRunners: installed});
            }
        })
    }

    fetchDownloadSizes(biz: any, version: any, lang: any, path: any, region_filter: any, callback: (data: any) => void) {
        invoke("get_download_sizes", {biz: biz, version: version, path: path, lang: lang, region: region_filter}).then(data => {
            if (data === null) {
                console.error("Could not get download sizes!");
            } else {
                const parsed = JSON.parse(data as string);
                callback(parsed);
                this.setState({downloadSizes: parsed});
            }
        });
    }

    async fetchManifestById(install: any) {
        // Use broad typing since manifest.extra may include optional fields like
        // switches, fps_unlock_options, preload, etc.
        let rslt: any;
        let data = await invoke("get_game_manifest_by_manifest_id", {id: install});
        if (data === null) {
            console.error("Failed to fetch game manifest info!");
            rslt = {latest_version: null, extra: {preload: {metadata: null}}};
        } else {
            rslt = JSON.parse(data as string);
        }
        return rslt;
    }

    fetchInstallResumeStates(install: any) {
        invoke("get_resume_states", {install: install}).then(async data => {
            if (data === null) {
                console.error("Failed to fetch install resume states!");
                this.setState(() => ({resumeStates: {downloading: false, updating: false, preloading: false, repairing: false}}));
            } else {
                let parsed = JSON.parse(data as string);
                this.setState(() => ({resumeStates: parsed}));
            }
        });
    }

    async refreshDownloadButtonInfo(existingInstall: boolean = false) {
        // Ensure versions in state
        this.fetchGameVersions(this.state.currentGame);
        await this.fetchCompatibilityVersions();
        // Delay a tiny bit before opening to allow state sync
        setTimeout(() => {
            this.fetchDownloadSizes(
                this.state.currentGame,
                this.state.gameVersions[0]?.value,
                "en-us",
                `${this.state.globalSettings.default_game_path}/${this.state.currentGame}`,
                "glb_official",
                (disk) => {
                    // @ts-ignore
                    const btn = document.getElementById("game_dl_btn");
                    // @ts-ignore
                    const freedisk = document.getElementById("game_disk_free");

                    // Always enable button initially - DownloadGame handles checkbox disabling
                    if (btn) {
                        // @ts-ignore
                        btn.removeAttribute("disabled");
                    }

                    // Styling based on available space
                    if (freedisk) {
                        // Use legacy fields from backend
                        // @ts-ignore
                        if (disk.game_decompressed_size_raw > disk.free_disk_space_raw) {
                            freedisk.classList.add("text-red-600");
                            // @ts-ignore
                            freedisk.classList.remove("text-white");
                            // @ts-ignore
                            freedisk.classList.add("font-bold");
                        } else {
                            // @ts-ignore
                            freedisk.classList.remove("text-red-600");
                            // @ts-ignore
                            freedisk.classList.add("text-white");
                            // @ts-ignore
                            freedisk.classList.remove("font-bold");
                        }
                    }
                }
            );
            this.setState({ openPopup: POPUPS.DOWNLOADGAME, openDownloadAsExisting: existingInstall });
        }, 20);
    }

    

    setOpenPopup(state: POPUPS) {this.setState({openPopup: state});}
    setCurrentGame(game: string) {this.setState({currentGame: game});}
    setDisplayName(name: string) {this.setState({displayName: name});}
    // Store the background transition timeout
    bgTransitionTimeout?: number;

    setBackground(file: string) {
        if (!file || file === this.state.gameBackground) return; // nothing to do

        // Cancel any previous transition timeout
        if (this.bgTransitionTimeout) {
            clearTimeout(this.bgTransitionTimeout);
            this.bgTransitionTimeout = undefined;
        }

        // Start loading: show gradient on the new image while it loads
        this.setState((prev: any) => ({
            bgLoading: true,
            previousBackground: prev.gameBackground || prev.previousBackground || "",
            gameBackground: file,
            transitioningBackground: prev.gameBackground !== "",
            bgVersion: prev.bgVersion + 1
        }), () => {
            if (this.state.transitioningBackground) {
                this.bgTransitionTimeout = window.setTimeout(() => {
                    // After animation remove previous; keep if multiple rapid switches occurred
                    this.setState({
                        transitioningBackground: false,
                        previousBackground: ""
                    });
                    this.bgTransitionTimeout = undefined;
                }, 480); // match CSS duration
            }
        });
    }
    setGameIcon(file: string) {this.setState({gameIcon: file});}
    setReposList(reposList: any) {this.setState({reposList: reposList});}
    setCurrentInstall(game: string) {this.setState({currentInstall: game});}
}
