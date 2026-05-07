import React from "react";

type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'outlined'
}

const base = "font-sans text-sm px-4 py-1.5 rounded cursor-pointer border-none disabled:opacity-50 disabled:cursor-not-allowed";

const variants = {
  primary: "bg-primary text-white hover:bg-primary-dark",
  outlined: "bg-transparent border border-gray-200 text-gray-500 hover:bg-gray-100",
} as const;

export function Button({variant = 'primary', className = '', ...props}: ButtonProps) {
  return <button className={`${base} ${variants[variant]} ${className}`} {...props}/>;
}
