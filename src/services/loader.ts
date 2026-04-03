import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Events } from "../constants/events.ts";
import { registerEvents } from "./events.ts";
import { isLinux, clearFailedImages, getFailedImageCount, isImagePreloaded } from "../utils/imagePreloader.ts";
import { showDialogAsync } from "../context/DialogContext.tsx";

const IMAGE_PRELOAD_TIMEOUT_MS = 20000;

export interface NetworkStatus {
  status: "online" | "slow" | "offline";
  latency_ms: number | null;
  message: string;
}

export async function checkNetworkConnectivity(): Promise<NetworkStatus> {
  try {
    return await invoke<NetworkStatus>("check_network_connectivity");
  } catch (e) {
    console.error("Failed to check network connectivity:", e);
    return { status: "offline", latency_ms: null, message: "Failed to check connectivity" };
  }
}

/**
 * Default handler for network issues that shows a dialog to the user.
 * Returns true if user chooses to continue with limited experience, false otherwise.
 */
export async function handleNetworkIssue(status: NetworkStatus): Promise<boolean> {
  const isOffline = status.status === "offline";
  const title = isOffline ? "No Internet Connection" : "Slow Internet Connection";
  const message = isOffline
    ? "Unable to connect to the internet. You can continue with limited functionality - you'll be able to launch installed games but won't be able to download or update."
    : "Your internet connection appears to be slow. Downloads and updates may take longer than expected. You can continue with limited functionality or retry the connection.";

  const buttonIndex = await showDialogAsync({
    type: "warning",
    title,
    message,
    buttons: [
      { label: "Retry", variant: "secondary" },
      { label: "Continue (Limited)", variant: "primary" },
    ],
  });

  // Button index -1 = dialog failed to show, continue with limited mode
  // Button index 1 = "Continue (Limited)"
  return buttonIndex === -1 || buttonIndex === 1;
}
export type SetProgressFn = (progress: number, message: string) => void;

export interface LoaderOptions {
  fetchSettings: () => Promise<void>;
  fetchRepositories: () => Promise<void>;
  fetchCompatibilityVersions: () => Promise<void>;
  fetchCompatibilityVersionsFiltered: () => Promise<void>;
  fetchInstalledRunners: () => Promise<void>;
  fetchSteamRTStatus: () => Promise<void>;
  getGamesInfo: () => any[];
  getInstalls: () => any[];
  preloadImages: (
    images: string[],
    onProgress: (loaded: number, total: number) => void,
    preloaded: Set<string>
  ) => Promise<void>;
  preloadedBackgrounds: Set<string>;
  setProgress: SetProgressFn;
  completeInitialLoading: () => void;
  // Event state wiring
  pushInstalls: () => void;
  applyEventState: (ns: Record<string, any> | ((prev: any) => Record<string, any>)) => void;
  getCurrentInstall: () => string;
  fetchInstallResumeStates: (install: string) => void;
  // Network status handler - called when network issues detected
  onNetworkIssue?: (status: NetworkStatus) => Promise<boolean>; // returns true to continue, false to abort
}

export interface RecoveryProgress {
  phase: "checking" | "loading_repos" | "loading_images" | "complete" | "idle";
  current: number;
  total: number;
  message: string;
}

export type RecoveryProgressCallback = (progress: RecoveryProgress) => void;

export interface LoaderController {
  cancel: () => void;
}

export function startInitialLoad(opts: LoaderOptions): LoaderController {
  let cancelled = false;
  const unlistenFns: UnlistenFn[] = [];

  const run = async () => {
    try {
      if (cancelled) return;

      // Small delay to let React mount DialogProvider
      await new Promise(resolve => setTimeout(resolve, 100));

      if (cancelled) return;
      opts.setProgress(0, "Checking network connectivity...");

      // Step 0: Network connectivity check with retry support
      let networkCheckPassed = false;
      let limitedMode = false;
      while (!networkCheckPassed && !cancelled) {
        try {
          const networkStatus = await checkNetworkConnectivity();
          if (cancelled) return;

          if (networkStatus.status === "online") {
            networkCheckPassed = true;
          } else {
            // Use custom handler or default handler
            const handler = opts.onNetworkIssue || handleNetworkIssue;
            const shouldContinue = await handler(networkStatus);

            if (shouldContinue) {
              // User chose to continue with limited experience
              networkCheckPassed = true;
              limitedMode = true;
              opts.applyEventState({ limitedMode: true, networkStatus: networkStatus.status });
            }
            // If not continuing, loop will retry the check
          }
        } catch (e) {
          console.error("Error checking network connectivity:", e);
          // Continue anyway if check fails
          networkCheckPassed = true;
        }
      }

      if (cancelled) return;

      if (limitedMode) {
        opts.setProgress(5, "Loading (limited mode)...");
      }

      if (cancelled) return;
      opts.setProgress(5, "Loading settings and repositories...");

      // Step 1+2: Settings and repositories in parallel (independent data)
      try {
        const [, ] = await Promise.all([
          opts.fetchSettings().catch(e => { console.error("Error loading settings:", e); }),
          opts.fetchRepositories().catch(e => { console.error("Error loading repositories:", e); }),
        ]);
        // Load runners if application is running on Linux (all independent, fetch in parallel)
        if (window.navigator.platform.includes("Linux")) {
          await Promise.all([
            opts.fetchCompatibilityVersions(),
            opts.fetchCompatibilityVersionsFiltered(),
            opts.fetchInstalledRunners(),
            opts.fetchSteamRTStatus(),
          ]);
        }
        if (cancelled) return;
        opts.setProgress(50, "Loading game data...");
      } catch (e) {
        console.error("Error during initial data load:", e);
        opts.setProgress(50, "Error loading data...");
      }

      // Step 3: Wait for gamesinfo to be populated (polling)
      let tries = 0;
      while (!cancelled && opts.getGamesInfo().length === 0 && tries < 40) {
        await new Promise((res) => setTimeout(res, 50));
        tries++;
      }
      if (cancelled) return;
      opts.setProgress(75, "Preloading images...");

      // Step 4: Preload images (backgrounds + icons) including installed assets for older/different versions
      // Also preload live/dynamic backgrounds for smooth transitions
      const games = opts.getGamesInfo() || [];
      const installs = opts.getInstalls ? (opts.getInstalls() || []) : [];
      const gameBackgrounds: string[] = games.map((g: any) => g?.assets?.game_background).filter(Boolean);
      // Skip live backgrounds on Linux - video backgrounds not supported
      const gameLiveBackgrounds: string[] = isLinux ? [] : games.map((g: any) => g?.assets?.game_live_background).filter(Boolean);
      const gameIcons: string[] = games.map((g: any) => g?.assets?.game_icon).filter(Boolean);
      const installBackgrounds: string[] = installs.map((i: any) => i?.game_background).filter(Boolean);
      const installIcons: string[] = installs.map((i: any) => i?.game_icon).filter(Boolean);
      // Deduplicate all URLs - live backgrounds are preloaded first for immediate use (if not on Linux)
      const images = Array.from(new Set([
        ...gameLiveBackgrounds,
        ...gameBackgrounds,
        ...gameIcons,
        ...installBackgrounds,
        ...installIcons
      ]));
      try {
        await Promise.race([
          opts.preloadImages(
            images,
            (loaded, total) => {
              if (cancelled) return;
              const progress = 75 + Math.round((loaded / total) * 25);
              opts.setProgress(progress, `Preloading images... (${loaded}/${total})`);
            },
            opts.preloadedBackgrounds
          ),
          // timeout
          new Promise<void>((resolve) => {
            setTimeout(() => {
              console.warn(`Image preloading did not finish within ${IMAGE_PRELOAD_TIMEOUT_MS}ms, continuing startup.`);
              resolve();
            }, IMAGE_PRELOAD_TIMEOUT_MS);
          }),
        ]);
      } catch (e) { console.error("Error during image preloading:", e); }
      if (cancelled) return;
      opts.setProgress(100, "Almost ready...");

      // Complete loading visuals immediately
      opts.completeInitialLoading();

      // Register events slightly later (to allow UI to settle)
      setTimeout(async () => {
        if (cancelled) return;
        for (const eventType of Events) {
          const unlisten = await listen(eventType, (event) => {
            if (cancelled) return;
            const ns = registerEvents(
              eventType,
              event,
              opts.pushInstalls,
              opts.getCurrentInstall,
              opts.fetchInstallResumeStates,
              opts.fetchInstalledRunners,
              opts.fetchSteamRTStatus
            );
            if (ns !== undefined) opts.applyEventState(ns);
          });
          unlistenFns.push(unlisten);
        }

        // Fetch current download queue state to sync after refresh
        // This ensures ongoing downloads are visible immediately after frontend reload
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          const currentQueueState = await invoke('get_download_queue_state');
          if (currentQueueState && !cancelled) {
            const parsed = JSON.parse(currentQueueState as string);
            opts.applyEventState({ downloadQueueState: parsed });
          }
        } catch (e) {
          console.warn("Could not fetch initial download queue state:", e);
        }
      }, 20);
    } catch (e) {
      console.error("Loader error:", e);
    }
  };

  // Fire and forget
  run();

  return {
    cancel: () => {
      cancelled = true;
      // Unregister all listeners
      unlistenFns.forEach((fn) => {
        try { fn(); } catch { }
      });
    },
  };
}

/**
 * Network monitor that periodically checks connectivity and triggers recovery when online.
 */
export class NetworkMonitor {
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private isRecovering = false;
  private checkIntervalMs: number;
  private onStatusChange: (status: NetworkStatus, isRecovering: boolean) => void;
  private onRecoveryProgress: RecoveryProgressCallback;
  private onConnectivityLost?: () => void;
  private previousStatus: NetworkStatus["status"] = "online";
  private consecutiveFailures = 0;
  private static readonly FAILURE_THRESHOLD = 3;
  private recoveryOpts: Pick<LoaderOptions, 'fetchRepositories' | 'fetchCompatibilityVersions' | 'fetchCompatibilityVersionsFiltered' | 'fetchInstalledRunners' | 'fetchSteamRTStatus' | 'getGamesInfo' | 'getInstalls' | 'preloadImages' | 'preloadedBackgrounds' | 'applyEventState'> | null = null;

  constructor(
    onStatusChange: (status: NetworkStatus, isRecovering: boolean) => void,
    onRecoveryProgress: RecoveryProgressCallback,
    checkIntervalMs: number = 15000, // Check every 15 seconds
    onConnectivityLost?: () => void
  ) {
    this.onStatusChange = onStatusChange;
    this.onRecoveryProgress = onRecoveryProgress;
    this.checkIntervalMs = checkIntervalMs;
    this.onConnectivityLost = onConnectivityLost;
  }

  /**
   * Set the recovery options (functions needed to reload data)
   */
  setRecoveryOptions(opts: typeof this.recoveryOpts) {
    this.recoveryOpts = opts;
  }

  /**
   * Start monitoring network connectivity
   */
  start() {
    if (this.intervalId) return;

    this.intervalId = setInterval(async () => {
      await this.check();
    }, this.checkIntervalMs);

    // Also do an immediate check
    this.check();
  }

  /**
   * Stop monitoring
   */
  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /**
   * Manually trigger a check and recovery
   */
  async check(): Promise<NetworkStatus> {
    const status = await checkNetworkConnectivity();

    if (status.status !== "online") {
      this.consecutiveFailures++;
    } else {
      this.consecutiveFailures = 0;
    }

    // Only trigger connectivity loss after consecutive failures to avoid false positives
    // during download congestion or transient network hiccups
    if (this.previousStatus === "online" && status.status !== "online" && this.consecutiveFailures >= NetworkMonitor.FAILURE_THRESHOLD) {
      if (this.onConnectivityLost) {
        this.onConnectivityLost();
      }
      if (this.recoveryOpts) {
        this.recoveryOpts.applyEventState({ limitedMode: true, networkStatus: status.status });
      }
      this.previousStatus = status.status;
    } else if (status.status === "online") {
      this.previousStatus = status.status;
    }
    // If not yet at threshold, keep previousStatus as "online" so the transition
    // logic can still fire once the threshold is reached

    this.onStatusChange(status, this.isRecovering);
    return status;
  }

  /**
   * Trigger recovery - reload repositories and images
   */
  async triggerRecovery(): Promise<boolean> {
    if (this.isRecovering || !this.recoveryOpts) return false;

    this.isRecovering = true;
    this.onRecoveryProgress({ phase: "checking", current: 0, total: 0, message: "Checking connection..." });

    try {
      // Check connectivity first
      const status = await checkNetworkConnectivity();
      if (status.status !== "online") {
        this.onRecoveryProgress({ phase: "idle", current: 0, total: 0, message: "Still offline" });
        this.isRecovering = false;
        return false;
      }

      // Clear failed images so they can be retried
      const failedCount = getFailedImageCount();
      if (failedCount > 0) {
        console.log(`Clearing ${failedCount} failed images for retry`);
        clearFailedImages();
      }

      // Reload repositories
      this.onRecoveryProgress({ phase: "loading_repos", current: 0, total: 1, message: "Loading repositories..." });
      try {
        await this.recoveryOpts.fetchRepositories();
        if (window.navigator.platform.includes("Linux")) {
          await Promise.all([
            this.recoveryOpts.fetchCompatibilityVersions(),
            this.recoveryOpts.fetchCompatibilityVersionsFiltered(),
            this.recoveryOpts.fetchInstalledRunners(),
            this.recoveryOpts.fetchSteamRTStatus(),
          ]);
        }
      } catch (e) {
        console.error("Error loading repositories during recovery:", e);
      }

      // Wait for games data
      let tries = 0;
      while (this.recoveryOpts.getGamesInfo().length === 0 && tries < 40) {
        await new Promise((res) => setTimeout(res, 50));
        tries++;
      }

      // Preload images
      const games = this.recoveryOpts.getGamesInfo() || [];
      const installs = this.recoveryOpts.getInstalls ? (this.recoveryOpts.getInstalls() || []) : [];
      const gameBackgrounds: string[] = games.map((g: any) => g?.assets?.game_background).filter(Boolean);
      const gameLiveBackgrounds: string[] = isLinux ? [] : games.map((g: any) => g?.assets?.game_live_background).filter(Boolean);
      const gameIcons: string[] = games.map((g: any) => g?.assets?.game_icon).filter(Boolean);
      const installBackgrounds: string[] = installs.map((i: any) => i?.game_background).filter(Boolean);
      const installIcons: string[] = installs.map((i: any) => i?.game_icon).filter(Boolean);

      const allImages = Array.from(new Set([
        ...gameLiveBackgrounds,
        ...gameBackgrounds,
        ...gameIcons,
        ...installBackgrounds,
        ...installIcons
      ]));

      // Filter to only images not already confirmed as successfully preloaded
      const imagesToLoad = allImages.filter((img) => !isImagePreloaded(img));

      if (imagesToLoad.length > 0) {
        this.onRecoveryProgress({
          phase: "loading_images",
          current: 0,
          total: imagesToLoad.length,
          message: `Loading images... (0/${imagesToLoad.length})`
        });

        try {
          await Promise.race([
            this.recoveryOpts.preloadImages(
              imagesToLoad,
              (loaded, total) => {
                this.onRecoveryProgress({
                  phase: "loading_images",
                  current: loaded,
                  total,
                  message: `Loading images... (${loaded}/${total})`
                });
              },
              this.recoveryOpts.preloadedBackgrounds
            ),
            new Promise<void>((resolve) => {
              setTimeout(() => {
                console.warn("Image preloading during recovery timed out");
                resolve();
              }, IMAGE_PRELOAD_TIMEOUT_MS);
            }),
          ]);
        } catch (e) {
          console.error("Error preloading images during recovery:", e);
        }
      }

      // Recovery complete
      this.onRecoveryProgress({ phase: "complete", current: 0, total: 0, message: "Connected!" });
      this.recoveryOpts.applyEventState({ limitedMode: false, networkStatus: "online" });

      // Reset to idle after a short delay
      setTimeout(() => {
        this.onRecoveryProgress({ phase: "idle", current: 0, total: 0, message: "" });
      }, 2000);

      this.isRecovering = false;
      return true;
    } catch (e) {
      console.error("Recovery failed:", e);
      this.onRecoveryProgress({ phase: "idle", current: 0, total: 0, message: "Recovery failed" });
      this.isRecovering = false;
      return false;
    }
  }

  /**
   * Check if currently recovering
   */
  get recovering(): boolean {
    return this.isRecovering;
  }
}
