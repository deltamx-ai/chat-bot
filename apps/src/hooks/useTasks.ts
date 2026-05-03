import useSWR from 'swr'

import type { TaskDto, TaskEventDto } from '../lib/taskApi'
import { demoTaskEvents, demoTasks } from '../lib/taskApi'

function normalizeTasks(payload: unknown): TaskDto[] {
  if (Array.isArray(payload)) return payload as TaskDto[]
  if (payload && typeof payload === 'object') {
    if ('tasks' in payload && Array.isArray((payload as { tasks?: unknown }).tasks)) {
      return (payload as { tasks: TaskDto[] }).tasks
    }
    if ('data' in payload && Array.isArray((payload as { data?: unknown }).data)) {
      return (payload as { data: TaskDto[] }).data
    }
  }
  return demoTasks
}

function normalizeEvents(payload: unknown, taskId: string): TaskEventDto[] {
  if (Array.isArray(payload)) return payload as TaskEventDto[]
  if (payload && typeof payload === 'object') {
    if ('events' in payload && Array.isArray((payload as { events?: unknown }).events)) {
      return (payload as { events: TaskEventDto[] }).events
    }
    if ('data' in payload && Array.isArray((payload as { data?: unknown }).data)) {
      return (payload as { data: TaskEventDto[] }).data
    }
  }
  return demoTaskEvents.filter((event) => event.taskId === taskId)
}

export function useTasks() {
  const query = useSWR<unknown>('/api/tasks')

  return {
    ...query,
    tasks: normalizeTasks(query.data),
  }
}

export function useTaskEvents(taskId: string) {
  const key = taskId ? `/api/tasks/${taskId}/events` : null
  const query = useSWR<unknown>(key)

  return {
    ...query,
    events: normalizeEvents(query.data, taskId),
  }
}
