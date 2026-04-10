export interface SeriesInfo {
  seriesId: number;
  title: string;
  totalEpisodes: number;
  url?: string;
  posterUrl?: string;
  episodeUrls: Record<number, string>;
  source: string;
  sourceUrl?: string;
  episodeKeys?: Record<number, HlsKeyInfo>;
  cookies?: [string, string][];
}

export interface HlsKeyInfo {
  key: string;
  iv: string;
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

export interface DomainSettings {
  titanDomain: string;
  baanjeenDomain: string;
  rongyokDomain: string;
  hsckDomain?: string;
  njavtvDomain?: string;
  njavDomain?: string;
}

// --- Library types ---

export interface LibraryEntry {
  id: number;
  parserSeriesId: number;
  title: string;
  source: string;
  sourceUrl?: string;
  posterPath?: string;
  totalEpisodes: number;
  dateAdded: string;
  lastDownloaded?: string;
  completedCount: number;
}

export interface LibraryEpisode {
  id: number;
  libraryId: number;
  episodeNumber: number;
  videoUrl?: string;
  filePath?: string;
  quality?: string;
  fileSize?: number;
  status: string;
}

export interface SeriesDetail {
  entry: LibraryEntry;
  episodes: LibraryEpisode[];
  canRefetch: boolean;
}

// --- Quality types ---

export interface QualityOption {
  resolution: string;
  bandwidth: number;
  label: string;
  streamUrl: string;
}

export interface QualityInfo {
  qualities: QualityOption[];
  defaultIndex: number;
}

// --- Scheduling types ---

export interface ScheduleConfig {
  enabled: boolean;
  activeStart?: string;
  activeEnd?: string;
  speedDuringActive: number;
  speedOutsideActive: number;
  autoPause: boolean;
  autoResume: boolean;
}

// --- Proxy types ---

export interface ProxyConfig {
  proxyType: 'Http' | 'Socks5' | 'Direct';
  host: string;
  port: number;
  username?: string;
  password?: string;
}

// --- Retry types ---

export interface RetryConfig {
  maxRetries: number;
  retryDelayMs: number;
  fallbackUrls: string[];
  autoRetry: boolean;
  skipFailedSegments: boolean;
}
