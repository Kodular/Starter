import React from "react";

const inputClass = "font-sans text-sm px-3 py-1.5 border border-gray-300 rounded outline-none focus:border-primary disabled:bg-gray-100 disabled:text-gray-400 disabled:cursor-not-allowed w-full";

export function Input(props: React.InputHTMLAttributes<HTMLInputElement>) {
  return <input className={inputClass} {...props}/>;
}
