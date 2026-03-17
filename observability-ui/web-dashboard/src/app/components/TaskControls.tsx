'use client';

import { useState } from 'react';

interface TaskControlsProps {
  taskId: string;
  status: string;
  onStatusChange?: (newStatus: string) => void;
}

export function TaskControls({ taskId, status, onStatusChange }: TaskControlsProps) {
  const [loading, setLoading] = useState(false);

  const handlePause = async () => {
    setLoading(true);
    try {
      await fetch(`/api/v1/tasks/${taskId}/pause`, { method: 'POST' });
      onStatusChange?.('PAUSED');
    } finally {
      setLoading(false);
    }
  };

  const handleResume = async () => {
    setLoading(true);
    try {
      await fetch(`/api/v1/tasks/${taskId}/resume`, { method: 'POST' });
      onStatusChange?.('RUNNING');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="task-controls">
      {status === 'RUNNING' && (
        <button onClick={handlePause} disabled={loading} aria-label="Pause task">
          {loading ? '...' : '暂停'}
        </button>
      )}
      {status === 'PAUSED' && (
        <button onClick={handleResume} disabled={loading} aria-label="Resume task">
          {loading ? '...' : '恢复'}
        </button>
      )}
    </div>
  );
}
