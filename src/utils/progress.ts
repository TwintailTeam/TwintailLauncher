export function toPercent(number: any, total: any): number {
  return (parseInt(number) / parseInt(total)) * 100;
}

export function formatBytes(bytes: any): string {
  const MiB = 1024 * 1024;
  const GiB = 1024 * MiB;
  const b = parseInt(bytes);
  if (b >= GiB) return (b / GiB).toFixed(2) + ' GiB';
  if (b >= MiB) return (b / MiB).toFixed(2) + ' MiB';
  return b + ' B';
}
