import React, { useState, useRef } from "react";
import { Play, FileVideo, AlertCircle } from "lucide-react";

interface VideoPreviewThumbnailProps {
  filePath: string;
  thumbnailTime?: number;
  onClick?: () => void;
  className?: string;
}

export const VideoPreviewThumbnail: React.FC<VideoPreviewThumbnailProps> = ({
  filePath,
  thumbnailTime = 5,
  onClick,
  className = "",
}) => {
  const [isHovered, setIsHovered] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  // Convert file path to asset protocol URL
  const getAssetUrl = (path: string) => {
    // For local files, use the asset protocol
    if (path.startsWith("/")) {
      // Tauri v2 uses asset:// for local files
      return `asset://fs${path}`;
    }
    return path;
  };

  const handleMouseEnter = () => {
    setIsHovered(true);
    if (videoRef.current && !hasError) {
      videoRef.current.currentTime = thumbnailTime;
      videoRef.current
        .play()
        .catch(() => {
          // Auto-play might be blocked, that's okay
        });
    }
  };

  const handleMouseLeave = () => {
    setIsHovered(false);
    if (videoRef.current) {
      videoRef.current.pause();
      videoRef.current.currentTime = 0;
    }
  };

  const handleLoadStart = () => {
    setIsLoading(true);
    setHasError(false);
  };

  const handleCanPlay = () => {
    setIsLoading(false);
  };

  const handleError = () => {
    setIsLoading(false);
    setHasError(true);
  };

  return (
    <div
      className={`relative aspect-video bg-slate-800 rounded-lg overflow-hidden cursor-pointer group ${className}`}
      onClick={onClick}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {isLoading && !hasError && (
        <div className="absolute inset-0 flex items-center justify-center bg-slate-900">
          <div className="flex flex-col items-center gap-2">
            <FileVideo size={32} className="text-slate-600" />
            <span className="text-xs text-slate-500">Loading...</span>
          </div>
        </div>
      )}

      {hasError ? (
        <div className="absolute inset-0 flex items-center justify-center bg-slate-900">
          <div className="flex flex-col items-center gap-2 text-center px-2">
            <AlertCircle size={24} className="text-amber-500" />
            <span className="text-xs text-slate-400">Preview unavailable</span>
          </div>
        </div>
      ) : (
        <video
          ref={videoRef}
          src={getAssetUrl(filePath)}
          className="w-full h-full object-cover"
          muted
          playsInline
          preload="metadata"
          onLoadStart={handleLoadStart}
          onCanPlay={handleCanPlay}
          onError={handleError}
        />
      )}

      {/* Play button overlay */}
      {!isHovered && !hasError && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/30 opacity-0 group-hover:opacity-100 transition-opacity">
          <div className="w-12 h-12 rounded-full bg-violet-500/80 flex items-center justify-center shadow-lg drop-shadow-[0_0_12px_rgba(139,92,246,0.5)]">
            <Play size={20} className="text-white ml-1" />
          </div>
        </div>
      )}

      {/* Hover indicator */}
      {isHovered && !hasError && (
        <div className="absolute top-2 right-2 px-2 py-1 bg-black/60 rounded text-xs text-white">
          Preview
        </div>
      )}
    </div>
  );
};

export default VideoPreviewThumbnail;
