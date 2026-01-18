import React from "react";
import { Maximize2, X, Download, Pause, Play } from "lucide-react";
import { ProgressBar } from "./ProgressBar";

interface MiniModeProps {
  isOpen: boolean;
  onClose: () => void;
  onExpand: () => void;
  progress: {
    episode: number;
    percentage: number;
    speed: number;
  };
  overallProgress: number;
  completedEpisodes: number;
  totalEpisodes: number;
  isPaused: boolean;
  onPause: () => void;
  onResume: () => void;
  seriesTitle?: string;
}

export const MiniMode: React.FC<MiniModeProps> = ({
  isOpen,
  onClose,
  onExpand,
  progress,
  overallProgress,
  completedEpisodes,
  totalEpisodes,
  isPaused,
  onPause,
  onResume,
  seriesTitle,
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 w-80 glass rounded-2xl border border-slate-700/50 shadow-2xl animate-slide-in">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-slate-700/50">
        <div className="flex items-center gap-2">
          <span className="icon-glow icon-glow-sm icon-glow-violet icon-glow-animated">
            <Download size={14} />
          </span>
          <span className="text-sm font-medium text-white truncate max-w-[160px]">
            {seriesTitle || "Downloading..."}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={onExpand}
            className="p-1.5 hover:bg-slate-700/50 rounded-lg transition-colors"
            title="Expand"
          >
            <Maximize2 size={14} className="text-slate-400" />
          </button>
          <button
            onClick={onClose}
            className="p-1.5 hover:bg-red-500/20 rounded-lg transition-colors"
            title="Close"
          >
            <X size={14} className="text-slate-400 hover:text-red-400" />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="p-4 space-y-3">
        {/* Current Episode */}
        <div>
          <div className="flex justify-between text-xs text-slate-400 mb-1">
            <span>Episode {progress.episode}</span>
            <span>{(progress.speed / 1024 / 1024).toFixed(1)} MB/s</span>
          </div>
          <ProgressBar percentage={progress.percentage} />
        </div>

        {/* Overall Progress */}
        <div>
          <div className="flex justify-between text-xs text-slate-400 mb-1">
            <span>Overall</span>
            <span>
              {completedEpisodes}/{totalEpisodes}
            </span>
          </div>
          <ProgressBar percentage={overallProgress} variant="success" />
        </div>

        {/* Controls */}
        <div className="flex justify-center pt-1">
          <button
            onClick={isPaused ? onResume : onPause}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all ${
              isPaused
                ? "bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30"
                : "bg-amber-500/20 text-amber-400 hover:bg-amber-500/30"
            }`}
          >
            {isPaused ? (
              <>
                <Play size={16} />
                <span className="text-sm">Resume</span>
              </>
            ) : (
              <>
                <Pause size={16} />
                <span className="text-sm">Pause</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default MiniMode;
