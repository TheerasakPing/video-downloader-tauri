import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface DownloadProgress {
  episode: number;
  downloaded: number;
  total: number;
  speed: number;
  percentage: number;
}

interface DownloadResult {
  episode: number;
  success: boolean;
  filePath?: string;
  error?: string;
}

interface UseEventListenersProps {
  onDownloadProgress: (progress: DownloadProgress) => void;
  onDownloadResult: (result: DownloadResult) => void;
  onMergeStarted: () => void;
  onMergeProgress: (progress: { percentage: number; currentTime: number; totalDuration: number }) => void;
  onMergeComplete: (file: string) => void;
  onMergeError: (error: string) => void;
  onLogInfo: (message: string) => void;
  onDetectionProgress: (progress: { message: string; progress: number }) => void;
}

export function useEventListeners(props: UseEventListenersProps) {
  useEffect(() => {
    const unsubscribers: (() => void)[] = [];

    const setupListeners = async () => {
      try {
        unsubscribers.push(
          await listen<DownloadProgress>('download-progress', (event) => {
            props.onDownloadProgress(event.payload);
          })
        );

        unsubscribers.push(
          await listen<DownloadResult>('download-result', (event) => {
            props.onDownloadResult(event.payload);
          })
        );

        unsubscribers.push(
          await listen('merge-started', () => {
            props.onMergeStarted();
          })
        );

        unsubscribers.push(
          await listen<{ percentage: number; currentTime: number; totalDuration: number }>(
            'merge-progress',
            (event) => {
              props.onMergeProgress(event.payload);
            }
          )
        );

        unsubscribers.push(
          await listen<string>('merge-complete', (event) => {
            props.onMergeComplete(event.payload);
          })
        );

        unsubscribers.push(
          await listen<string>('merge-error', (event) => {
            props.onMergeError(event.payload);
          })
        );

        unsubscribers.push(
          await listen<string>('log-info', (event) => {
            props.onLogInfo(event.payload);
          })
        );

        unsubscribers.push(
          await listen<{ message: string; progress: number }>(
            'detection-progress',
            (event) => {
              props.onDetectionProgress(event.payload);
            }
          )
        );
      } catch (error) {
        console.error('Failed to setup event listeners:', error);
      }
    };

    setupListeners();

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, [props]);
}
