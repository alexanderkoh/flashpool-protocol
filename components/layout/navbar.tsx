'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { menuItems } from '@/lib/constants';
import { BoltIcon, MenuIcon, XIcon } from 'lucide-react';
import { cn } from '@/lib/utils';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useWallet } from '@/hooks/use-wallet';

export function Navbar() {
  const pathname = usePathname();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  
  // Updated destructuring to use isConnected from the new useWallet hook
  const { isConnected: connected, publicKey, connect, disconnect, isConnecting } = useWallet();

  const toggleMenu = () => {
    setMobileMenuOpen(!mobileMenuOpen);
  };

  return (
    <header className="sticky top-0 z-50 w-full border-b border-border/40 bg-background/80 backdrop-blur">
      <div className="container mx-auto flex h-16 items-center justify-between px-4">
        <div className="flex items-center gap-2">
          <Link href="/" className="flex items-center gap-2">
            <BoltIcon className="h-8 w-8 text-primary" />
            <span className="text-xl font-bold tracking-tight">
              Flash<span className="text-primary">Pool</span>
            </span>
          </Link>
        </div>

        {/* Desktop Navigation */}
        <nav className="hidden md:flex items-center gap-6">
          {menuItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "text-sm font-medium transition-colors hover:text-primary",
                pathname === item.href
                  ? "text-foreground"
                  : "text-muted-foreground"
              )}
            >
              {item.name}
            </Link>
          ))}
        </nav>

        <div className="flex items-center gap-4">
          {connected ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" className="font-mono text-xs">
                  {publicKey && `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}`}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem asChild>
                  <Link href="/rewards">My Rewards</Link>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={disconnect}>
                  Disconnect
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          ) : (
            <Button onClick={connect} disabled={isConnecting}>
              {isConnecting ? "Connecting..." : "Connect Wallet"}
            </Button>
          )}

          {/* Mobile Menu Button */}
          <Button
            variant="ghost"
            size="icon"
            className="md:hidden"
            onClick={toggleMenu}
          >
            {mobileMenuOpen ? (
              <XIcon className="h-6 w-6" />
            ) : (
              <MenuIcon className="h-6 w-6" />
            )}
          </Button>
        </div>
      </div>

      {/* Mobile Navigation */}
      {mobileMenuOpen && (
        <div className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm md:hidden" onClick={toggleMenu}>
          <div 
            className="fixed inset-y-0 right-0 w-full max-w-xs bg-background p-6 shadow-lg z-50 overflow-y-auto" 
            onClick={e => e.stopPropagation()}
          >
            <div className="flex items-center justify-between mb-8">
              <Link href="/" className="flex items-center gap-2" onClick={toggleMenu}>
                <BoltIcon className="h-6 w-6 text-primary" />
                <span className="text-lg font-bold tracking-tight">
                  Flash<span className="text-primary">Pool</span>
                </span>
              </Link>
              <Button variant="ghost" size="icon" onClick={toggleMenu}>
                <XIcon className="h-5 w-5" />
              </Button>
            </div>
            
            <nav className="flex flex-col gap-4">
              {menuItems.map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className={cn(
                    "text-sm font-medium py-2 transition-colors hover:text-primary",
                    "text-white",
                    pathname === item.href
                      ? "font-semibold"
                      : "opacity-80 hover:opacity-100",
                  )}
                  onClick={toggleMenu}
                >
                  {item.name}
                </Link>
              ))}
            </nav>
          </div>
        </div>
      )}
    </header>
  );
}