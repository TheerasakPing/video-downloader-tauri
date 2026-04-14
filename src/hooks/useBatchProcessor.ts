import { useState, useEffect, useRef, useCallback } from "react";
import { SeriesInfo } from "../types";

export interface BatchItem {
  url: string;
  status: "pending" | "fetching" | "ready" | "error" | "downloading" | "completed" | "failed";
  info?: SeriesInfo;
  error?: string;
}

interface UseBatchProcessorDeps {
  isFetching: boolean;
  setIsFetching: (v: boolean) => void;
  isDownloading: boolean;
  runDownload: (series: SeriesInfo, episodes: Set<number>) => Promise<boolean>;
  log: (msg: string) => void;
  error: (msg: string) => void;
}

export function useBatchProcessor(deps: UseBatchProcessorDeps) {
  const { isFetching, setIsFetching, isDownloading, runDownload, log, error } = deps;

  const [isBatchMode, setIsBatchMode] = useState(false);
  const [batchQueue, setBatchQueue] = useState<BatchItem[]>([]);
  const [isBatchProcessing, setIsBatchProcessing] = useState(false);
  const [isAutoCapture, setIsAutoCapture] = useState(false);
  const [isBatchItemRunning, setIsBatchItemRunning] = useState(false);
  const [currentBatchItem, setCurrentBatchItem] = useState<BatchItem | null>(null);
  const isBatchItemRunningRef = useRef(false);

  const toggleBatchProcessing = useCallback(() => {
    setIsBatchProcessing((p) => !p);
  }, []);

  // Batch fetcher: pending -> ready
  useEffect(() => {
    const pendingIdx = batchQueue.findIndex((i) => i.status === "pending");
    if (pendingIdx === -1 || isFetching) return;

    const item = batchQueue[pendingIdx];

    const fetchItem = async () => {
      setBatchQueue((prev) =>
        prev.map((it, idx) => (idx === pendingIdx ? { ...it, status: "fetching" } : it)),
      );
      setIsFetching(true);

      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke<SeriesInfo>("fetch_series", { url: item.url });
        setBatchQueue((prev) =>
          prev.map((it, idx) => (idx === pendingIdx ? { ...it, status: "ready", info: result } : it)),
        );
        log(`Queue: Ready - ${result.title}`);
      } catch (e) {
        error(`Queue Fetch failed for ${item.url}: ${e}`);
        setBatchQueue((prev) =>
          prev.map((it, idx) => (idx === pendingIdx ? { ...it, status: "failed", error: String(e) } : it)),
        );
      } finally {
        setIsFetching(false);
      }
    };

    fetchItem();
  }, [batchQueue, isFetching, log, error, setIsFetching]);

  // Continuous batch processing
  useEffect(() => {
    if (!isBatchProcessing || isDownloading) return;
    if (isBatchItemRunning || isBatchItemRunningRef.current) return;

    const nextIdx = batchQueue.findIndex((i) => i.status === "ready" && i.info);
    if (nextIdx === -1) return;

    const item = batchQueue[nextIdx];

    const processItem = async () => {
      isBatchItemRunningRef.current = true;
      setCurrentBatchItem(item);
      setIsBatchItemRunning(true);

      setBatchQueue((prev) =>
        prev.map((it, idx) => (idx === nextIdx ? { ...it, status: "downloading" } : it)),
      );

      try {
        if (!item.info) throw new Error("Series info missing for batch item");

        const allEpisodes = new Set(
          Array.from({ length: item.info.totalEpisodes }, (_, i) => i + 1),
        );

        const dlSuccess = await runDownload(item.info, allEpisodes);

        setBatchQueue((prev) =>
          prev.map((it, idx) =>
            (idx === nextIdx ? { ...it, status: dlSuccess ? "completed" : "failed" } : it)),
          );
      } catch (e) {
        setBatchQueue((prev) =>
          prev.map((it, idx) =>
            (idx === nextIdx ? { ...it, status: "failed", error: String(e) } : it)),
          );
      } finally {
        isBatchItemRunningRef.current = false;
        setIsBatchItemRunning(false);
        setCurrentBatchItem(null);
      }
    };

    processItem();
  }, [batchQueue, isBatchProcessing, isDownloading, isBatchItemRunning, runDownload]);

  const resetBatchFlags = useCallback(() => {
    isBatchItemRunningRef.current = false;
    setIsBatchItemRunning(false);
    setCurrentBatchItem(null);
    setIsBatchProcessing(false);
    setIsBatchMode(false);
  }, []);

  return {
    isBatchMode,
    setIsBatchMode,
    batchQueue,
    setBatchQueue,
    isBatchProcessing,
    setIsBatchProcessing,
    isAutoCapture,
    setIsAutoCapture,
    isBatchItemRunning,
    currentBatchItem,
    toggleBatchProcessing,
    resetBatchFlags,
  };
}
