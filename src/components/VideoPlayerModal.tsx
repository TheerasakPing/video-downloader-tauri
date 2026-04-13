import React, { useState, useRef, useEffect } from "react";
import { X, Maximize2, Minimize2, Volume2, VolumeX } from "lucide-react";
import { Button } from "./Button";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "../hooks/useI18n";

interface VideoPlayerModalProps {
  isOpen: boolean;
  filePath: string;
  title?: string;
  onClose: () => void;
}

export const VideoPlayerModal: React.FC<VideoPlayerModalProps> = ({
  isOpen,
  filePath,
  title,
  onClose,
}) => {
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [isMuted, setIsMuted] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const { t } = useI18n();

  // Resolve file path using Tauri's expand_path command
  const [resolvedPath, setResolvedPath] = useState<string>("");

  useEffect(() => {
    const resolvePath = async () => {
      try {
        // Try to expand the path to get the full path
        const expanded = await invoke<string>("expand_path", { path: filePath });
        setResolvedPath(expanded);
      } catch {
        // If expand_path doesn't exist, use the original path with asset protocol
        if (filePath.startsWith("/")) {
          setResolvedPath(`asset://fs${filePath}`);
        } else {
          setResolvedPath(filePath);
        }
      }
    };

    if (isOpen && filePath) {
      resolvePath();
    }
  }, [isOpen, filePath]);

  // Handle ESC key to close
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isOpen, onClose]);

  // Toggle fullscreen
  const toggleFullscreen = async () => {
    if (!containerRef.current) return;

    if (!document.fullscreenElement) {
      try {
        await containerRef.current.requestFullscreen();
        setIsFullscreen(true);
      } catch {
        // Fullscreen might not be supported
      }
    } else {
      try {
        await document.exitFullscreen();
        setIsFullscreen(false);
      } catch {
        // Ignore errors
      }
    }
  };

  // Listen for fullscreen changes
  useEffect(() => {
    const handleFullscreenChange = () => {
      setIsFullscreen(!!document.fullscreenElement);
    };

    document.addEventListener("fullscreenchange", handleFullscreenChange);
    return () => document.removeEventListener("fullscreenchange", handleFullscreenChange);
  }, []);

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

  const toggleMute = () => {
    if (videoRef.current) {
      videoRef.current.muted = !videoRef.current.muted;
      setIsMuted(videoRef.current.muted);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      ref={containerRef}
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/95 backdrop-blur-sm animate-fade-in"
    >
      {/* Close button */}
      <button
        onClick={onClose}
        className="absolute top-4 right-4 z-10 p-2 hover:bg-white/10 rounded-lg transition-colors group"
        aria-label={t("player.close")}
      >
        <X size={24} className="text-slate-400 group-hover:text-white" />
      </button>

      {/* Video Container */}
      <div className="relative w-full h-full flex flex-col">
        {/* Header */}
        {title && (
          <div className="flex-shrink-0 px-6 py-4 bg-gradient-to-b from-black/80 to-transparent">
            <h2 className="text-white text-lg font-medium truncate">{title}</h2>
          </div>
        )}

        {/* Video Area */}
        <div className="flex-1 flex items-center justify-center p-4">
          <div className="relative w-full h-full max-w-6xl aspect-video bg-black rounded-lg overflow-hidden shadow-2xl">
            {isLoading && !hasError && (
              <div className="absolute inset-0 flex items-center justify-center bg-slate-900">
                <div className="flex flex-col items-center gap-3">
                  <div className="w-12 h-12 border-4 border-violet-500 border-t-transparent rounded-full animate-spin" />
                  <span className="text-slate-400 text-sm">{t("player.loading")}</span>
                </div>
              </div>
            )}

            {hasError ? (
              <div className="absolute inset-0 flex items-center justify-center bg-slate-900">
                <div className="flex flex-col items-center gap-3 text-center px-4">
                  <div className="w-16 h-16 rounded-full bg-red-500/20 flex items-center justify-center">
                    <X size={32} className="text-red-400" />
                  </div>
                  <div>
                    <p className="text-white font-medium">{t("player.failed")}</p>
                    <p className="text-slate-400 text-sm mt-1">
                      {t("player.fileNotSupported")}
                    </p>
                  </div>
                  <Button variant="secondary" onClick={onClose} className="mt-2">
                    {t("common.close")}
                  </Button>
                </div>
              </div>
            ) : resolvedPath ? (
              <video
                ref={videoRef}
                src={resolvedPath}
                className="w-full h-full"
                autoPlay
                controls
                onLoadStart={handleLoadStart}
                onCanPlay={handleCanPlay}
                onError={handleError}
              >
                {t("player.unsupported")}
              </video>
            ) : null}

            {/* Custom controls overlay */}
            {!hasError && resolvedPath && (
              <div className="absolute bottom-4 right-4 flex gap-2">
                <button
                  onClick={toggleMute}
                  className="p-2 bg-black/60 hover:bg-black/80 rounded-lg transition-colors"
                  title={isMuted ? t("player.unmute") : t("player.mute")}
                  aria-label={isMuted ? t("player.unmute") : t("player.mute")}
                >
                  {isMuted ? (
                    <VolumeX size={20} className="text-white" />
                  ) : (
                    <Volume2 size={20} className="text-white" />
                  )}
                </button>
                <button
                  onClick={toggleFullscreen}
                  className="p-2 bg-black/60 hover:bg-black/80 rounded-lg transition-colors"
                  title={isFullscreen ? t("player.exitFullscreen") : t("player.fullscreen")}
                  aria-label={t("player.toggleFullscreen")}
                >
                  {isFullscreen ? (
                    <Minimize2 size={20} className="text-white" />
                  ) : (
                    <Maximize2 size={20} className="text-white" />
                  )}
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default VideoPlayerModal;
