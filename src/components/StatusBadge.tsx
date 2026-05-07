export function StatusBadge({label, isRunning}: { label: string; isRunning: boolean | undefined }) {
  return (
    <div className="flex items-center gap-2">
      <div className="relative flex items-center justify-center size-2 shrink-0">
        {isRunning && (
          <span className="absolute inline-flex size-full rounded-full bg-green-400 opacity-60 animate-ping"/>
        )}
        <span className={`relative size-2 rounded-full ${isRunning ? 'bg-green-500' : 'bg-red-400'}`}/>
      </div>
      <span className="text-sm text-gray-500">{label}</span>
      <span className={`text-sm font-medium ${isRunning ? 'text-green-700' : isRunning === undefined ? 'text-gray-400' : 'text-red-500'}`}>
        {isRunning === undefined ? 'Checking…' : isRunning ? 'Running' : 'Stopped'}
      </span>
    </div>
  )
}
