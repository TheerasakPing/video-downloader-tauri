import {
  ListOrdered,
  ArrowUp,
  ArrowDown,
  Trash2,
  Pause,
  Clock,
  CheckCircle,
  XCircle,
  Loader2,
} from "lucide-react";

export interface QueueItem {
  id: string;
  seriesId: number;
  seriesTitle: string;
  episode: number;
  status: "pending" | "downloading" | "completed" | "failed";
  progress: number;
  priority: number;
}

interface DownloadQueueProps {
  queue: QueueItem[];
  onMoveUp: (id: string) => void;
  onMoveDown: (id: string) => void;
  onRemove: (id: string) => void;
  onPause: (id: string) => void;
  onResume?: (id: string) => void;
}

export function DownloadQueue({
  queue,
  onMoveUp,
  onMoveDown,
  onRemove,
  onPause,
}: DownloadQueueProps) {
  const pendingCount = queue.filter((q) => q.status === "pending").length;
  const downloadingCount = queue.filter((q) => q.status === "downloading").length;
  const completedCount = queue.filter((q) => q.status === "completed").length;

  const getStatusIcon = (status: QueueItem["status"]) => {
    switch (status) {
      case "pending":
        return <Clock size={16} className="text-slate-400" />;
      case "downloading":
        return <Loader2 size={16} className="text-violet-400 animate-spin" />;
      case "completed":
        return <CheckCircle size={16} className="text-emerald-400" />;
      case "failed":
        return <XCircle size={16} className="text-red-400" />;
    }
  };

  return (
    <div className="bg-slate-800/50 rounded-xl border border-slate-700 overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 bg-slate-800/50 border-b border-slate-700">
        <h3 className="text-sm font-medium text-slate-300 flex items-center gap-2">
          <ListOrdered size={16} />
          Download Queue
        </h3>
        <div className="flex items-center gap-3 text-xs">
          <span className="text-slate-500">
            Pending: <span className="text-white">{pendingCount}</span>
          </span>
          <span className="text-slate-500">
            Active: <span className="text-violet-400">{downloadingCount}</span>
          </span>
          <span className="text-slate-500">
            Done: <span className="text-emerald-400">{completedCount}</span>
          </span>
        </div>
      </div>

      <div className="max-h-64 overflow-y-auto">
        {queue.length === 0 ? (
          <div className="p-8 text-center text-slate-500">
            Download queue is empty
          </div>
        ) : (
          <div className="divide-y divide-slate-700/50">
            {queue.map((item, index) => (
              <div
                key={item.id}
                className="flex items-center gap-3 px-4 py-2 hover:bg-slate-700/30 transition-colors"
              >
                {/* Priority/Status */}
                <div className="w-8 text-center">
                  {getStatusIcon(item.status)}
                </div>

                {/* Info */}
                <div className="flex-1 min-w-0">
                  <div className="text-sm text-white truncate">
                    {item.seriesTitle} - Episode {item.episode}
                  </div>
                  {item.status === "downloading" && (
                    <div className="mt-1">
                      <div className="h-1 bg-slate-700 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-violet-500 rounded-full transition-all"
                          style={{ width: `${item.progress}%` }}
                        />
                      </div>
                    </div>
                  )}
                </div>

                {/* Actions */}
                <div className="flex items-center gap-1">
                  {item.status === "pending" && (
                    <>
                      <button
                        onClick={() => onMoveUp(item.id)}
                        disabled={index === 0}
                        className="p-1 text-slate-500 hover:text-white hover:bg-slate-700 rounded disabled:opacity-30"
                      >
                        <ArrowUp size={14} />
                      </button>
                      <button
                        onClick={() => onMoveDown(item.id)}
                        disabled={index === queue.length - 1}
                        className="p-1 text-slate-500 hover:text-white hover:bg-slate-700 rounded disabled:opacity-30"
                      >
                        <ArrowDown size={14} />
                      </button>
                    </>
                  )}
                  {item.status === "downloading" && (
                    <button
                      onClick={() => onPause(item.id)}
                      className="p-1 text-slate-500 hover:text-amber-400 hover:bg-slate-700 rounded"
                    >
                      <Pause size={14} />
                    </button>
                  )}
                  {item.status === "pending" && (
                    <button
                      onClick={() => onRemove(item.id)}
                      className="p-1 text-slate-500 hover:text-red-400 hover:bg-slate-700 rounded"
                    >
                      <Trash2 size={14} />
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
