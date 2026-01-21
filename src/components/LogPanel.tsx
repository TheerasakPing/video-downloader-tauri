import { LogEntry } from "../types";
import { Copy, Trash2 } from "lucide-react";

interface LogPanelProps {
  logs: LogEntry[];
  onClear: () => void;
}

export function LogPanel({ logs, onClear }: LogPanelProps) {
  const copyLogs = () => {
    const text = logs
      .map(
        (log) =>
          `[${log.timestamp.toLocaleTimeString()}] [${log.level.toUpperCase()}] ${log.message}`
      )
      .join("\n");
    navigator.clipboard.writeText(text);
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case "success":
        return "text-emerald-400";
      case "warning":
        return "text-amber-400";
      case "error":
        return "text-red-400";
      default:
        return "text-slate-300";
    }
  };

  return (
    <div className="flex flex-col h-full bg-slate-900 rounded-xl overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 bg-slate-800/50 border-b border-slate-700">
        <h3 className="text-sm font-medium text-slate-300">Debug Log</h3>
        <div className="flex gap-2">
          <button
            onClick={copyLogs}
            className="p-1.5 text-blue-400 hover:text-white hover:bg-slate-700 rounded-lg transition-colors"
            title="Copy logs"
          >
            <Copy size={16} className="drop-shadow-[0_0_4px_currentColor]" />
          </button>
          <button
            onClick={onClear}
            className="p-1.5 text-red-400 hover:text-red-300 hover:bg-slate-700 rounded-lg transition-colors"
            title="Clear logs"
          >
            <Trash2 size={16} className="drop-shadow-[0_0_4px_currentColor]" />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-3 font-mono text-xs space-y-1">
        {logs.length === 0 ? (
          <div className="text-slate-500 text-center py-8">No logs yet</div>
        ) : (
          logs.map((log) => (
            <div key={log.id} className="flex gap-2">
              <span className="text-slate-500 shrink-0">
                [{log.timestamp.toLocaleTimeString()}]
              </span>
              <span className={getLevelColor(log.level)}>{log.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
