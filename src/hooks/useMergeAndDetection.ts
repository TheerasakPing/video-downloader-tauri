import { useState, useCallback } from "react";

interface MergeState {
  isMerging: boolean;
  mergedFile: string | null;
  mergeError: string | null;
  progress: number;
  currentTime: number;
  totalDuration: number;
}

interface DetectionState {
  isDetecting: boolean;
  message: string;
  progress: number;
}

interface UseMergeAndDetectionDeps {
  success: (msg: string) => void;
  error: (msg: string) => void;
  refreshFiles: () => void;
}

export function useMergeAndDetection(deps: UseMergeAndDetectionDeps) {
  const { success, error, refreshFiles } = deps;

  const [mergeState, setMergeState] = useState<MergeState>({
    isMerging: false, mergedFile: null, mergeError: null,
    progress: 0, currentTime: 0, totalDuration: 0,
  });

  const [detectionState, setDetectionState] = useState<DetectionState>({
    isDetecting: false, message: "", progress: 0,
  });

  // Merge event callbacks (to be passed to useEventListeners)
  const onMergeStarted = useCallback(() => {
    setMergeState((prev) => ({ ...prev, isMerging: true, progress: 0 }));
  }, []);

  const onMergeProgress = useCallback((p: { percentage: number; currentTime: number; totalDuration: number }) => {
    setMergeState((prev) => ({ ...prev, progress: p.percentage, currentTime: p.currentTime, totalDuration: p.totalDuration }));
  }, []);

  const onMergeComplete = useCallback((file: string) => {
    setMergeState({ isMerging: false, mergedFile: file, mergeError: null, progress: 100, currentTime: 0, totalDuration: 0 });
    success(`Videos merged: ${file.split("/").pop()}`);
    refreshFiles();
  }, [success, refreshFiles]);

  const onMergeError = useCallback((err: string) => {
    setMergeState((prev) => ({ ...prev, isMerging: false, mergeError: err }));
    error(`Merge failed: ${err}`);
  }, [error]);

  const onDetectionProgress = useCallback((p: { message: string; progress: number }) => {
    setDetectionState({ isDetecting: true, message: p.message, progress: p.progress });
    if (p.progress >= 100) {
      setTimeout(() => setDetectionState({ isDetecting: false, message: "", progress: 0 }), 500);
    }
  }, []);

  return {
    mergeState,
    detectionState,
    onMergeStarted,
    onMergeProgress,
    onMergeComplete,
    onMergeError,
    onDetectionProgress,
  };
}
