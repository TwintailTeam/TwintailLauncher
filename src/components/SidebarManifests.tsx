import React from "react";
import { POPUPS } from "./popups/POPUPS.ts";
import { DownloadIcon, X } from "lucide-react";

export default function SidebarManifests({
  isOpen,
  onToggle,
  popup: _popup
}: {
  isOpen: boolean,
  onToggle: () => void,
  popup: POPUPS
}) {

  return (
    <React.Fragment>
      <div
        className="group text-white hover:text-white/55 w-8 h-10 mb-0 cursor-pointer flex-initial relative flex items-center justify-center"
        onClick={onToggle}
        aria-label={isOpen ? 'Hide manifests' : 'Show manifests'}
        title={isOpen ? 'Hide manifests' : 'Show manifests'}
      >
        {/* Animated icon swap: Download when closed -> X when open */}
        <span className="absolute inset-0 flex items-center justify-center">
          <DownloadIcon
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
      </div>
      
    </React.Fragment>
  );
}
