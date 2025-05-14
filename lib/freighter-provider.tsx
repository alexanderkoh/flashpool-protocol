'use client';

import { createContext, useContext, useState, useEffect } from 'react';

type FreighterContextType = {
  connected: boolean;
  publicKey: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  isConnecting: boolean;
};

const FreighterContext = createContext<FreighterContextType>({
  connected: false,
  publicKey: null,
  connect: async () => {},
  disconnect: () => {},
  isConnecting: false,
});

export function FreighterProvider({ children }: { children: React.ReactNode }) {
  const [connected, setConnected] = useState(false);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);

  // Check for existing connection on mount
  useEffect(() => {
    // Browser-only code to avoid SSR issues
    if (typeof window !== 'undefined') {
      const savedPublicKey = localStorage.getItem('flashpool-wallet');
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
        localStorage.setItem('flashpool-wallet', mockPublicKey);
      }
      
      setConnected(true);
      setPublicKey(mockPublicKey);
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnect = () => {
    if (typeof window !== 'undefined') {
      localStorage.removeItem('flashpool-wallet');
    }
    setConnected(false);
    setPublicKey(null);
  };

  return (
    <FreighterContext.Provider 
      value={{ 
        connected, 
        publicKey, 
        connect, 
        disconnect, 
        isConnecting 
      }}
    >
      {children}
    </FreighterContext.Provider>
  );
}

export function useWallet() {
  return useContext(FreighterContext);
}