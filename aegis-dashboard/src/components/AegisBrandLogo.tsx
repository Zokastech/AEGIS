// AEGIS — zokastech.fr — Apache 2.0 / MIT — gradient shield aligned with landing / Zokastech docs

import * as React from "react";
import { cn } from "@/lib/utils";

const sizeClass = {
  sm: "h-9 w-9",
  md: "h-11 w-11",
  lg: "h-16 w-16",
  xl: "h-[4.5rem] w-[4.5rem]",
} as const;

/** Shield icon (same geometry as the React landing). */
export function AegisBrandMark({
  className,
  size = "md",
  glow = false,
  "aria-hidden": ariaHidden = true,
}: {
  className?: string;
  size?: keyof typeof sizeClass;
  glow?: boolean;
  "aria-hidden"?: boolean;
}) {
  const uid = React.useId().replaceAll(":", "");
  const g1 = `aegis-sg1-${uid}`;
  const g2 = `aegis-sg2-${uid}`;

  return (
    <svg
      className={cn(sizeClass[size], glow && "aegis-shield-glow", className)}
      viewBox="0 0 64 72"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden={ariaHidden}
    >
      <defs>
        <linearGradient id={g1} x1="8" y1="4" x2="56" y2="68" gradientUnits="userSpaceOnUse">
          <stop stopColor="#ff8a00" />
          <stop offset="0.5" stopColor="#e52e71" />
          <stop offset="1" stopColor="#4361ee" />
        </linearGradient>
        <linearGradient id={g2} x1="20" y1="20" x2="48" y2="52" gradientUnits="userSpaceOnUse">
          <stop stopColor="#ff8a00" stopOpacity="0.2" />
          <stop offset="1" stopColor="#4361ee" stopOpacity="0" />
        </linearGradient>
      </defs>
      <path
        d="M32 4L8 16v20c0 14 10 26 24 32 14-6 24-18 24-32V16L32 4z"
        fill={`url(#${g1})`}
        stroke="#e52e71"
        strokeWidth="1.2"
        strokeOpacity="0.45"
      />
      <path
        d="M32 14L18 22v12c0 8 6 15 14 18 8-3 14-10 14-18V22L32 14z"
        fill={`url(#${g2})`}
      />
      <path d="M28 34h8v-8h-8v8zm0 4h8v8h-8v-8z" fill="#ffffff" fillOpacity="0.35" />
    </svg>
  );
}

/** "AEGIS" wordmark in signature gradient. */
export function AegisWordmark({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "font-display font-extrabold tracking-[0.14em] bg-gradient-to-r from-brand-orange via-brand-pink to-brand-blue bg-clip-text text-transparent",
        className
      )}
    >
      AEGIS
    </span>
  );
}
