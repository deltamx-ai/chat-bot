import type { TaskDto } from '../../lib/taskApi'

interface ConversationListProps {
  tasks: TaskDto[]
  selectedTaskId: string
  onSelectTask: (taskId: string) => void
}

export function ConversationList({ tasks, selectedTaskId, onSelectTask }: ConversationListProps) {
  return (
    <section className="flex h-full min-h-[820px] flex-col rounded-[28px] border border-white/10 bg-[#060913] p-4 text-slate-100 shadow-[0_20px_80px_rgba(0,0,0,0.35)]">
      <header className="mb-4 flex items-center justify-between px-2">
        <h2 className="text-lg font-semibold text-white">所有会话</h2>
        <button className="rounded-xl border border-white/10 px-2 py-1 text-xs text-slate-400 transition hover:bg-white/5 hover:text-white">
          筛选
        </button>
      </header>

      <div className="mb-4 px-2 text-sm text-slate-500">今天</div>

      <div className="space-y-2">
        {tasks.map((task) => {
          const active = task.id === selectedTaskId
          return (
            <button
              key={task.id}
              onClick={() => onSelectTask(task.id)}
              className={`flex w-full items-center justify-between rounded-2xl border px-4 py-4 text-left transition ${
                active
                  ? 'border-violet-400/50 bg-white/5 shadow-[inset_3px_0_0_0_rgba(167,139,250,1)]'
                  : 'border-transparent hover:border-white/10 hover:bg-white/[0.03]'
              }`}
            >
              <div className="min-w-0">
                <div className="truncate text-sm font-medium text-white">{task.title}</div>
                <div className="mt-1 truncate text-xs text-slate-500">{task.goal}</div>
              </div>
              <div className="ml-4 text-xs text-slate-500">{task.steps.length}步</div>
            </button>
          )
        })}
      </div>
    </section>
  )
}
