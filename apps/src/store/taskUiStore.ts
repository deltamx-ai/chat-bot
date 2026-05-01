import { create } from 'zustand'

interface TaskUiState {
  selectedTaskId: string
  setSelectedTaskId: (taskId: string) => void
}

export const useTaskUiStore = create<TaskUiState>((set) => ({
  selectedTaskId: '',
  setSelectedTaskId: (selectedTaskId) => set({ selectedTaskId }),
}))
