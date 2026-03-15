import type { DownloadPhase } from '../types/downloadQueue.ts';

export type EventStateUpdate = Record<string, any> | ((prev: any) => Record<string, any>);

// Convert numeric phase from backend to string phase for frontend
function parsePhase(phaseNum: string | number | undefined): DownloadPhase | undefined {
  if (phaseNum === undefined) return undefined;
  const num = typeof phaseNum === 'string' ? parseInt(phaseNum) : phaseNum;
  // Phase: 0=idle, 1=verifying, 2=downloading, 3=installing, 4=validating, 5=moving
  switch (num) {
    case 0: return 'idle';
    case 1: return 'verifying';
    case 2: return 'downloading';
    case 3: return 'installing';
    case 4: return 'validating';
    case 5: return 'moving';
    default: return undefined;
  }
}

function parseOptionalInt(value: unknown): number | undefined {
  if (value === undefined || value === null) return undefined;
  const parsed = parseInt(String(value), 10);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function clampProgressValue(progress: number | undefined, total: number | undefined): number | undefined {
  if (progress === undefined) return undefined;
  if (total !== undefined && total > 0) {
    return Math.max(0, Math.min(total, progress));
  }
  return Math.max(0, progress);
}

function parseProgressPair(progressRaw: unknown, totalRaw: unknown): { progress?: number; total?: number } {
  const total = parseOptionalInt(totalRaw);
  const progress = clampProgressValue(parseOptionalInt(progressRaw), total);
  return { progress, total };
}

export function registerEvents(
  eventType: string,
  event: any,
  pushInstalls: () => void,
  getCurrentInstall: () => string,
  fetchInstallResumeStates: (install: string) => void,
  fetchInstalledRunners?: () => void,
  fetchSteamRTStatus?: () => void
): EventStateUpdate | undefined {
  switch (eventType) {
    case 'download_queue_state': {
      // Note: disableRun and disableInstallEdit are now calculated per-install in the UI
      return {
        downloadQueueState: event.payload,
        // keep legacy fields hidden (UI is replaced)
        hideProgressBar: true,
        // disableInstallEdit is now calculated per-install in the UI (allows editing settings for non-downloading games)
        // disableRun is now calculated per-install in the UI (allows playing installed games while others download)
        // Allow queueing new downloads/updates even while work is in progress
        // The queue system handles multiple jobs
        disableUpdate: false,
        disableDownload: false,
        disablePreload: false,
        disableResume: false,
      };
    }
    case 'game_closed':
    case 'move_complete':
    case 'download_removed':
    case 'download_complete':
    case 'update_complete':
    case 'repair_complete':
    case 'preload_complete': {
      pushInstalls();

      // Refresh resume states for the current install after completion
      const currentInstall = getCurrentInstall();
      if (currentInstall) {
        fetchInstallResumeStates(currentInstall);
      }

      // Refresh runner status after downloads complete (runner/steamrt downloads)
      // This ensures the Play button becomes enabled when dependencies are ready
      if (fetchInstalledRunners) fetchInstalledRunners();
      if (fetchSteamRTStatus) fetchSteamRTStatus();

      // Misc downloads (proton, steamrt, etc.) use name as job ID
      // Remove from progress tracking when complete
      const completedName = typeof event.payload === 'string' ? event.payload : event.payload?.name;
      if (completedName) {
        return (prev) => {
          const next = { ...(prev?.downloadProgressByJobId || {}) };
          delete next[completedName];
          return { downloadProgressByJobId: next };
        };
      }
      return undefined;
    }
    case 'move_progress': {
      const jobId = event.payload.install_id;
      if (!jobId) return undefined;
      const { progress, total } = parseProgressPair(event.payload.progress, event.payload.total);
      const { progress: installProgress, total: installTotal } = parseProgressPair(event.payload.install_progress, event.payload.install_total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        next[jobId] = {
          jobId,
          name: event.payload.install_name,
          progress,
          total,
          speed: 0,
          disk: 0,
          installProgress,
          installTotal,
          phase: parsePhase(event.payload.phase),
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'download_progress': {
      // Use job_id if present, otherwise use name as fallback for misc downloads (proton, steamrt, etc.)
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId ?? event?.payload?.name;
      if (!jobId) return undefined;
      const { progress, total } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      const { progress: installProgress, total: installTotal } = parseProgressPair(event?.payload?.install_progress, event?.payload?.install_total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        next[jobId] = {
          jobId,
          name: event.payload.name,
          progress,
          total,
          speed: parseOptionalInt(event.payload.speed),
          disk: parseOptionalInt(event.payload.disk),
          // Include install progress if present in the same event (Sophon downloads)
          installProgress,
          installTotal,
          // Phase: verifying, downloading, installing, validating, moving
          phase: parsePhase(event.payload.phase),
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'download_installing': {
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId;
      if (!jobId) return undefined;
      const { progress: installProgress, total: installTotal } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        const existing = next[jobId] || {};
        next[jobId] = {
          ...existing,
          jobId,
          name: event.payload.name || existing.name,
          // Keep existing download progress, add installation progress
          installProgress: installProgress ?? existing.installProgress,
          installTotal: installTotal ?? existing.installTotal,
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'download_paused': {
      const currentInstall = getCurrentInstall();
      if (currentInstall) {
        fetchInstallResumeStates(currentInstall);
      }
      return undefined;
    }
    case 'update_progress': {
      // Use job_id if present, otherwise use name as fallback for misc updates (steamrt, etc.)
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId ?? event?.payload?.name;
      if (!jobId) return undefined;
      const { progress, total } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      const { progress: installProgress, total: installTotal } = parseProgressPair(event?.payload?.install_progress, event?.payload?.install_total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        next[jobId] = {
          jobId,
          name: event.payload.name,
          progress,
          total,
          speed: parseOptionalInt(event.payload.speed),
          disk: parseOptionalInt(event.payload.disk),
          // Include install progress if present in the same event (Sophon downloads)
          installProgress,
          installTotal,
          // Phase: verifying, downloading, installing, validating, moving
          phase: parsePhase(event.payload.phase),
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'update_installing': {
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId;
      if (!jobId) return undefined;
      const { progress: installProgress, total: installTotal } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        const existing = next[jobId] || {};
        next[jobId] = {
          ...existing,
          jobId,
          name: event.payload.name || existing.name,
          // Keep existing download progress, add installation progress
          installProgress: installProgress ?? existing.installProgress,
          installTotal: installTotal ?? existing.installTotal,
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'repair_progress': {
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId;
      if (!jobId) return undefined;
      const { progress, total } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      const { progress: installProgress, total: installTotal } = parseProgressPair(event?.payload?.install_progress, event?.payload?.install_total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        next[jobId] = {
          jobId,
          name: event.payload.name,
          progress,
          total,
          speed: parseOptionalInt(event.payload.speed),
          disk: parseOptionalInt(event.payload.disk),
          // Include install progress if present in the same event (Sophon downloads)
          installProgress,
          installTotal,
          // Phase: verifying, downloading, installing, validating, moving
          phase: parsePhase(event.payload.phase),
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'repair_installing': {
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId;
      if (!jobId) return undefined;
      const { progress: installProgress, total: installTotal } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        const existing = next[jobId] || {};
        next[jobId] = {
          ...existing,
          jobId,
          name: event.payload.name || existing.name,
          // Keep existing download progress, add installation progress
          installProgress: installProgress ?? existing.installProgress,
          installTotal: installTotal ?? existing.installTotal,
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'preload_progress': {
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId;
      if (!jobId) return undefined;
      const { progress, total } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      const { progress: installProgress, total: installTotal } = parseProgressPair(event?.payload?.install_progress, event?.payload?.install_total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        next[jobId] = {
          jobId,
          name: event.payload.name,
          progress,
          total,
          speed: parseOptionalInt(event.payload.speed),
          disk: parseOptionalInt(event.payload.disk),
          // Include install progress if present in the same event (Sophon downloads)
          installProgress,
          installTotal,
          // Phase: verifying, downloading, installing, validating, moving
          phase: parsePhase(event.payload.phase),
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
    case 'preload_installing': {
      const jobId = event?.payload?.job_id ?? event?.payload?.jobId;
      if (!jobId) return undefined;
      const { progress, total } = parseProgressPair(event?.payload?.progress, event?.payload?.total);
      return (prev) => {
        const next = { ...(prev?.downloadProgressByJobId || {}) };
        next[jobId] = {
          jobId,
          name: event.payload.name,
          progress,
          total,
          speed: parseOptionalInt(event.payload.speed),
          disk: parseOptionalInt(event.payload.disk),
          eventType,
        };
        return { downloadProgressByJobId: next };
      };
    }
  }
}
