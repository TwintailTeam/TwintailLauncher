import { useEffect, useState, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { invoke } from '@tauri-apps/api/core';
import type {
    DownloadJobProgress,
    DownloadPhase,
    DownloadQueueStatePayload,
    QueueJobView,
} from '../../types/downloadQueue';
import { formatBytes, toPercent } from '../../utils/progress';
import { ArrowLeft, DownloadCloud } from "lucide-react";
import { PAGES } from './PAGES';
import { CachedImage } from '../common/CachedImage';

/* Telemetry sample for graph */
interface TelemetrySample {
    net: number;
    disk: number;
}

/* Install view for banner display */
interface InstallView {
    id: string;
    name?: string;
    game_icon?: string;
    game_background?: string;
}

/* Props for DownloadsPage */
interface DownloadsPageProps {
    setCurrentPage: (page: PAGES) => void;
    queue: DownloadQueueStatePayload | null;
    progressByJobId: Record<string, DownloadJobProgress>;
    installs: InstallView[];
    speedHistory: TelemetrySample[];
    onSpeedSample: (sample: TelemetrySample) => void;
    onClearHistory: () => void;
    downloadSpeedLimitKB: number;
    imageVersion?: number; // Used to force image re-load after network recovery
}

/* Format speed in bytes per second */
const formatSpeed = (bytesPerSecond: number): string => {
    if (!Number.isFinite(bytesPerSecond) || bytesPerSecond <= 0) return '0 B/s';
    return `${formatBytes(bytesPerSecond)}/s`;
};

/* Format time duration */
const formatTime = (seconds: number): string => {
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ${seconds % 60}s`;
    const hours = Math.floor(minutes / 60);
    return `${hours}h ${minutes % 60}m`;
};

const INSTALL_ETA_PHASES: DownloadPhase[] = ['installing', 'extracting', 'validating', 'moving'];
const MAX_ETA_SECONDS = 60 * 60 * 24 * 7; // Hide unrealistic ETAs (>7 days)

/* Calculate ETA using average of recent speeds */
const calculateETA = (
    downloadTotalBytes: number,
    downloadProgressBytes: number,
    installTotalBytes: number,
    installProgressBytes: number,
    speedHistory: TelemetrySample[],
    phase: DownloadPhase | undefined,
    preferInstallETA: boolean
): string => {
    if (speedHistory.length === 0) return '—';

    const useInstallProgress = installTotalBytes > 0 && (preferInstallETA || (phase !== undefined && INSTALL_ETA_PHASES.includes(phase)));
    const totalBytes = useInstallProgress ? installTotalBytes : downloadTotalBytes;
    const progressBytes = useInstallProgress ? installProgressBytes : downloadProgressBytes;
    const remaining = Math.max(totalBytes - progressBytes, 0);
    if (remaining === 0) return '—';

    // Use last 10 samples (or all if less than 10), and ignore zero/invalid samples.
    const recentSamples = speedHistory.slice(-10);
    const sampleSpeeds = recentSamples
        .map((sample) => useInstallProgress ? Math.max(sample.disk, sample.net) : sample.net)
        .filter((speed) => Number.isFinite(speed) && speed > 0);
    if (sampleSpeeds.length === 0) return '—';

    const avgSpeed = sampleSpeeds.reduce((sum, speed) => sum + speed, 0) / sampleSpeeds.length;
    if (!Number.isFinite(avgSpeed) || avgSpeed <= 0) return '—';

    const seconds = Math.ceil(remaining / avgSpeed);
    if (!Number.isFinite(seconds) || seconds > MAX_ETA_SECONDS) return '—';
    return formatTime(seconds);
};

/* Format kind label */
function formatKind(kind: QueueJobView['kind']): string {
    switch (kind) {
        case 'game_download': return 'Game';
        case 'game_update': return 'Update';
        case 'game_preload': return 'Predownload';
        case 'game_repair': return 'Repair';
        case 'runner_download': return 'Runner';
        case 'steamrt_download': return 'SteamRT';
        case 'steamrt4_download': return 'SteamRT';
        case 'xxmi_download': return 'XXMI';
        case 'extras_download': return 'Extra';
        default: return 'Download';
    }
}

/* Format status label */
function formatStatus(status: QueueJobView['status'], isPaused: boolean): string {
    if (isPaused && status === 'running') return 'Paused';
    switch (status) {
        case 'queued': return 'Queued';
        case 'running': return 'Downloading';
        case 'completed': return 'Completed';
        case 'failed': return 'Failed';
        case 'cancelled': return 'Paused';
        case 'paused': return 'Paused';
    }
}

/* Format download phase - only Verifying (when resuming) or Downloading */
function formatDownloadPhase(phase: DownloadPhase | undefined): string {
    return phase === 'verifying' ? 'Verifying' : 'Downloading';
}

/* Format install phase substatus */
function formatInstallPhase(phase: DownloadPhase | undefined): string {
    switch (phase) {
        case 'validating': return 'Validating';
        case 'moving': return 'Moving';
        case 'extracting': return 'Extracting';
        default: return 'Installing';
    }
}

/* Get download phase color class */
function getDownloadPhaseColor(phase: DownloadPhase | undefined): string {
    return phase === 'verifying' ? 'text-yellow-400' : 'text-blue-400';
}

/* Get install phase color class */
function getInstallPhaseColor(phase: DownloadPhase | undefined): string {
    switch (phase) {
        case 'validating': return 'text-yellow-400';
        case 'moving': return 'text-purple-400';
        case 'extracting': return 'text-cyan-400';
        default: return 'text-green-400';
    }
}

/**
 * Downloads Page - Full-page view for download progress, graph, and queue
 */
export default function DownloadsPage({
    setCurrentPage,
    queue,
    progressByJobId,
    installs,
    speedHistory,
    onSpeedSample,
    onClearHistory,
    downloadSpeedLimitKB,
    imageVersion = 0,
}: DownloadsPageProps) {
    // Canvas ref for graph
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [canvasSize, setCanvasSize] = useState({ width: 0, height: 0 });

    // Ref to track last sampled speed to avoid duplicate samples
    const lastSampleRef = useRef<{ net: number; disk: number; time: number } | null>(null);
    const progressSnapshotRef = useRef<{ downloadBytes: number; installBytes: number; time: number } | null>(null);

    // Hover state for graph tooltip
    const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
    const [mousePos, setMousePos] = useState<{ x: number; y: number } | null>(null);

    // Use completed items from backend queue state
    const completedItems = queue?.completed ?? [];

    // Peak speed tracking
    const [peakSpeed, setPeakSpeed] = useState<number>(0);

    // Track previous job ID to detect job changes and reset history
    const previousJobIdRef = useRef<string | null>(null);

    // Drag and drop state
    const [draggedJobId, setDraggedJobId] = useState<string | null>(null);
    const [dragOverTarget, setDragOverTarget] = useState<string | null>(null);

    // Get running, queued, and paused jobs from backend
    const runningJobs = queue?.running ?? [];
    const queuedJobs = queue?.queued ?? [];
    const pausedJobs = queue?.pausedJobs ?? [];
    const pausingInstalls = queue?.pausingInstalls ?? [];
    const isQueuePaused = queue?.paused ?? false;
    const allJobs = [...runningJobs, ...queuedJobs];

    // Current download - either from running queue or paused jobs from backend
    const pausedJob = pausedJobs[0] ?? null;
    const currentJob = runningJobs[0] ?? pausedJob ?? null;
    const currentProgress = currentJob
        ? progressByJobId[currentJob.id] ?? null
        : null;

    // Derive pausing state from backend (persists across navigation)
    const isPausing = currentJob ? pausingInstalls.includes(currentJob.installId) : false;

    // Derive paused state - job is paused if it's in pausedJobs or queue is paused
    const isPaused = isQueuePaused || currentJob?.status === 'paused' || (pausedJob !== null && runningJobs.length === 0);

    // Calculate progress values
    const progressBytes = currentProgress?.progress ?? 0;
    const totalBytes = currentProgress?.total ?? 0;
    const downloadProgress = totalBytes > 0 ? toPercent(progressBytes, totalBytes) : 0;
    const currentSpeed = currentProgress?.speed ?? 0;
    const currentDisk = currentProgress?.disk ?? 0;
    const latestSample = speedHistory.length > 0 ? speedHistory[speedHistory.length - 1] : undefined;
    const displayNetSpeed = Math.max(currentSpeed, latestSample?.net ?? 0);
    const displayDiskSpeed = Math.max(currentDisk, latestSample?.disk ?? 0);

    // Installation progress (for extraction/verification phase)
    const installProgressBytes = currentProgress?.installProgress ?? 0;
    const installTotalBytes = currentProgress?.installTotal ?? 0;
    const installProgress = installTotalBytes > 0 ? toPercent(installProgressBytes, installTotalBytes) : 0;
    const hasInstallProgress = installTotalBytes > 0;

    // Current phase/status
    const currentPhase = currentProgress?.phase;
    const etaText = calculateETA(totalBytes, progressBytes, installTotalBytes, installProgressBytes, speedHistory, currentPhase, currentJob?.kind === 'game_repair');


    // Reset graph history and peak speed when active job changes
    useEffect(() => {
        const currentJobId = currentJob?.id ?? null;
        if (previousJobIdRef.current !== null && currentJobId !== previousJobIdRef.current) {
            onClearHistory();
            setPeakSpeed(0);
            lastSampleRef.current = null;
            progressSnapshotRef.current = null;
            onSpeedSample({ net: 0, disk: 0 });
        }
        previousJobIdRef.current = currentJobId;
    }, [currentJob?.id, onClearHistory, onSpeedSample]);

    // Sample speed/disk for graph when we have a running job
    useEffect(() => {
        if (!currentJob || isPaused) return;

        const now = Date.now();
        const last = lastSampleRef.current;
        if (last && now - last.time < 900) return;

        const snapshot = progressSnapshotRef.current;
        const isDownloadingPhase = currentPhase === 'downloading';
        let derivedNet = 0;
        let derivedDisk = 0;
        if (snapshot && now > snapshot.time) {
            const elapsedSeconds = (now - snapshot.time) / 1000;
            const downloadDelta = Math.max(0, progressBytes - snapshot.downloadBytes);
            const installDelta = Math.max(0, installProgressBytes - snapshot.installBytes);
            if (elapsedSeconds > 0) {
                // Only infer network throughput from progress deltas in real download phase.
                // Repair verify/validate can move download progress without network traffic.
                if (isDownloadingPhase) derivedNet = Math.round(downloadDelta / elapsedSeconds);
                derivedDisk = Math.round(installDelta / elapsedSeconds);
            }
        }

        progressSnapshotRef.current = { downloadBytes: progressBytes, installBytes: installProgressBytes, time: now };
        const sampledNet = currentSpeed > 0 ? currentSpeed : (isDownloadingPhase ? derivedNet : 0);
        const sampledDisk = currentDisk > 0 ? currentDisk : derivedDisk;

        lastSampleRef.current = { net: sampledNet, disk: sampledDisk, time: now };
        onSpeedSample({ net: sampledNet, disk: sampledDisk });
        setPeakSpeed(prev => Math.max(prev, sampledNet));
    }, [currentSpeed, currentDisk, progressBytes, installProgressBytes, currentPhase, currentJob, isPaused, onSpeedSample]);

    // Draw canvas graph
    const GRAPH_SLOTS = 60;

    // Handle Resize
    useEffect(() => {
        if (!containerRef.current) return;

        const observer = new ResizeObserver((entries) => {
            const entry = entries[0];
            if (entry) {
                // Use contentBoxSize if available for better precision, fallback to contentRect
                const width = entry.contentRect.width;
                const height = entry.contentRect.height;
                setCanvasSize({ width, height });
            }
        });

        observer.observe(containerRef.current);
        return () => observer.disconnect();
    }, []);

    useEffect(() => {
        if (!canvasRef.current || canvasSize.width === 0 || canvasSize.height === 0) return;

        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        // Handle high-DPI displays
        const dpr = window.devicePixelRatio || 1;

        // Set actual canvas size (resolution)
        canvas.width = canvasSize.width * dpr;
        canvas.height = canvasSize.height * dpr;

        // Scale context to match
        ctx.resetTransform();
        ctx.scale(dpr, dpr);

        // Logical dimensions for drawing
        const width = canvasSize.width;
        const height = canvasSize.height;

        const paddedHistory: { net: number; disk: number }[] = [];
        const emptySlots = GRAPH_SLOTS - speedHistory.length;
        for (let i = 0; i < emptySlots; i++) {
            paddedHistory.push({ net: 0, disk: 0 });
        }
        paddedHistory.push(...speedHistory);

        const minGraphScale = 128 * 1024;
        const maxNet = Math.max(...paddedHistory.map(s => s.net), minGraphScale);
        const maxDisk = Math.max(...paddedHistory.map(s => s.disk), minGraphScale);
        const maxValue = Math.max(maxNet, maxDisk);

        ctx.clearRect(0, 0, width, height);

        const barWidth = width / GRAPH_SLOTS;
        const fadeLeftAlpha = 0.05;
        const fadeRightAlpha = 1.0;
        const fadeExponent = 1.6;

        // Draw network bars
        paddedHistory.forEach((sample, index) => {
            const x = barWidth * index;
            const barHeight = (sample.net / maxValue) * height;
            const y = height - barHeight;

            const actualIndex = index - emptySlots;
            const isHighlighted = hoveredIndex !== null && actualIndex === hoveredIndex;
            const t = index / (GRAPH_SLOTS - 1);
            const tAdjusted = Math.pow(t, fadeExponent);
            const alpha = isHighlighted ? 0.98 : (fadeLeftAlpha + tAdjusted * (fadeRightAlpha - fadeLeftAlpha));

            ctx.fillStyle = `rgba(59, 130, 246, ${alpha})`;
            // Used Math.floor/ceil to avoid subpixel gaps
            ctx.fillRect(x, y, barWidth - 1, barHeight);
        });

        // Draw disk line with gradient
        const diskGradient = ctx.createLinearGradient(0, 0, width, 0);
        diskGradient.addColorStop(0, 'rgba(16,185,129,0.12)');
        diskGradient.addColorStop(0.5, 'rgba(16,185,129,0.65)');
        diskGradient.addColorStop(1, 'rgba(16,185,129,1.0)');

        ctx.strokeStyle = diskGradient;
        ctx.lineWidth = 2;
        ctx.beginPath();
        paddedHistory.forEach((sample, index) => {
            const x = barWidth * index + barWidth / 2;
            const y = height - (sample.disk / maxValue) * height;
            if (index === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });
        ctx.stroke();

        // Draw hover indicator
        if (hoveredIndex !== null && hoveredIndex < speedHistory.length) {
            const paddedIndex = emptySlots + hoveredIndex;
            const x = barWidth * paddedIndex + barWidth / 2;
            const sample = speedHistory[hoveredIndex];
            const y = height - (sample.disk / maxValue) * height;

            ctx.beginPath();
            ctx.arc(x, y, 6, 0, Math.PI * 2);
            ctx.fillStyle = 'rgba(16, 185, 129, 0.3)';
            ctx.fill();

            ctx.beginPath();
            ctx.arc(x, y, 4, 0, Math.PI * 2);
            ctx.fillStyle = '#10b981';
            ctx.fill();
            ctx.strokeStyle = '#fff';
            ctx.lineWidth = 2;
            ctx.stroke();
        }
    }, [speedHistory, hoveredIndex, canvasSize]);

    // Mouse handlers for canvas
    const handleCanvasMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
        if (!canvasRef.current) return;

        const rect = canvasRef.current.getBoundingClientRect();
        // Mouse X relative to the canvas
        const x = e.clientX - rect.left;

        // Canvas is now responsive, so logical width matches layout width.
        // We just need to map x to the correct slot.
        const logicalWidth = rect.width;

        const barWidth = logicalWidth / GRAPH_SLOTS;
        const paddedIndex = Math.floor(x / barWidth);

        const emptySlots = GRAPH_SLOTS - speedHistory.length;
        const actualIndex = paddedIndex - emptySlots;

        if (actualIndex >= 0 && actualIndex < speedHistory.length) {
            setHoveredIndex(actualIndex);
            setMousePos({ x: e.clientX, y: e.clientY });
        } else {
            setHoveredIndex(null);
            setMousePos(null);
        }
    }, [speedHistory.length]);

    const handleCanvasMouseLeave = useCallback(() => {
        setHoveredIndex(null);
        setMousePos(null);
    }, []);

    // Pause/Resume handlers
    const handlePause = async () => {
        if (!currentJob) return;
        try {
            // Pausing state is now tracked in backend via queue state
            await invoke('pause_game_download', { installId: currentJob.installId });
        } catch (error) {
            console.error('Failed to pause download:', error);
        }
    };

    const handleResume = async () => {
        if (!pausedJob) return;

        const installId = pausedJob.installId;

        try {
            // Use the new queue_resume_job command to resume the paused job
            await invoke('queue_resume_job', { installId });
        } catch (error) {
            console.error('Failed to resume download:', error);
        }
    };

    // Queue reordering handlers
    const handleMoveUp = async (jobId: string) => {
        try {
            await invoke('queue_move_up', { jobId });
        } catch (error) {
            console.error('Failed to move job up:', error);
        }
    };

    const handleMoveDown = async (jobId: string) => {
        try {
            await invoke('queue_move_down', { jobId });
        } catch (error) {
            console.error('Failed to move job down:', error);
        }
    };

    const handleRemove = async (jobId: string) => {
        try {
            await invoke('queue_remove', { jobId });
        } catch (error) {
            console.error('Failed to remove job from queue:', error);
        }
    };

    const handleActivateJob = async (jobId: string) => {
        try {
            await invoke('queue_activate_job', { jobId });
        } catch (error) {
            console.error('Failed to activate job:', error);
        }
    };

    const handleClearCompleted = async () => {
        try {
            await invoke('queue_clear_completed');
        } catch (error) {
            console.error('Failed to clear completed:', error);
        }
    };

    const handleReorder = async (jobId: string, newPosition: number) => {
        try {
            await invoke('queue_reorder', { jobId, newPosition });
        } catch (error) {
            console.error('Failed to reorder job:', error);
        }
    };

    // Drag and drop handlers
    const handleDragStart = (e: React.DragEvent, jobId: string) => {
        setDraggedJobId(jobId);
        e.dataTransfer.effectAllowed = 'move';
        e.dataTransfer.setData('text/plain', jobId);
    };

    const handleDragEnd = () => {
        setDraggedJobId(null);
        setDragOverTarget(null);
    };

    const handleDragOver = (e: React.DragEvent, targetId: string) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
        setDragOverTarget(targetId);
    };

    const handleDragLeave = () => {
        setDragOverTarget(null);
    };

    const handleDropOnQueue = async (e: React.DragEvent, targetIndex: number) => {
        e.preventDefault();
        const jobId = e.dataTransfer.getData('text/plain');
        if (jobId && draggedJobId) {
            await handleReorder(jobId, targetIndex);
        }
        setDraggedJobId(null);
        setDragOverTarget(null);
    };

    const handleDropOnActive = async (e: React.DragEvent) => {
        e.preventDefault();
        const jobId = e.dataTransfer.getData('text/plain');
        if (jobId && draggedJobId) {
            await handleActivateJob(jobId);
        }
        setDraggedJobId(null);
        setDragOverTarget(null);
    };

    // Get install info for banner
    const currentInstall = currentJob ? installs.find(i => i.id === currentJob.installId) : null;
    const bannerImage = currentInstall?.game_background;

    // Speed limit display
    const limitText = downloadSpeedLimitKB > 0
        ? (downloadSpeedLimitKB >= 1000
            ? `${(downloadSpeedLimitKB / 1000).toFixed(1)} MB/s`
            : `${Math.round(downloadSpeedLimitKB)} KB/s`)
        : '';

    return (
        <div
            className="flex-1 flex flex-col h-full overflow-hidden animate-fadeIn"
            style={{ willChange: 'opacity', backfaceVisibility: 'hidden', transform: 'translateZ(0)' }}
        >
            {/* Hover Tooltip - portaled to body to avoid transform containing block offset */}
            {hoveredIndex !== null && mousePos && speedHistory[hoveredIndex] && createPortal(
                (() => {
                    const tooltipWidth = 220;
                    const tooltipHeight = 90;
                    const padding = 8;
                    let left = mousePos.x + 20;
                    let top = mousePos.y - tooltipHeight - 8;

                    if (left + tooltipWidth + padding > window.innerWidth) {
                        left = Math.max(padding, mousePos.x - tooltipWidth - 8);
                    }
                    if (top < padding) top = mousePos.y + 8;

                    const sample = speedHistory[hoveredIndex];
                    return (
                        <div
                            className="fixed z-50 bg-zinc-900/95 border border-white/10 rounded-xl px-3 py-2 shadow-2xl ring-1 ring-white/5 pointer-events-none"
                            style={{ left: `${left}px`, top: `${top}px`, minWidth: tooltipWidth }}
                        >
                            <div className="text-xs space-y-1.5 min-w-[180px]">
                                <div className="border-b border-white/5 pb-1.5 mb-1.5">
                                    <div className="flex items-center gap-1.5 text-orange-400 font-medium">
                                        <DownloadCloud className="w-3 h-3" />
                                        <span className="break-all">{currentJob?.name ?? 'Download'}</span>
                                    </div>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-blue-400 font-semibold">Network</span>
                                    <span className="text-blue-400 font-semibold">{formatSpeed(sample.net)}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-green-400 font-semibold">Disk</span>
                                    <span className="text-green-400 font-semibold">{formatSpeed(sample.disk)}</span>
                                </div>
                            </div>
                        </div>
                    );
                })(),
                document.body
            )}

            {/* Page Header */}
            <div className="flex items-center gap-4 px-8 py-5 border-b border-white/5">
                <button
                    onClick={() => setCurrentPage(PAGES.NONE)}
                    className="p-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 hover:border-white/10 transition-all duration-200 hover:scale-105 hover:shadow-[0_0_12px_rgba(59,130,246,0.15)] active:scale-95"
                >
                    <ArrowLeft className="w-5 h-5 text-white/70" />
                </button>
                <div className="flex items-center gap-4">
                    <div className="p-3 bg-blue-500/15 rounded-xl border border-blue-500/20 shadow-[0_0_15px_rgba(59,130,246,0.2)]">
                        <DownloadCloud className="w-6 h-6 text-blue-400" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-bold bg-gradient-to-r from-white to-white/70 bg-clip-text text-transparent">Download Manager</h1>
                        {currentJob && (
                            <p className="text-sm text-white/50">
                                {formatSpeed(displayNetSpeed)} • {allJobs.length} item{allJobs.length !== 1 ? 's' : ''} in queue
                            </p>
                        )}
                    </div>
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-y-auto">
                {/* Current Download Section */}
                {currentJob ? (
                    <div
                        className={`flex flex-col transition-all ${draggedJobId && dragOverTarget === 'active'
                            ? 'ring-2 ring-blue-500 ring-inset bg-blue-900/10'
                            : ''
                            }`}
                        onDragOver={(e) => { e.preventDefault(); setDragOverTarget('active'); }}
                        onDragLeave={() => setDragOverTarget(null)}
                        onDrop={handleDropOnActive}
                    >
                        {/* Graph and Progress Area */}
                        <div className="border-b border-white/5">
                            <div className="flex gap-0 relative items-end pl-72">
                                {/* Banner Background - Extended */}
                                <div className="absolute left-0 top-0 bottom-0 w-[65%] overflow-hidden pointer-events-none z-0">
                                    {bannerImage ? (
                                        <div
                                            className="w-full h-full"
                                            style={{
                                                maskImage: 'linear-gradient(to right, rgba(0,0,0,1) 0%, rgba(0,0,0,1) 40%, rgba(0,0,0,0.5) 70%, rgba(0,0,0,0) 100%)',
                                                WebkitMaskImage: 'linear-gradient(to right, rgba(0,0,0,1) 0%, rgba(0,0,0,1) 40%, rgba(0,0,0,0.5) 70%, rgba(0,0,0,0) 100%)'
                                            }}
                                        >
                                            <CachedImage key={`banner-v${imageVersion}`} src={bannerImage} alt="" className="w-full h-full object-cover" />
                                        </div>
                                    ) : (
                                        <div className="w-full h-full bg-gray-800/50" />
                                    )}

                                    {/* Title overlay */}
                                    <div className="absolute left-6 top-6 z-10 text-left pointer-events-none">
                                        <h3
                                            className="text-2xl font-semibold text-white mb-1 break-words max-w-sm"
                                            style={{
                                                textShadow: "0 2px 3px rgba(0,0,0,0.95), 0 0 12px rgba(0,0,0,0.75)",
                                                WebkitTextStroke: "0.35px rgba(0,0,0,0.8)"
                                            }}
                                        >
                                            {currentProgress?.name ?? currentJob.name}
                                        </h3>
                                        <p
                                            className="text-sm text-gray-200"
                                            style={{
                                                textShadow: "0 1px 2px rgba(0,0,0,0.95), 0 0 8px rgba(0,0,0,0.7)",
                                                WebkitTextStroke: "0.25px rgba(0,0,0,0.8)"
                                            }}
                                        >
                                            {formatKind(currentJob.kind)}
                                        </p>
                                    </div>
                                </div>

                                {/* Graph - Wrapper to keep full width layout but constrain graph */}
                                <div className="flex-1 relative z-10 min-w-0 flex items-end justify-end">
                                    <div className="w-full max-w-[800px]">
                                        <div ref={containerRef} className="w-full h-[140px] overflow-hidden">
                                            <canvas
                                                ref={canvasRef}
                                                className="w-full h-full cursor-pointer touch-none block"
                                                style={{ width: '100%', height: '100%' }}
                                                onMouseMove={handleCanvasMouseMove}
                                                onMouseLeave={handleCanvasMouseLeave}
                                            />
                                        </div>
                                    </div>
                                </div>

                                {/* Stats Panel - Wider with background */}
                                <div className="w-[420px] flex flex-col justify-between z-10 px-6 py-4 bg-black/90 border-l border-white/5">
                                    <div className="space-y-6">
                                        {/* Stats Row */}
                                        <div className="mt-3">
                                            <div className="flex items-start gap-8 justify-between">
                                                <div className="flex flex-col text-xs">
                                                    <div className="flex items-center gap-2 text-gray-400 uppercase tracking-wider">
                                                        <div className="flex items-end gap-0.5 h-3">
                                                            <div className="w-0.5 h-1.5 bg-blue-400 rounded-sm"></div>
                                                            <div className="w-0.5 h-2.5 bg-blue-400 rounded-sm"></div>
                                                            <div className="w-0.5 h-2 bg-blue-400 rounded-sm"></div>
                                                        </div>
                                                        <span>Network</span>
                                                    </div>
                                                    <div className="text-sm font-medium text-blue-400 mt-1">{formatSpeed(displayNetSpeed)}</div>
                                                </div>

                                                <div className="flex flex-col text-xs">
                                                    <div className="flex items-center gap-2 text-gray-400 uppercase tracking-wider">
                                                        <div className="flex items-end gap-0.5 h-3">
                                                            <div className="w-0.5 h-1.5 bg-blue-400 rounded-sm"></div>
                                                            <div className="w-0.5 h-2.5 bg-blue-400 rounded-sm"></div>
                                                            <div className="w-0.5 h-2 bg-blue-400 rounded-sm"></div>
                                                        </div>
                                                        <span>Peak</span>
                                                    </div>
                                                    <div className="text-sm font-medium text-blue-400 mt-1">{formatSpeed(peakSpeed)}</div>
                                                </div>

                                                <div className="flex flex-col text-xs">
                                                    <div className="flex items-center gap-2 text-gray-400 uppercase tracking-wider">
                                                        <div className="flex items-center h-3">
                                                            <div className="w-3 h-0.5 bg-green-400 rounded-full"></div>
                                                        </div>
                                                        <span>Disk</span>
                                                    </div>
                                                    <div className="text-sm font-medium text-green-400 mt-1">{formatSpeed(displayDiskSpeed)}</div>
                                                </div>
                                            </div>

                                            {/* Speed limit */}
                                            {limitText && (
                                                <div className="text-xs text-gray-400 uppercase tracking-wider mt-2">
                                                    Downloads limited to: <span className="text-gray-300">{limitText}</span>
                                                </div>
                                            )}
                                        </div>

                                        {/* Progress Bar */}
                                        <div>
                                            <div className="flex items-center justify-between mb-2">
                                                <div className="text-xs">
                                                    {isPaused ? (
                                                        <span className="text-yellow-400 font-medium">Paused</span>
                                                    ) : (
                                                        <div className="flex items-center gap-3">
                                                            {/* Phase indicator */}
                                                            <div className={`flex items-center gap-1.5 ${getDownloadPhaseColor(currentPhase)}`}>
                                                                <div className="w-1.5 h-1.5 rounded-full bg-current animate-pulse" />
                                                                <span className="font-medium">{formatDownloadPhase(currentPhase)}</span>
                                                            </div>
                                                            {/* Progress bytes */}
                                                            <div className="text-gray-400">
                                                                <span className="text-white font-medium">
                                                                    {formatBytes(progressBytes)}
                                                                </span>
                                                                <span className="mx-1 text-gray-400">/</span>
                                                                <span>{formatBytes(totalBytes)}</span>
                                                            </div>
                                                        </div>
                                                    )}
                                                </div>
                                                <div className="text-sm text-gray-300 font-medium">
                                                    {downloadProgress.toFixed(1)}%
                                                </div>
                                            </div>

                                            <div className="w-full bg-white/10 rounded-full h-2 mb-2 overflow-hidden">
                                                <div className={`h-2 rounded-full transition-all duration-500 ${isPaused ? 'bg-gray-500' : 'bg-blue-500'}`}
                                                    style={{
                                                        width: `${downloadProgress}%`,
                                                        boxShadow: isPaused ? 'none' : '0 0 10px rgba(59, 130, 246, 0.5)'
                                                    }}
                                                />
                                            </div>

                                            {/* Installing Progress Bar */}
                                            {hasInstallProgress && (
                                                <div className="mb-2">
                                                    <div className="flex items-center justify-between mb-1">
                                                        <div className={`flex items-center gap-1.5 text-xs ${getInstallPhaseColor(currentPhase)}`}>
                                                            <div className="w-1.5 h-1.5 rounded-full bg-current animate-pulse" />
                                                            <span className="font-medium uppercase tracking-wider">Installing</span>
                                                            <span className="text-gray-500">•</span>
                                                            <span className="font-medium">{formatInstallPhase(currentPhase)}</span>
                                                        </div>
                                                        <div className="text-sm text-gray-300 font-medium">
                                                            {installProgress.toFixed(1)}%
                                                        </div>
                                                    </div>
                                                    <div className="w-full bg-white/10 rounded-full h-2 overflow-hidden">
                                                        <div className={`h-2 rounded-full transition-all duration-500 ${isPaused ? 'bg-gray-500' : 'bg-green-500'}`}
                                                            style={{
                                                                width: `${installProgress}%`,
                                                                boxShadow: isPaused ? 'none' : '0 0 10px rgba(34, 197, 94, 0.5)'
                                                            }}
                                                        />
                                                    </div>
                                                </div>
                                            )}

                                            {/* ETA and Pause Button */}
                                            <div className="flex items-center justify-between text-xs text-gray-400 mb-3">
                                                <div className="flex-1">
                                                    {isPausing && !isPaused ? (
                                                        <div className="flex items-center">
                                                            <svg className="animate-spin w-3 h-3 mr-2 text-yellow-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                                                                <circle cx="12" cy="12" r="10" strokeOpacity="0.25" />
                                                                <path d="M12 2a10 10 0 0 1 10 10" strokeLinecap="round" />
                                                            </svg>
                                                            <span className="text-yellow-400 font-medium">Pausing...</span>
                                                        </div>
                                                    ) : !isPaused ? (
                                                        etaText !== '—' ? (
                                                            <div>
                                                                <span className="uppercase tracking-wider">Estimate:</span>
                                                                <span className="ml-2 text-white font-medium">{etaText}</span>
                                                            </div>
                                                        ) : null
                                                    ) : null}
                                                </div>
                                                <button
                                                    onClick={isPaused ? handleResume : handlePause}
                                                    disabled={isPausing && !isPaused}
                                                    className={`w-9 h-9 rounded-md flex items-center justify-center transition-colors shadow-sm ${isPausing && !isPaused
                                                        ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                                                        : 'bg-blue-600 hover:bg-blue-700 text-white'
                                                        }`}
                                                >
                                                    {isPaused ? (
                                                        <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                                            <path d="M5 3v18l15-9L5 3z" />
                                                        </svg>
                                                    ) : (
                                                        <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                                                            <path d="M6 4h4v16H6zM14 4h4v16h-4z" />
                                                        </svg>
                                                    )}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                    <div className="h-2" />
                                </div>
                            </div>
                        </div>
                    </div>
                ) : (
                    <div className="p-12 text-center text-gray-500">
                        <DownloadCloud className="w-16 h-16 mx-auto mb-4 opacity-50" />
                        <p className="text-lg">No active downloads</p>
                        <p className="text-sm mt-2 text-gray-600">Downloads will appear here when you start them</p>
                    </div>
                )}

                {/* Queue Section */}
                <div className="">
                    <div className="p-6 border-b border-white/5">
                        <h3 className="text-lg font-semibold text-white">Queue ({queuedJobs.length})</h3>
                    </div>
                    <div className="p-4">
                        {queuedJobs.length === 0 ? (
                            <div className="text-center py-8 text-gray-500">
                                <p className="text-sm">No items in queue</p>
                            </div>
                        ) : (
                            <div className="flex flex-col gap-2">
                                {queuedJobs.map((job, index) => {
                                    const jobProgress = progressByJobId[job.id];
                                    const install = installs.find(i => i.id === job.installId);
                                    const name = jobProgress?.name ?? job.name ?? install?.name ?? job.installId;
                                    const isDragging = draggedJobId === job.id;
                                    const isDragOver = dragOverTarget === job.id;

                                    return (
                                        <div
                                            key={job.id}
                                            draggable
                                            onDragStart={(e) => handleDragStart(e, job.id)}
                                            onDragEnd={handleDragEnd}
                                            onDragOver={(e) => handleDragOver(e, job.id)}
                                            onDragLeave={handleDragLeave}
                                            onDrop={(e) => handleDropOnQueue(e, index)}
                                            className={`
                        flex items-center gap-4 p-3 rounded-lg border transition-all cursor-grab active:cursor-grabbing
                        ${isDragging ? 'opacity-50 scale-95' : 'opacity-100'}
                        ${isDragOver ? 'border-blue-500 bg-blue-900/20' : 'border-white/5 bg-white/5'}
                        hover:bg-white/10`}>
                                            {/* Drag Handle */}
                                            <div className="flex-shrink-0 text-gray-500 hover:text-gray-300">
                                                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                                                </svg>
                                            </div>

                                            {/* Game Icon */}
                                            <div className="flex-shrink-0 w-12 h-12 rounded-lg overflow-hidden bg-white/10">
                                                {install?.game_icon ? (
                                                    <img key={`icon-${job.id}-v${imageVersion}`} src={install.game_icon} alt="" className="w-full h-full object-cover" />
                                                ) : (
                                                    <div className="w-full h-full flex items-center justify-center text-gray-500">
                                                        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                                        </svg>
                                                    </div>
                                                )}
                                            </div>

                                            {/* Info */}
                                            <div className="flex-1 min-w-0">
                                                <h4 className="text-sm font-medium text-white break-all" title={name}>
                                                    {name}
                                                </h4>
                                                <p className="text-xs text-gray-400 mt-0.5">{formatKind(job.kind)} • {formatStatus(job.status, false)}</p>
                                            </div>

                                            {/* Actions */}
                                            <div className="flex gap-1 flex-shrink-0">
                                                <button
                                                    onClick={() => handleActivateJob(job.id)}
                                                    className="px-3 py-1.5 text-xs font-medium text-white bg-blue-600 hover:bg-blue-700 rounded transition-colors"
                                                    title="Start downloading this now">
                                                    Start Now
                                                </button>
                                                {index > 0 && (
                                                    <button
                                                        onClick={() => handleMoveUp(job.id)}
                                                        className="p-1.5 text-gray-400 hover:text-white hover:bg-white/10 rounded transition-colors"
                                                        title="Move up">
                                                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
                                                        </svg>
                                                    </button>
                                                )}
                                                {index < queuedJobs.length - 1 && (
                                                    <button
                                                        onClick={() => handleMoveDown(job.id)}
                                                        className="p-1.5 text-gray-400 hover:text-white hover:bg-white/10 rounded transition-colors"
                                                        title="Move down">
                                                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                                                        </svg>
                                                    </button>
                                                )}
                                                <button
                                                    onClick={() => handleRemove(job.id)}
                                                    className="p-1.5 text-red-400 hover:text-red-300 hover:bg-red-900/30 rounded transition-colors"
                                                    title="Remove from queue">
                                                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                                    </svg>
                                                </button>
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        )}
                    </div>

                    {/* Completed Section */}
                    {completedItems.length > 0 && (
                        <div className="p-6 border-t border-white/5">
                            <div className="flex items-center justify-between mb-4">
                                <h3 className="text-lg font-semibold text-white">
                                    Completed <span className="text-gray-400 font-normal ml-2">({completedItems.length})</span>
                                </h3>
                                <button
                                    onClick={handleClearCompleted}
                                    className="px-3 py-1.5 text-xs font-medium text-gray-400 hover:text-white bg-white/5 hover:bg-white/10 rounded transition-colors">
                                    Clear All
                                </button>
                            </div>
                            <div className="flex flex-col gap-2">
                                {completedItems.map((job) => {
                                    const install = installs.find(i => i.id === job.installId);
                                    const statusColor = job.status === 'failed' ? 'text-red-400' : 'text-green-400';
                                    const statusText = job.status === 'failed' ? 'Failed' : 'Completed';
                                    return (
                                        <div key={job.id} className="flex items-center gap-4 p-3 rounded-lg border border-white/5 bg-white/5">
                                            {/* Game Icon */}
                                            <div className="flex-shrink-0 w-10 h-10 rounded-lg overflow-hidden bg-white/10">
                                                {install?.game_icon ? (
                                                    <img key={`icon-${job.id}-v${imageVersion}`} src={install.game_icon} alt="" className="w-full h-full object-cover" />
                                                ) : (
                                                    <div className="w-full h-full flex items-center justify-center text-gray-500">
                                                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                                        </svg>
                                                    </div>
                                                )}
                                            </div>
                                            {/* Info */}
                                            <div className="flex-1 min-w-0">
                                                <h4 className="text-sm font-medium text-white break-all" title={job.name}>
                                                    {job.name}
                                                </h4>
                                                <p className="text-xs text-gray-400 mt-0.5">{formatKind(job.kind)}</p>
                                            </div>
                                            <div className={`text-xs ${statusColor} font-medium flex items-center gap-1`}>
                                                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                                                </svg>
                                                {statusText}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
