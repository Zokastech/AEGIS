// AEGIS — zokastech.fr — Apache 2.0 / MIT

import * as React from "react";
import { cn } from "@/lib/utils";

const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, type, ...props }, ref) => (
    <input
      type={type}
      className={cn(
        "flex h-9 w-full rounded-md border border-[#e2e8f0] bg-white px-3 py-1 text-sm text-zokastech-dark shadow-sm transition-[border-color,box-shadow] duration-200 ease-smooth file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-[#94a3b8] focus-visible:outline-none focus-visible:border-brand-orange focus-visible:ring-2 focus-visible:ring-[rgba(255,138,0,0.2)] disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      ref={ref}
      {...props}
    />
  )
);
Input.displayName = "Input";

export { Input };
