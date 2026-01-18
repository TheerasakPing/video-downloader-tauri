import { useState, useCallback } from "react";
import { LogEntry, LogLevel } from "../types";

export function useLogger() {
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const addLog = useCallback((level: LogLevel, message: string) => {
    const entry: LogEntry = {
      id: crypto.randomUUID(),
      timestamp: new Date(),
      level,
      message,
    };
    setLogs((prev) => [...prev, entry]);
  }, []);

  const log = useCallback((message: string) => addLog("info", message), [addLog]);
  const success = useCallback((message: string) => addLog("success", message), [addLog]);
  const warning = useCallback((message: string) => addLog("warning", message), [addLog]);
  const error = useCallback((message: string) => addLog("error", message), [addLog]);

  const clearLogs = useCallback(() => setLogs([]), []);

  return { logs, log, success, warning, error, clearLogs };
}
