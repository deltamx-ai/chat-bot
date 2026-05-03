interface PlanOverviewProps {
  steps: string[]
}

export function PlanOverview({ steps }: PlanOverviewProps) {
  return (
    <section className="rounded-3xl border border-white/10 bg-[#070a12]/78 p-5 transition-colors">
      <div className="mb-4 flex flex-wrap items-center gap-2 text-xs text-slate-400">
        <span className="rounded-full border border-emerald-400/25 bg-emerald-500/10 px-3 py-1 text-emerald-200">
          Plan ready
        </span>
        <span className="rounded-full border border-white/10 px-3 py-1">
          来源：项目技能管理
        </span>
        <span className="rounded-full border border-white/10 px-3 py-1">
          更新于刚刚
        </span>
      </div>

      <h2 className="text-lg font-medium text-white">实施概览</h2>
      <p className="mt-3 text-sm leading-7 text-slate-400 sm:text-[15px]">
        当前建议把 skill 文件统一收敛到 Alma 的技能目录，避免仓库副本和运行时目录不同步。整个执行过程、审批节点和结果统一收在右侧详情面板里。
      </p>

      <ol className="mt-5 list-decimal space-y-2 pl-5 leading-8 text-slate-100">
        {steps.map((step) => (
          <li key={step}>{step}</li>
        ))}
      </ol>

      <div className="mt-5 rounded-2xl bg-white/4 p-4 text-sm leading-6 text-slate-400 dark:bg-white/4">
        预计影响：只会修改统一 skills 目录，不动源目录；后续结果会同步到任务状态和执行日志。
      </div>
    </section>
  )
}
