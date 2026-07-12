import React from "react";

type IconButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  active?: boolean
}

export function IconButton({active, ...props}: IconButtonProps) {
  return (
    <button
      className="bg-transparent border-none cursor-pointer text-xl size-8 flex items-center justify-center rounded text-gray-500 hover:bg-primary-light hover:text-primary data-[active=true]:bg-primary-light data-[active=true]:text-primary"
      data-active={active}
      {...props}
    />
  );
}
