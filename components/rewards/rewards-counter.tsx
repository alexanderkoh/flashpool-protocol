'use client';

import { useEffect, useRef } from 'react';
import { cn } from '@/lib/utils';

interface RewardsCounterProps {
  value: number;
  className?: string;
}

export function RewardsCounter({ value, className }: RewardsCounterProps) {
  const counterRef = useRef<HTMLSpanElement>(null);
  const previousValue = useRef<number>(0);
  
  useEffect(() => {
    if (!counterRef.current) return;
    
    const start = previousValue.current;
    const end = value;
    const duration = 1000;
    const frameDuration = 1000 / 60;
    const totalFrames = Math.round(duration / frameDuration);
    
    let frame = 0;
    const counter = setInterval(() => {
      frame++;
      
      const progress = frame / totalFrames;
      const currentValue = Math.floor(start + (end - start) * progress);
      
      if (counterRef.current) {
        counterRef.current.textContent = currentValue.toLocaleString(undefined, {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2
        });
      }
      
      if (frame === totalFrames) {
        clearInterval(counter);
        previousValue.current = value;
      }
    }, frameDuration);
    
    return () => clearInterval(counter);
  }, [value]);
  
  return (
    <div className={cn("text-center", className)}>
      <p className="text-sm text-muted-foreground mb-2">Available to Claim</p>
      <div className="font-mono text-4xl md:text-5xl font-bold mb-2 gradient-text">
        <span ref={counterRef}>
          {value.toLocaleString(undefined, {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2
          })}
        </span>
        <span className="ml-2">FLASH</span>
      </div>
      <p className="text-sm text-muted-foreground">Ready to be transferred to your wallet</p>
    </div>
  );
}