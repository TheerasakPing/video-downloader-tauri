import React from "react";
import { Download, X, RefreshCw, Sparkles, ArrowUpCircle, ExternalLink } from "lucide-react";
import { Button } from "./Button";
import { ProgressBar } from "./ProgressBar";
import { UpdateInfo } from "../hooks/useUpdater";

interface UpdateDialogProps {
  isOpen: boolean;
  updateInfo: UpdateInfo | null;
  downloading: boolean;
  progress: number;
  error: string | null;
  onDownload: () => void;
  onDismiss: () => void;
}

export const UpdateDialog: React.FC<UpdateDialogProps> = ({
  isOpen,
  updateInfo,
  downloading,
  progress,
  error,
  onDownload,
  onDismiss,
}) => {
  if (!isOpen || !updateInfo) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 animate-fade-in">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onDismiss}
      />

      {/* Dialog */}
      <div className="relative glass rounded-2xl border border-slate-700/50 p-6 max-w-md w-full shadow-2xl animate-scale-in">
        {/* Close button */}
        <button
          onClick={onDismiss}
          className="absolute top-4 right-4 p-1 text-slate-400 hover:text-white transition-colors"
        >
          <X size={20} />
        </button>

        {/* Icon */}
        <div className="flex justify-center mb-4">
          <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-emerald-500 to-teal-500 flex items-center justify-center shadow-lg shadow-emerald-500/20 animate-bounce-subtle">
            <Sparkles size={32} className="text-white" />
          </div>
        </div>

        {/* Title */}
        <h2 className="text-xl font-bold text-center mb-2">
          Update Available!
        </h2>

        {/* Version info */}
        <div className="flex items-center justify-center gap-2 text-sm text-slate-400 mb-4">
          <span>{updateInfo.currentVersion}</span>
          <ArrowUpCircle size={16} className="text-emerald-400" />
          <span className="text-emerald-400 font-medium">{updateInfo.version}</span>
        </div>

        {/* Release notes */}
        <div className="bg-slate-800/50 rounded-xl p-4 mb-4 max-h-40 overflow-y-auto">
          <h3 className="text-sm font-medium text-slate-300 mb-2">What's New:</h3>
          <div className="text-sm text-slate-400 whitespace-pre-wrap">
            {updateInfo.body}
          </div>
        </div>

        {/* Progress bar when downloading */}
        {downloading && (
          <div className="mb-4 animate-fade-in">
            <ProgressBar
              percentage={progress}
              label="Downloading update..."
              sublabel={`${progress.toFixed(0)}%`}
              variant="success"
            />
          </div>
        )}

        {/* Error message */}
        {error && (
          <div className="mb-4 p-3 bg-red-500/20 rounded-lg border border-red-500/30 animate-fade-in">
            <div className="text-red-300 text-sm font-medium mb-1">Update Failed</div>
            <div className="text-red-400/80 text-xs">{error}</div>
            {error.includes("manually") && (
              <a
                href="https://github.com/TheerasakPing/video-downloader-tauri/releases/latest"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 mt-2 text-xs text-blue-400 hover:text-blue-300"
              >
                <ExternalLink size={12} />
                Open GitHub Releases
              </a>
            )}
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-3">
          <Button
            variant="secondary"
            onClick={onDismiss}
            disabled={downloading}
            className="flex-1"
          >
            Later
          </Button>
          <Button
            onClick={onDownload}
            disabled={downloading}
            isLoading={downloading}
            leftIcon={downloading ? <RefreshCw size={16} className="animate-spin" /> : <Download size={16} />}
            className="flex-1 bg-gradient-to-r from-emerald-500 to-teal-500 hover:from-emerald-600 hover:to-teal-600"
          >
            {downloading ? "Updating..." : error ? "Retry Update" : "Update Now"}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default UpdateDialog;
