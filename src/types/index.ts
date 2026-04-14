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
  javwowDomain?: string;
  avkuyDomain?: string;
  jav18tvDomain?: string;
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
  favorite: boolean;
  tags: LibraryTag[];
  watchedCount?: number;
  description?: string;
  rating?: number;
  year?: number;
  genre?: string;
  duration?: string;
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
  watched: boolean;
  watchedAt?: string;
}

export interface SeriesDetail {
  entry: LibraryEntry;
  episodes: LibraryEpisode[];
  canRefetch: boolean;
}

export interface LibraryTag {
  id: number;
  name: string;
  usageCount: number;
}

export interface LibraryQuery {
  sort?: string;
  order?: string;
  source?: string;
  status?: string;
  tagId?: number;
  favoriteOnly?: boolean;
  search?: string;
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

export interface ScheduleEntry {
  id: number;
  name: string;
  url: string;
  outputDir: string;
  cronExpression: string;
  enabled: boolean;
  lastRun?: string;
  nextRun?: string;
  createdAt: string;
}

export interface CreateScheduleRequest {
  name: string;
  url: string;
  outputDir: string;
  cronExpression: string;
}

export interface UpdateScheduleRequest {
  id: number;
  name?: string;
  url?: string;
  outputDir?: string;
  cronExpression?: string;
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

export interface SearchResult {
  title: string;
  posterUrl?: string;
  url: string;
  source: string;
  totalEpisodes?: number;
  description?: string;
  rating?: number;
  year?: number;
  genre?: string;
  duration?: string;
}

export interface SiteCategory {
  id: string;
  label: string;
  source: string;
}

export interface SearchResponse {
  results: SearchResult[];
  source: string;
  page: number;
  hasMore: boolean;
}
