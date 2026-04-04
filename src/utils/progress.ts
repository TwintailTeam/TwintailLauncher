export function toPercent(number: any, total: any): number {
  const current = Number(number);
  const max = Number(total);
  if (!Number.isFinite(current) || !Number.isFinite(max) || max <= 0) {
    return 0;
  }

  const percent = (current / max) * 100;
  return Math.max(0, Math.min(100, percent));
}

export function formatBytes(bytes: number): string {
  if (bytes > 1000 * 1000 * 1000) {
    return (bytes / 1000.0 / 1000.0 / 1000.0).toFixed(2) + ' GB';
  } else if (bytes > 1000 * 1000) {
    return (bytes / 1000.0 / 1000.0).toFixed(2) + ' MB';
  } else if (bytes > 1000) {
    return (bytes / 1000.0).toFixed(2) + ' KB';
  } else {
    return bytes.toFixed(2) + ' B';
  }
}
