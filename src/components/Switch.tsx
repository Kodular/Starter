import React from "react";

type SwitchProps = Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> & {
  label: string;
};

export function Switch({label, className, ...props}: SwitchProps) {
  return (
    <label className={`relative inline-flex items-center cursor-pointer ${className ?? ''}`}>
      <input type="checkbox" className="sr-only peer" {...props} />
      <div className="w-9 h-5 bg-slate-200 rounded-full peer-checked:bg-primary transition-colors" />
      <div className="absolute left-0.5 top-0.5 w-4 h-4 bg-white rounded-full border border-slate-300 transition-transform peer-checked:translate-x-4" />
      <span className="sr-only">{label}</span>
    </label>
  );
}
