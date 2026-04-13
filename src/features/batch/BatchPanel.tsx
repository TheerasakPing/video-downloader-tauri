import React, { useCallback } from 'react';
import { Play, Pause, X } from 'lucide-react';
import { AppState, AppAction } from '../../hooks/useAppState';

interface BatchPanelProps {
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
  onToggleBatchProcessing: () => void;
  onClearBatch: () => void;
}

export const BatchPanel: React.FC<BatchPanelProps> = ({
  state,
  dispatch,
  onToggleBatchProcessing,
  onClearBatch,
}) => {
  const { batch } = state;

  const handleToggleAutoCapture = useCallback(() => {
    dispatch({ type: 'TOGGLE_AUTO_CAPTURE' });
  }, [dispatch]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Batch Mode</h3>
        <div className="flex gap-2">
          <button
            onClick={handleToggleAutoCapture}
            className={`px-3 py-1 rounded-lg text-sm transition-colors ${
              batch.isAutoCapture
                ? 'bg-green-600 hover:bg-green-700 text-white'
                : 'bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600'
            }`}
          >
            Auto Capture {batch.isAutoCapture ? 'ON' : 'OFF'}
          </button>
          <button
            onClick={onToggleBatchProcessing}
            className={`flex items-center gap-2 px-3 py-1 rounded-lg text-sm transition-colors ${
              batch.isBatchProcessing
                ? 'bg-yellow-600 hover:bg-yellow-700 text-white'
                : 'bg-blue-600 hover:bg-blue-700 text-white'
            }`}
          >
            {batch.isBatchProcessing ? (
              <>
                <Pause className="w-4 h-4" />
                Pause
              </>
            ) : (
              <>
                <Play className="w-4 h-4" />
                Start
              </>
            )}
          </button>
          <button
            onClick={onClearBatch}
            className="flex items-center gap-2 px-3 py-1 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm transition-colors"
          >
            <X className="w-4 h-4" />
            Clear
          </button>
        </div>
      </div>

      <div className="text-sm text-gray-600 dark:text-gray-400">
        {batch.isBatchItemRunning && (
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse" />
            Processing batch item...
          </div>
        )}
      </div>
    </div>
  );
};

export default BatchPanel;
