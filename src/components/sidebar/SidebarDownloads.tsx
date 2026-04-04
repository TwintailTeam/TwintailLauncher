import React, { useEffect, useRef, useState } from "react";
import {
  arrow,
  autoUpdate,
  flip,
  FloatingArrow,
  offset,
  shift,
  useFloating,
  useHover,
  useInteractions,
} from "@floating-ui/react";
import { POPUPS } from "../popups/POPUPS";
import { PAGES } from "../pages/PAGES";
import { DownloadIcon } from "lucide-react";

export default function SidebarDownloads({
  setOpenPopup,
  popup,
  hasDownloads,
  queueCount,
  progressPercent,
  currentPage,
  setCurrentPage,
}: {
  setOpenPopup: (a: POPUPS) => void;
  popup: POPUPS;
  hasDownloads: boolean;
  queueCount: number;
  progressPercent: number | undefined;
  currentPage?: PAGES;
  setCurrentPage?: (page: PAGES) => void;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const [iconPop, setIconPop] = useState(false);
  const [addedBurstTick, setAddedBurstTick] = useState(0);
  const prevQueueCountRef = useRef<number | null>(null);
  const iconPopTimeoutRef = useRef<number | null>(null);

  const arrowRef = useRef(null);
  const { refs, floatingStyles, context } = useFloating({
    open: isOpen,
    onOpenChange: setIsOpen,
    middleware: [offset(25), flip(), shift(), arrow({ element: arrowRef })],
    whileElementsMounted: autoUpdate,
    placement: "right",
  });

  const hover = useHover(context, { move: false });
  const { getReferenceProps, getFloatingProps } = useInteractions([hover]);

  const ringPercent = progressPercent === undefined ? undefined : Math.max(0, Math.min(100, progressPercent));
  const normalizedQueueCount = Math.max(0, queueCount);
  const showQueueBadge = normalizedQueueCount >= 1;
  const queueBadgeLabel = normalizedQueueCount > 99 ? "99+" : String(normalizedQueueCount);
  const cx = 16;
  const cy = 20;
  const r = 14;
  const c = 2 * Math.PI * r;
  const dashOffset = ringPercent !== undefined ? c * (1 - ringPercent / 100) : c;

  const showActivityDot = hasDownloads && ringPercent === undefined && !showQueueBadge;
  const iconClass =
    ringPercent !== undefined
      ? "w-4 h-5"
      : "w-8 h-10";
  const burstIconClass =
    ringPercent !== undefined
      ? "w-7 h-8"
      : "w-10 h-12";

  const isActive = currentPage === PAGES.DOWNLOADS;

  useEffect(() => {
    if (prevQueueCountRef.current === null) {
      prevQueueCountRef.current = normalizedQueueCount;
      return;
    }

    if (normalizedQueueCount > prevQueueCountRef.current) {
      setAddedBurstTick((tick) => tick + 1);
      setIconPop(true);

      if (iconPopTimeoutRef.current !== null) {
        window.clearTimeout(iconPopTimeoutRef.current);
      }

      iconPopTimeoutRef.current = window.setTimeout(() => {
        setIconPop(false);
        iconPopTimeoutRef.current = null;
      }, 200);
    }

    prevQueueCountRef.current = normalizedQueueCount;
  }, [normalizedQueueCount]);

  useEffect(() => {
    return () => {
      if (iconPopTimeoutRef.current !== null) {
        window.clearTimeout(iconPopTimeoutRef.current);
      }
    };
  }, []);

  return (
    <React.Fragment>
      <div
        ref={refs.setReference}
        {...getReferenceProps()}
        className={`relative flex items-center justify-center w-10 h-10 rounded-xl cursor-pointer transition-all duration-200 ${isActive ? 'text-purple-400 bg-purple-500/15 shadow-[0_0_12px_rgba(147,51,234,0.3)]' : 'text-white/70 hover:text-white hover:bg-white/5 hover:shadow-[0_0_12px_rgba(147,51,234,0.15)]'} ${iconPop ? "animate-download-attention-pop" : ""} active:scale-95`}
        onClick={() => {
          if (setCurrentPage) {
            setCurrentPage(currentPage === PAGES.DOWNLOADS ? PAGES.NONE : PAGES.DOWNLOADS);
          } else {
            setOpenPopup(popup === POPUPS.DOWNLOADS ? POPUPS.NONE : POPUPS.DOWNLOADS);
          }
        }}
      >
        {ringPercent !== undefined && (
          <svg
            className="absolute inset-0 w-full h-full pointer-events-none drop-shadow-[0_0_8px_rgba(168,85,247,0.6)]"
            viewBox="0 0 32 40"
            aria-hidden="true"
          >
            <circle
              cx={cx}
              cy={cy}
              r={r}
              fill="none"
              stroke="currentColor"
              strokeWidth={3}
              className="text-white/10"
            />
            <circle
              cx={cx}
              cy={cy}
              r={r}
              fill="none"
              stroke="currentColor"
              strokeWidth={3}
              className="text-purple-400"
              strokeLinecap="round"
              strokeDasharray={c}
              strokeDashoffset={dashOffset}
              transform={`rotate(-90 ${cx} ${cy})`}
            />
          </svg>
        )}

        <DownloadIcon
          className={`relative z-10 flex-initial transition-all duration-200 ease-out ${iconClass}`}
        />

        {addedBurstTick > 0 && (
          <span
            key={addedBurstTick}
            aria-hidden="true"
            className="absolute inset-0 z-20 flex items-center justify-center pointer-events-none"
          >
            <DownloadIcon
              className={`${burstIconClass} text-purple-300/90 drop-shadow-[0_0_12px_rgba(168,85,247,0.9)] animate-download-added-burst`}
            />
          </span>
        )}

        {showQueueBadge && (
          <span
            key={`queue-badge-${normalizedQueueCount}-${addedBurstTick}`}
            className="absolute -top-1.5 -right-1.5 z-30 min-w-[16px] h-4 px-1 rounded-full bg-purple-500 text-[10px] leading-none font-bold text-white flex items-center justify-center shadow-[0_0_10px_rgba(168,85,247,0.85)] border border-white/20 animate-download-queue-badge-pop"
            aria-label={`${normalizedQueueCount} items in download queue`}
          >
            {queueBadgeLabel}
          </span>
        )}

        {showActivityDot && (
          <span className="absolute top-1 right-0.5 z-20 flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-purple-500 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-purple-400 shadow-[0_0_8px_rgba(168,85,247,0.9)]"></span>
          </span>
        )}
      </div>

      {isOpen && popup === POPUPS.NONE && currentPage === PAGES.NONE && (
        <div
          ref={refs.setFloating}
          style={floatingStyles}
          {...getFloatingProps()}
          className="bg-black/75 rounded-md p-2 min-w-max z-50"
        >
          <FloatingArrow ref={arrowRef} context={context} className="fill-black/75" />
          <span className="text-white z-50">Downloads</span>
        </div>
      )}
    </React.Fragment>
  );
}
