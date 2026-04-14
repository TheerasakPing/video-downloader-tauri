import { ListOrdered, Loader2, CheckCircle, X, Image as ImageIcon, Pause, Play } from "lucide-react";
import { Button } from "./Button";
import { useI18n } from "../hooks/useI18n";
import { BatchItem } from "../hooks/useBatchProcessor";
import { DownloadProgress } from "../types";

interface BatchQueueDrawerProps {
  isOpen: boolean;
  batchQueue: BatchItem[];
  setBatchQueue: React.Dispatch<React.SetStateAction<BatchItem[]>>;
  isBatchProcessing: boolean;
  toggleBatchProcessing: () => void;
  setIsBatchProcessing: (v: boolean) => void;
  setIsBatchMode: (v: boolean) => void;
  progress: DownloadProgress;
}

export function BatchQueueDrawer({
  isOpen,
  batchQueue,
  setBatchQueue,
  isBatchProcessing,
  toggleBatchProcessing,
  setIsBatchProcessing,
  setIsBatchMode,
  progress,
}: BatchQueueDrawerProps) {
  const { t } = useI18n();

  if (!isOpen) return null;

  return (
    <>
      <div className="fixed inset-y-0 right-0 w-80 bg-slate-900/98 backdrop-blur-xl border-l border-slate-700/50 shadow-2xl shadow-black/50 z-50 flex flex-col animate-slide-in-right">
        <div className="px-3 py-2.5 border-b border-slate-700/50 flex items-center justify-between bg-slate-800/60">
          <div className="flex items-center gap-2">
            <ListOrdered size={14} className="text-emerald-400" />
            <span className="text-xs font-bold text-slate-200 tracking-wide">Queue</span>
            <span className="bg-slate-700/80 text-slate-300 px-1.5 py-0.5 rounded-full text-[10px]">{batchQueue.length}</span>
          </div>
          <div className="flex items-center gap-1">
            <Button size="sm" variant={isBatchProcessing ? "amber" : "success"} className="h-6 w-6 p-0"
              onClick={toggleBatchProcessing} title={isBatchProcessing ? t("queue.pauseDownload") : t("app.startQueue")}>
              {isBatchProcessing ? <Pause size={11} /> : <Play size={11} />}
            </Button>
            <Button size="sm" variant="ghost" className="h-6 w-6 p-0 text-slate-500 hover:text-red-400"
              onClick={() => { setBatchQueue([]); setIsBatchProcessing(false); }}>
              <X size={12} />
            </Button>
            <Button size="sm" variant="ghost" className="h-6 w-6 p-0 text-slate-400 hover:text-white"
              onClick={() => setIsBatchMode(false)}>
              <X size={14} />
            </Button>
          </div>
        </div>
        <div className="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-1.5">
          {batchQueue.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-slate-500 opacity-40">
              <ListOrdered size={28} className="mb-2" />
              <p className="text-[11px]">{t("queue.empty")}</p>
            </div>
          ) : (
            batchQueue.map((item, idx) => (
              <div key={item.url} className={`p-2 rounded-lg border flex gap-2.5 text-xs relative group transition-all ${
                item.status === "downloading" ? "bg-cyan-500/5 border-cyan-500/30 shadow-[0_0_12px_rgba(34,211,238,0.08)]"
                : item.status === "completed" ? "bg-emerald-500/5 border-emerald-500/20"
                : item.status === "error" || item.status === "failed" ? "bg-red-500/5 border-red-500/20"
                : "bg-slate-800/30 border-slate-700/40"
              }`}>
                <div className="w-11 h-14 bg-slate-900/80 rounded overflow-hidden flex-shrink-0 relative">
                  {item.info?.posterUrl ? (
                    <img src={item.info.posterUrl} className="w-full h-full object-cover" alt="" />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center"><ImageIcon size={14} className="text-slate-600" /></div>
                  )}
                  {item.status === "downloading" && (
                    <div className="absolute inset-0 bg-black/40 flex items-center justify-center"><Loader2 size={12} className="animate-spin text-cyan-400" /></div>
                  )}
                  {item.status === "completed" && (
                    <div className="absolute inset-0 bg-black/30 flex items-center justify-center"><CheckCircle size={12} className="text-emerald-400" /></div>
                  )}
                </div>
                <div className="flex-1 min-w-0 flex flex-col gap-0.5">
                  <div className="font-medium truncate text-slate-200 text-[11px]">{item.info?.title || item.url}</div>
                  <div className="text-[10px] text-slate-400">
                    {item.status === "pending" && t("queue.pending")}
                    {item.status === "fetching" && t("app.fetchingInfo")}
                    {item.status === "ready" && `${t("app.ready")} (${item.info?.totalEpisodes})`}
                    {item.status === "downloading" && <span className="text-cyan-400">{t("app.downloadingStatus")} {progress.percentage.toFixed(0)}%</span>}
                    {item.status === "completed" && <span className="text-emerald-400">{t("app.completed")}</span>}
                    {item.status === "error" && <span className="text-red-400">{t("app.error")}</span>}
                  </div>
                  {item.status === "downloading" && (
                    <div className="h-1 bg-slate-700/50 rounded-full overflow-hidden mt-0.5">
                      <div className="h-full bg-cyan-400/80 transition-all duration-300 shadow-[0_0_4px_currentColor] rounded-full"
                        style={{ width: `${progress.percentage}%` }} />
                    </div>
                  )}
                </div>
                <button onClick={(e) => { e.stopPropagation(); setBatchQueue((prev) => prev.filter((_, i) => i !== idx)); }}
                  className="absolute top-1 right-1 opacity-0 group-hover:opacity-100 hover:text-red-400 transition-opacity text-slate-600"
                  aria-label={t("app.removeFromBatch")}>
                  <X size={11} />
                </button>
              </div>
            ))
          )}
        </div>
      </div>
      <div className="fixed inset-0 bg-black/30 z-40" onClick={() => setIsBatchMode(false)} />
    </>
  );
}
