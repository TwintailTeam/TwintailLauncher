export const Events = [
  'download_queue_state',
  'download_progress',
  'download_installing',
  'download_complete',
  'download_paused',
  'update_progress',
  'update_installing',
  'update_complete',
  'repair_progress',
  'repair_installing',
  'repair_complete',
  'preload_progress',
  'preload_installing',
  'preload_complete',
  'move_progress',
  'move_complete',
  'game_closed',
] as const;

export type AppEvent = typeof Events[number];
