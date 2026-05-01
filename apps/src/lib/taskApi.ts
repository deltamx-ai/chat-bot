export type TaskStatus =
  | 'Draft'
  | 'Pending'
  | 'Running'
  | 'Blocked'
  | 'Succeeded'
  | 'Failed'
  | 'Cancelled'
  | 'Archived'

export type StepStatus =
  | 'Pending'
  | 'Ready'
  | 'Running'
  | 'Succeeded'
  | 'Failed'
  | 'Skipped'
  | 'Cancelled'

export interface TaskStepDto {
  id: string
  index: number
  title: string
  action: string
  toolName: string
  status: StepStatus
}

export interface TaskDto {
  id: string
  title: string
  goal: string
  status: TaskStatus
  kind: string
  priority: string
  tags: string[]
  steps: TaskStepDto[]
}

export interface TaskEventDto {
  id: string
  taskId: string
  stepId?: string
  kind: string
  payload: Record<string, unknown>
}

export interface RunTaskResponseDto {
  ok: boolean
  results?: Array<Record<string, unknown>>
  error?: string
}

export const demoTasks: TaskDto[] = [
  {
    id: 'task_plan_default_1',
    title: 'Server run task',
    goal: 'Run task through server skeleton',
    status: 'Succeeded',
    kind: 'Feature',
    priority: 'Normal',
    tags: ['runtime', 'server'],
    steps: [
      {
        id: 'step_task_plan_default_1_1',
        index: 0,
        title: 'Inspect input',
        action: 'Read',
        toolName: 'read',
        status: 'Succeeded',
      },
      {
        id: 'step_task_plan_default_1_2',
        index: 1,
        title: 'Validate request',
        action: 'Validate',
        toolName: 'validate',
        status: 'Succeeded',
      },
    ],
  },
  {
    id: 'task_demo_pending',
    title: 'Desktop task panel',
    goal: 'Connect frontend with task runtime contract',
    status: 'Running',
    kind: 'Feature',
    priority: 'High',
    tags: ['frontend', 'tasks'],
    steps: [
      {
        id: 'step_demo_pending_1',
        index: 0,
        title: 'Define DTO contract',
        action: 'Write',
        toolName: 'write',
        status: 'Succeeded',
      },
      {
        id: 'step_demo_pending_2',
        index: 1,
        title: 'Render task panel',
        action: 'Write',
        toolName: 'write',
        status: 'Running',
      },
    ],
  },
]

export const demoTaskEvents: TaskEventDto[] = [
  {
    id: 'evt_task_plan_default_1_1',
    taskId: 'task_plan_default_1',
    kind: 'TaskStarted',
    payload: {},
  },
  {
    id: 'evt_task_plan_default_1_2',
    taskId: 'task_plan_default_1',
    stepId: 'step_task_plan_default_1_1',
    kind: 'StepSucceeded',
    payload: { tool: 'read' },
  },
  {
    id: 'evt_task_plan_default_1_3',
    taskId: 'task_plan_default_1',
    stepId: 'step_task_plan_default_1_2',
    kind: 'StepSucceeded',
    payload: { tool: 'validate' },
  },
]
