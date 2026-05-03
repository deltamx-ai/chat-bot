const ghostButtonClass =
  'rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-slate-300 transition hover:border-violet-400/40 hover:text-white'

export function ComposerPanel() {
  return (
    <section className="rounded-3xl border border-white/10 bg-[#070a12]/74 p-4 transition-colors">
      <div className="mb-3 flex gap-2.5">
        <button className={ghostButtonClass}>执行</button>
        <button className={ghostButtonClass}>待办</button>
      </div>
      <textarea
        className="min-h-32 w-full resize-y rounded-2xl border border-white/10 bg-white/3 px-4 py-3 text-slate-100 outline-none transition placeholder:text-slate-500 focus:border-violet-400/80"
        placeholder="按 Shift + Return 执行"
        rows={5}
      />
    </section>
  )
}
