import { ProgressBar } from './ProgressBar';

interface SubTask {
  task_id: string;
  status: string;
  progress: number;
  sub_tasks?: SubTask[];
}

interface TaskTreeProps {
  task: SubTask;
}

export function TaskTree({ task }: TaskTreeProps) {
  return (
    <div className="task-node" style={{ marginLeft: '16px', borderLeft: '2px solid #ccc', paddingLeft: '8px' }}>
      <div className="task-info" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <span className="task-id">{task.task_id}</span>
        <span className={`task-status status-${task.status.toLowerCase()}`}>{task.status}</span>
        <ProgressBar taskId={task.task_id} progress={task.progress} />
      </div>
      {task.sub_tasks?.map(sub => (
        <TaskTree key={sub.task_id} task={sub} />
      ))}
    </div>
  );
}
