/**
 * Preloads a list of image URLs and tracks progress.
 * @param urls Array of image URLs to preload
 * @param onProgress Optional callback for progress updates (loaded, total)
 * @param cache Optional Set to track already loaded images
 * @returns Promise that resolves when all images are loaded
 */
export function preloadImages(
  urls: string[],
  onProgress?: (loaded: number, total: number) => void,
  cache?: Set<string>
): Promise<void> {
  return new Promise((resolve) => {
    const loadedCache = cache || new Set<string>();
    const toLoad = urls.filter((src) => src && !loadedCache.has(src));
    const total = toLoad.length;
    if (total === 0) return resolve();
    let loaded = 0;
    toLoad.forEach((src) => {
      const img = new window.Image();
      img.onload = () => {
        loadedCache.add(src);
        loaded++;
        if (onProgress) onProgress(loaded, total);
        if (loaded === total) resolve();
      };
      img.onerror = () => {
        loaded++;
        if (onProgress) onProgress(loaded, total);
        if (loaded === total) resolve();
      };
      img.src = src;
    });
  });
}

