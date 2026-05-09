import React from "react";

type CheckboxProps = Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> & {
  label: string;
}

export function Checkbox({label, className, ...props}: CheckboxProps) {
  return (
    <label className="flex items-center gap-2 cursor-pointer select-none">
      <input type="checkbox" className={`w-4 h-4 accent-primary ${className ?? ''}`} {...props} />
      <span className="text-sm">{label}</span>
    </label>
  );
}
