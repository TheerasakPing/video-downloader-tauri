import { useReducer, Reducer } from 'react';

// State types
export interface DownloadState {
  isDownloading: boolean;
  isPaused: boolean;
  currentEpisode: number;
  completedEpisodes: number[];
  failedEpisodes: number[];
  totalSelected: number;
}

export interface BatchState {
  isBatchMode: boolean;
  isBatchProcessing: boolean;
  isAutoCapture: boolean;
  isBatchItemRunning: boolean;
}

export interface UIState {
  activeTab: 'download' | 'library' | 'browse' | 'files' | 'history' | 'settings' | 'logs';
  showMiniMode: boolean;
  showShortcuts: boolean;
  isDragging: boolean;
}

export interface AppState {
  download: DownloadState;
  batch: BatchState;
  ui: UIState;
}

// Action types
export type AppAction =
  | { type: 'START_DOWNLOAD'; payload: { totalSelected: number } }
  | { type: 'STOP_DOWNLOAD' }
  | { type: 'PAUSE_DOWNLOAD' }
  | { type: 'RESUME_DOWNLOAD' }
  | { type: 'UPDATE_CURRENT_EPISODE'; payload: number }
  | { type: 'ADD_COMPLETED_EPISODE'; payload: number }
  | { type: 'ADD_FAILED_EPISODE'; payload: number }
  | { type: 'ENABLE_BATCH_MODE' }
  | { type: 'DISABLE_BATCH_MODE' }
  | { type: 'START_BATCH_PROCESSING' }
  | { type: 'STOP_BATCH_PROCESSING' }
  | { type: 'TOGGLE_AUTO_CAPTURE' }
  | { type: 'SET_BATCH_ITEM_RUNNING'; payload: boolean }
  | { type: 'SET_ACTIVE_TAB'; payload: UIState['activeTab'] }
  | { type: 'TOGGLE_MINI_MODE' }
  | { type: 'TOGGLE_SHORTCUTS' }
  | { type: 'SET_DRAGGING'; payload: boolean };

// Initial state
const initialState: AppState = {
  download: {
    isDownloading: false,
    isPaused: false,
    currentEpisode: 0,
    completedEpisodes: [],
    failedEpisodes: [],
    totalSelected: 0,
  },
  batch: {
    isBatchMode: false,
    isBatchProcessing: false,
    isAutoCapture: false,
    isBatchItemRunning: false,
  },
  ui: {
    activeTab: 'download',
    showMiniMode: false,
    showShortcuts: false,
    isDragging: false,
  },
};

// Reducer
const appReducer: Reducer<AppState, AppAction> = (state, action) => {
  switch (action.type) {
    case 'START_DOWNLOAD':
      return {
        ...state,
        download: {
          ...state.download,
          isDownloading: true,
          isPaused: false,
          currentEpisode: 0,
          completedEpisodes: [],
          failedEpisodes: [],
          totalSelected: action.payload.totalSelected,
        },
      };

    case 'STOP_DOWNLOAD':
      return {
        ...state,
        download: {
          ...state.download,
          isDownloading: false,
          isPaused: false,
          currentEpisode: 0,
        },
      };

    case 'PAUSE_DOWNLOAD':
      return {
        ...state,
        download: { ...state.download, isPaused: true },
      };

    case 'RESUME_DOWNLOAD':
      return {
        ...state,
        download: { ...state.download, isPaused: false },
      };

    case 'UPDATE_CURRENT_EPISODE':
      return {
        ...state,
        download: { ...state.download, currentEpisode: action.payload },
      };

    case 'ADD_COMPLETED_EPISODE':
      return {
        ...state,
        download: {
          ...state.download,
          completedEpisodes: [...state.download.completedEpisodes, action.payload],
        },
      };

    case 'ADD_FAILED_EPISODE':
      return {
        ...state,
        download: {
          ...state.download,
          failedEpisodes: [...state.download.failedEpisodes, action.payload],
        },
      };

    case 'ENABLE_BATCH_MODE':
      return {
        ...state,
        batch: { ...state.batch, isBatchMode: true },
      };

    case 'DISABLE_BATCH_MODE':
      return {
        ...state,
        batch: { ...state.batch, isBatchMode: false },
      };

    case 'START_BATCH_PROCESSING':
      return {
        ...state,
        batch: { ...state.batch, isBatchProcessing: true },
      };

    case 'STOP_BATCH_PROCESSING':
      return {
        ...state,
        batch: { ...state.batch, isBatchProcessing: false },
      };

    case 'TOGGLE_AUTO_CAPTURE':
      return {
        ...state,
        batch: { ...state.batch, isAutoCapture: !state.batch.isAutoCapture },
      };

    case 'SET_BATCH_ITEM_RUNNING':
      return {
        ...state,
        batch: { ...state.batch, isBatchItemRunning: action.payload },
      };

    case 'SET_ACTIVE_TAB':
      return {
        ...state,
        ui: { ...state.ui, activeTab: action.payload },
      };

    case 'TOGGLE_MINI_MODE':
      return {
        ...state,
        ui: { ...state.ui, showMiniMode: !state.ui.showMiniMode },
      };

    case 'TOGGLE_SHORTCUTS':
      return {
        ...state,
        ui: { ...state.ui, showShortcuts: !state.ui.showShortcuts },
      };

    case 'SET_DRAGGING':
      return {
        ...state,
        ui: { ...state.ui, isDragging: action.payload },
      };

    default:
      return state;
  }
};

// Custom hook
export function useAppState() {
  const [state, dispatch] = useReducer(appReducer, initialState);
  return { state, dispatch };
}
