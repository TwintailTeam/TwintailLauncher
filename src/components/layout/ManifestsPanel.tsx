import React, { useEffect } from "react";
import SidebarIconManifest from "../sidebar/SidebarIconManifest.tsx";
import { POPUPS } from "../popups/POPUPS.ts";
import { isLinux } from "../../utils/imagePreloader";
import { motion } from "framer-motion";

interface GameInfoItem {
  manifest_enabled: boolean;
  assets: { game_icon: string; game_background: string, game_live_background: string };
  filename: string;
  icon: string;
  display_name: string;
  biz: string;
}

interface ManifestsPanelProps {
  openPopup: POPUPS;
  manifestsOpenVisual: boolean;
  manifestsInitialLoading: boolean;
  gamesinfo: GameInfoItem[];
  manifestsPanelRef: React.RefObject<HTMLDivElement>;
  currentGame?: string;
  setCurrentGame: (id: string) => void;
  setOpenPopup: (p: POPUPS) => void;
  setDisplayName: (name: string) => void;
  setBackground: (src: string) => void;
  setCurrentInstall: (id: string) => void;
  setGameIcon: (src: string) => void;
  onRequestClose?: () => void;
  imageVersion?: number; // Used to force image re-load after network recovery
}

const ManifestsPanel: React.FC<ManifestsPanelProps> = ({
  openPopup,
  manifestsOpenVisual,
  gamesinfo,
  manifestsPanelRef,
  currentGame,
  setCurrentGame,
  setOpenPopup,
  setDisplayName,
  setBackground,
  setCurrentInstall,
  setGameIcon,
  onRequestClose,
  imageVersion = 0,
}) => {
  // Close when clicking anywhere outside the manifests panel
  useEffect(() => {
    const handleDocMouseDown = (e: MouseEvent) => {
      if (!manifestsOpenVisual || openPopup !== POPUPS.NONE) return;
      // Ignore clicks on the sidebar toggle to avoid conflicting state flips
      const toggle = document.getElementById('sidebar_manifests_toggle');
      if (toggle && toggle.contains(e.target as Node)) return;
      const panel = manifestsPanelRef.current;
      if (!panel) return;
      if (!panel.contains(e.target as Node)) {
        onRequestClose?.();
      }
    };
    document.addEventListener('mousedown', handleDocMouseDown);
    return () => document.removeEventListener('mousedown', handleDocMouseDown);
  }, [manifestsOpenVisual, openPopup, manifestsPanelRef, onRequestClose]);

  const variantsContainer = {
    closed: {
      clipPath: "inset(0 100% 0 0)",
      opacity: 0,
      transition: {
        type: "spring",
        stiffness: 500,
        damping: 40,
        staggerChildren: 0.02,
        staggerDirection: -1
      }
    },
    open: {
      clipPath: "inset(0 0% 0 0)",
      opacity: 1,
      transition: {
        type: "spring",
        stiffness: 450,
        damping: 35,
        staggerChildren: 0.04,
        delayChildren: 0
      }
    },
    hidden: {
      opacity: 0,
      y: -10,
      pointerEvents: "none" as const, // explicitly cast to prevent type issues
    }
  };

  const variantsItem = {
    closed: { opacity: 0, x: -20, scale: 0.8 },
    open: {
      opacity: 1,
      x: 0,
      scale: 1,
      transition: { type: "spring", stiffness: 450, damping: 25 }
    }
  };

  // Determine the animation state
  let animationState = "closed";
  if (openPopup === POPUPS.NONE) {
    if (manifestsOpenVisual) {
      animationState = "open";
    }
  } else {
    // If a popup is open, we hide the panel entirely, similar to the original logic
    animationState = "hidden";
  }

  return (
    <div className="absolute top-0 left-16 right-0 z-20 pointer-events-none">
      <div className="pl-3 pt-2 pr-6">
        <motion.div
          ref={manifestsPanelRef}
          className="relative inline-flex rounded-2xl border border-white/10 bg-black/50 shadow-2xl overflow-hidden pointer-events-auto origin-left"
          initial="closed"
          animate={animationState}
          // @ts-ignore
          variants={variantsContainer}
        >
          <div className="flex flex-row items-center gap-2 overflow-x-auto px-3 py-2 scrollbar-none select-none">
            {gamesinfo.map((game, _index) => {
              // Use dynamic background if available (skip on Linux), otherwise fall back to static
              // Using || handles undefined, null, and empty string cases
              let bg = (!isLinux && game.assets.game_live_background) || game.assets.game_background;

              return (
                <motion.div
                  key={`${game.biz}-v${imageVersion}`}
                  // @ts-ignore
                  variants={variantsItem}
                  layout
                >
                  <SidebarIconManifest
                    variant="floating"
                    sizeClass="w-12 h-12"
                    popup={openPopup}
                    icon={game.assets.game_icon}
                    background={bg}
                    name={game.display_name}
                    enabled={game.manifest_enabled}
                    id={game.biz}
                    currentGame={currentGame}
                    setCurrentGame={setCurrentGame}
                    setOpenPopup={setOpenPopup}
                    setDisplayName={setDisplayName}
                    setBackground={setBackground}
                    setCurrentInstall={setCurrentInstall}
                    setGameIcon={setGameIcon}
                  />
                </motion.div>
              )
            })}
          </div>

          {/* Edge fades to hint horizontal scroll */}
          {/* We treat these as purely decorative; no need to animate individually, they just fade with container */}
          <div className="pointer-events-none absolute top-0 left-0 h-full w-10 bg-gradient-to-r from-black/50 to-transparent" />
          <div className="pointer-events-none absolute top-0 right-0 h-full w-12 bg-gradient-to-l from-black/50 to-transparent" />
        </motion.div>
      </div>
    </div>
  );
};

export default ManifestsPanel;
