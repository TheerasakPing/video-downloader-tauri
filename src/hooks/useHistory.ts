import { useState, useEffect } from "react";

export interface DownloadRecord {
  id: string;
  seriesId: number;
  seriesTitle: string;
  episodes: number[];
  completedEpisodes: number[];
  failedEpisodes: number[];
  startTime: string;
  endTime?: string;
  totalSize: number;
  status: "completed" | "failed" | "partial";
}

const STORAGE_KEY = "rongyok-history";
const MAX_RECORDS = 50;

export function useHistory() {
  const [history, setHistory] = useState<DownloadRecord[]>([]);

  // Load history on mount
  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        setHistory(JSON.parse(stored));
      } catch (e) {
        console.error("Failed to parse history:", e);
      }
    }
  }, []);

  // Save history when changed
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
  }, [history]);

  const addRecord = (record: Omit<DownloadRecord, "id">) => {
    const newRecord: DownloadRecord = {
      ...record,
      id: crypto.randomUUID(),
    };
    setHistory((prev) => [newRecord, ...prev].slice(0, MAX_RECORDS));
    return newRecord.id;
  };

  const updateRecord = (id: string, updates: Partial<DownloadRecord>) => {
    setHistory((prev) =>
      prev.map((r) => (r.id === id ? { ...r, ...updates } : r))
    );
  };

  const deleteRecord = (id: string) => {
    setHistory((prev) => prev.filter((r) => r.id !== id));
  };

  const clearHistory = () => {
    setHistory([]);
  };

  const getStats = () => {
    const totalDownloads = history.length;
    const totalEpisodes = history.reduce(
      (sum, r) => sum + r.completedEpisodes.length,
      0
    );
    const totalSize = history.reduce((sum, r) => sum + r.totalSize, 0);
    const successRate =
      totalDownloads > 0
        ? (history.filter((r) => r.status === "completed").length /
            totalDownloads) *
          100
        : 0;

    return { totalDownloads, totalEpisodes, totalSize, successRate };
  };

  return { history, addRecord, updateRecord, deleteRecord, clearHistory, getStats };
}
