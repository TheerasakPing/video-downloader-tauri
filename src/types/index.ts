export interface SeriesInfo {
  seriesId: number;
  title: string;
  totalEpisodes: number;
  posterUrl?: string;
  episodeUrls: Record<number, string>;
}

export interface EpisodeInfo {
  episodeNumber: number;
  title: string;
  videoUrl: string;
}

export interface DownloadProgress {
  episode: number;
  downloaded: number;
  total: number;
  speed: number;
  percentage: number;
}

export interface DownloadState {
  isDownloading: boolean;
  isPaused: boolean;
  currentEpisode: number;
  completedEpisodes: number[];
  failedEpisodes: number[];
  totalSelected: number;
}

export type LogLevel = "info" | "success" | "warning" | "error";

export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  message: string;
}
