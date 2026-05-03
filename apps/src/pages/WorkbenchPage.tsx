import { useMemo } from 'react'

import { GlobalErrorToast } from '../components/GlobalErrorToast'
import { useGlobalErrorToast } from '../hooks/useGlobalErrorToast'
import { useTaskEvents, useTasks } from '../hooks/useTasks'
import { useTaskUiStore } from '../store/taskUiStore'
import { ConversationList } from '../components/workbench/ConversationList'
import { Sidebar } from '../components/workbench/Sidebar'
import { TaskWorkspace } from '../components/workbench/TaskWorkspace'

function WorkbenchPage() {
  const { tasks } = useTasks()
  const selectedTaskId = useTaskUiStore((state) => state.selectedTaskId)
  const setSelectedTaskId = useTaskUiStore((state) => state.setSelectedTaskId)
  const activeTaskId = selectedTaskId || tasks[0]?.id || ''
  const { events } = useTaskEvents(activeTaskId)
  const { message, clear } = useGlobalErrorToast()
  const selectedTask = useMemo(
    () => tasks.find((task) => task.id === activeTaskId) ?? tasks[0],
    [tasks, activeTaskId],
  )

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top_left,rgba(124,92,255,0.35),transparent_28%),radial-gradient(circle_at_top_right,rgba(104,211,255,0.22),transparent_24%),#0a0d14] px-8 py-8 text-slate-100">
      <div className="mx-auto grid max-w-[1800px] grid-cols-[320px_420px_minmax(0,1fr)] gap-4">
        <Sidebar />
        <ConversationList
          tasks={tasks}
          selectedTaskId={activeTaskId}
          onSelectTask={setSelectedTaskId}
        />
        <TaskWorkspace task={selectedTask} events={events} />
      </div>

      <GlobalErrorToast message={message} onClose={clear} />
    </main>
  )
}

export default WorkbenchPage
