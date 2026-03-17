interface ProgressBarProps {
  taskId: string;
  progress: number;
}

export function ProgressBar({ taskId, progress }: ProgressBarProps) {
  const clamped = Math.min(100, Math.max(0, progress));
  return (
    <div className="progress-container" aria-label={`Task ${taskId} progress`}>
      <div
        className="progress-bar"
        role="progressbar"
        aria-valuenow={clamped}
        aria-valuemin={0}
        aria-valuemax={100}
        style={{ width: `${clamped}%` }}
      />
      <span className="progress-text">{clamped.toFixed(1)}%</span>
    </div>
  );
}
