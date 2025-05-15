'use client';

import { createContext, useContext, useState, useEffect } from 'react';

// Renamed to avoid conflict and mark as deprecated
type FreighterContext_DEPRECATED_Type = {
  connected: boolean;
  publicKey: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  isConnecting: boolean;
};

// Renamed to avoid conflict
const FreighterContext_DEPRECATED = createContext<FreighterContext_DEPRECATED_Type>({
  connected: false,
  publicKey: null,
  connect: async () => {},
  disconnect: () => {},
  isConnecting: false,
});

// Renamed to avoid conflict and mark as deprecated
export function FreighterProvider_DEPRECATED({ children }: { children: React.ReactNode }) {
  const [connected, setConnected] = useState(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);

  // Check for existing connection on mount
  useEffect(() => {
    // Browser-only code to avoid SSR issues
    if (typeof window !== 'undefined') {
      const savedPublicKey = localStorage.getItem('flashpool-wallet-deprecated'); // Changed LS key
      if (savedPublicKey) {
        setConnected(true);
        setPublicKey(savedPublicKey);
      }
    }
  }, []);

  // Mock implementation for Freighter wallet
  const connect = async () => {
    setIsConnecting(true);
    
    try {
      // In a real implementation, this would use the Freighter wallet SDK
      // For now, we'll mock the connection
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const mockPublicKey = 'GDRA72QGAFDVSRXGT3WFIEK2HZWPJ72ENTVLXCDWKFG6OBKEWKNJOEAF';
      
      // Save to localStorage for persistence
      if (typeof window !== 'undefined') {
        localStorage.setItem('flashpool-wallet-deprecated', mockPublicKey); // Changed LS key
      }
      
      setConnected(true);
      setPublicKey(mockPublicKey);
    } catch (error) {
      console.error('Failed to connect wallet (deprecated):', error);
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnect = () => {
    if (typeof window !== 'undefined') {
      localStorage.removeItem('flashpool-wallet-deprecated'); // Changed LS key
    }
    setConnected(false);
    setPublicKey(null);
  };

  return (
    <FreighterContext_DEPRECATED.Provider 
      value={{ 
        connected, 
        publicKey, 
        connect, 
        disconnect, 
        isConnecting 
      }}
    >
      {children}
    </FreighterContext_DEPRECATED.Provider>
  );
}

// Renamed to avoid conflict and mark as deprecated
export function useFreighterWallet_DEPRECATED() {
  return useContext(FreighterContext_DEPRECATED);
}