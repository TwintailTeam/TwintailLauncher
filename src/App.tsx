import "./App.css";
import React from "react";
import { POPUPS } from "./components/popups/POPUPS.ts";
import { PAGES } from "./components/pages/PAGES.ts";
import { invoke } from "@tauri-apps/api/core";
import SidebarSettings from "./components/sidebar/SidebarSettings.tsx";
import SidebarIconInstall from "./components/sidebar/SidebarIconInstall.tsx";
import SidebarLink from "./components/sidebar/SidebarLink.tsx";
import { preloadImages, isLinux, isImagePreloaded } from "./utils/imagePreloader";
import AppLoadingScreen from "./components/AppLoadingScreen";
import SidebarManifests from "./components/sidebar/SidebarManifests.tsx";
import { determineButtonType } from "./utils/determineButtonType";
import BackgroundLayer from "./components/layout/BackgroundLayer";
import ManifestsPanel from "./components/layout/ManifestsPanel";
import ActionBar from "./components/layout/ActionBar";
import PopupOverlay from "./components/layout/PopupOverlay";
import PageViewContainer from "./components/pages/PageViewContainer";
import GameInfoOverlay from "./components/layout/GameInfoOverlay";
import PlayStatsOverlay from "./components/layout/PlayStatsOverlay";
import { startInitialLoad, NetworkMonitor, type RecoveryProgress, type NetworkStatus } from "./services/loader";
import { showDialogAsync, closeCurrentDialog } from "./context/DialogContext";
import SidebarRunners from "./components/sidebar/SidebarRunners.tsx";
import SidebarDownloads from "./components/sidebar/SidebarDownloads";
import { toPercent } from "./utils/progress";
import BackgroundControls from "./components/common/BackgroundControls";


export default class App extends React.Component<any, any> {
    loaderController?: { cancel: () => void };
    preloadedBackgrounds: Set<string>;
    // Ref to measure floating manifests panel width to prevent snap during close
    manifestsPanelRef: React.RefObject<HTMLDivElement>;
    // Network monitor for periodic connectivity checks
    networkMonitor?: NetworkMonitor;
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
        this.fetchSteamRTStatus = this.fetchSteamRTStatus.bind(this);
        this.handleSpeedSample = this.handleSpeedSample.bind(this);
        this.handleClearSpeedHistory = this.handleClearSpeedHistory.bind(this);
        this.setCurrentPage = this.setCurrentPage.bind(this);
        this.updateAvailableBackgrounds = this.updateAvailableBackgrounds.bind(this);

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
            steamrtInstalled: true,
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
            progressSpeed: "",
            progressPretty: 0,
            progressPrettyTotal: 0,
            downloadQueueState: null,
            downloadProgressByJobId: {},
            resumeStates: {},
            openDownloadAsExisting: false,
            downloadManagerOpen: false,
            speedHistory: [] as { net: number; disk: number }[],
            downloadsPageOpen: false,
            currentPage: PAGES.NONE,
            availableBackgrounds: [] as { src: string; label: string; isDynamic: boolean }[],
            limitedMode: false,
            networkStatus: "online" as "online" | "slow" | "offline",
            recoveryProgress: null as RecoveryProgress | null,
            imageVersion: 0, // Increments on recovery to force image re-loads
            // Drag and drop state for sidebar installs
            dragIndex: null as number | null,
            dragTargetIndex: null as number | null,
            dropCompleted: false, // Flag to prevent handleDragEnd from resetting after successful drop
            droppedItemId: null as string | null, // Track which item was just dropped for pop animation
        }
    }

    render() {
        const runningJobs = this.state.downloadQueueState?.running || [];
        const queuedJobs = this.state.downloadQueueState?.queued || [];
        const pausedJobs = this.state.downloadQueueState?.pausedJobs || [];
        const downloadQueueCount = new Set(
            [...runningJobs, ...queuedJobs, ...pausedJobs].map((j: any) => j.id)
        ).size;

        const isCurrentInstallDownloading = runningJobs.some((j: any) => j.installId === this.state.currentInstall);
        const isCurrentInstallQueued = queuedJobs.some((j: any) => j.installId === this.state.currentInstall);
        const hasDownloads = downloadQueueCount > 0;

        // Check if runner dependencies are ready (Linux only)
        const isLinux = window.navigator.platform.includes("Linux");
        const currentInstallData = this.state.installs.find((i: any) => i.id === this.state.currentInstall);
        const currentRunnerVersion = currentInstallData?.runner_version ?? "";
        const isRunnerInstalled = this.state.installedRunners.some((r: any) => r.version === currentRunnerVersion && r.is_installed);
        const runnerDepsNotReady = isLinux && currentInstallData && (!this.state.steamrtInstalled || !isRunnerInstalled);

        // Check if any extras/dependencies required by the current install are downloading or queued
        const allJobs = [...runningJobs, ...queuedJobs];
        const xxmiPackageIds = ['xxmi', 'gimi', 'srmi', 'zzmi', 'himi', 'wwmi', 'ssmi', 'efmi'];
        const isXxmiDownloading = this.state.installSettings?.use_xxmi && allJobs.some((j: any) =>
            j.kind === 'xxmi_download' || (j.kind === 'extras_download' && xxmiPackageIds.includes(j.installId))
        );
        const isJadeiteDownloading = this.state.installSettings?.use_jadeite && allJobs.some((j: any) =>
            j.kind === 'extras_download' && j.installId === 'v5.0.1-hotfix'
        );
        const isFpsUnlockDownloading = this.state.installSettings?.use_fps_unlock && allJobs.some((j: any) =>
            j.kind === 'extras_download' && j.installId === 'keqing_unlock'
        );
        const isSteamRTDownloading = isLinux && allJobs.some((j: any) =>
            j.kind === 'steamrt_download' || j.kind === 'steamrt4_download'
        );
        const isRunnerDownloading = isLinux && currentRunnerVersion && allJobs.some((j: any) =>
            j.kind === 'runner_download' && j.installId === currentRunnerVersion
        );
        const extrasDownloading = !!(isXxmiDownloading || isJadeiteDownloading || isFpsUnlockDownloading || isSteamRTDownloading || isRunnerDownloading);

        const primaryRunningJobId = runningJobs.length > 0 ? runningJobs[0].id : undefined;
        const primaryProgress = primaryRunningJobId ? this.state.downloadProgressByJobId?.[primaryRunningJobId] : undefined;
        const downloadsPercent =
            typeof primaryProgress?.progress === "number" && typeof primaryProgress?.total === "number" && primaryProgress.total > 0 ? Math.max(0, Math.min(100, toPercent(primaryProgress.progress, primaryProgress.total))) : undefined;
        const buttonType = determineButtonType({
            currentInstall: this.state.currentInstall,
            installSettings: this.state.installSettings,
            gameManifest: this.state.gameManifest,
            preloadAvailable: this.state.preloadAvailable,
            resumeStates: this.state.resumeStates,
            isDownloading: isCurrentInstallDownloading,
            isQueued: isCurrentInstallQueued,
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
                        pageOpen={this.state.currentPage !== PAGES.NONE}
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
                    {/* Limited Mode Banner */}
                    {this.state.limitedMode && (
                        <div className="absolute top-0 left-0 right-0 z-50 flex justify-center pointer-events-none">
                            <div className="mt-2 px-4 py-2 bg-amber-500/95 rounded-lg shadow-lg pointer-events-auto flex items-center gap-2 animate-fadeIn">
                                {/* Icon - changes based on recovery state */}
                                {this.state.recoveryProgress?.phase && this.state.recoveryProgress.phase !== "idle" && this.state.recoveryProgress.phase !== "complete" ? (
                                    <svg className="w-4 h-4 text-amber-900 animate-spin" fill="none" viewBox="0 0 24 24">
                                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                                    </svg>
                                ) : this.state.recoveryProgress?.phase === "complete" ? (
                                    <svg className="w-4 h-4 text-green-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                    </svg>
                                ) : (
                                    <svg className="w-4 h-4 text-amber-900" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                    </svg>
                                )}

                                {/* Status text - shows progress when recovering */}
                                <span className="text-sm font-medium text-amber-900">
                                    {this.state.recoveryProgress?.phase === "checking" && "Checking connection..."}
                                    {this.state.recoveryProgress?.phase === "loading_repos" && "Loading repositories..."}
                                    {this.state.recoveryProgress?.phase === "loading_images" && (
                                        <>Loading images... ({this.state.recoveryProgress.current}/{this.state.recoveryProgress.total})</>
                                    )}
                                    {this.state.recoveryProgress?.phase === "complete" && "Connected!"}
                                    {(!this.state.recoveryProgress?.phase || this.state.recoveryProgress.phase === "idle") && (
                                        <>{this.state.networkStatus === "offline" ? "Offline Mode" : "Limited Mode"} - Downloads unavailable</>
                                    )}
                                </span>

                                {/* Progress bar for image loading */}
                                {this.state.recoveryProgress?.phase === "loading_images" && this.state.recoveryProgress.total > 0 && (
                                    <div className="w-20 h-1.5 bg-amber-200 rounded-full overflow-hidden">
                                        <div
                                            className="h-full bg-amber-700 transition-all duration-300"
                                            style={{ width: `${(this.state.recoveryProgress.current / this.state.recoveryProgress.total) * 100}%` }}
                                        />
                                    </div>
                                )}

                                {/* Retry button - disabled during recovery */}
                                {(!this.state.recoveryProgress?.phase || this.state.recoveryProgress.phase === "idle") && (
                                    <button
                                        onClick={() => this.networkMonitor?.triggerRecovery()}
                                        className="ml-2 px-2 py-0.5 text-xs font-medium text-amber-900 bg-amber-200 hover:bg-amber-100 rounded transition-colors"
                                    >
                                        Retry
                                    </button>
                                )}
                            </div>
                        </div>
                    )}
                    {/* Solid dark backdrop - instant visibility to prevent black flash on WebKitGTK */}
                    {/* This layer uses visibility (not opacity) for truly instant show/hide */}
                    <div
                        className="pointer-events-none absolute inset-0 left-16 z-[15]"
                        style={{
                            visibility: (this.state.openPopup !== POPUPS.NONE || this.state.currentPage !== PAGES.NONE) ? 'visible' : 'hidden',
                            background: 'rgba(0,0,0,0.5)',
                            backfaceVisibility: 'hidden',
                            WebkitBackfaceVisibility: 'hidden',
                            transform: 'translateZ(0)'
                        }}
                    />
                    {/* Frost overlay - always rendered to prevent DOM insertion flash on WebKitGTK */}
                    <div
                        className={`pointer-events-none absolute inset-0 ${this.state.openPopup != POPUPS.NONE ? "z-40" : "z-20"}`}
                        style={{
                            // Use visibility instead of opacity for instant show/hide without compositor flash on WebKitGTK
                            visibility: (this.state.openPopup !== POPUPS.NONE || this.state.currentPage !== PAGES.NONE) ? 'visible' : 'hidden',
                            backfaceVisibility: 'hidden',
                            WebkitBackfaceVisibility: 'hidden',
                            transform: 'translateZ(0)'
                        }}
                    >
                        {/* Frost content - static decorative layers, no animation needed */}
                        {/* Frost-like light veil */}
                        <div className="absolute inset-0 bg-gradient-to-br from-white/[0.08] via-white/[0.03] to-white/[0.06]" />
                        {/* Subtle dark vignette for depth */}
                        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,transparent_0%,rgba(0,0,0,0.25)_70%,rgba(0,0,0,0.4)_100%)]" />
                        {/* Light grid texture for frost feel */}
                        <div className="absolute inset-0 backdrop-fallback-grid opacity-[0.06]" />
                    </div>
                    {/* Top floating manifest panel (slides out from left), toggled by the sidebar chevron */}
                    <ManifestsPanel
                        openPopup={this.state.openPopup}
                        manifestsOpenVisual={this.state.manifestsOpenVisual}
                        manifestsInitialLoading={this.state.manifestsInitialLoading}
                        gamesinfo={this.state.currentGame !== "" ? this.state.gamesinfo : []}
                        manifestsPanelRef={this.manifestsPanelRef}
                        currentGame={this.state.currentGame}
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
                        imageVersion={this.state.imageVersion}
                    />
                    <div className="h-full w-16 p-2 bg-black/50 flex flex-col items-center justify-start animate-slideInLeft relative z-30" style={{ animationDelay: '100ms' }}>
                        {/* Separate, centered section for the download/manifests toggle */}
                        <div className="flex items-center justify-center h-16 animate-slideInLeft" style={{ animationDelay: '150ms' }}>
                            <SidebarManifests
                                isOpen={this.state.manifestsOpenVisual}
                                popup={this.state.openPopup}
                                hasInstalls={(this.state.installs?.length || 0) > 0}
                                currentPage={this.state.currentPage}
                                setCurrentPage={this.setCurrentPage}
                                onToggle={() => {
                                    const nextOpen = !this.state.manifestsOpenVisual;
                                    // Instant visual flip for spam-safe reversible transitions
                                    this.setState((prev: any) => ({
                                        manifestsOpenVisual: nextOpen,
                                        globalSettings: { ...prev.globalSettings, hide_manifests: !nextOpen }
                                    }));
                                    // Persist setting asynchronously, no timeouts
                                    invoke("update_settings_manifests_hide", { enabled: !nextOpen }).then(() => { });
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
                            <div className={`w-full h-px bg-gradient-to-r from-transparent via-white/20 to-transparent transition-all duration-500 ${this.state.manifestsClosing ? 'animate-slideUpToPosition' : ''}`} style={{
                                animationDelay: this.state.manifestsClosing ? "100ms" : "0ms",
                                '--target-y': this.state.manifestsClosing ? `-${(this.state.gamesinfo.length * 56) + 12}px` : '0px'
                            } as React.CSSProperties} />
                            <div className={`gap-3 flex flex-col items-center scrollbar-none overflow-scroll select-none transition-all duration-500 ${this.state.manifestsClosing ? 'animate-slideUpToPosition' : (this.state.globalSettings.hide_manifests ? '' : 'animate-slideDownToPosition')}`} style={{
                                animationDelay: this.state.manifestsClosing ? "100ms" : "0ms",
                                '--target-y': this.state.manifestsClosing ? `-${(this.state.gamesinfo.length * 56) + 12}px` : '0px'
                            } as React.CSSProperties}>
                                {this.state.installs.map((install: any, index: number) => {
                                    // Find corresponding game manifest info by manifest_id
                                    const game = (this.state.gamesinfo || []).find((g: any) => g.manifest_id === install.manifest_id);
                                    const latest = game?.latest_version ?? null;
                                    const hasUpdate = !!(latest && install?.version && latest !== install.version && !install?.ignore_updates && !install?.steam_imported);
                                    // Only apply slideInLeft animation during initial loading, not during drag reorder
                                    const isInitialAnim = this.state.isInitialLoading || this.state.showLoadingOverlay;
                                    const isJustDropped = this.state.droppedItemId === install.id;
                                    return (
                                        <div key={`${install.id}-v${this.state.imageVersion}`} className={`w-12 ${isInitialAnim ? 'animate-slideInLeft' : ''} ${isJustDropped ? 'animate-drop-pop' : ''}`} style={isInitialAnim ? { animationDelay: `${index * 100 + 600}ms` } : undefined}>
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
                                                currentPage={this.state.currentPage}
                                                setCurrentPage={this.setCurrentPage}
                                                setDisplayName={this.setDisplayName}
                                                setBackground={this.setBackground}
                                                setGameIcon={this.setGameIcon}
                                                installSettings={this.state.installSettings}
                                                // Drag and drop props
                                                index={index}
                                                onDragStart={this.handleDragStart}
                                                onDragEnd={this.handleDragEnd}
                                                onDragOver={this.handleDragOver}
                                                onDrop={this.handleDrop}
                                                isDragging={this.state.dragIndex === index}
                                                isDragTarget={this.state.dragTargetIndex === index}
                                                onOpenInstallSettings={async () => {
                                                    this.setCurrentInstall(install.id);
                                                    this.setDisplayName(install.name);
                                                    this.setGameIcon(install.game_icon);

                                                    // Find game manifest to get dynamic background if available
                                                    const game = this.state.gamesinfo.find((g: any) => g.manifest_id === install.manifest_id);
                                                    const dynamicBg = game?.assets?.game_live_background;
                                                    const staticBg = game?.assets?.game_background || install.game_background;
                                                    // Prefer dynamic background (video) over static, skip video on Linux
                                                    const bestBackground = (!isLinux && dynamicBg) ? dynamicBg : staticBg;

                                                    this.setBackground(bestBackground);

                                                    // Preload images and fetch settings in parallel
                                                    await Promise.all([
                                                        preloadImages([bestBackground, install.game_icon].filter(Boolean)),
                                                        this.fetchInstallSettings(install.id)
                                                    ]);
                                                    this.setOpenPopup(POPUPS.INSTALLSETTINGS);
                                                }}
                                                onRefreshSettings={() => { this.fetchInstallSettings(install.id); }}
                                            />
                                        </div>
                                    )
                                })}
                                {/* Drop zone for last position */}
                                {this.state.dragIndex !== null && (
                                    <div
                                        className="w-12 flex flex-col items-center"
                                        onDragOver={(e) => {
                                            e.preventDefault();
                                            e.dataTransfer.dropEffect = 'move';
                                            // Set target to length (after all items)
                                            const lastIndex = this.state.installs.length;
                                            if (this.state.dragIndex !== lastIndex - 1) {
                                                this.setState({ dragTargetIndex: lastIndex });
                                            }
                                        }}
                                        onDrop={(e) => {
                                            e.preventDefault();
                                            this.handleDropAtEnd();
                                        }}
                                    >
                                        <div
                                            className={`w-12 flex items-center justify-center transition-all duration-200 ease-out overflow-hidden ${this.state.dragTargetIndex === this.state.installs.length ? 'h-14' : 'h-6'}`}
                                        >
                                            {this.state.dragTargetIndex === this.state.installs.length && (
                                                <div className="w-12 h-12 rounded-lg border-2 border-dashed border-purple-500/70 bg-purple-500/10 flex items-center justify-center animate-pulse">
                                                    <div className="w-6 h-0.5 rounded-full bg-purple-500/50" />
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>
                        {/* Sidebar bottom icons - single container with dynamic stagger delays */}
                        {(() => {
                            const isAnimating = this.state.isInitialLoading || this.state.showLoadingOverlay;
                            const baseDelay = 900; // Starting delay in ms
                            const staggerStep = 50; // Delay increment between items

                            // Build array of sidebar bottom items (conditional items included when applicable)
                            const bottomItems: React.ReactNode[] = [
                                // Divider
                                <div key="divider" className="w-full h-px bg-gradient-to-r from-transparent via-white/20 to-transparent" />,
                                // Downloads
                                <SidebarDownloads
                                    key="downloads"
                                    popup={this.state.openPopup}
                                    setOpenPopup={this.setOpenPopup}
                                    hasDownloads={hasDownloads}
                                    queueCount={downloadQueueCount}
                                    progressPercent={downloadsPercent}
                                    currentPage={this.state.currentPage}
                                    setCurrentPage={this.setCurrentPage}
                                />,
                                // Runners (Linux only)
                                ...(window.navigator.platform.includes("Linux") ? [
                                    <SidebarRunners key="runners" popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} currentPage={this.state.currentPage} setCurrentPage={this.setCurrentPage} />
                                ] : []),
                                // Settings
                                <SidebarSettings key="settings" popup={this.state.openPopup} setOpenPopup={this.setOpenPopup} currentPage={this.state.currentPage} setCurrentPage={this.setCurrentPage} />,
                                // Discord
                                <SidebarLink key="discord" popup={this.state.openPopup} title={"Discord"} iconType={"discord"} uri={"https://discord.gg/nDMJDwuj7s"} />,
                                // Donate
                                <SidebarLink key="donate" popup={this.state.openPopup} title={"Support the project"} iconType={"donate"} uri={"https://ko-fi.com/twintailteam"} />,
                            ];

                            return (
                                <div className="flex flex-col gap-4 flex-shrink overflow-visible scrollbar-none mt-auto items-center">
                                    {bottomItems.map((item, index) => (
                                        <div
                                            key={index}
                                            className={isAnimating ? 'animate-slideInLeft-stagger' : ''}
                                            style={isAnimating ? { '--stagger-delay': `${baseDelay + (index * staggerStep)}ms` } as React.CSSProperties : undefined}
                                        >
                                            {item}
                                        </div>
                                    ))}
                                </div>
                            );
                        })()}
                    </div>
                    <GameInfoOverlay
                        displayName={this.state.displayName}
                        gameIcon={this.state.gameIcon}
                        version={(() => {
                            // For installed games, use installSettings.version
                            if (this.state.currentInstall) {
                                return this.state.installSettings?.version;
                            }
                            // For manifest games (not installed), show latest version
                            const game = this.state.gamesinfo.find((g: any) => g.biz === this.state.currentGame);
                            return game?.latest_version;
                        })()}
                        hasUpdate={(() => {
                            const install = this.state.installs.find((i: any) => i.id === this.state.currentInstall);
                            const game = this.state.gamesinfo.find((g: any) => g.manifest_id === install?.manifest_id);
                            const latest = game?.latest_version ?? null;
                            return !!(latest && install?.version && latest !== install.version && !install?.ignore_updates && !install?.steam_imported);
                        })()}
                        isVisible={(this.state.currentInstall !== "" || this.state.currentGame !== "") && this.state.openPopup === POPUPS.NONE && this.state.currentPage === PAGES.NONE}
                        imageVersion={this.state.imageVersion}
                    />
                    <PlayStatsOverlay
                        lastPlayedTime={this.state.installSettings?.last_played_time}
                        totalPlaytime={this.state.installSettings?.total_playtime}
                        isVisible={this.state.currentInstall !== "" && this.state.openPopup === POPUPS.NONE && this.state.currentPage === PAGES.NONE}
                    />

                    <ActionBar
                        currentInstall={this.state.currentInstall}
                        preloadAvailable={this.state.preloadAvailable}
                        disablePreload={this.state.disablePreload}
                        disableInstallEdit={isCurrentInstallDownloading || isCurrentInstallQueued}
                        disableResume={this.state.disableResume}
                        disableDownload={this.state.disableDownload}
                        disableRun={isCurrentInstallDownloading || isCurrentInstallQueued || runnerDepsNotReady || extrasDownloading}
                        disableUpdate={this.state.disableUpdate}
                        resumeStates={this.state.resumeStates}
                        globalSettings={this.state.globalSettings}
                        installSettings={this.state.installSettings}
                        gameManifest={this.state.gameManifest}
                        buttonType={buttonType}
                        refreshDownloadButtonInfo={this.refreshDownloadButtonInfo}
                        isVisible={this.state.openPopup === POPUPS.NONE && this.state.currentPage === PAGES.NONE}
                        isPausing={this.state.downloadQueueState?.pausingInstalls?.includes(this.state.currentInstall) ?? false}
                        onOpenInstallSettings={() => {
                            this.setState({ disableInstallEdit: true }, async () => {
                                // Get current install for image preloading
                                const currentInstall = this.state.installs.find((i: any) => i.id === this.state.currentInstall);
                                // Preload images and fetch settings in parallel
                                await Promise.all([
                                    currentInstall ? preloadImages([currentInstall.game_background, currentInstall.game_icon].filter(Boolean)) : Promise.resolve(),
                                    this.fetchInstallSettings(this.state.currentInstall)
                                ]);
                                this.setState({ openPopup: POPUPS.INSTALLSETTINGS, disableInstallEdit: false });
                            });
                        }}
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
                        gameManifest={this.state.gameManifest}
                        setCurrentGame={this.setCurrentGame}
                        fetchInstallSettings={this.fetchInstallSettings}
                        installGameSwitches={this.state.installGameSwitches}
                        installGameFps={this.state.installGameFps}
                        installs={this.state.installs}
                        setCurrentPage={this.setCurrentPage}
                        setDisplayName={this.setDisplayName}
                        setGameIcon={this.setGameIcon}
                        imageVersion={this.state.imageVersion}
                    />
                    <BackgroundControls
                        currentBackground={this.state.gameBackground}
                        availableBackgrounds={this.state.availableBackgrounds}
                        onBackgroundChange={this.handleBackgroundChange}
                        isVisible={this.state.openPopup === POPUPS.NONE && this.state.currentPage === PAGES.NONE && (this.state.currentInstall !== "" || this.state.currentGame !== "")}
                    />
                </main>
                {/* Page View Container - always rendered to prevent WebKitGTK DOM insertion flash */}
                <PageViewContainer
                    currentPage={this.state.currentPage}
                    setCurrentPage={this.setCurrentPage}
                    globalSettings={this.state.globalSettings}
                    fetchSettings={this.fetchSettings}
                    downloadQueueState={this.state.downloadQueueState}
                    downloadProgressByJobId={this.state.downloadProgressByJobId}
                    installs={this.state.installs}
                    speedHistory={this.state.speedHistory}
                    onSpeedSample={this.handleSpeedSample}
                    onClearHistory={this.handleClearSpeedHistory}
                    downloadSpeedLimitKB={this.state.globalSettings?.download_speed_limit ?? 0}
                    runners={this.state.runners}
                    installedRunners={this.state.installedRunners}
                    fetchInstalledRunners={this.fetchInstalledRunners}
                    imageVersion={this.state.imageVersion}
                />
                {this.state.showLoadingOverlay && (
                    <AppLoadingScreen
                        progress={this.state.loadingProgress}
                        message={this.state.loadingMessage}
                        fadingOut={this.state.overlayFadingOut}
                        onSkip={() => this.completeInitialLoading()}
                    />
                )}
            </>
        )
    }

    async componentDidMount() {
        // Create network monitor for periodic connectivity checks
        this.networkMonitor = new NetworkMonitor(
            (status: NetworkStatus, isRecovering: boolean) => {
                // Only update state if we're in limited mode and status changes
                if (this.state.limitedMode && status.status === "online" && !isRecovering) {
                    // Auto-close the "Connection Lost" dialog if it's still open
                    closeCurrentDialog();
                    // Auto-trigger recovery when connectivity is restored
                    this.networkMonitor?.triggerRecovery();
                }
            },
            (progress: RecoveryProgress) => {
                this.setState({ recoveryProgress: progress });
                // Increment imageVersion when recovery completes to force image re-loads
                if (progress.phase === "complete") {
                    this.setState((prev: any) => ({ imageVersion: prev.imageVersion + 1 }));
                }
            },
            15000, // Check every 15 seconds
            // Callback when connectivity is lost
            () => {
                // Show a popup when connectivity is lost while using the launcher
                // Don't await - let it show in the background
                showDialogAsync({
                    type: "warning",
                    title: "Connection Lost",
                    message: "Your internet connection was lost. Some features like downloads and updates won't be available until the connection is restored.\n\nThe launcher will automatically reconnect when possible.",
                    buttons: [{ label: "OK", variant: "primary" }],
                });
            }
        );

        // Kick off background-style initial loading via service
        this.loaderController = startInitialLoad({
            fetchSettings: this.fetchSettings,
            fetchRepositories: this.fetchRepositories,
            fetchCompatibilityVersions: this.fetchCompatibilityVersions,
            fetchInstalledRunners: this.fetchInstalledRunners,
            fetchSteamRTStatus: this.fetchSteamRTStatus,
            getGamesInfo: () => this.state.gamesinfo,
            getInstalls: () => this.state.installs,
            preloadImages: (images, onProgress, preloaded) => preloadImages(images, onProgress, preloaded),
            preloadedBackgrounds: this.preloadedBackgrounds,
            setProgress: (progress, message) => this.setState({ loadingProgress: progress, loadingMessage: message }),
            completeInitialLoading: () => this.completeInitialLoading(),
            pushInstalls: this.pushInstalls,
            applyEventState: (ns) => this.setState(ns as any),
            getCurrentInstall: () => this.state.currentInstall,
            fetchInstallResumeStates: this.fetchInstallResumeStates,
        });

        // Set up network monitor with recovery options
        this.networkMonitor.setRecoveryOptions({
            fetchRepositories: this.fetchRepositories,
            fetchCompatibilityVersions: this.fetchCompatibilityVersions,
            fetchInstalledRunners: this.fetchInstalledRunners,
            fetchSteamRTStatus: this.fetchSteamRTStatus,
            getGamesInfo: () => this.state.gamesinfo,
            getInstalls: () => this.state.installs,
            preloadImages: (images, onProgress, preloaded) => preloadImages(images, onProgress, preloaded),
            preloadedBackgrounds: this.preloadedBackgrounds,
            applyEventState: (ns) => this.setState(ns as any),
        });

        // Start monitoring (will auto-recover when connectivity is restored)
        this.networkMonitor.start();
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
        if (this.networkMonitor) {
            this.networkMonitor.stop();
        }
    }

    componentDidUpdate(_prevProps: any, prevState: any) {
        if (this.state.currentInstall && this.state.currentInstall !== prevState.currentInstall) {
            // Capture install ID now — state may change before the deferred callback fires
            const install = this.state.currentInstall;
            // Defer all IPC calls so WebKitGTK can paint the new background before
            // dispatching the 5 webkit.messageHandlers messages (each has synchronous overhead).
            setTimeout(() => {
                Promise.all([
                    this.fetchInstallSettings(install),
                    this.fetchInstallResumeStates(install),
                    this.fetchCompatibilityVersions(),
                    this.fetchInstalledRunners(),
                    this.fetchSteamRTStatus(),
                ]);
                this.updateAvailableBackgrounds();
            }, 0);
        }

        // Update available backgrounds when current game changes (for manifests without installs)
        if (this.state.currentGame && this.state.currentGame !== prevState.currentGame && !this.state.currentInstall) {
            this.updateAvailableBackgrounds();
        }

        // Update available backgrounds when switching from install view to manifest view
        // This handles the case where user clicks on a manifest for a game they have installed
        // (e.g., to see install options) - the install's bg preference shouldn't persist
        if (!this.state.currentInstall && prevState.currentInstall && this.state.currentGame) {
            this.updateAvailableBackgrounds(true); // Force reset to default bg
        }

        // Update available backgrounds when gamesinfo updates (handles late-loading dynamic backgrounds)
        // This ensures dynamic backgrounds show up even if data wasn't ready when user first clicked a game
        if (this.state.gamesinfo !== prevState.gamesinfo && this.state.gamesinfo.length > 0 && (this.state.currentGame || this.state.currentInstall)) {
            this.updateAvailableBackgrounds();
        }

        // Update available backgrounds when installs list changes (handles async pushInstalls completion)
        // This fixes the race condition where setCurrentInstall is called before pushInstalls completes
        if (this.state.installs !== prevState.installs && this.state.currentInstall) {
            const updatedInstall = this.state.installs.find((i: any) => i.id === this.state.currentInstall);
            if (updatedInstall) {
                const updates: any = {};
                if (updatedInstall.name !== this.state.displayName) updates.displayName = updatedInstall.name;
                if (updatedInstall.game_icon !== this.state.gameIcon) updates.gameIcon = updatedInstall.game_icon;
                // Merge fresh install fields into installSettings (syncs version, name, icon, etc.)
                // Only if installSettings belongs to the current install to avoid cross-install contamination
                if (this.state.installSettings?.id === this.state.currentInstall) {
                    updates.installSettings = { ...this.state.installSettings, ...updatedInstall };
                }
                if (Object.keys(updates).length > 0) this.setState(updates);
            }
            this.updateAvailableBackgrounds();
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
                    let g = games.find(g => g.filename.toLowerCase().replace(".json", "") === e.biz.toLowerCase());
                    // @ts-ignore
                    e["manifest_id"] = g.id;
                    // @ts-ignore
                    e["manifest_enabled"] = g.enabled;
                    // @ts-ignore
                    e["manifest_file"] = g.filename;
                });

                this.setState(() => ({ gamesinfo: gi }), () => {
                    // Reset initial loading state after animations complete
                    if (this.state.manifestsInitialLoading && gi.length > 0) {
                        const maxDelay = (gi.length - 1) * 100 + 400; // Last item delay
                        const animationDuration = 600; // slideInLeft duration
                        setTimeout(() => { this.setState({ manifestsInitialLoading: false }); }, maxDelay + animationDuration + 50);
                    }

                    if (this.state.installs.length === 0) {
                        if (games.length > 0 && this.state.currentGame == "") {
                            // Use dynamic background if available (unless on Linux), otherwise fall back to static
                            let bg = (!isLinux && gi[0].assets.game_live_background) || gi[0].assets.game_background;
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
                this.setState(() => ({ installs: gi }), () => {
                    // Also preload installed-specific assets (older/different versions)
                    try {
                        const backgrounds: string[] = (this.state.installs || []).map((i: any) => i?.game_background).filter((s: any) => !!s);
                        const icons: string[] = (this.state.installs || []).map((i: any) => i?.game_icon).filter((s: any) => !!s);
                        const images = Array.from(new Set([...(backgrounds as string[]), ...(icons as string[])]));
                        // Only preload ones we haven't already cached
                        const notPreloaded = images.filter((u) => !this.preloadedBackgrounds.has(u));
                        if (notPreloaded.length > 0) { preloadImages(notPreloaded, undefined, this.preloadedBackgrounds).then(() => { }); }
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
        return invoke("get_install_by_id", { id: install }).then(async data => {
            if (data === null) {
                console.error("Failed to fetch install settings!");
                this.setState(() => ({ installSettings: null, gameManifest: null, preloadAvailable: false, installGameSwitches: {}, installGameFps: [] }));
            } else {
                let parsed = JSON.parse(data as string);
                let md = await this.fetchManifestById(parsed.manifest_id);
                // @ts-ignore
                let isPreload = md.extra.preload['metadata'] !== null;
                // Prepare switches and fps list for SettingsInstall (keep newer fields if present)
                const switches = md?.extra?.switches ?? {};
                const fpsList = Array.isArray(md?.extra?.fps_unlock_options) ? md.extra.fps_unlock_options.map((e: any) => ({ value: `${e}`, name: `${e}` })) : [];
                this.setState(() => ({ installSettings: parsed, gameManifest: md, preloadAvailable: isPreload, installGameSwitches: switches, installGameFps: fpsList }));
            }
        });
    }

    fetchGameVersions(biz: string): Promise<void> {
        return new Promise((resolve) => {
            let game = this.state.gamesinfo.filter((g: any) => g.biz == biz)[0];
            let tmp: { value: any; name: any; background?: string; liveBackground?: string; }[] = [];
            game.game_versions.forEach((g: any) => {
                // Only use version-specific live background if it exists for this version
                // Don't fall back to game's global live background for older versions
                const versionLiveBackground = g.assets?.game_live_background || "";
                const staticBackground = g.assets?.game_background || "";
                tmp.push({
                    value: g.metadata.version,
                    name: (game.latest_version === g.metadata.version) ? `Latest (${g.metadata.version})` : g.metadata.version,
                    background: staticBackground,
                    liveBackground: versionLiveBackground // Only use version-specific, no fallback
                });
            });
            this.setState({ gameVersions: tmp }, resolve);
        });
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
                    e.versions.forEach((v: any) => dxvks.push({ value: v.version, name: v.version }));
                });
                r.filter((e: any) => !e.display_name.toLowerCase().includes("dxvk") && !e.display_name.toLowerCase().includes("wine")).forEach((e: any) => {
                    e.versions.forEach((v: any) => wines.push({ value: v.version, name: v.version }));
                });
                let d = r.filter((e: any) => !e.display_name.toLowerCase().includes("dxvk") && !e.display_name.toLowerCase().includes("wine"));
                this.setState({ runnerVersions: wines, dxvkVersions: dxvks, runners: d });
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
                r.filter((e: any) => e.is_installed).forEach((e: any) => { installed.push(e); });
                this.setState({ installedRunners: installed });
            }
        })
    }


    fetchSteamRTStatus() {
        // Only check on Linux
        if (!window.navigator.platform.includes("Linux")) {
            this.setState({ steamrtInstalled: true });
            return Promise.resolve();
        }
        return invoke<boolean>("is_steamrt_installed").then(installed => {
            this.setState({ steamrtInstalled: installed });
        }).catch(() => {
            this.setState({ steamrtInstalled: false });
        });
    }

    fetchDownloadSizes(biz: any, version: any, lang: any, path: any, region_filter: any, callback: (data: any) => void) {
        invoke("get_download_sizes", { biz: biz, version: version, path: path, lang: lang, region: region_filter }).then(data => {
            if (data === null) {
                console.error("Could not get download sizes!");
            } else {
                const parsed = JSON.parse(data as string);
                callback(parsed);
                this.setState({ downloadSizes: parsed });
            }
        });
    }

    async fetchManifestById(install: any) {
        // Use broad typing since manifest.extra may include optional fields like
        // switches, fps_unlock_options, preload, etc.
        let rslt: any;
        let data = await invoke("get_game_manifest_by_manifest_id", { id: install });
        if (data === null) {
            console.error("Failed to fetch game manifest info!");
            rslt = { latest_version: null, extra: { preload: { metadata: null } } };
        } else {
            rslt = JSON.parse(data as string);
        }
        return rslt;
    }

    fetchInstallResumeStates(install: any) {
        return invoke("get_resume_states", { install: install }).then(async data => {
            if (data === null) {
                console.error("Failed to fetch install resume states!");
                this.setState(() => ({ resumeStates: { downloading: false, updating: false, preloading: false, repairing: false } }));
            } else {
                let parsed = JSON.parse(data as string);
                this.setState(() => ({ resumeStates: parsed }));
            }
        });
    }

    async refreshDownloadButtonInfo(existingInstall: boolean = false) {
        await Promise.all([
            this.fetchGameVersions(this.state.currentGame),
            this.fetchCompatibilityVersions(),
        ]);

        // Fetch download sizes and open popup only after data is ready
        this.fetchDownloadSizes(
            this.state.currentGame,
            this.state.gameVersions[0]?.value,
            "en-us",
            `${this.state.globalSettings.default_game_path}/${this.state.currentGame}`,
            "glb_official",
            () => {
                // Open popup after download sizes are fetched
                this.setState({ openPopup: POPUPS.DOWNLOADGAME, openDownloadAsExisting: existingInstall });
            }
        );
    }

    setOpenPopup(state: POPUPS) { this.setState({ openPopup: state }); }
    setCurrentGame(game: string) { this.setState({ currentGame: game }); }
    setDisplayName(name: string) { this.setState({ displayName: name }); }
    setCurrentPage(page: PAGES) { this.setState({ currentPage: page }); }

    // Handle speed sample from DownloadManager for telemetry graph
    handleSpeedSample(sample: { net: number; disk: number }) {
        this.setState((prev: any) => {
            const history = [...(prev.speedHistory || []), sample];
            // Keep last 60 samples (~60 seconds of data)
            return { speedHistory: history.slice(-60) };
        });
    }

    // Clear speed history when switching to a different download job
    handleClearSpeedHistory() {
        this.setState({ speedHistory: [] });
    }

    // Update available backgrounds for the current game/install
    // forceReset: If true, always reset to default background (used when switching from install to manifest view)
    updateAvailableBackgrounds(forceReset: boolean = false) {
        const backgrounds: { src: string; label: string; isDynamic: boolean }[] = [];
        const seen = new Set<string>();

        const addBg = (src: string, label: string, isDynamic: boolean) => {
            if (src && !seen.has(src)) {
                backgrounds.push({ src, label, isDynamic });
                seen.add(src);
            }
        };

        // If we have a current install, get backgrounds from the install
        if (this.state.currentInstall) {
            const install = this.state.installs.find((i: any) => i.id === this.state.currentInstall);
            if (install) {
                // Try to find the game manifest by ID first, then by title
                let game = this.state.gamesinfo.find((g: any) => g.manifest_id === install.manifest_id);

                if (!game) {
                    game = this.state.gamesinfo.find((g: any) =>
                        g.display_name === install.name ||
                        g.game_versions?.some((v: any) => v.metadata?.versioned_name === install.name)
                    );
                }

                if (!game) {
                    console.warn(`Could not find game manifest for install: ${install.name} (${install.manifest_id})`);
                }

                if (game) {
                    // Add dynamic background if available (skip on Linux)
                    if (!isLinux && game.assets?.game_live_background) {
                        addBg(game.assets.game_live_background, "Dynamic", true);
                    }
                    // Add static background from game manifest
                    if (game.assets?.game_background) {
                        addBg(game.assets.game_background, "Static", false);
                    }
                }

                // If the install has a stored background that isn't in the current manifest
                // (e.g. an outdated/imported version), clear manifest backgrounds and show only
                // the install's own background — no navigation alternatives for outdated installs
                if (install.game_background && !backgrounds.some(b => b.src === install.game_background)) {
                    backgrounds.length = 0;
                    seen.clear();
                    addBg(install.game_background, "Static", false);
                }
            }
        } else if (this.state.currentGame) {
            // If no install, get backgrounds from the current game manifest
            const game = this.state.gamesinfo.find((g: any) => g.biz === this.state.currentGame);
            if (game) {
                // Add dynamic background if available (skip on Linux)
                if (!isLinux && game.assets?.game_live_background) {
                    addBg(game.assets.game_live_background, "Dynamic", true);
                }
                if (game.assets?.game_background) {
                    addBg(game.assets.game_background, "Static", false);
                }
            }
        }

        // Sort: Dynamic first
        backgrounds.sort((a, b) => (a.isDynamic === b.isDynamic ? 0 : a.isDynamic ? -1 : 1));

        // Increment version so older racing calls become stale
        const updateVersion = ++this.bgUpdateVersion;

        this.setState({ availableBackgrounds: backgrounds }, () => {
            // If a newer updateAvailableBackgrounds call happened, skip this stale callback
            if (updateVersion !== this.bgUpdateVersion) return;

            // Check if the user has a saved preference for this install
            const install = this.state.currentInstall
                ? this.state.installs.find((i: any) => i.id === this.state.currentInstall)
                : null;

            if (install?.game_background && !forceReset) {
                // Use the install's stored game_background if it's in the available list
                const installedBg = backgrounds.find(b => b.src === install.game_background);
                if (installedBg && this.state.gameBackground !== installedBg.src) {
                    this.setBackground(installedBg.src);
                } else if (!installedBg && backgrounds.length > 0) {
                    // Stored bg not in available list (version mismatch) - fall back to first available
                    const currentBgInList = backgrounds.some(b => b.src === this.state.gameBackground);
                    if (!currentBgInList) { this.setBackground(backgrounds[0].src); }
                }
            } else if (backgrounds.length > 0) {
                // No saved preference (or forceReset) - use the first available background (dynamic preferred due to sorting)
                // Always set if current background isn't in the new game's available list, or if forceReset is true
                const currentBgInList = backgrounds.some(b => b.src === this.state.gameBackground);
                if (forceReset || !currentBgInList) {
                    this.setBackground(backgrounds[0].src);
                }
            }
        });
    }

    // Store the background transition timeout
    bgTransitionTimeout?: number;
    // Guard against racing updateAvailableBackgrounds calls
    bgUpdateVersion: number = 0;
    // Track the latest intended background to prevent stale setState guard checks
    pendingGameBackground: string = "";

    setBackground(file: string, savePreference: boolean = false) {
        if (!file || (file === this.state.gameBackground && file === this.pendingGameBackground)) return; // nothing to do
        this.pendingGameBackground = file;

        // Cancel any previous transition timeout
        if (this.bgTransitionTimeout) {
            clearTimeout(this.bgTransitionTimeout);
            this.bgTransitionTimeout = undefined;
        }

        // Save user preference if this is a manual change
        if (savePreference && this.state.currentInstall) {
            invoke("update_install_game_background", {
                id: this.state.currentInstall,
                background: file
            }).catch(console.error);

            // Also update the local state so the preference is immediately reflected
            this.setState((prev: any) => ({
                installs: prev.installs.map((i: any) =>
                    i.id === this.state.currentInstall
                        ? { ...i, game_background: file }
                        : i
                )
            }));
        }

        // Only show loading state if the image isn't already preloaded
        // This prevents the brief flash of loading gradient on WebKitGTK when switching
        // between already-cached backgrounds
        const needsLoading = !isImagePreloaded(file);

        // Start loading: show gradient on the new image while it loads
        this.setState((prev: any) => ({
            bgLoading: needsLoading,
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

    // Wrapper for user-initiated background changes (saves preference)
    handleBackgroundChange = (file: string) => {
        this.setBackground(file, true);
    };
    setGameIcon(file: string) { this.setState({ gameIcon: file }); }
    setReposList(reposList: any) { this.setState({ reposList: reposList }); }
    setCurrentInstall(game: string) { this.setState({ currentInstall: game }); }

    // Drag and drop handlers for sidebar install reordering
    handleDragStart = (index: number) => {
        this.setState({ dragIndex: index, dragTargetIndex: null, dropCompleted: false });
    };

    handleDragEnd = () => {
        // Only reset if drop wasn't completed (i.e., drag was cancelled)
        if (!this.state.dropCompleted) {
            this.setState({ dragIndex: null, dragTargetIndex: null });
        }
        // Reset the flag for next drag operation
        this.setState({ dropCompleted: false });
    };

    handleDragOver = (index: number) => {
        const { dragIndex } = this.state;
        if (dragIndex === null || dragIndex === index) return;

        // Don't show indicator if the drop would result in the same position
        // When dragging down to the immediately next item, the result is the same position
        if (dragIndex < index && index === dragIndex + 1) {
            this.setState({ dragTargetIndex: null });
            return;
        }

        this.setState({ dragTargetIndex: index });
    };

    handleDrop = (targetIndex: number) => {
        const { dragIndex, installs } = this.state;
        if (dragIndex === null || dragIndex === targetIndex) {
            this.setState({ dragIndex: null, dragTargetIndex: null });
            return;
        }

        // When dragging down to immediately next item, it's a no-op
        if (dragIndex < targetIndex && targetIndex === dragIndex + 1) {
            this.setState({ dragIndex: null, dragTargetIndex: null });
            return;
        }

        // Reorder the installs array
        const newInstalls = [...installs];
        const [draggedItem] = newInstalls.splice(dragIndex, 1);

        // When dragging down, the target index shifts after removal
        // Adjust insertion index so item lands where the indicator showed
        const insertIndex = dragIndex < targetIndex ? targetIndex - 1 : targetIndex;
        newInstalls.splice(insertIndex, 0, draggedItem);

        // Update state immediately for responsive UI with pop animation
        this.setState({ installs: newInstalls, dragIndex: null, dragTargetIndex: null, dropCompleted: true, droppedItemId: draggedItem.id });
        // Clear the dropped item ID after animation completes
        setTimeout(() => this.setState({ droppedItemId: null }), 250);

        // Persist the new order to backend
        const orderUpdates: [string, number][] = newInstalls.map((install: any, idx: number) => [install.id, idx]);
        invoke("set_installs_order", { order: orderUpdates }).catch((e) => {
            console.error("Failed to persist install order:", e);
        });
    };

    handleDropAtEnd = () => {
        const { dragIndex, installs } = this.state;
        if (dragIndex === null) {
            this.setState({ dragIndex: null, dragTargetIndex: null, dropCompleted: true });
            return;
        }

        // If already at the end, no-op
        if (dragIndex === installs.length - 1) {
            this.setState({ dragIndex: null, dragTargetIndex: null, dropCompleted: true });
            return;
        }

        // Move item to end
        const newInstalls = [...installs];
        const [draggedItem] = newInstalls.splice(dragIndex, 1);
        newInstalls.push(draggedItem);

        // Update state immediately for responsive UI with pop animation
        this.setState({ installs: newInstalls, dragIndex: null, dragTargetIndex: null, dropCompleted: true, droppedItemId: draggedItem.id });
        // Clear the dropped item ID after animation completes
        setTimeout(() => this.setState({ droppedItemId: null }), 250);

        // Persist the new order to backend
        const orderUpdates: [string, number][] = newInstalls.map((install: any, idx: number) => [install.id, idx]);
        invoke("set_installs_order", { order: orderUpdates }).catch((e) => {
            console.error("Failed to persist install order:", e);
        });
    };
}
