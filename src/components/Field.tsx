import { cn } from "#/lib/cn";

type FieldProps = {
  label: string
  htmlFor?: string
  layout?: 'vertical' | 'horizontal'
  className?: string
  children: React.ReactNode
}

export function Field({label, htmlFor, layout = 'vertical', className, children}: FieldProps) {
  const wrapperClass = layout === 'horizontal'
    ? 'flex flex-row items-start gap-4'
    : 'flex flex-col gap-1';

  return (
    <div className={cn(wrapperClass, className)}>
      <label
        htmlFor={htmlFor}
        className={cn(
          'text-xs text-gray-500',
          layout === 'horizontal' && 'min-w-[10rem] flex-shrink-0',
        )}
      >
        {label}
      </label>
      {children}
    </div>
  );
}
