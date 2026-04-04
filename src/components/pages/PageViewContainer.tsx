import { PAGES } from "./PAGES";
import SettingsPage from "./SettingsPage";
import DownloadsPage from "./DownloadsPage";
import RunnersPage from "./RunnersPage";
import { useEffect } from "react";
import type { DownloadJobProgress, DownloadQueueStatePayload } from "../../types/downloadQueue";

interface TelemetrySample {
    net: number;
    disk: number;
}

interface InstallView {
    id: string;
    name?: string;
    game_icon?: string;
    game_background?: string;
}

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

interface PageViewContainerProps {
    currentPage: PAGES;
    setCurrentPage: (page: PAGES) => void;

    // Settings props
    globalSettings: any;
    fetchSettings: () => void;

    // Downloads props
    downloadQueueState: DownloadQueueStatePayload | null;
    downloadProgressByJobId: Record<string, DownloadJobProgress>;
    installs: InstallView[];
    speedHistory: TelemetrySample[];
    onSpeedSample: (sample: TelemetrySample) => void;
    onClearHistory: () => void;
    downloadSpeedLimitKB: number;

    // Runners props
    runners: RunnerManifest[];
    installedRunners: InstalledRunner[];
    fetchInstalledRunners: () => void;
    pushInstalls: () => void;

    // Network recovery
    imageVersion?: number;
}

export default function PageViewContainer({
    currentPage,
    setCurrentPage,
    globalSettings,
    fetchSettings,
    downloadQueueState,
    downloadProgressByJobId,
    installs,
    speedHistory,
    onSpeedSample,
    onClearHistory,
    downloadSpeedLimitKB,
    runners,
    installedRunners,
    fetchInstalledRunners,
    pushInstalls,
    imageVersion = 0,
}: PageViewContainerProps) {
    const isOpen = currentPage !== PAGES.NONE;

    useEffect(() => {
        if (!isOpen) return;

        const onKeyDown = (event: KeyboardEvent) => {
            if (event.key !== "Escape" || event.repeat) return;
            setCurrentPage(PAGES.NONE);
        };

        document.addEventListener("keydown", onKeyDown);
        return () => {
            document.removeEventListener("keydown", onKeyDown);
        };
    }, [isOpen, setCurrentPage]);

    return (
        <div
            className={`absolute inset-0 left-16 z-30 flex flex-col overflow-hidden border-l border-white/10 ${isOpen ? '' : 'pointer-events-none'}`}
            style={{
                backfaceVisibility: 'hidden',
                WebkitBackfaceVisibility: 'hidden',
                transform: 'translateZ(0)',
                // Use visibility instead of opacity for instant show/hide without compositor flash
                visibility: isOpen ? 'visible' : 'hidden',
                background: 'rgba(0,0,0,0.5)'
            }}
        >
            {/* Animated content wrapper for smooth page transitions */}
            <div
                className="absolute inset-0 transition-all duration-300 ease-out animate-slideInRight"
                style={{
                    opacity: isOpen ? 1 : 0,
                    transform: isOpen ? 'translateX(0) scale(1)' : 'translateX(20px) scale(0.98)',
                    willChange: 'opacity, transform',
                    // Coordinated timing: page starts after background, closes before background
                    transitionDelay: isOpen ? '50ms' : '0ms'
                }}
            >
                {currentPage === PAGES.SETTINGS && (
                    <SettingsPage
                        settings={globalSettings}
                        fetchSettings={fetchSettings}
                        setCurrentPage={setCurrentPage}
                    />
                )}
                {currentPage === PAGES.DOWNLOADS && (
                    <DownloadsPage
                        setCurrentPage={setCurrentPage}
                        queue={downloadQueueState}
                        progressByJobId={downloadProgressByJobId}
                        installs={installs}
                        speedHistory={speedHistory}
                        onSpeedSample={onSpeedSample}
                        onClearHistory={onClearHistory}
                        downloadSpeedLimitKB={downloadSpeedLimitKB}
                        imageVersion={imageVersion}
                    />
                )}
                {currentPage === PAGES.RUNNERS && (
                    <RunnersPage
                        setCurrentPage={setCurrentPage}
                        runners={runners}
                        installedRunners={installedRunners}
                        fetchInstalledRunners={fetchInstalledRunners}
                        pushInstalls={pushInstalls}
                        downloadQueueState={downloadQueueState}
                        downloadProgressByJobId={downloadProgressByJobId}
                    />
                )}
            </div>
        </div>
    );
}
