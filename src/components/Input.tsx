import React from "react";
import {cn} from "../lib/cn.ts";

const inputClass = "font-sans text-sm px-3 py-1 border border-gray-300 rounded outline-none focus:border-primary disabled:bg-gray-100 disabled:text-gray-400 disabled:cursor-not-allowed w-full";

export function Input({className, ...props}: React.InputHTMLAttributes<HTMLInputElement>) {
  return <input className={cn(inputClass, className)} {...props}/>;
}
