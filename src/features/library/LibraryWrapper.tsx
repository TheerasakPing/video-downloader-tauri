import React from 'react';
import LibraryPanel from '../../components/LibraryPanel';
import ErrorBoundary from '../../components/ErrorBoundary';

interface LibraryWrapperProps {
  isActive: boolean;
}

export const LibraryWrapper: React.FC<LibraryWrapperProps> = ({ isActive }) => {
  if (!isActive) return null;

  return (
    <ErrorBoundary>
      <LibraryPanel />
    </ErrorBoundary>
  );
};

export default LibraryWrapper;
