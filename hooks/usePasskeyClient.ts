'use client';

import { useState, useEffect, useCallback } from 'react';
import { PasskeyKit } from 'passkey-kit';

// Define the shape of the context or hook return value
interface PasskeyClientControls {
  passkeyKit: PasskeyKit | null;
  isInitializing: boolean;
  error: string | null;
  // Add more methods as needed, e.g., for registration, authentication
  // registerPasskey: () => Promise<void>;
  // authenticatePasskey: () => Promise<void>;
}

export function usePasskeyClient(): PasskeyClientControls {
  const [passkeyKit, setPasskeyKit] = useState<PasskeyKit | null>(null);
  const [isInitializing, setIsInitializing] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    try {
      // Ensure environment variables are defined
      const rpcUrl = process.env.NEXT_PUBLIC_RPC_URL;
      const networkPassphrase = process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE;
      const walletWasmHash = process.env.NEXT_PUBLIC_WALLET_WASM_HASH;

      if (!rpcUrl || !networkPassphrase || !walletWasmHash) {
        throw new Error(
          'Missing required environment variables for PasskeyKit: NEXT_PUBLIC_RPC_URL, NEXT_PUBLIC_NETWORK_PASSPHRASE, or NEXT_PUBLIC_WALLET_WASM_HASH'
        );
      }

      const kit = new PasskeyKit({
        rpcUrl,
        networkPassphrase,
        walletWasmHash,
        // Optionally, you can provide a server_url if your API routes are set up
        // server_url: '/api/passkey', // Example if you host your PasskeyServer routes at /api/passkey
      });
      setPasskeyKit(kit);
    } catch (e: any) {
      console.error('Failed to initialize PasskeyKit:', e);
      setError(e.message || 'Failed to initialize PasskeyKit');
    } finally {
      setIsInitializing(false);
    }
  }, []);

  // Placeholder for registration logic (example)
  // const registerPasskey = useCallback(async () => {
  //   if (!passkeyKit) {
  //     setError('PasskeyKit not initialized');
  //     return;
  //   }
  //   try {
  //     // Example: const result = await passkeyKit.register();
  //     // Handle result
  //   } catch (e: any) {
  //     setError(e.message || 'Registration failed');
  //   }
  // }, [passkeyKit]);

  // Placeholder for authentication logic (example)
  // const authenticatePasskey = useCallback(async () => {
  //   if (!passkeyKit) {
  //     setError('PasskeyKit not initialized');
  //     return;
  //   }
  //   try {
  //     // Example: const result = await passkeyKit.authenticate();
  //     // Handle result
  //   } catch (e: any) {
  //     setError(e.message || 'Authentication failed');
  //   }
  // }, [passkeyKit]);

  return {
    passkeyKit,
    isInitializing,
    error,
    // registerPasskey,
    // authenticatePasskey,
  };
} 