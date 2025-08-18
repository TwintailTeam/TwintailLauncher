import { toPercent, formatBytes } from './progress.ts';

export function registerEvents(eventType: string, event: any, pushInstalls: () => void) {
  switch (eventType) {
    case 'move_complete':
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
        progressPrettyTotal: 0,
      };
    }
    case 'move_progress': {
      return {
        hideProgressBar: false,
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
        progressPrettyTotal: `${formatBytes(event.payload.total)}`,
      };
    }
    case 'download_progress': {
      return {
        hideProgressBar: false,
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
        progressPrettyTotal: `${formatBytes(event.payload.total)}`,
      };
    }
    case 'update_progress': {
      return {
        hideProgressBar: false,
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
        progressPrettyTotal: `${formatBytes(event.payload.total)}`,
      };
    }
    case 'repair_progress': {
      return {
        hideProgressBar: false,
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
        progressPrettyTotal: `${formatBytes(event.payload.total)}`,
      };
    }
    case 'preload_progress': {
      return {
        hideProgressBar: false,
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
        progressPrettyTotal: `${formatBytes(event.payload.total)}`,
      };
    }
  }
}
