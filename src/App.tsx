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
import TooltipIcon from "./components/common/TooltipIcon.tsx";
import CollapsableTooltip from "./components/common/CollapsableTooltip.tsx";
import {emit, listen} from "@tauri-apps/api/event";
import SidebarCommunity from "./components/SidebarCommunity.tsx";

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
    this.preloadBackgrounds = this.preloadBackgrounds.bind(this);

    // Track which backgrounds have been preloaded to avoid duplicate fetches
    // @ts-ignore
    this.preloadedBackgrounds = new Set();

        this.state = {
            isInitialLoading: true,
            isContentLoaded: false,
            loadingProgress: 0,
            loadingMessage: "Initializing...",
            openPopup: POPUPS.NONE,
            currentGame: "",
            currentInstall: "",
            displayName: "",
            gameBackground: "",
            previousBackground: "",
            transitioningBackground: false,
            bgVersion: 0,
            gameIcon: "",
            gamesinfo: [],
            reposList: [],
            installs: [],
            globalSettings: {},
            preloadAvailable: false,
            gameVersions: [],
            installSettings: {},
            manifestsClosing: false,
            manifestsOpening: false,
            manifestsInitialLoading: true,
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
            disablePreload: false,
            disableResume: false,
            hideProgressBar: true,
            progressName: "?",
            progressVal: 0,
            progressPercent: "0%",
            progressPretty: 0,
            progressPrettyTotal: 0,
            resumeStates: {}
        }
    }

    render() {
        let buttonType = this.determineButtonType();
        
        // Show loading screen while app is initializing
        if (this.state.isInitialLoading) {
            return (
                <main className="w-full h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 scrollbar-none">
                    <div className="flex flex-col items-center space-y-6 animate-fadeIn">
                        {/* App Logo/Icon */}
                        <div className="relative w-16 h-16 rounded-xl animate-pulse shadow-2xl shadow-blue-500/20 overflow-hidden bg-slate-700/50">
                            <img src="/launcher-icon.png" srcSet="/launcher-icon.png 1x, /launcher-icon-128.png 2x" alt="TwintailLauncher" className="w-full h-full object-cover rounded-xl"
                                onError={(e) => {
                                    e.currentTarget.style.display = 'none';
                                    e.currentTarget.parentElement!.style.background = 'linear-gradient(135deg, rgb(59 130 246), rgb(147 51 234))';
                                }}
                            />
                            <div className="absolute inset-0 bg-gradient-to-tr from-white/10 to-transparent rounded-xl"></div>
                        </div>
                        
                        {/* App Name */}
                        <div className="text-center">
                            <h1 className="text-2xl font-bold text-white mb-2 animate-slideUp">TwintailLauncher</h1>
                            <p className="text-slate-400 text-sm animate-slideUp delay-100">{this.state.loadingMessage}</p>
                        </div>
                        
                        {/* Loading Bar */}
                        <div className="w-64 h-1 bg-slate-700 rounded-full overflow-hidden animate-slideUp delay-200">
                            <div className="h-full bg-gradient-to-r from-blue-500 to-purple-600 rounded-full transition-all duration-500 ease-out animate-shimmer" style={{ width: `${this.state.loadingProgress}%` }}></div>
                        </div>
                        
                        {/* Loading Dots */}
                        <div className="flex space-x-1 animate-slideUp delay-300">
                            <div className="w-2 h-2 bg-blue-500 rounded-full animate-bounce"></div>
                            <div className="w-2 h-2 bg-purple-500 rounded-full animate-bounce delay-100"></div>
                            <div className="w-2 h-2 bg-pink-500 rounded-full animate-bounce delay-200"></div>
                        </div>
                    </div>
                </main>
            );
        }
        
        return (
            <main className={`w-full h-screen flex flex-row bg-transparent overflow-x-hidden transition-opacity duration-500 ${this.state.isContentLoaded ? 'opacity-100' : 'opacity-0'} ${this.state.openPopup != POPUPS.NONE ? "popup-open" : ""}`}>
                <div className="absolute inset-0 -z-10 pointer-events-none overflow-hidden">
                    {this.state.transitioningBackground && this.state.previousBackground && (
                        <img key={`prev-${this.state.bgVersion}`} className={`w-full h-screen object-cover object-center absolute inset-0 transition-none animate-bg-fade-out ${this.state.openPopup != POPUPS.NONE ? "scale-[1.03] brightness-[0.45] saturate-75" : ""}`} alt={"previous background"} src={this.state.previousBackground} loading="lazy" decoding="async"/>
                    )}
                    <img id="app-bg" key={`curr-${this.state.bgVersion}`} className={`w-full h-screen object-cover object-center transition-all duration-300 ease-out ${this.state.transitioningBackground ? "animate-bg-fade-in" : ""} ${this.state.openPopup != POPUPS.NONE ? "scale-[1.03] brightness-[0.45] saturate-75" : ""}`} alt={"?"} src={this.state.gameBackground} loading="lazy" decoding="async"
                        onLoad={() => {
                            // Ensure content is marked as loaded when main background loads
                            if (!this.state.isContentLoaded) {
                                setTimeout(() => {
                                    this.setState({ isContentLoaded: true });
                                }, 100);
                            }
                        }}
                    />
                </div>
                {this.state.openPopup != POPUPS.NONE && (
                    <div className="pointer-events-none absolute top-0 bottom-0 left-16 right-0 z-10 animate-fadeIn">
                        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(10,10,15,0.55)_0%,rgba(5,5,10,0.70)_55%,rgba(0,0,0,0.82)_100%)]"/>
                        <div className="absolute inset-0 popup-noise opacity-30 mix-blend-overlay"/>
                        <div className="absolute inset-0 backdrop-fallback-grid opacity-[0.04]"/>
                    </div>
                )}
                <div className="h-full w-16 p-2 bg-black/50 flex flex-col items-center justify-between animate-slideInLeft" style={{ animationDelay: '100ms' }}>
                    <div className="flex flex-col pb-2 gap-2 flex-shrink overflow-scroll scrollbar-none animate-slideInLeft" style={{ animationDelay: '200ms' }}>
                        <CollapsableTooltip text={this.state.globalSettings.hide_manifests ? "Show manifests" : "Hide manifests"} icon={<ChevronDown color="white" onClick={() => {
                            // If we're about to hide manifests, trigger closing animation first
                            if (!this.state.globalSettings.hide_manifests) {
                                // Start closing animation
                                this.setState({ manifestsClosing: true });
                                // Calculate total animation time: base animation (200ms) + max stagger delay
                                const maxStaggerDelay = (this.state.gamesinfo.length - 1) * 50;
                                const totalAnimationTime = 200 + maxStaggerDelay + 100; // Add extra buffer
                                // After all animations complete, hide manifests
                                setTimeout(() => {
                                    invoke("update_settings_manifests_hide", {enabled: true}).then(() => {});
                                    this.setState((prevState: any) => ({
                                        globalSettings: {
                                            ...prevState.globalSettings,
                                            hide_manifests: true
                                        },
                                        manifestsClosing: false
                                    }));
                                }, totalAnimationTime);
                            } else {
                                // Show manifests with opening animation
                                this.setState({ manifestsOpening: true });
                                invoke("update_settings_manifests_hide", {enabled: false}).then(() => {});
                                this.setState((prevState: any) => ({
                                    globalSettings: {
                                        ...prevState.globalSettings,
                                        hide_manifests: false
                                    }
                                }));
                                // Reset opening state after all animations complete
                                const maxDelay = (this.state.gamesinfo.length - 1) * 60 + 50; // Last item delay
                                const animationDuration = 300; // slideInLeft duration
                                setTimeout(() => {
                                    this.setState({ manifestsOpening: false });
                                }, maxDelay + animationDuration + 50); // Add small buffer
                            }
                        }} className={`h-5 w-14 align-middle border-transparent transition cursor-pointer duration-300 pb-0 mb-0 ${this.state.globalSettings.hide_manifests ? "rotate-00" : "rotate-180"}`}/>}/>
                        <div className={"w-full transition-all duration-500 ease-in-out overflow-visible scrollbar-none gap-3 flex flex-col flex-shrink items-center"} style={{
                            maxHeight: (this.state.globalSettings.hide_manifests && !this.state.manifestsClosing) ? "0px" : (this.state.gamesinfo.length * 120) + "px",
                            opacity: (this.state.globalSettings.hide_manifests && !this.state.manifestsClosing) ? 0 : 1,
                            transform: (this.state.globalSettings.hide_manifests && !this.state.manifestsClosing) ? "translateY(-10px)" : "translateY(0px)"
                        }}>
                            {this.state.currentGame != "" && (!this.state.globalSettings.hide_manifests || this.state.manifestsClosing) && this.state.gamesinfo.map((game: { manifest_enabled: boolean; assets: any; filename: string; icon: string; display_name: string; biz: string; }, index: number) => {
                                return (
                                    <div key={game.biz} className={this.state.manifestsClosing ? "animate-slideOutLeft" :
                                        (this.state.manifestsOpening ? "animate-slideInLeft" :
                                        (this.state.manifestsInitialLoading ? "animate-slideInLeft" : ""))
                                    } style={{animationDelay: this.state.manifestsClosing ? `${(this.state.gamesinfo.length - index - 1) * 50}ms` :
                                                      (this.state.manifestsOpening ? `${index * 60 + 50}ms` : 
                                                      (this.state.manifestsInitialLoading ? `${index * 100 + 400}ms` : "0ms"))
                                    }}>
                                        <SidebarIconManifest popup={this.state.openPopup} icon={game.assets.game_icon} background={game.assets.game_background} name={game.display_name} enabled={game.manifest_enabled} id={game.biz} setCurrentGame={this.setCurrentGame} setOpenPopup={this.setOpenPopup} setDisplayName={this.setDisplayName} setBackground={this.setBackground} setCurrentInstall={this.setCurrentInstall} setGameIcon={this.setGameIcon} />
                                    </div>
                                )
                            })}
                        </div>
                        <hr className={`text-white/20 bg-white/20 p-0 transition-all duration-500 ${this.state.manifestsClosing ? 'animate-slideUpToPosition' : ''}`} style={{
                            borderColor: "rgb(255 255 255 / 0.2)",
                            animationDelay: this.state.manifestsClosing ? "100ms" : "0ms",
                            '--target-y': this.state.manifestsClosing ? `-${(this.state.gamesinfo.length * 56) + 12}px` : '0px'
                        } as React.CSSProperties}/>
                        <div className={`gap-3 flex flex-col items-center scrollbar-none overflow-scroll transition-all duration-500 ${this.state.manifestsClosing ? 'animate-slideUpToPosition' : (this.state.globalSettings.hide_manifests ? '' : 'animate-slideDownToPosition')}`} style={{
                            animationDelay: this.state.manifestsClosing ? "100ms" : "0ms",
                            '--target-y': this.state.manifestsClosing ? `-${(this.state.gamesinfo.length * 56) + 12}px` : '0px'
                        } as React.CSSProperties}>
                            {this.state.installs.map((install: { game_background: string; game_icon: string; manifest_id: string; name: string; id: string; }, index: number) => {
                                return (
                                    <div key={install.id} className="animate-slideInLeft" style={{ animationDelay: `${index * 100 + 600}ms` }}>
                                        <SidebarIconInstall popup={this.state.openPopup} icon={install.game_icon} background={install.game_background} name={install.name} enabled={true} id={install.id} setCurrentInstall={this.setCurrentInstall} setOpenPopup={this.setOpenPopup} setDisplayName={this.setDisplayName} setBackground={this.setBackground} setGameIcon={this.setGameIcon} />
                                    </div>
                                )
                            })}
                        </div>
                    </div>
                    <div className="flex flex-col gap-4 flex-shrink overflow-visible scrollbar-none animate-slideInLeft" style={{ animationDelay: '900ms' }}>
                        <hr className="text-white/20 bg-white/20 p-0 animate-slideInLeft" style={{borderColor: "rgb(255 255 255 / 0.2)", animationDelay: '950ms'}}/>
                        <div className="animate-slideInLeft" style={{ animationDelay: '1000ms' }}>
                            <SidebarRepos popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                        </div>
                        <div className="animate-slideInLeft" style={{ animationDelay: '1100ms' }}>
                            <SidebarSettings popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} />
                        </div>
                        <div className="animate-slideInLeft" style={{ animationDelay: '1200ms' }}>
                            <SidebarCommunity popup={this.state.openPopup} uri={"https://discord.gg/nDMJDwuj7s"} />
                        </div>
                    </div>
                </div>
                <div className="flex flex-row absolute bottom-8 right-16 gap-4 animate-slideInRight" style={{ animationDelay: '900ms' }}>
                    {(this.state.currentInstall !== "" && this.state.preloadAvailable) ? (<button disabled={this.state.disablePreload} onClick={() => {
                        emit("start_game_preload", {install: this.state.currentInstall, biz: "", lang: ""}).then(() => {});
                    }}><TooltipIcon side={"top"} text={"Predownload update"} icon={<DownloadIcon className="text-yellow-600 hover:text-yellow-700 w-8 h-8"/>}/>
                    </button>): null}
                    {(this.state.currentInstall !== "") ? <button id={`install_settings_btn`} disabled={this.state.disableInstallEdit} onClick={() => {
                        // Delay for very unnoticeable time to prevent popup opening before state is synced
                        setTimeout(() => {this.setState({openPopup: POPUPS.INSTALLSETTINGS});}, 20);
                    }}><TooltipIcon side={"top"} text={"Install settings"} icon={<Settings fill={"white"} className="hover:stroke-neutral-500 stroke-black w-8 h-8"/>}/></button> : null}
                    <GameButton resumeStates={this.state.resumeStates} disableResume={this.state.disableResume} disableDownload={this.state.disableDownload} disableRun={this.state.disableRun} disableUpdate={this.state.disableUpdate} currentInstall={this.state.currentInstall} globalSettings={this.state.globalSettings} refreshDownloadButtonInfo={this.refreshDownloadButtonInfo} buttonType={buttonType}/>
                </div>
                <div className={`absolute items-center justify-center bottom-0 left-96 right-72 p-8 z-20 [top:82%] ${this.state.hideProgressBar ? "hidden" : ""}`} id={"progress_bar"}>
                    <h4 className={"pl-4 pb-1 text-white text-stroke inline"} id={"progress_name"}>{this.state.progressName}</h4>
                    <h4 className={"pl-4 pb-1 text-white text-stroke inline"}>(<span id={"progress_percent"}>{this.state.progressPercent}</span> | <span id={"progress_pretty"}>{this.state.progressPretty} / {this.state.progressPrettyTotal}</span>)</h4>
                    <ProgressBar id={"progress_value"} progress={this.state.progressVal} className={"transition-all duration-500 ease-out"}/>
                </div>
                <div className={`absolute items-center justify-center top-0 bottom-0 left-16 right-0 p-8 z-20 ${this.state.openPopup == POPUPS.NONE ? "hidden" : "flex bg-white/10"}`} onClick={(e) => {
                    // Close popup when clicking on the overlay (but not on the popup content)
                    if (e.target === e.currentTarget) {this.setOpenPopup(POPUPS.NONE);}
                }}>
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
        // Set minimum loading time to account for startup
        const startTime = Date.now();
        const minimumLoadTime = 2800; // 2.8 seconds minimum
        // Start loading with progress animation
        this.animateLoadingProgress();
        
        // Update loading messages with timing
        setTimeout(() => this.setState({ loadingMessage: "Loading settings..." }), 900);
        setTimeout(() => this.setState({ loadingMessage: "Connecting to repositories..." }), 1400);
        setTimeout(() => this.setState({ loadingMessage: "Loading game data..." }), 1900);
        
        // Start fetching data
        Promise.all([
            this.fetchSettings(),
            this.fetchRepositories()
        ]).then(() => {
            this.setState({ loadingMessage: "Almost ready..." });
            
            // Calculate remaining time needed
            const elapsedTime = Date.now() - startTime;
            const remainingTime = Math.max(0, minimumLoadTime - elapsedTime);
            
            // Complete loading after minimum time has passed
            setTimeout(() => {
                this.completeInitialLoading();
            }, remainingTime);
        }).catch((error) => {
            console.error("Error during initialization:", error);
            this.setState({ loadingMessage: "Finishing up..." });
            // Still complete loading even if there are errors
            setTimeout(() => {
                this.completeInitialLoading();
            }, 1000);
        });
        
        // Set up event listeners after a delay
        setTimeout(async () => {
            for (const eventType of EVENTS) {
                const unlisten = await listen<string>(eventType, (event) => {
                    const newState = registerEvents(eventType, event, this.pushInstalls);
                    if (newState !== undefined) this.setState(() => ({...newState}));
                });
                this.unlistenFns.push(unlisten);
            }
        }, 20);
    }

    animateLoadingProgress() {
        const duration = 1800; // 1.8 seconds
        const interval = 100; // Update every 100ms
        const steps = duration / interval;
        let currentStep = 0;

        const progressInterval = setInterval(() => {
            currentStep++;
            // Use eased progress that starts fast then slows down
            const rawProgress = currentStep / steps;
            const easedProgress = 1 - Math.pow(1 - rawProgress, 3); // Ease-out cubic
            const progress = Math.min(easedProgress * 90, 90); // Cap at 90% until data loads
            this.setState({ loadingProgress: progress });
            if (currentStep >= steps) {clearInterval(progressInterval);}
        }, interval);
    }

    completeInitialLoading() {
        // Finish the progress bar quickly
        this.setState({ loadingProgress: 100 });
        // Wait a moment then hide loading screen with smooth transition
        setTimeout(() => {
            this.setState({isInitialLoading: false});
            // Mark content as loaded after a brief delay for smoother transition
            setTimeout(() => {this.setState({ isContentLoaded: true });}, 200);
        }, 300);
    }

    componentWillUnmount() {
        this.unlistenFns.forEach((fn) => fn());
    }

    componentDidUpdate(_prevProps: any, prevState: any) {
        if (this.state.currentInstall && this.state.currentInstall !== prevState.currentInstall) {
            this.fetchInstallSettings(this.state.currentInstall);
            this.fetchInstallResumeStates(this.state.currentInstall);
            this.fetchCompatibilityVersions();
        }
        if (this.state.openPopup !== prevState.openPopup) {
            if (this.state.openPopup !== POPUPS.NONE) {
                document.documentElement.classList.add("no-scroll");
                document.body.classList.add("no-scroll");
            } else {
                document.documentElement.classList.remove("no-scroll");
                document.body.classList.remove("no-scroll");
            }
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
                    // Preload all backgrounds once games are available
                    this.preloadBackgrounds();
                    
                    // Reset initial loading state after animations complete
                    if (this.state.manifestsInitialLoading && gi.length > 0) {
                        const maxDelay = (gi.length - 1) * 100 + 400; // Last item delay
                        const animationDuration = 600; // slideInLeft duration
                        setTimeout(() => {this.setState({ manifestsInitialLoading: false });}, maxDelay + animationDuration + 50);
                    }
                    
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
        return invoke("list_settings").then(data => {
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

    refreshDownloadButtonInfo() {
        this.fetchGameVersions(this.state.currentGame);
        this.fetchCompatibilityVersions();
        setTimeout(() => {
            this.fetchDownloadSizes(this.state.currentGame, this.state.gameVersions[0].value, "en-us", `${this.state.globalSettings.default_game_path}/${this.state.currentGame}`, (disk) => {
                // @ts-ignore
                let btn = document.getElementById("game_dl_btn");
                // @ts-ignore
                let freedisk = document.getElementById("game_disk_free");

                // Always enable button initially - the DownloadGame component will handle disabling based on checkbox state
                // @ts-ignore
                btn.removeAttribute("disabled");

                // Set disk space styling based on available space (but don't disable button)

                if (disk.game_decompressed_size_raw > disk.free_disk_space_raw) {
                    // @ts-ignore
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
            });
            this.setState({openPopup: POPUPS.DOWNLOADGAME});
        }, 20);
    }

    determineButtonType() {
        let buttonType: "download" | "update" | "launch" | "resume";

        if (!this.state.currentInstall || this.state.currentInstall === "") {
            buttonType = "download";
        } else if (this.state.installSettings.version !== this.state.gameManifest.latest_version && !this.state.preloadAvailable && !this.state.installSettings.ignore_updates) {
            if (this.state.gameManifest.latest_version !== null) {
                if (this.state.resumeStates.updating || this.state.resumeStates.downloading || this.state.resumeStates.preloading || this.state.resumeStates.repairing) {
                    buttonType = "resume";
                } else {
                    buttonType = "update";
                }
            } else {
                if (this.state.resumeStates.updating || this.state.resumeStates.downloading || this.state.resumeStates.preloading || this.state.resumeStates.repairing) {
                    buttonType = "resume";
                } else {
                    buttonType = "launch";
                }
            }
        } else {
            if (this.state.resumeStates.updating || this.state.resumeStates.downloading || this.state.resumeStates.preloading || this.state.resumeStates.repairing) {
                buttonType = "resume";
            } else {
                buttonType = "launch";
            }
        }
        return buttonType;
    }

    setOpenPopup(state: POPUPS) {this.setState({openPopup: state});}
    setCurrentGame(game: string) {this.setState({currentGame: game});}
    setDisplayName(name: string) {this.setState({displayName: name});}
    // Store the background transition timeout ID
    bgTransitionTimeout?: number;

    setBackground(file: string) {
        if (!file || file === this.state.gameBackground) return; // nothing to do

        // Cancel any previous transition timeout
        if (this.bgTransitionTimeout) {
            clearTimeout(this.bgTransitionTimeout);
            this.bgTransitionTimeout = undefined;
        }

        // Preload image to avoid flash of old bg when new not ready
        const img = new Image();
        img.onload = () => {
            this.setState((prev: any) => ({
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
        };
        img.src = file;
    }
    setGameIcon(file: string) {this.setState({gameIcon: file});}
    setReposList(reposList: any) {this.setState({reposList: reposList});}
    setCurrentInstall(game: string) {this.setState({currentInstall: game});}

    // === PRELOAD BACKGROUNDS ===
    preloadBackgrounds() {
        // @ts-ignore
        const cache: Set<string> = this.preloadedBackgrounds;
        const list = this.state.gamesinfo || [];
        if (!list.length) return;
        list.forEach((g: any, idx: number) => {
            const src = g?.assets?.game_background;
            if (!src || cache.has(src)) return;
            // Stagger slightly to avoid network burst
            setTimeout(() => {
                const img = new Image();
                img.onload = () => { cache.add(src); };
                img.onerror = () => { /* Ignore failures */ };
                img.src = src;
            }, idx * 40);
        });
    }
}

// === UTILITY ===
function registerEvents(eventType: string, event: any, pushInstalls: () => void) {
    switch (eventType) {
        case "move_complete":
        case 'download_complete':
        case 'update_complete':
        case 'repair_complete':
        case 'preload_complete': {
            pushInstalls();
            return {
                hideProgressBar: true,
                disableInstallEdit: false,
                disableRun: false,
                disableUpdate: false,
                disableDownload: false,
                disablePreload: false,
                disableResume: false,
                progressName: `?`,
                progressVal: 0,
                progressPercent: `0%`,
                progressPretty: 0,
                progressPrettyTotal: 0
            };
        }
        case 'move_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                disablePreload: true,
                disableResume: true,
                progressName: `Moving "${event.payload.file}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`,
                progressPretty: `${formatBytes(event.payload.progress)}`,
                progressPrettyTotal: `${formatBytes(event.payload.total)}`
            };
        }
        case 'download_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                disablePreload: true,
                disableResume: true,
                progressName: `Downloading "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`,
                progressPretty: `${formatBytes(event.payload.progress)}`,
                progressPrettyTotal: `${formatBytes(event.payload.total)}`
            };
        }
        case 'update_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                disablePreload: true,
                disableResume: true,
                progressName: `Updating "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`,
                progressPretty: `${formatBytes(event.payload.progress)}`,
                progressPrettyTotal: `${formatBytes(event.payload.total)}`
            };
        }
        case 'repair_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                disablePreload: true,
                disableResume: true,
                progressName: `Repairing "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`,
                progressPretty: `${formatBytes(event.payload.progress)}`,
                progressPrettyTotal: `${formatBytes(event.payload.total)}`
            };
        }
        case 'preload_progress': {
            return {hideProgressBar: false,
                disableInstallEdit: true,
                disableRun: true,
                disableUpdate: true,
                disableDownload: true,
                disablePreload: true,
                disableResume: true,
                progressName: `Predownloading "${event.payload.name}"`,
                progressVal: Math.round(toPercent(event.payload.progress, event.payload.total)),
                progressPercent: `${toPercent(event.payload.progress, event.payload.total).toFixed(2)}%`,
                progressPretty: `${formatBytes(event.payload.progress)}`,
                progressPrettyTotal: `${formatBytes(event.payload.total)}`
            };
        }
    }
}

function toPercent(number: any, total: any) { return (parseInt(number) / parseInt(total)) * 100; }

function formatBytes(bytes: any) {
    const MiB = 1024 * 1024;
    const GiB = 1024 * MiB;
    let b =  parseInt(bytes);

    if (b >= GiB) {
        return (b / GiB).toFixed(2) + ' GiB';
    } else if (b >= MiB) {
        return (b / MiB).toFixed(2) + ' MiB';
    } else {
        return b + ' bytes';
    }
}
