type FieldProps = {
  label: string
  htmlFor?: string
  children: React.ReactNode
}

export function Field({label, htmlFor, children}: FieldProps) {
  return (
    <div className="flex flex-col gap-1">
      <label htmlFor={htmlFor} className="text-xs text-gray-500">{label}</label>
      {children}
    </div>
  );
}
