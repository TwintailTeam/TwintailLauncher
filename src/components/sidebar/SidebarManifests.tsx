import React from "react";
import { POPUPS } from "../popups/POPUPS.ts";
import { PAGES } from "../pages/PAGES.ts";
import { PlusCircle, X } from "lucide-react";

export default function SidebarManifests({
  isOpen,
  onToggle,
  popup: _popup,
  hasInstalls,
  currentPage,
  setCurrentPage,
}: {
  isOpen: boolean,
  onToggle: () => void,
  popup: POPUPS,
  hasInstalls: boolean,
  currentPage?: PAGES,
  setCurrentPage?: (page: PAGES) => void,
}) {

  const handleClick = () => {
    // Close any open page when toggling manifests
    if (setCurrentPage && currentPage !== PAGES.NONE) {
      setCurrentPage(PAGES.NONE);
    }
    onToggle();
  };

  return (
    <React.Fragment>
      <div
        id="sidebar_manifests_toggle"
        className="group text-white hover:text-white/55 w-8 h-16 mb-0 cursor-pointer flex-initial relative flex items-center justify-center"
        onClick={handleClick}>
        {/* Animated icon swap: Add when closed -> X when open */}
        <span className="absolute inset-0 flex items-center justify-center">
          <PlusCircle
            className={`w-8 h-10 transition-all duration-300 ease-out ${isOpen ? 'opacity-0 scale-75 rotate-90' : 'opacity-100 scale-100 rotate-0'}`}
            aria-hidden="true"
          />
        </span>
        <span className="absolute inset-0 flex items-center justify-center">
          <X
            className={`w-8 h-10 transition-all duration-300 ease-out ${isOpen ? 'opacity-100 scale-100 rotate-0' : 'opacity-0 scale-75 -rotate-90'}`}
            aria-hidden="true"
          />
        </span>

        {/* Purple shining dot over the Add icon when there are no installs and panel is closed */}
        {!hasInstalls && !isOpen && (
          <>
            <span className="absolute top-4 right-0.5 flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-purple-500 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-purple-400 shadow-[0_0_8px_rgba(168,85,247,0.9)]"></span>
            </span>
          </>
        )}
      </div>

    </React.Fragment>
  );
}

