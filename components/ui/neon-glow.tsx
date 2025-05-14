import React from 'react';
import { cn } from '@/lib/utils';

interface NeonGlowProps {
  color: 'magenta' | 'yellow';
  className?: string;
}

export function NeonGlow({ color, className }: NeonGlowProps) {
  return (
    <div 
      className={cn(
        'w-64 h-64 rounded-full opacity-40 animate-glow-pulse',
        color === 'magenta' ? 'neon-glow-magenta bg-magenta/10' : 'neon-glow-yellow bg-yellow/10',
        className
      )}
    />
  );
}