'use client';

// Re-export the useStellarWallet hook from the StellarWalletsKitProvider
// This ensures a single source of truth for wallet functionality using the new kit.
export { useStellarWallet as useWallet } from '@/lib/stellar-wallets-kit-provider'; 