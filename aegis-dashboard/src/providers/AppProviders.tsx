// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { type ReactNode, useState } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { createQueryClient } from "@/lib/queryClient";

type AppProvidersProps = { children: ReactNode };

/**
 * App root: TanStack Query (one client per mount, avoids a fragile global singleton under HMR).
 */
export function AppProviders({ children }: AppProvidersProps) {
  const [queryClient] = useState(() => createQueryClient());
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}
