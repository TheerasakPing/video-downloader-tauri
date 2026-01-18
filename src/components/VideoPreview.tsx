import React, { useState } from "react";
import { Eye, X, Play, Loader2 } from "lucide-react";
import { Button } from "./Button";

interface VideoPreviewProps {
  isOpen: boolean;
  onClose: () => void;
  videoUrl?: string;
  episodeNumber?: number;
  seriesTitle?: string;
}

export const VideoPreview: React.FC<VideoPreviewProps> = ({
  isOpen,
  onClose,
  videoUrl,
  episodeNumber,
  seriesTitle,
}) => {
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/80 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative w-full max-w-4xl glass rounded-2xl border border-slate-700/50 shadow-2xl animate-scale-in overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700/50">
          <div className="flex items-center gap-2">
            <span className="icon-glow icon-glow-sm icon-glow-violet">
              <Eye size={16} />
            </span>
            <span className="text-sm font-medium text-white">
              {seriesTitle ? `${seriesTitle} - Episode ${episodeNumber}` : `Episode ${episodeNumber} Preview`}
            </span>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-red-500/20 rounded-lg transition-colors"
          >
            <X size={18} className="text-slate-400 hover:text-red-400" />
          </button>
        </div>

        {/* Video Container */}
        <div className="relative aspect-video bg-black">
          {isLoading && !hasError && (
            <div className="absolute inset-0 flex items-center justify-center bg-slate-900">
              <div className="flex flex-col items-center gap-3">
                <Loader2 size={48} className="text-violet-400 animate-spin" />
                <span className="text-slate-400 text-sm">Loading preview...</span>
              </div>
            </div>
          )}

          {hasError ? (
            <div className="absolute inset-0 flex items-center justify-center bg-slate-900">
              <div className="flex flex-col items-center gap-3 text-center px-4">
                <div className="icon-glow icon-glow-lg icon-glow-red">
                  <X size={32} />
                </div>
                <span className="text-slate-300 font-medium">Preview not available</span>
                <span className="text-slate-500 text-sm">
                  This video cannot be previewed. Try downloading it instead.
                </span>
              </div>
            </div>
          ) : videoUrl ? (
            <video
              src={videoUrl}
              controls
              autoPlay
              className="w-full h-full"
              onLoadStart={() => setIsLoading(true)}
              onCanPlay={() => setIsLoading(false)}
              onError={() => {
                setIsLoading(false);
                setHasError(true);
              }}
            >
              Your browser does not support the video tag.
            </video>
          ) : (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="flex flex-col items-center gap-3">
                <div className="icon-glow icon-glow-lg icon-glow-violet">
                  <Play size={48} />
                </div>
                <span className="text-slate-400 text-sm">Select an episode to preview</span>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 px-4 py-3 border-t border-slate-700/50">
          <Button variant="secondary" onClick={onClose}>
            Close
          </Button>
        </div>
      </div>
    </div>
  );
};

export default VideoPreview;
