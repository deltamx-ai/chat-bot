import { ThemeToggle } from '../ThemeToggle'

interface WorkbenchHeaderProps {
  metrics: Array<{ label: string; value: string }>
}

export function WorkbenchHeader({ metrics }: WorkbenchHeaderProps) {
  return (
    <header className="mb-5 flex flex-col gap-4 border-b border-white/10 pb-5 lg:flex-row lg:items-center lg:justify-between">
      <div>
        <div className="mb-2 inline-flex items-center rounded-full border border-violet-400/30 bg-violet-500/10 px-3 py-1 text-xs uppercase tracking-[0.3em] text-violet-200">
          Draft session
        </div>
        <h1 className="text-2xl font-semibold text-white sm:text-3xl">
          统一 skills 目录到 Alma
        </h1>
        <p className="mt-2 max-w-3xl text-sm leading-7 text-slate-400 sm:text-base">
          把执行计划、任务状态、运行日志和输入输出放到同一个工作台里，方便在开发前先审阅、批准、再执行。
        </p>
      </div>

      <div className="flex flex-col items-end gap-3">
        <ThemeToggle />
        <div className="grid gap-3 sm:grid-cols-3 lg:w-[480px]">
          {metrics.map((item) => (
            <div
              key={item.label}
              className="rounded-2xl border border-white/10 bg-white/[0.03] p-4 text-center transition-colors"
            >
              <div className="text-xs uppercase tracking-[0.25em] text-slate-500">{item.label}</div>
              <div className="mt-2 text-2xl font-semibold text-white">
                {item.value}
              </div>
            </div>
          ))}
        </div>
      </div>
    </header>
  )
}
