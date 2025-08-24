/**
 * Preloads a list of image URLs and tracks progress.
 * @param urls Array of image URLs to preload
 * @param onProgress Optional callback for progress updates (loaded, total)
 * @param cache Optional Set to track already loaded images
 * @returns Promise that resolves when all images are loaded
 */
// Keep strong references to preloaded images to prevent GC and maximize cache hits
const imageElementCache: Map<string, HTMLImageElement> = new Map();

export function isImagePreloaded(url: string): boolean {
  return imageElementCache.has(url);
}

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
      // Hint the browser this is important and should be cached if allowed
      try {
        // @ts-ignore fetchPriority isn't typed on HTMLImageElement in all TS versions
        img.fetchPriority = "high";
      } catch {}
      img.decoding = "async";
      img.loading = "eager";
      img.onload = () => {
        loadedCache.add(src);
        imageElementCache.set(src, img);
        loaded++;
        if (onProgress) onProgress(loaded, total);
        if (loaded === total) resolve();
      };
      img.onerror = () => {
        // Keep a reference anyway to avoid repeated retries during session
        imageElementCache.set(src, img);
        loaded++;
        if (onProgress) onProgress(loaded, total);
        if (loaded === total) resolve();
      };
      img.src = src;
    });
  });
}

