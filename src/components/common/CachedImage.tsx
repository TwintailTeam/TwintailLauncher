import { useEffect, useRef, useState } from "react";
import { cacheImage, isImageFailed, isImagePreloaded, isVideoUrl } from "../../utils/imagePreloader";

interface CachedImageProps {
    src: string;
    alt?: string;
    className?: string;
}

export function CachedImage({ src, alt = "", className = "" }: CachedImageProps) {
    const currentSrcRef = useRef(src);
    const [isReady, setIsReady] = useState(() => !!src && isImagePreloaded(src) && !isImageFailed(src));

    useEffect(() => {
        currentSrcRef.current = src;
        setIsReady(!!src && isImagePreloaded(src) && !isImageFailed(src));
    }, [src]);

    if (!src) {
        return <div className="contents" data-ready={false} />;
    }

    const isVideo = isVideoUrl(src);

    return (
        <div className="contents" data-ready={isReady}>
            {isVideo ? (
                <video
                    key={`video-${src}`}
                    src={src}
                    className={className}
                    muted
                    playsInline
                    autoPlay
                    loop
                    preload="auto"
                    onLoadedData={(e) => {
                        if (currentSrcRef.current !== src) return;
                        cacheImage(src, e.currentTarget, false);
                        setIsReady(true);
                        e.currentTarget.play().catch(() => { });
                    }}
                    onError={(e) => {
                        if (currentSrcRef.current !== src) return;
                        cacheImage(src, e.currentTarget, true);
                        setIsReady(true);
                    }}
                />
            ) : (
                <img
                    key={`img-${src}`}
                    src={src}
                    alt={alt}
                    className={className}
                    loading="eager"
                    decoding="async"
                    onLoad={(e) => {
                        if (currentSrcRef.current !== src) return;
                        const img = e.currentTarget;
                        const complete = () => {
                            cacheImage(src, img, false);
                            setIsReady(true);
                        };

                        if (typeof img.decode === "function") {
                            img.decode().catch(() => { }).finally(complete);
                        } else {
                            complete();
                        }
                    }}
                    onError={(e) => {
                        if (currentSrcRef.current !== src) return;
                        cacheImage(src, e.currentTarget, true);
                        setIsReady(true);
                    }}
                />
            )}
        </div>
    );
}

export default CachedImage;
