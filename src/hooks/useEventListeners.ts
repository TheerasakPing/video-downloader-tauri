import { useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { DownloadProgress } from '../types';

export interface DownloadResult {
  episode: number;
  success: boolean;
  filePath?: string;
  error?: string;
}

export interface MergeProgress {
  percentage: number;
  currentTime: number;
  totalDuration: number;
}

export interface DetectionProgress {
  message: string;
  progress: number;
}

interface UseEventListenersProps {
  onDownloadProgress: (progress: DownloadProgress) => void;
  onDownloadResult: (result: DownloadResult) => void;
  onMergeStarted: () => void;
  onMergeProgress: (progress: MergeProgress) => void;
  onMergeComplete: (file: string) => void;
  onMergeError: (error: string) => void;
  onLogInfo: (message: string) => void;
  onDetectionProgress: (progress: DetectionProgress) => void;
}

/**
 * Stable event listener hook. Uses refs internally so the Tauri listen/unlisten
 * cycle runs exactly once, regardless of how often parent callbacks change.
 */
export function useEventListeners(props: UseEventListenersProps) {
  const propsRef = useRef(props);
  propsRef.current = props;

  useEffect(() => {
    const unsubscribers: (() => void)[] = [];

    const setupListeners = async () => {
      try {
        unsubscribers.push(
          await listen<DownloadProgress>('download-progress', (event) => {
            propsRef.current.onDownloadProgress(event.payload);
          }),
        );

        unsubscribers.push(
          await listen<DownloadResult>('download-result', (event) => {
            propsRef.current.onDownloadResult(event.payload);
          }),
        );

        unsubscribers.push(
          await listen('merge-started', () => {
            propsRef.current.onMergeStarted();
          }),
        );

        unsubscribers.push(
          await listen<MergeProgress>('merge-progress', (event) => {
            propsRef.current.onMergeProgress(event.payload);
          }),
        );

        unsubscribers.push(
          await listen<string>('merge-complete', (event) => {
            propsRef.current.onMergeComplete(event.payload);
          }),
        );

        unsubscribers.push(
          await listen<string>('merge-error', (event) => {
            propsRef.current.onMergeError(event.payload);
          }),
        );

        unsubscribers.push(
          await listen<string>('log-info', (event) => {
            propsRef.current.onLogInfo(event.payload);
          }),
        );

        unsubscribers.push(
          await listen<DetectionProgress>('detection-progress', (event) => {
            propsRef.current.onDetectionProgress(event.payload);
          }),
        );
      } catch (err) {
        console.error('Failed to setup event listeners:', err);
      }
    };

    setupListeners();

    return () => {
      unsubscribers.forEach((unsub) => unsub());
    };
  }, []);
}
