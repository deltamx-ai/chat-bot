import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface TaskUiState {
  selectedTaskId: string
  activeConversationId: string
  setSelectedTaskId: (taskId: string) => void
  setActiveConversationId: (conversationId: string) => void
}

export const useTaskUiStore = create<TaskUiState>()(
  persist(
    (set) => ({
      selectedTaskId: '',
      activeConversationId: '',
      setSelectedTaskId: (selectedTaskId) => set({ selectedTaskId }),
      setActiveConversationId: (activeConversationId) => set({ activeConversationId }),
    }),
    {
      name: 'chat-bot-task-ui',
      partialize: (state) => ({
        selectedTaskId: state.selectedTaskId,
        activeConversationId: state.activeConversationId,
      }),
    },
  ),
)
