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
        <div className="glass rounded-xl p-4 border border-violet-500/30 bg-violet-500/10">
          <div className="flex items-center gap-2 text-violet-400 mb-2">
            <Film size={16} className="drop-shadow-[0_0_4px_currentColor]" />
            <span className="text-xs font-medium">Total Downloads</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {stats.totalDownloads}
          </div>
        </div>

        <div className="glass rounded-xl p-4 border border-emerald-500/30 bg-emerald-500/10">
          <div className="flex items-center gap-2 text-emerald-400 mb-2">
            <CheckCircle size={16} className="drop-shadow-[0_0_4px_currentColor]" />
            <span className="text-xs font-medium">Episodes</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {stats.totalEpisodes}
          </div>
        </div>

        <div className="glass rounded-xl p-4 border border-blue-500/30 bg-blue-500/10">
          <div className="flex items-center gap-2 text-blue-400 mb-2">
            <HardDrive size={16} className="drop-shadow-[0_0_4px_currentColor]" />
            <span className="text-xs font-medium">Total Size</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {formatBytes(stats.totalSize)}
          </div>
        </div>

        <div className="glass rounded-xl p-4 border border-amber-500/30 bg-amber-500/10">
          <div className="flex items-center gap-2 text-amber-400 mb-2">
            <TrendingUp size={16} className="drop-shadow-[0_0_4px_currentColor]" />
            <span className="text-xs font-medium">Success Rate</span>
          </div>
          <div className="text-2xl font-bold text-white">
            {stats.successRate.toFixed(0)}%
          </div>
        </div>
      </div>

      {/* History List */}
      <div className="glass rounded-lg border border-slate-700/50 overflow-hidden">
        <div className="flex items-center justify-between px-4 py-3 bg-slate-800/30 border-b border-slate-700/50">
          <h3 className="text-sm font-medium text-slate-300 flex items-center gap-2">
            <Clock size={16} className="text-cyan-400 drop-shadow-[0_0_4px_currentColor]" />
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

        <div className="max-h-[60vh] overflow-y-auto">
          {history.length === 0 ? (
            <div className="p-8 text-center text-slate-500">
              <Clock size={24} className="mx-auto mb-3 drop-shadow-[0_0_4px_currentColor]" />
              No download history yet
            </div>
          ) : (
            <div className="divide-y divide-slate-700/30">
              {history.map((record, index) => (
                <div
                  key={record.id}
                  className="px-4 py-3 hover:bg-slate-700/20 transition-all group"
                  style={{ animationDelay: `${index * 50}ms` }}
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        {record.status === "completed" && (
                          <CheckCircle size={14} className="text-emerald-400 drop-shadow-[0_0_4px_currentColor]" />
                        )}
                        {record.status === "failed" && (
                          <XCircle size={14} className="text-red-400 drop-shadow-[0_0_4px_currentColor]" />
                        )}
                        {record.status === "partial" && (
                          <AlertCircle size={14} className="text-amber-400 drop-shadow-[0_0_4px_currentColor]" />
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
                      className="p-1.5 text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                    >
                      <Trash2 size={14} className="drop-shadow-[0_0_4px_currentColor]" />
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
