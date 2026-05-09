const variants = {
  checking: { dot: 'bg-gray-300', text: 'text-gray-400', label: 'Checking…', ping: false },
  running:  { dot: 'bg-green-500', text: 'text-green-700', label: 'Running', ping: true },
  stopped:  { dot: 'bg-red-400', text: 'text-red-500', label: 'Stopped', ping: false },
} as const;

type StatusBadgeVariant = keyof typeof variants;

export function StatusBadge({ label, variant }: { label: string; variant: StatusBadgeVariant }) {
  const v = variants[variant];
  return (
    <div className="flex items-center gap-2 whitespace-nowrap">
      <div className="relative flex items-center justify-center size-2 shrink-0">
        {v.ping && <span className="absolute inline-flex size-full rounded-full bg-green-400 opacity-60 animate-ping"/>}
        <span className={`relative size-2 rounded-full ${v.dot}`}/>
      </div>
      <span className="text-sm text-gray-500">{label}</span>
      <span className={`text-sm font-medium ${v.text}`}>{v.label}</span>
    </div>
  )
}
