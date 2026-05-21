import { approveApprovalRequest, rejectApprovalRequest, useApprovalRequests } from '../../lib/approvalApi'

interface ApprovalPanelProps {
  activeTaskId?: string
}

export function ApprovalPanel({ activeTaskId }: ApprovalPanelProps) {
  const { approvals } = useApprovalRequests()
  const pending = approvals.filter((item) => item.status === 'Pending')

  return (
    <aside className="rounded-2xl border border-white/10 bg-[#090d18] p-4">
      <div className="mb-3 flex items-center justify-between">
        <div className="text-sm font-medium text-white">待审批</div>
        <div className="text-xs text-slate-500">{pending.length}</div>
      </div>
      <div className="space-y-3">
        {pending.map((approval) => (
          <div key={approval.id} className="rounded-2xl border border-white/10 bg-black/20 p-3 text-sm text-slate-300">
            <div className="font-medium text-white">{approval.title}</div>
            <div className="mt-1 text-xs text-slate-500">{approval.kind} · {approval.task_id}</div>
            <div className="mt-2 text-xs text-slate-400 whitespace-pre-wrap">
              {String(approval.payload.summary ?? approval.payload.goal ?? '')}
            </div>
            <div className="mt-3 flex gap-2">
              <button
                type="button"
                onClick={() => {
                  void approveApprovalRequest(approval.id, undefined, activeTaskId)
                }}
                className="rounded-xl border border-emerald-400/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-200 transition hover:brightness-110"
              >
                批准
              </button>
              <button
                type="button"
                onClick={() => {
                  void rejectApprovalRequest(approval.id, undefined, activeTaskId)
                }}
                className="rounded-xl border border-rose-400/30 bg-rose-500/10 px-3 py-2 text-xs text-rose-200 transition hover:brightness-110"
              >
                拒绝
              </button>
            </div>
          </div>
        ))}
        {pending.length === 0 ? (
          <div className="rounded-2xl border border-dashed border-white/10 p-3 text-xs text-slate-500">
            现在没有待审批项
          </div>
        ) : null}
      </div>
    </aside>
  )
}
