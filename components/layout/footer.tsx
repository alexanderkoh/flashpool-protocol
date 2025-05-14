import Link from 'next/link';
import { BoltIcon, TwitterIcon, GithubIcon, BookOpenIcon } from 'lucide-react';

export function Footer() {
  return (
    <footer className="border-t border-border/40 bg-background/80 backdrop-blur mt-auto">
      <div className="container mx-auto px-4 py-8">
        <div className="flex flex-col md:flex-row justify-between items-center">
          <div className="flex items-center gap-2 mb-4 md:mb-0">
            <Link href="/" className="flex items-center gap-2">
              <BoltIcon className="h-6 w-6 text-primary" />
              <span className="text-lg font-bold tracking-tight">
                Flash<span className="text-primary">Pool</span>
              </span>
            </Link>
            <span className="text-sm text-muted-foreground ml-2">
              © 2025 FlashPool | Powered by Stellar Soroban
            </span>
          </div>
          
          <div className="flex items-center gap-4">
            <Link 
              href="https://twitter.com" 
              target="_blank" 
              rel="noopener noreferrer"
              className="text-muted-foreground hover:text-primary transition-colors"
            >
              <TwitterIcon className="h-5 w-5" />
              <span className="sr-only">Twitter</span>
            </Link>
            <Link 
              href="https://github.com" 
              target="_blank" 
              rel="noopener noreferrer"
              className="text-muted-foreground hover:text-primary transition-colors"
            >
              <GithubIcon className="h-5 w-5" />
              <span className="sr-only">GitHub</span>
            </Link>
            <Link 
              href="/docs" 
              className="text-muted-foreground hover:text-primary transition-colors"
            >
              <BookOpenIcon className="h-5 w-5" />
              <span className="sr-only">Documentation</span>
            </Link>
          </div>
        </div>
        
        <div className="mt-6 text-center md:text-left text-xs text-muted-foreground">
          <p>FlashPool is not a financial advisor. Trading and investing in cryptocurrencies involve substantial risk.</p>
        </div>
      </div>
    </footer>
  );
}