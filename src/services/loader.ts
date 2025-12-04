import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from "@tauri-apps/api/event";
import { Events } from "../constants/events.ts";
import { registerEvents } from "./events.ts";

export type SetProgressFn = (progress: number, message: string) => void;

export interface LoaderOptions {
  fetchSettings: () => Promise<void>;
  fetchRepositories: () => Promise<void>;
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
  applyEventState: (ns: Record<string, any>) => void;
  getCurrentInstall: () => string;
  fetchInstallResumeStates: (install: string) => void;
}

export interface LoaderController {
  cancel: () => void;
}

export function startInitialLoad(opts: LoaderOptions): LoaderController {
  let cancelled = false;
  const unlistenFns: UnlistenFn[] = [];

  const run = async () => {
    try {
      if (cancelled) return;
      opts.setProgress(0, "Loading settings...");

      // Step 1: Settings
      try {
        await opts.fetchSettings();
        if (cancelled) return;
        opts.setProgress(25, "Connecting to repositories...");
      } catch (e) {
        console.error("Error loading settings:", e);
        opts.setProgress(0, "Error loading settings...");
      }

      // Step 2: Repositories
      try {
        await opts.fetchRepositories();
        if (cancelled) return;
        opts.setProgress(50, "Loading game data...");
      } catch (e) {
        console.error("Error loading repositories:", e);
        opts.setProgress(50, "Error loading repositories...");
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
      const games = opts.getGamesInfo() || [];
      const installs = opts.getInstalls ? (opts.getInstalls() || []) : [];
      const gameBackgrounds: string[] = games.map((g: any) => g?.assets?.game_background).filter(Boolean);
      //const gameLiveBackgrounds: string[] = games.map((g: any) => g?.assets?.game_live_background).filter(Boolean);
      const gameIcons: string[] = games.map((g: any) => g?.assets?.game_icon).filter(Boolean);
      const installBackgrounds: string[] = installs.map((i: any) => i?.game_background).filter(Boolean);
      const installIcons: string[] = installs.map((i: any) => i?.game_icon).filter(Boolean);
      const images = Array.from(new Set([/*...(gameLiveBackgrounds as string[]), */...(gameBackgrounds as string[]), ...(gameIcons as string[]), ...(installBackgrounds as string[]), ...(installIcons as string[])]));
      await opts.preloadImages(
        images,
        (loaded, total) => {
          if (cancelled) return;
          const progress = 75 + Math.round((loaded / total) * 25);
          opts.setProgress(progress, `Preloading images... (${loaded}/${total})`);
        },
        opts.preloadedBackgrounds
      );
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
            const ns = registerEvents(eventType, event, opts.pushInstalls, opts.getCurrentInstall, opts.fetchInstallResumeStates);
            if (ns !== undefined) opts.applyEventState(ns);
          });
          unlistenFns.push(unlisten);
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
        try { fn(); } catch {}
      });
      
      
      
    },
  };
}
