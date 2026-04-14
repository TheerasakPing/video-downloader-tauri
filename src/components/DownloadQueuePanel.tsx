import {
  ListOrdered,
  Loader2,
  CheckCircle,
  AlertTriangle,
  Clock,
  Download,
  Image as ImageIcon,
  Pause,
  Play,
  Clipboard,
  Trash2,
  X,
  Layers,
} from "lucide-react";
import { BatchItem } from "../hooks/useBatchProcessor";
import { DownloadProgress } from "../types";

interface DownloadQueuePanelProps {
  batchQueue: BatchItem[];
  setBatchQueue: React.Dispatch<React.SetStateAction<BatchItem[]>>;
  isBatchProcessing: boolean;
  toggleBatchProcessing: () => void;
  setIsBatchProcessing: (v: boolean) => void;
  isAutoCapture: boolean;
  setIsAutoCapture: React.Dispatch<React.SetStateAction<boolean>>;
  progress: DownloadProgress;
}

export function DownloadQueuePanel({
  batchQueue,
  setBatchQueue,
  isBatchProcessing,
  toggleBatchProcessing,
  setIsBatchProcessing,
  isAutoCapture,
  setIsAutoCapture,
  progress,
}: DownloadQueuePanelProps) {
  const pendingCount = batchQueue.filter(
    (i) => i.status === "pending" || i.status === "fetching" || i.status === "ready"
  ).length;
  const activeCount = batchQueue.filter((i) => i.status === "downloading").length;
  const completedCount = batchQueue.filter((i) => i.status === "completed").length;
  const failedCount = batchQueue.filter(
    (i) => i.status === "error" || i.status === "failed"
  ).length;

  const statusIcon = (status: BatchItem["status"]) => {
    switch (status) {
      case "pending":
        return <Clock size={12} className="text-slate-500" />;
      case "fetching":
        return <Loader2 size={12} className="text-amber-400 animate-spin" />;
      case "ready":
        return <Download size={12} className="text-cyan-400" />;
      case "downloading":
        return <Loader2 size={12} className="text-cyan-400 animate-spin" />;
      case "completed":
        return <CheckCircle size={12} className="text-emerald-400" />;
      case "error":
      case "failed":
        return <AlertTriangle size={12} className="text-red-400" />;
    }
  };

  const statusLabel = (status: BatchItem["status"]) => {
    switch (status) {
      case "pending":
        return "Waiting";
      case "fetching":
        return "Fetching info\u2026";
      case "ready":
        return "Ready";
      case "downloading":
        return `${Math.round(progress.percentage)}%`;
      case "completed":
        return "Done";
      case "error":
        return "Error";
      case "failed":
        return "Failed";
    }
  };

  const statusColor = (status: BatchItem["status"]) => {
    switch (status) {
      case "pending":
        return "text-slate-500";
      case "fetching":
        return "text-amber-400";
      case "ready":
        return "text-cyan-400";
      case "downloading":
        return "text-cyan-400";
      case "completed":
        return "text-emerald-400";
      case "error":
      case "failed":
        return "text-red-400";
    }
  };

  return (
    <div className="h-full flex flex-col bg-slate-900/60 border-l border-slate-700/40">
      {/* Header */}
      <div className="px-3 py-2.5 border-b border-slate-700/40 flex items-center justify-between bg-slate-800/40">
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-md bg-cyan-500/15 flex items-center justify-center border border-cyan-500/30">
            <ListOrdered size={13} className="text-cyan-400" />
          </div>
          <span className="text-xs font-bold text-slate-200 tracking-wide">Queue</span>
          {batchQueue.length > 0 && (
            <span className="bg-cyan-500/15 text-cyan-400 px-1.5 py-0.5 rounded-full text-[10px] font-bold border border-cyan-500/25">
              {batchQueue.length}
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setIsAutoCapture((p) => !p)}
            className={`h-6 px-1.5 rounded-md text-[10px] font-medium flex items-center gap-1 transition-all border ${
              isAutoCapture
                ? "bg-amber-500/15 text-amber-300 border-amber-500/40 shadow-[0_0_8px_rgba(245,158,11,0.2)]"
                : "bg-transparent text-slate-500 border-transparent hover:text-amber-400 hover:bg-amber-500/10"
            }`}
            title="Auto Capture Clipboard"
          >
            <Clipboard size={11} className={isAutoCapture ? "animate-pulse" : ""} />
            <span className="hidden xl:inline">Auto</span>
          </button>

          <button
            onClick={toggleBatchProcessing}
            className={`h-6 w-6 rounded-md flex items-center justify-center transition-all border ${
              isBatchProcessing
                ? "bg-amber-500/15 text-amber-300 border-amber-500/40 shadow-[0_0_8px_rgba(245,158,11,0.2)]"
                : "bg-emerald-500/15 text-emerald-300 border-emerald-500/40 shadow-[0_0_8px_rgba(16,185,129,0.2)]"
            }`}
            title={isBatchProcessing ? "Pause Queue" : "Start Queue"}
          >
            {isBatchProcessing ? <Pause size={11} /> : <Play size={11} />}
          </button>

          {batchQueue.length > 0 && (
            <button
              onClick={() => {
                if (confirm("Clear entire queue?")) {
                  setBatchQueue([]);
                  setIsBatchProcessing(false);
                }
              }}
              className="h-6 w-6 rounded-md flex items-center justify-center text-slate-500 hover:text-red-400 hover:bg-red-500/10 transition-all border border-transparent hover:border-red-500/30"
              title="Clear All"
            >
              <Trash2 size={11} />
            </button>
          )}
        </div>
      </div>

      {/* Stats Bar */}
      {batchQueue.length > 0 && (
        <div className="px-3 py-1.5 border-b border-slate-700/30 flex items-center gap-3 text-[10px] bg-slate-800/20">
          <span className="flex items-center gap-1">
            <Clock size={9} className="text-slate-500" />
            <span className="text-slate-400">{pendingCount}</span>
          </span>
          {activeCount > 0 && (
            <span className="flex items-center gap-1">
              <Loader2 size={9} className="text-cyan-400 animate-spin" />
              <span className="text-cyan-400">{activeCount}</span>
            </span>
          )}
          {completedCount > 0 && (
            <span className="flex items-center gap-1">
              <CheckCircle size={9} className="text-emerald-400" />
              <span className="text-emerald-400">{completedCount}</span>
            </span>
          )}
          {failedCount > 0 && (
            <span className="flex items-center gap-1">
              <AlertTriangle size={9} className="text-red-400" />
              <span className="text-red-400">{failedCount}</span>
            </span>
          )}
          <span className="ml-auto text-slate-600 font-mono">
            {completedCount}/{batchQueue.length}
          </span>
        </div>
      )}

      {/* Queue Items */}
      <div className="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-1">
        {batchQueue.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-slate-600">
            <div className="w-10 h-10 rounded-xl bg-slate-800/60 flex items-center justify-center mb-2 border border-slate-700/30">
              <ListOrdered size={18} className="text-slate-600" />
            </div>
            <p className="text-[11px] text-slate-500 mb-0.5">No items in queue</p>
            <p className="text-[9px] text-slate-600 text-center px-4">
              Paste URLs or enable Auto Capture to add items
            </p>
          </div>
        ) : (
          batchQueue.map((item, idx) => (
            <div
              key={item.url}
              className={`p-2 rounded-lg border flex gap-2 text-xs transition-all group relative ${
                item.status === "downloading"
                  ? "bg-cyan-500/5 border-cyan-500/30 shadow-[0_0_12px_rgba(34,211,238,0.06)]"
                  : item.status === "completed"
                  ? "bg-emerald-500/5 border-emerald-500/20"
                  : item.status === "error" || item.status === "failed"
                  ? "bg-red-500/5 border-red-500/20"
                  : item.status === "fetching"
                  ? "bg-amber-500/5 border-amber-500/20"
                  : "bg-slate-800/30 border-slate-700/30 hover:border-slate-600/40"
              }`}
            >
              <div className="w-10 h-13 bg-slate-900/80 rounded overflow-hidden flex-shrink-0 relative">
                {item.info?.posterUrl ? (
                  <img src={item.info.posterUrl} className="w-full h-full object-cover" alt="" />
                ) : (
                  <div className="w-full h-full flex items-center justify-center">
                    <ImageIcon size={12} className="text-slate-600" />
                  </div>
                )}
                {item.status === "downloading" && (
                  <div className="absolute inset-0 bg-cyan-500/20 flex items-center justify-center">
                    <Loader2 size={11} className="animate-spin text-cyan-400" />
                  </div>
                )}
                {item.status === "completed" && (
                  <div className="absolute inset-0 bg-emerald-500/20 flex items-center justify-center">
                    <CheckCircle size={11} className="text-emerald-400" />
                  </div>
                )}
              </div>

              <div className="flex-1 min-w-0 flex flex-col gap-0.5">
                <div className="font-medium truncate text-slate-300 text-[11px] leading-tight">
                  {item.info?.title || item.url}
                </div>
                <div className={`text-[10px] flex items-center gap-1 ${statusColor(item.status)}`}>
                  {statusIcon(item.status)}
                  {statusLabel(item.status)}
                </div>
                {item.status === "downloading" && (
                  <div className="h-1 bg-slate-700/50 rounded-full overflow-hidden mt-0.5">
                    <div
                      className="h-full bg-cyan-400/80 transition-all duration-300 shadow-[0_0_4px_currentColor] rounded-full"
                      style={{ width: `${progress.percentage}%` }}
                    />
                  </div>
                )}
                {item.info && (
                  <div className="text-[9px] text-slate-600">
                    <span className="flex items-center gap-0.5"><Layers size={8} className="text-slate-500" /> {item.info!.totalEpisodes} eps</span>
                  </div>
                )}
              </div>

              {(item.status === "pending" || item.status === "ready" || item.status === "failed" || item.status === "error") && (
                <button
                  onClick={() => setBatchQueue((prev) => prev.filter((_, i) => i !== idx))}
                  className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 text-slate-600 hover:text-red-400 transition-all p-0.5 rounded hover:bg-red-500/10"
                >
                  <X size={10} />
                </button>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
