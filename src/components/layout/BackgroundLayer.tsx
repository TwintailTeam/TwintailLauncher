import React from "react";

interface BackgroundLayerProps {
  currentSrc: string;
  previousSrc?: string;
  transitioning: boolean;
  bgVersion: number;
  popupOpen: boolean;
  onMainLoad?: () => void;
}

const BackgroundLayer: React.FC<BackgroundLayerProps> = ({
  currentSrc,
  previousSrc,
  transitioning,
  bgVersion,
  popupOpen,
  onMainLoad,
}) => {
  return (
    <div className="absolute inset-0 -z-10 pointer-events-none overflow-hidden">
      {transitioning && previousSrc && (
        <img
          key={`prev-${bgVersion}`}
          className={`w-full h-screen object-cover object-center absolute inset-0 transition-none animate-bg-fade-out ${popupOpen ? "scale-[1.03] brightness-[0.45] saturate-75" : ""}`}
          alt={"previous background"}
          src={previousSrc}
          loading="lazy"
          decoding="async"
        />
      )}
      <img
        id="app-bg"
        key={`curr-${bgVersion}`}
        className={`w-full h-screen object-cover object-center transition-all duration-300 ease-out ${transitioning ? "animate-bg-fade-in" : ""} ${popupOpen ? "scale-[1.03] brightness-[0.45] saturate-75" : ""}`}
        alt={"?"}
        src={currentSrc}
        loading="lazy"
        decoding="async"
        onLoad={() => onMainLoad && onMainLoad()}
      />
    </div>
  );
};

export default BackgroundLayer;
