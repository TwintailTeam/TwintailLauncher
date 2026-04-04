export type QueueJobKind = 'game_download' | 'game_update' | 'game_preload' | 'game_repair' | 'runner_download' | 'steamrt_download' | 'steamrt4_download' | 'xxmi_download' | 'extras_download';

export type QueueJobStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'paused';

export interface QueueJobView {
  id: string;
  kind: QueueJobKind;
  installId: string;
  name: string;
  status: QueueJobStatus;
}

export interface DownloadQueueStatePayload {
  maxConcurrent: number;
  paused: boolean;
  running: QueueJobView[];
  queued: QueueJobView[];
  completed?: QueueJobView[];
  pausedJobs?: QueueJobView[];
  pausingInstalls?: string[];
}

export type DownloadPhase = 'verifying' | 'downloading' | 'installing' | 'extracting' | 'validating' | 'moving' | 'idle';

export interface DownloadJobProgress {
  jobId: string;
  name?: string;
  progress?: number;
  total?: number;
  speed?: number;
  disk?: number;
  installProgress?: number;
  installTotal?: number;
  phase?: DownloadPhase;
  eventType: string;
}
