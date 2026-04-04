import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, AtomIcon, DownloadCloud, FolderOpen, Trash2, Check } from "lucide-react";
import { PAGES } from "./PAGES";
import { SettingsSidebar, SettingsTab } from "../sidebar/SettingsSidebar.tsx";
import { SettingsSection } from "../common/SettingsComponents.tsx";
import type { DownloadJobProgress, DownloadQueueStatePayload } from "../../types/downloadQueue";

interface RunnerVersion {
    version: string;
    url: string;
}

interface RunnerManifest {
    display_name: string;
    versions: RunnerVersion[];
}

interface InstalledRunner {
    version: string;
    is_installed: boolean;
}

interface RunnersPageProps {
    setCurrentPage: (page: PAGES) => void;
    runners: RunnerManifest[];
    installedRunners: InstalledRunner[];
    fetchInstalledRunners: () => void;
    pushInstalls: () => void;
    downloadQueueState: DownloadQueueStatePayload | null;
    downloadProgressByJobId: Record<string, DownloadJobProgress>;
}

function RunnerItem({
    version,
    isInstalled,
    isDownloading,
    isQueued,
    progress,
    onInstall,
    onRemove,
    onOpenFolder,
    isLastInstalled = false,
}: {
    version: string;
    isInstalled: boolean;
    isDownloading: boolean;
    isQueued: boolean;
    progress?: DownloadJobProgress;
    onInstall: () => void;
    onRemove: () => void;
    onOpenFolder: () => void;
    isLastInstalled?: boolean;
}) {
    const [isLoading, setIsLoading] = useState(false);

    const handleInstall = async () => {
        setIsLoading(true);
        try {
            await onInstall();
        } finally {
            setIsLoading(false);
        }
    };

    const handleRemove = async () => {
        setIsLoading(true);
        try {
            await onRemove();
        } finally {
            setIsLoading(false);
        }
    };

    const progressPercent = (progress?.progress && progress?.total && progress.total > 0) ? Math.round((progress.progress / progress.total) * 100) : 0;
    const downloading = isDownloading || isQueued;

    return (
        <div className="flex items-center justify-between px-4 py-3 rounded-lg bg-zinc-900/60 hover:bg-zinc-800/80 border border-white/5 hover:border-white/10 transition-all duration-200 group">
            <div className="flex items-center gap-3">
                <div className={`w-2 h-2 rounded-full ${isInstalled ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' : downloading ? 'bg-purple-500 shadow-[0_0_8px_rgba(147,51,234,0.5)] animate-pulse' : 'bg-zinc-600'}`} />
                <span className="text-white/90 text-sm font-medium">{version}</span>
            </div>
            <div className="flex items-center gap-2">
                {isInstalled ? (
                    <>
                        <button
                            onClick={onOpenFolder}
                            className="p-2 rounded-lg text-white/50 hover:text-purple-400 hover:bg-purple-500/10 transition-all duration-200"
                            title="Open folder"
                        >
                            <FolderOpen className="w-4 h-4" />
                        </button>
                        <button
                            onClick={handleRemove}
                            disabled={isLoading || isLastInstalled}
                            className="p-2 rounded-lg text-white/50 hover:text-red-400 hover:bg-red-500/10 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:text-white/50 disabled:hover:bg-transparent"
                            title={isLastInstalled ? "Cannot remove the last installed runner" : "Remove"}
                        >
                            <Trash2 className="w-4 h-4" />
                        </button>
                        <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
                            <Check className="w-3.5 h-3.5 text-emerald-400" />
                            <span className="text-xs text-emerald-400 font-medium">Installed</span>
                        </div>
                    </>
                ) : downloading ? (
                    <div className="relative flex items-center gap-2 px-3 py-1.5 rounded-lg border border-purple-500/30 overflow-hidden">
                        {/* Fill background */}
                        <div
                            className="absolute inset-0 bg-purple-500/25 transition-all duration-300 ease-out"
                            style={{ width: isDownloading && progressPercent > 0 ? `${progressPercent}%` : isQueued ? '0%' : '100%' }}
                        />
                        <svg className="relative animate-spin w-4 h-4 text-purple-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                            <circle cx="12" cy="12" r="10" strokeOpacity="0.25" />
                            <path d="M12 2a10 10 0 0 1 10 10" strokeLinecap="round" />
                        </svg>
                        <span className="relative text-xs font-medium text-purple-300">
                            {isQueued ? 'Queued' : progressPercent > 0 ? `${progressPercent}%` : 'Installing...'}
                        </span>
                    </div>
                ) : (
                    <button
                        onClick={handleInstall}
                        disabled={isLoading}
                        className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-purple-500/20 hover:bg-purple-500/30 border border-purple-500/30 text-purple-300 hover:text-purple-200 transition-all duration-200 disabled:opacity-50"
                    >
                        {isLoading ? (
                            <svg className="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                                <circle cx="12" cy="12" r="10" strokeOpacity="0.25" />
                                <path d="M12 2a10 10 0 0 1 10 10" strokeLinecap="round" />
                            </svg>
                        ) : (
                            <DownloadCloud className="w-4 h-4" />
                        )}
                        <span className="text-xs font-medium">Install</span>
                    </button>
                )}
            </div>
        </div>
    );
}

export default function RunnersPage({
    setCurrentPage,
    runners,
    installedRunners,
    fetchInstalledRunners,
    pushInstalls,
    downloadQueueState,
    downloadProgressByJobId,
}: RunnersPageProps) {
    const runningJobs = downloadQueueState?.running || [];
    const queuedJobs = downloadQueueState?.queued || [];
    // Generate tabs from runner manifests
    const tabs: SettingsTab[] = useMemo(() => {
        return runners.map((runner, index) => ({
            id: runner.display_name.toLowerCase().replace(/\s+/g, '-'),
            label: runner.display_name,
            icon: AtomIcon, // We could use different icons if we had map, but Atom is good for generic runner
            color: ["purple", "blue", "green", "orange", "pink", "yellow"][index % 6], // Cycle colors
            // Store original name to find data easily
            _originalName: runner.display_name
        }));
    }, [runners]);

    const [activeTab, setActiveTab] = useState<string>("");

    // Set initial tab 
    useEffect(() => {
        if (tabs.length > 0 && !activeTab) {
            setActiveTab(tabs[0].id);
        }
    }, [tabs, activeTab]);

    // Track animation class state for transitions
    const [animClass, setAnimClass] = useState("animate-fadeIn");

    const handleTabChange = (newTabId: string) => {
        const oldIndex = tabs.findIndex(t => t.id === activeTab);
        const newIndex = tabs.findIndex(t => t.id === newTabId);
        const direction = newIndex > oldIndex ? "animate-slideUp" : "animate-slideDown";

        setAnimClass(direction);
        setActiveTab(newTabId);
    };

    // Calculate totals
    const totalVersions = runners.reduce((sum, r) => sum + r.versions.length, 0);
    const totalInstalled = installedRunners.filter(r => r.is_installed).length;

    // Find active runner data
    const activeRunner = runners.find(r =>
        r.display_name.toLowerCase().replace(/\s+/g, '-') === activeTab
    );

    return (
        <div
            className="flex-1 flex flex-col h-full overflow-hidden animate-fadeIn"
            style={{ willChange: 'opacity', backfaceVisibility: 'hidden', transform: 'translateZ(0)' }}
        >
            {/* Page Header */}
            <div className="flex items-center gap-4 px-8 py-5 border-b border-white/5">
                <button
                    onClick={() => setCurrentPage(PAGES.NONE)}
                    className="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 hover:border-white/10 transition-all duration-200 hover:scale-105 hover:shadow-[0_0_12px_rgba(147,51,234,0.15)] active:scale-95"
                >
                    <ArrowLeft className="w-5 h-5 text-white/70" />
                </button>
                <div className="flex items-center gap-4">
                    <div className="p-3 bg-purple-500/15 rounded-xl border border-purple-500/20 shadow-[0_0_15px_rgba(147,51,234,0.2)]">
                        <AtomIcon className="w-6 h-6 text-purple-400" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-bold bg-gradient-to-r from-white to-white/70 bg-clip-text text-transparent">
                            Runner Manager
                        </h1>
                        <p className="text-sm text-white/50">
                            {totalInstalled} installed / {totalVersions} available versions
                        </p>
                    </div>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex flex-1 overflow-hidden">
                {runners.length > 0 ? (
                    <>
                        <SettingsSidebar tabs={tabs} activeTab={activeTab} onTabChange={handleTabChange} />

                        {/* Content Area */}
                        <div
                            key={activeTab}
                            className={`flex-1 overflow-y-auto p-8 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent ${animClass}`}
                        >
                            {activeRunner && (
                                <SettingsSection title={`${activeRunner.display_name} Versions`}>
                                    <div className="flex flex-col gap-2">
                                        {activeRunner.versions.map((v) => {
                                            const isInstalled = installedRunners.some(
                                                r => r.version === v.version && r.is_installed
                                            );
                                            const runningJob = runningJobs.find(j => j.kind === 'runner_download' && j.installId === v.version);
                                            const isDownloading = !!runningJob;
                                            const isQueued = queuedJobs.some(j => j.kind === 'runner_download' && j.installId === v.version);
                                            const progress = runningJob ? downloadProgressByJobId[runningJob.id] || downloadProgressByJobId[v.version] : undefined;
                                            return (
                                                <RunnerItem
                                                    key={v.version}
                                                    version={v.version}
                                                    isInstalled={isInstalled}
                                                    isDownloading={isDownloading}
                                                    isQueued={isQueued}
                                                    progress={progress}
                                                    isLastInstalled={isInstalled && totalInstalled <= 1}
                                                    onInstall={async () => {
                                                        await invoke("add_installed_runner", {
                                                            runnerUrl: v.url,
                                                            runnerVersion: v.version
                                                        });
                                                        fetchInstalledRunners();
                                                    }}
                                                    onRemove={async () => {
                                                        await invoke("remove_installed_runner", {
                                                            runnerVersion: v.version
                                                        });
                                                        fetchInstalledRunners();
                                                        pushInstalls();
                                                    }}
                                                    onOpenFolder={() => {
                                                        invoke("open_folder", {
                                                            runnerVersion: v.version,
                                                            manifestId: "",
                                                            installId: "",
                                                            pathType: "runner_global"
                                                        });
                                                    }}
                                                />
                                            );
                                        })}
                                    </div>
                                </SettingsSection>
                            )}
                        </div>
                    </>
                ) : (
                    <div className="flex flex-col items-center justify-center w-full h-full text-center">
                        <AtomIcon className="w-16 h-16 text-white/20 mb-4" />
                        <h3 className="text-lg font-medium text-white/70">No runners available</h3>
                        <p className="text-sm text-white/40 mt-2">
                            Runner versions will appear here when available
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
}
