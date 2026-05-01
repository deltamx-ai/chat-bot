import useSWR from 'swr'

import type { TaskDto, TaskEventDto } from '../lib/taskApi'
import { demoTaskEvents, demoTasks } from '../lib/taskApi'

export function useTasks() {
  const query = useSWR<TaskDto[]>('/api/tasks')

  return {
    ...query,
    tasks: query.data ?? demoTasks,
  }
}

export function useTaskEvents(taskId: string) {
  const key = taskId ? `/api/tasks/${taskId}/events` : null
  const query = useSWR<TaskEventDto[]>(key)

  return {
    ...query,
    events: query.data ?? demoTaskEvents.filter((event) => event.taskId === taskId),
  }
}
