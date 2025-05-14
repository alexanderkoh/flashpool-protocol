import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatCurrency(value: number, usd = false): string {
  let formattedValue;
  
  if (value >= 1000000) {
    formattedValue = (value / 1000000).toFixed(1) + 'M';
  } else if (value >= 1000) {
    formattedValue = (value / 1000).toFixed(1) + 'K';
  } else {
    formattedValue = value.toLocaleString(undefined, {
      maximumFractionDigits: 2
    });
  }
  
  return usd ? '$' + formattedValue : formattedValue;
}