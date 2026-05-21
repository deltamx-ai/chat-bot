import { mutate } from 'swr'
import { useCallback, useMemo } from 'react'

import { GlobalErrorToast } from '../components/GlobalErrorToast'
import { useGlobalErrorToast } from '../hooks/useGlobalErrorToast'
import { useTaskEvents, useTasks } from '../hooks/useTasks'
import { useTaskUiStore } from '../store/taskUiStore'
import { pendingByConversation, usePendingOpsStore } from '../store/pendingOpsStore'
import { ConversationList } from '../components/workbench/ConversationList'
import { Sidebar } from '../components/workbench/Sidebar'
import { TaskWorkspace } from '../components/workbench/TaskWorkspace'
import {
  createConversation,
  deleteConversation,
  renameConversation,
  useConversations,
  useMessages,
  useModels,
  useSendMessage,
  type SendMessageRequestDto,
} from '../lib/messageApi'
import { httpClient } from '../lib/http'

function WorkbenchPage() {
  const { tasks } = useTasks()
  const { conversations } = useConversations()
  const { models } = useModels()
  const { send } = useSendMessage()
  const selectedTaskId = useTaskUiStore((state) => state.selectedTaskId)
  const setSelectedTaskId = useTaskUiStore((state) => state.setSelectedTaskId)
  const storedActiveConversationId = useTaskUiStore((state) => state.activeConversationId)
  const setActiveConversationId = useTaskUiStore((state) => state.setActiveConversationId)
  const ops = usePendingOpsStore((state) => state.ops)
  const activeTaskId = selectedTaskId || tasks[0]?.id || ''
  const storedIsValid =
    storedActiveConversationId.length > 0 &&
    conversations.some((conv) => conv.id === storedActiveConversationId)
  const activeConversationId = storedIsValid
    ? storedActiveConversationId
    : conversations[0]?.id || 'conv_default'
  const { events } = useTaskEvents(activeTaskId)
  const { messages } = useMessages(activeConversationId)
  const { message, clear } = useGlobalErrorToast()
  const selectedTask = useMemo(
    () => tasks.find((task) => task.id === activeTaskId) ?? tasks[0],
    [tasks, activeTaskId],
  )
  const pendingForActive = useMemo(
    () => pendingByConversation(ops, activeConversationId),
    [ops, activeConversationId],
  )
  const isSendingMessage = pendingForActive.length > 0

  const handleSend = useCallback(
    (req: SendMessageRequestDto) => send(req, { taskId: activeTaskId }),
    [send, activeTaskId],
  )

  const handleStop = useCallback(() => {
    usePendingOpsStore.getState().abortByConversation(activeConversationId)
  }, [activeConversationId])

  const handleCreateConversation = useCallback(async () => {
    const created = await createConversation()
    setActiveConversationId(created.id)
  }, [setActiveConversationId])

  const handleRequestRun = useCallback(async () => {
    if (!selectedTask) return
    try {
      await httpClient.post(`/api/tasks/${selectedTask.id}/runs`)
      await mutate('/api/tasks')
      await mutate('/api/approval-requests')
      await mutate(`/api/tasks/${selectedTask.id}/events`)
    } catch (error) {
      console.error('createRun failed', error)
    }
  }, [selectedTask])

  const handleRenameConversation = useCallback(
    async (id: string, currentTitle: string) => {
      const next = window.prompt('会话标题', currentTitle)
      if (next === null) return
      const trimmed = next.trim()
      if (trimmed.length === 0 || trimmed === currentTitle) return
      try {
        await renameConversation(id, trimmed)
      } catch (error) {
        console.error('renameConversation failed', error)
      }
    },
    [],
  )

  const handleDeleteConversation = useCallback(
    async (id: string, currentTitle: string) => {
      if (!window.confirm(`确认删除会话 "${currentTitle}" ?`)) return
      try {
        await deleteConversation(id)
        if (id === activeConversationId) {
          const next = conversations.find((conv) => conv.id !== id)
          setActiveConversationId(next?.id ?? '')
        }
      } catch (error) {
        console.error('deleteConversation failed', error)
      }
    },
    [activeConversationId, conversations, setActiveConversationId],
  )

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top_left,rgba(124,92,255,0.35),transparent_28%),radial-gradient(circle_at_top_right,rgba(104,211,255,0.22),transparent_24%),#0a0d14] px-8 py-8 text-slate-100">
      <div className="mx-auto grid max-w-[1800px] grid-cols-[320px_420px_minmax(0,1fr)] gap-4">
        <Sidebar />
        <ConversationList
          tasks={tasks}
          selectedTaskId={activeTaskId}
          onSelectTask={setSelectedTaskId}
          pendingOps={ops}
        />
        <TaskWorkspace
          task={selectedTask}
          events={events}
          messages={messages}
          models={models}
          conversationId={activeConversationId}
          conversations={conversations}
          pendingOps={ops}
          onSelectConversation={setActiveConversationId}
          onCreateConversation={handleCreateConversation}
          onRenameConversation={handleRenameConversation}
          onDeleteConversation={handleDeleteConversation}
          onSendMessage={handleSend}
          onRequestRun={handleRequestRun}
          onStopStreaming={handleStop}
          isSendingMessage={isSendingMessage}
        />
      </div>

      <GlobalErrorToast message={message} onClose={clear} />
    </main>
  )
}

export default WorkbenchPage
