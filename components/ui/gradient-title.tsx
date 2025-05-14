import { cn } from "@/lib/utils";

interface GradientTitleProps {
  children: React.ReactNode;
  className?: string;
}

export function GradientTitle({ children, className }: GradientTitleProps) {
  return (
    <h1 className={cn("gradient-text font-bold", className)}>
      {children}
    </h1>
  );
}