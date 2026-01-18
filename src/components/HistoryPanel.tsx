import { DownloadRecord } from "../hooks/useHistory";
import {
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  Trash2,
  HardDrive,
  Film,
  TrendingUp,
} from "lucide-react";
import { Button } from "./Button";

interface HistoryPanelProps {
  history: DownloadRecord[];
  stats: {
    totalDownloads: number;
    totalEpisodes: number;
    totalSize: number;
    successRate: number;
  };
  onDelete: (id: string) => void;
  onClear: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString("th-TH", {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function HistoryPanel({
  history,
  stats,
  onDelete,
  onClear,
}: HistoryPanelProps) {
  return (
    <div className="space-y-6">
      {/* Stats Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-gradient-to-br from-violet-600/20 to-violet-900/20 rounded-xl p-4 border border-violet-500/30">
          <div className="flex items-center gap-2 text-violet-400 mb-2">
            <Film size={18} />
            <span className="text-xs font-medium">Total Downloads</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {stats.totalDownloads}
          </div>
        </div>

        <div className="bg-gradient-to-br from-emerald-600/20 to-emerald-900/20 rounded-xl p-4 border border-emerald-500/30">
          <div className="flex items-center gap-2 text-emerald-400 mb-2">
            <CheckCircle size={18} />
            <span className="text-xs font-medium">Episodes</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {stats.totalEpisodes}
          </div>
        </div>

        <div className="bg-gradient-to-br from-blue-600/20 to-blue-900/20 rounded-xl p-4 border border-blue-500/30">
          <div className="flex items-center gap-2 text-blue-400 mb-2">
            <HardDrive size={18} />
            <span className="text-xs font-medium">Total Size</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {formatBytes(stats.totalSize)}
          </div>
        </div>

        <div className="bg-gradient-to-br from-amber-600/20 to-amber-900/20 rounded-xl p-4 border border-amber-500/30">
          <div className="flex items-center gap-2 text-amber-400 mb-2">
            <TrendingUp size={18} />
            <span className="text-xs font-medium">Success Rate</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {stats.successRate.toFixed(0)}%
          </div>
        </div>
      </div>

      {/* History List */}
      <div className="bg-slate-800/50 rounded-xl border border-slate-700 overflow-hidden">
        <div className="flex items-center justify-between px-4 py-3 bg-slate-800/50 border-b border-slate-700">
          <h3 className="text-sm font-medium text-slate-300 flex items-center gap-2">
            <Clock size={16} />
            Download History
          </h3>
          {history.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              leftIcon={<Trash2 size={14} />}
              onClick={onClear}
            >
              Clear All
            </Button>
          )}
        </div>

        <div className="max-h-96 overflow-y-auto">
          {history.length === 0 ? (
            <div className="p-8 text-center text-slate-500">
              No download history yet
            </div>
          ) : (
            <div className="divide-y divide-slate-700/50">
              {history.map((record) => (
                <div
                  key={record.id}
                  className="px-4 py-3 hover:bg-slate-700/30 transition-colors"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        {record.status === "completed" && (
                          <CheckCircle size={16} className="text-emerald-400 shrink-0" />
                        )}
                        {record.status === "failed" && (
                          <XCircle size={16} className="text-red-400 shrink-0" />
                        )}
                        {record.status === "partial" && (
                          <AlertCircle size={16} className="text-amber-400 shrink-0" />
                        )}
                        <span className="text-sm font-medium text-white truncate">
                          {record.seriesTitle}
                        </span>
                      </div>
                      <div className="flex items-center gap-3 mt-1 text-xs text-slate-500">
                        <span>ID: {record.seriesId}</span>
                        <span>•</span>
                        <span>
                          {record.completedEpisodes.length}/{record.episodes.length} episodes
                        </span>
                        <span>•</span>
                        <span>{formatBytes(record.totalSize)}</span>
                      </div>
                      <div className="text-xs text-slate-600 mt-1">
                        {formatDate(record.startTime)}
                      </div>
                    </div>
                    <button
                      onClick={() => onDelete(record.id)}
                      className="p-1.5 text-slate-500 hover:text-red-400 hover:bg-slate-700 rounded-lg transition-colors"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
