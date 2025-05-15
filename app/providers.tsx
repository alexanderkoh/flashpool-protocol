'use client';

// import { ThemeProvider } from "next-themes";
// import { Toaster } from "@/components/ui/toaster";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <>
      {children}
      {/* <Toaster /> */}
    </>
  );
}