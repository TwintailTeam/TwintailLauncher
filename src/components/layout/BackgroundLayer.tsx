import React, {useCallback, useEffect, useLayoutEffect, useRef} from "react";
import {getPreloadedImage, isImageFailed, isImagePreloaded, isLinux, isVideoUrl, preloadImage} from "../../utils/imagePreloader";

interface BackgroundLayerProps {
  currentSrc: string;
  previousSrc?: string;
  transitioning: boolean;
  bgVersion: number;
  popupOpen: boolean;
  pageOpen?: boolean;
  bgLoading?: boolean;
  onMainLoad?: () => void;
}

const isVideo = (src?: string) => !!src && isVideoUrl(src) && !isLinux;

// helper to detect MP4 specifically (for treating MP4 looping differently)
const isMp4 = (src?: string) => {
  const normalized = src?.split("?")[0]?.split("#")[0]?.toLowerCase() || "";
  return normalized.endsWith(".mp4");
};

const BackgroundLayer: React.FC<BackgroundLayerProps> = ({
  currentSrc,
  previousSrc,
  transitioning,
  bgVersion,
  popupOpen,
  pageOpen,
  bgLoading,
  onMainLoad,
}) => {
  const currentContainerRef = useRef<HTMLDivElement | null>(null);
  const previousContainerRef = useRef<HTMLDivElement | null>(null);
  const currentElementRef = useRef<HTMLImageElement | HTMLVideoElement | null>(null);
  const currentSrcRef = useRef<string>("");
  // Track pending preload to prevent race condition on Linux where effect re-runs
  // before preload completes, causing duplicate image appends
  const pendingPreloadRef = useRef<string | null>(null);

  // MP4 loop restart handler
  const restartMp4 = useCallback(() => {
    const el = currentElementRef.current;
    if (el && el instanceof HTMLVideoElement) {
      try {
        el.currentTime = 0;
        el.play().catch(() => { });
      } catch {
        // ignore
      }
    }
  }, []);

  // Create and configure video element from preloaded or new
  const createVideoElement = useCallback((src: string, className: string, onLoad?: () => void): HTMLVideoElement => {
    const preloaded = isImageFailed(src) ? undefined : getPreloadedImage(src);
    let video: HTMLVideoElement;

    if (preloaded && preloaded instanceof HTMLVideoElement) {
      // Clone the preloaded video to reuse buffered data
      video = preloaded.cloneNode(true) as HTMLVideoElement;
    } else {
      video = document.createElement("video");
      video.src = src;
    }

    video.className = className;
    video.muted = true;
    video.playsInline = true;
    video.preload = "auto";
    video.autoplay = false;

    if (isMp4(src)) {
      video.loop = false;
      video.onended = restartMp4;
    } else {
      video.loop = true;
    }

    if (onLoad) {
      video.onloadeddata = onLoad;
    }

    return video;
  }, [restartMp4]);

  // Create and configure image element from preloaded or new
  const createImageElement = useCallback((src: string, className: string, onLoad?: () => void): HTMLImageElement => {
    const preloaded = isImageFailed(src) ? undefined : getPreloadedImage(src);
    let img: HTMLImageElement;

    if (preloaded && preloaded instanceof HTMLImageElement) {
      // Clone the preloaded image to reuse cached data
      img = preloaded.cloneNode(true) as HTMLImageElement;
    } else {
      img = document.createElement("img");
      img.src = src;
    }

    img.className = className;
    img.alt = "background";
    img.loading = "eager";
    img.decoding = "async";

    if (onLoad) {
      // Use isImagePreloaded — cloned images on WebKitGTK always have complete=false
      // even when the image data is in cache, making complete unreliable
      if (isImagePreloaded(src)) {
        setTimeout(onLoad, 0);
      } else {
        img.onload = onLoad;
      }
    }

    return img;
  }, []);

  // Fast path for preloaded images: useLayoutEffect fires before paint so the
  // background animation starts in the same frame as the UI update.
  // The existing useEffect below handles videos and non-preloaded images;
  // its (currentSrcRef === currentSrc && currentElementRef exists) guard returns
  // early for anything already handled here.
  useLayoutEffect(() => {
    const container = currentContainerRef.current;
    if (!container || !currentSrc || isVideo(currentSrc)) return;
    if (!isImagePreloaded(currentSrc)) return;
    if (currentSrcRef.current === currentSrc && currentElementRef.current) return;

    const srcAtCallTime = currentSrc;
    currentSrcRef.current = srcAtCallTime;
    pendingPreloadRef.current = null;

    const baseClass = `w-full h-screen object-cover object-center absolute inset-0 transition-transform duration-300 ease-out will-change-transform`;
    const oldElement = currentElementRef.current;
    const element = createImageElement(srcAtCallTime, baseClass, () => { onMainLoad?.(); });
    element.id = "app-bg";

    const revealImage = () => {
      if (currentSrcRef.current !== srcAtCallTime || currentElementRef.current !== element) { element.remove(); return; }
      // No void offsetWidth needed — this is a fresh element so there's no stale animation
      // state to reset. The bgFadeIn keyframe (0% { opacity:0 }) defines the initial state,
      // so the browser starts the animation correctly without a forced layout.
      element.style.opacity = "";
      element.classList.add("animate-bg-fade-in");
      if (oldElement && oldElement.parentNode) { setTimeout(() => { if (oldElement.parentNode) oldElement.remove(); }, 350); }
    };

    element.style.opacity = "0";
    container.appendChild(element);
    currentElementRef.current = element;
    // queueMicrotask runs after useLayoutEffect but before browser paint,
    // so the animation class is applied before the first frame renders.
    queueMicrotask(revealImage);
  }, [currentSrc, bgVersion, createImageElement, onMainLoad]);

  // Effect to handle current background
  useEffect(() => {
    const container = currentContainerRef.current;
    if (!container) return;

    // Skip if same source and element already exists
    if (currentSrcRef.current === currentSrc && currentElementRef.current) {
      // Just update classes for popup state changes - handled by separate useEffect below
      return;
    }

    // Skip if preload is already pending for this source (prevents race condition on Linux
    // where effect re-runs due to transitioning/popup changes before preload completes)
    if (pendingPreloadRef.current === currentSrc) {
      return;
    }

    currentSrcRef.current = currentSrc;
    pendingPreloadRef.current = null; // Clear any stale pending state

    // DON'T clear the container here - keep old element visible until new one is ready
    // This prevents black flash on WebKitGTK when switching backgrounds

    if (!currentSrc) {
      // Only clear if source becomes empty
      container.innerHTML = "";
      currentElementRef.current = null;
      return;
    }

    const baseClass = `w-full h-screen object-cover object-center absolute inset-0 transition-transform duration-300 ease-out will-change-transform`;

    // Ensure preloaded before creating element
    const createAndAppend = (srcAtCallTime: string) => {
      // Guard: only append if source hasn't changed since preload started
      if (currentSrcRef.current !== srcAtCallTime) {
        return;
      }

      pendingPreloadRef.current = null;

      let element: HTMLImageElement | HTMLVideoElement;

      if (isVideo(srcAtCallTime)) {
        element = createVideoElement(srcAtCallTime, baseClass, () => {
          onMainLoad?.();
        });

        // For videos, we need to wait until the first frame is ready before showing
        // to prevent the black flash during transition
        element.style.opacity = "0";
        element.id = "app-bg";

        // Store reference to old element to remove after new video is ready
        const oldElement = currentElementRef.current;

        // Append new video (hidden) alongside old one temporarily
        container.appendChild(element);
        currentElementRef.current = element;

        // Function to reveal the video once it's ready
        const revealVideo = () => {
          // Guard: ensure source hasn't changed and this is still the current element
          if (currentSrcRef.current !== srcAtCallTime || currentElementRef.current !== element) {
            element.remove();
            return;
          }

          // Make the new video visible with smooth fade-in
          // Remove the hidden state and apply fresh animation class
          element.style.opacity = "";
          // Force reflow to ensure animation restarts cleanly
          void element.offsetWidth;
          // Apply the fade-in animation class fresh (remove first to restart if already applied)
          element.classList.remove("animate-bg-fade-in");
          element.classList.add("animate-bg-fade-in");

          // Start playback
          try {
            if ("play" in element) {
              element.play().catch(() => {
              });
            }
          } catch { /* ignore */ }

          // Remove old element AFTER a short delay to allow crossfade
          // This ensures smooth video-to-video transitions
          if (oldElement && oldElement.parentNode) {
            setTimeout(() => {
              if (oldElement.parentNode) {
                oldElement.remove();
              }
            }, 350); // Slightly longer than the 300ms fade animation
          }
        };

        // Wait for video to have enough data to show first frame
        // readyState 3 = HAVE_FUTURE_DATA, 4 = HAVE_ENOUGH_DATA
        const checkAndReveal = () => {
          if ("readyState" in element && element.readyState >= 3) {
            revealVideo();
          } else {
            // Listen for canplay event (fires when readyState >= 3)
            element.addEventListener("canplay", revealVideo, { once: true });
            // Also set a fallback timeout in case video loading is slow
            // This ensures we don't wait forever
            setTimeout(() => {
              if (element.style.opacity === "0" && currentElementRef.current === element) {
                revealVideo();
              }
            }, 500);
          }
        };

        // Reset and load the video
        try {
          element.pause();
          try { element.currentTime = 0; } catch { /* ignore */ }
          element.load();
          checkAndReveal();
        } catch {
          // If anything fails, just reveal immediately
          revealVideo();
        }
      } else {
        // Store reference to old element for crossfade
        const oldElement = currentElementRef.current;

        element = createImageElement(srcAtCallTime, baseClass, () => {
          onMainLoad?.();
        });

        element.id = "app-bg";

        // For smooth crossfade (especially dynamic-to-static), keep old element during fade
        // Start hidden, reveal with animation once loaded
        const revealImage = () => {
          // Guard: ensure source hasn't changed
          if (currentSrcRef.current !== srcAtCallTime || currentElementRef.current !== element) {
            element.remove();
            return;
          }

          // Apply fade-in animation
          element.style.opacity = "";
          void element.offsetWidth;
          element.classList.remove("animate-bg-fade-in");
          element.classList.add("animate-bg-fade-in");

          // Remove old element after crossfade completes
          if (oldElement && oldElement.parentNode) {
            setTimeout(() => {
              if (oldElement.parentNode) {
                oldElement.remove();
              }
            }, 350);
          }
        };

        // Use isImagePreloaded instead of element.complete — WebKitGTK cloned images
        // always report complete=false even when data is in cache, causing unnecessary
        // onload waits and visible delay before the background starts animating.
        element.style.opacity = "0";
        container.appendChild(element);
        currentElementRef.current = element;
        if (isImagePreloaded(srcAtCallTime)) {
          // Image is in cache: reveal via microtask so DOM is settled before animation
          queueMicrotask(revealImage);
        } else {
          element.onload = revealImage;
          // Fallback for browsers that don't fire onload for cached images
          setTimeout(() => {
            const imageElement = element as HTMLImageElement;
            if (
              imageElement.style.opacity === "0" &&
              currentElementRef.current === imageElement &&
              imageElement.complete &&
              imageElement.naturalHeight !== 0
            ) {
              revealImage();
            }
          }, 500);
        }
      }
    };

    // If already preloaded, create immediately; otherwise wait for preload
    if (isImagePreloaded(currentSrc)) {
      createAndAppend(currentSrc);
    } else {
      // Track that we're preloading this source
      pendingPreloadRef.current = currentSrc;
      const srcToPreload = currentSrc;
      preloadImage(srcToPreload).then(() => createAndAppend(srcToPreload));
    }
  }, [currentSrc, bgVersion, createVideoElement, createImageElement, onMainLoad]);

  // Effect to handle previous background (for transitions)
  useEffect(() => {
    const container = previousContainerRef.current;
    if (!container) return;

    // Clear previous content
    container.innerHTML = "";

    if (!transitioning || !previousSrc) return;

    const baseClass = `w-full h-screen object-cover object-center absolute inset-0 transition-none animate-bg-fade-out ${(popupOpen || pageOpen) ? "scale-[1.03]" : ""}`;

    if (isVideo(previousSrc)) {
      const preloaded = isImageFailed(previousSrc) ? undefined : getPreloadedImage(previousSrc);
      let video: HTMLVideoElement;

      if (preloaded && preloaded instanceof HTMLVideoElement) {
        video = preloaded.cloneNode(true) as HTMLVideoElement;
      } else {
        video = document.createElement("video");
        video.src = previousSrc;
      }

      video.className = baseClass;
      video.muted = true;
      video.playsInline = true;
      video.loop = true;
      video.autoplay = false;
      video.preload = "auto";

      container.appendChild(video);
    } else {
      const preloaded = isImageFailed(previousSrc) ? undefined : getPreloadedImage(previousSrc);
      let img: HTMLImageElement;

      if (preloaded && preloaded instanceof HTMLImageElement) {
        img = preloaded.cloneNode(true) as HTMLImageElement;
      } else {
        img = document.createElement("img");
        img.src = previousSrc;
      }

      img.className = baseClass;
      img.alt = "previous background";
      img.loading = "eager";
      img.decoding = "async";

      container.appendChild(img);
    }
  }, [transitioning, previousSrc, bgVersion, popupOpen, pageOpen]);

  // Effect to update popup scale without re-creating elements or touching the animation class.
  // Previously this did a full className= replacement including animate-bg-fade-in, which
  // restarted the fade-in animation every install switch. Now only scale-[1.03] is toggled.
  useEffect(() => {
    const el = currentElementRef.current;
    if (!el || !currentSrc) return;
    if (popupOpen || pageOpen) { el.classList.add("scale-[1.03]"); }
    else { el.classList.remove("scale-[1.03]"); }
  }, [popupOpen, pageOpen, currentSrc]);

  return (
    <div className="absolute inset-0 -z-10 pointer-events-none overflow-hidden bg-zinc-950">
      {/* Previous background container (for fade-out transition) */}
      <div ref={previousContainerRef} className="contents" />

      {/* Current background container */}
      <div
        ref={currentContainerRef}
        className="contents"
        style={bgLoading ? {
          backgroundImage: 'radial-gradient(circle at 20% 20%, rgba(90,70,140,0.35), rgba(20,15,30,0.9) 60%), radial-gradient(circle at 80% 80%, rgba(60,100,160,0.25), rgba(10,10,20,0.95) 55%)',
          backgroundSize: 'cover',
          backgroundPosition: 'center'
        } : undefined}
      />

      {/* Dimming overlay - replaces expensive brightness/saturate filters with cheap alpha blending */}
      {/* No transition on open to prevent black flash on WebKitGTK; instant visibility */}
      <div
        className={`absolute inset-0 pointer-events-none ${(popupOpen || pageOpen) ? "opacity-100" : "opacity-0"}`}
        style={{
          background: 'linear-gradient(to bottom, rgba(0,0,0,0.55), rgba(0,0,0,0.6))',
          willChange: 'opacity',
          backfaceVisibility: 'hidden',
          WebkitBackfaceVisibility: 'hidden',
          transform: 'translateZ(0)',
        }}
      />

      {/* Loading gradient overlay */}
      {(bgLoading || !currentSrc) ? (
        <div className="absolute inset-0">
          <div className={`w-full h-full ${(popupOpen || pageOpen) ? "scale-[1.03]" : ""}`} style={{
            backgroundImage: 'radial-gradient(circle at 20% 20%, rgba(90,70,140,0.35), rgba(20,15,30,0.9) 60%), radial-gradient(circle at 80% 80%, rgba(60,100,160,0.25), rgba(10,10,20,0.95) 55%)'
          }} />
        </div>
      ) : null}

      {/* Loading spinner */}
      {bgLoading ? (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="h-10 w-10 rounded-full border-4 border-purple-500/20 border-t-purple-400/80 animate-spin" />
        </div>
      ) : null}
    </div>
  );
};

export default BackgroundLayer;
