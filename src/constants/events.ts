export const Events = [
  'download_progress',
  'download_complete',
  'update_progress',
  'update_complete',
  'repair_progress',
  'repair_complete',
  'preload_progress',
  'preload_complete',
  'move_progress',
  'move_complete',
] as const;

export type AppEvent = typeof Events[number];
