// AEGIS — zokastech.fr — Apache 2.0 / MIT

import * as React from "react";
import { cn } from "@/lib/utils";

const Textarea = React.forwardRef<HTMLTextAreaElement, React.TextareaHTMLAttributes<HTMLTextAreaElement>>(
  ({ className, ...props }, ref) => (
    <textarea
      className={cn(
        "flex min-h-[120px] w-full rounded-md border border-[#e2e8f0] bg-white px-3 py-2 text-sm text-zokastech-dark shadow-sm transition-[border-color,box-shadow] duration-200 ease-smooth placeholder:text-[#94a3b8] focus-visible:outline-none focus-visible:border-brand-orange focus-visible:ring-2 focus-visible:ring-[rgba(255,138,0,0.2)] disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      ref={ref}
      {...props}
    />
  )
);
Textarea.displayName = "Textarea";

export { Textarea };
