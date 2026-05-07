import React from "react";
import {cn} from "../lib/cn.ts";

type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'outlined'
}

const base = "font-sans text-sm px-4 py-1.5 rounded cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed";

const variants = {
  primary: "border-none bg-primary text-white hover:bg-primary-dark",
  outlined: "border border-gray-200 bg-transparent text-gray-500 hover:bg-gray-100",
} as const;

export function Button({variant = 'primary', className, ...props}: ButtonProps) {
  return <button className={cn(base, variants[variant], className)} {...props}/>;
}
