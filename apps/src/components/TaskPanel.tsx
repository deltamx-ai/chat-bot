import type { TaskDto, TaskEventDto } from '../lib/taskApi'

interface TaskPanelProps {
  tasks: TaskDto[]
  selectedTaskId: string
  events: TaskEventDto[]
  onSelectTask: (taskId: string) => void
}

const statusTone: Record<string, string> = {
  Draft: 'bg-slate-500/15 text-slate-300 border-slate-500/20',
  Pending: 'bg-amber-500/15 text-amber-300 border-amber-500/20',
  Running: 'bg-sky-500/15 text-sky-300 border-sky-500/20',
  Blocked: 'bg-rose-500/15 text-rose-300 border-rose-500/20',
  Succeeded: 'bg-emerald-500/15 text-emerald-300 border-emerald-500/20',
  Failed: 'bg-red-500/15 text-red-300 border-red-500/20',
  Cancelled: 'bg-zinc-500/15 text-zinc-300 border-zinc-500/20',
  Archived: 'bg-indigo-500/15 text-indigo-300 border-indigo-500/20',
}

export function TaskPanel({ tasks, selectedTaskId, events, onSelectTask }: TaskPanelProps) {
  const selectedTask = tasks.find((task) => task.id === selectedTaskId) ?? tasks[0]
  const filteredEvents = events.filter((event) => event.taskId === selectedTask?.id)

  return (
    <section className="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]">
      <aside className="rounded-3xl border border-white/10 bg-[#070a12]/74 p-4">
        <div className="mb-4 flex items-center justify-between">
          <div>
            <div className="text-sm text-slate-400">Task runtime</div>
            <h3 className="text-lg font-semibold text-white">任务面板</h3>
          </div>
          <span className="rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-slate-300">
            {tasks.length} tasks
          </span>
        </div>

        <div className="space-y-3">
          {tasks.map((task) => {
            const active = task.id === selectedTask?.id
            return (
              <button
                key={task.id}
                type="button"
                onClick={() => onSelectTask(task.id)}
                className={`w-full rounded-2xl border p-3 text-left transition ${
                  active
                    ? 'border-violet-400/50 bg-violet-500/10'
                    : 'border-white/10 bg-white/[0.03] hover:border-white/20 hover:bg-white/[0.05]'
                }`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <div className="text-sm font-medium text-white">{task.title}</div>
                    <div className="mt-1 text-xs leading-5 text-slate-400">{task.goal}</div>
                  </div>
                  <span
                    className={`rounded-full border px-2 py-0.5 text-[11px] ${statusTone[task.status] ?? statusTone.Pending}`}
                  >
                    {task.status}
                  </span>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {task.tags.map((tag) => (
                    <span
                      key={tag}
                      className="rounded-full border border-white/10 bg-white/[0.03] px-2 py-0.5 text-[11px] text-slate-300"
                    >
                      #{tag}
                    </span>
                  ))}
                </div>
              </button>
            )
          })}
        </div>
      </aside>

      <section className="rounded-3xl border border-white/10 bg-[#070a12]/74 p-5">
        {selectedTask ? (
          <>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <div className="text-sm text-slate-400">Task detail</div>
                <h3 className="text-xl font-semibold text-white">{selectedTask.title}</h3>
                <p className="mt-1 text-sm text-slate-400">{selectedTask.goal}</p>
              </div>
              <span
                className={`rounded-full border px-3 py-1 text-xs ${statusTone[selectedTask.status] ?? statusTone.Pending}`}
              >
                {selectedTask.status}
              </span>
            </div>

            <div className="mt-5 grid gap-4 lg:grid-cols-[minmax(0,1fr)_320px]">
              <div className="rounded-2xl border border-white/10 bg-white/[0.03] p-4">
                <div className="mb-3 text-sm font-medium text-white">Steps</div>
                <div className="space-y-3">
                  {selectedTask.steps.map((step) => (
                    <div
                      key={step.id}
                      className="rounded-2xl border border-white/10 bg-black/20 p-3"
                    >
                      <div className="flex items-center justify-between gap-3">
                        <div>
                          <div className="text-sm text-white">{step.title}</div>
                          <div className="mt-1 text-xs text-slate-400">
                            {step.action} · {step.toolName}
                          </div>
                        </div>
                        <span
                          className={`rounded-full border px-2 py-0.5 text-[11px] ${statusTone[step.status] ?? statusTone.Pending}`}
                        >
                          {step.status}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <aside className="rounded-2xl border border-white/10 bg-white/[0.03] p-4">
                <div className="mb-3 text-sm font-medium text-white">Events</div>
                <div className="space-y-3 text-sm text-slate-300">
                  {filteredEvents.map((event) => (
                    <div key={event.id} className="rounded-2xl border border-white/10 bg-black/20 p-3">
                      <div className="font-medium text-white">{event.kind}</div>
                      <div className="mt-1 text-xs text-slate-400">
                        {event.stepId ? `step: ${event.stepId}` : 'task event'}
                      </div>
                    </div>
                  ))}
                  {filteredEvents.length === 0 ? (
                    <div className="rounded-2xl border border-dashed border-white/10 p-3 text-xs text-slate-500">
                      暂时还没有事件
                    </div>
                  ) : null}
                </div>
              </aside>
            </div>
          </>
        ) : null}
      </section>
    </section>
  )
}
