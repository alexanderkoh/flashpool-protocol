'use client';

import { ThemeProvider } from "next-themes";
import { Toaster } from "@/components/ui/toaster";
import { FreighterProvider } from "@/lib/freighter-provider";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false}>
      <FreighterProvider>
        {children}
        <Toaster />
      </FreighterProvider>
    </ThemeProvider>
  );
}