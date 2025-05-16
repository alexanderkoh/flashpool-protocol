'use client';

import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import {
  StellarWalletsKit,
  WalletNetwork,
  allowAllModules,
  ISupportedWallet,
  XBULL_ID,
  // Potentially import specific wallet IDs if needed for a default, e.g., FREIGHTER_ID
} from '@creit.tech/stellar-wallets-kit';
import { Networks } from '@stellar/stellar-sdk';

// --- Configuration ---
const APP_NETWORK = process.env.network ?? WalletNetwork.TESTNET; // Or WalletNetwork.PUBLIC
const STELLAR_NETWORK_PASSPHRASE = APP_NETWORK === WalletNetwork.PUBLIC ? Networks.PUBLIC : Networks.TESTNET;
const APP_NAME = 'FlashPool'; // As per your README

interface StellarWalletContextType {
  publicKey: string | null;
  isConnected: boolean;
  isConnecting: boolean; // True when modal is open or an async wallet action is in progress
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  signTransaction: (xdr: string) => Promise<string | null>; // Returns signed XDR or null
}

const StellarWalletContext = createContext<StellarWalletContextType | undefined>(undefined);

export function StellarWalletsKitProvider({ children }: { children: ReactNode }) {
  const [kit, setKit] = useState<StellarWalletsKit | null>(null);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);

  useEffect(() => {
    const storedWalletId = localStorage.getItem('flashpool-selected-wallet-id');
    const isValidStoredWalletId = storedWalletId && storedWalletId !== 'undefined' && storedWalletId !== 'null' && storedWalletId !== '';

    const initialSelectedWalletId = isValidStoredWalletId ? storedWalletId : XBULL_ID;
    
    const newKit = new StellarWalletsKit({
      network: APP_NETWORK === 'PUBLIC' ? WalletNetwork.PUBLIC : 
           APP_NETWORK === 'TESTNET' ? WalletNetwork.TESTNET :
           APP_NETWORK === 'FUTURENET' ? WalletNetwork.FUTURENET :
           APP_NETWORK === 'SANDBOX' ? WalletNetwork.SANDBOX :
           APP_NETWORK === 'STANDALONE' ? WalletNetwork.STANDALONE :
           WalletNetwork.TESTNET,
      selectedWalletId: initialSelectedWalletId,
      modules: allowAllModules(),
    });
    setKit(newKit);

    if (isValidStoredWalletId && storedWalletId) {
      (async () => {
        try {
          const { address } = await newKit.getAddress();
          if (address) {
            setPublicKey(address);
            localStorage.setItem('flashpool-stellar-pk', address);
            console.log('Restored session for wallet:', storedWalletId, 'Address:', address);
          } else {
            localStorage.removeItem('flashpool-selected-wallet-id');
            localStorage.removeItem('flashpool-stellar-pk');
            setPublicKey(null);
          }
        } catch (error: any) {
          console.warn(`Could not auto-connect to previously selected wallet (${storedWalletId}):`, error.message);
          localStorage.removeItem('flashpool-selected-wallet-id');
          localStorage.removeItem('flashpool-stellar-pk');
          setPublicKey(null);
        }
      })();
    } else {
      localStorage.removeItem('flashpool-stellar-pk');
      setPublicKey(null);
    }
  }, []);

  const connect = useCallback(async () => {
    if (!kit) return;
    setIsConnecting(true);
    try {
      await kit.openModal({
        onWalletSelected: async (option: ISupportedWallet) => {
          setIsConnecting(true);
          localStorage.setItem('flashpool-selected-wallet-id', option.id);
          try {
            kit.setWallet(option.id);
            const { address } = await kit.getAddress();
            setPublicKey(address);
            localStorage.setItem('flashpool-stellar-pk', address);
            console.log(`Wallet connected: ${option.name} - ${address}`);
          } catch (e: any) {
            console.error('Error setting wallet or getting address:', e.message);
            setPublicKey(null);
            localStorage.removeItem('flashpool-stellar-pk');
            localStorage.removeItem('flashpool-selected-wallet-id');
          } finally {
            setIsConnecting(false);
          }
        },
        onClosed: () => {
          if (!publicKey) {
            setIsConnecting(false);
          }
          console.log('Modal closed by user');
        },
      });
    } catch (error: any) {
      console.error('Error opening wallet modal:', error.message);
      setIsConnecting(false);
    }
  }, [kit, publicKey]);

  const disconnect = useCallback(async () => {
    if (!kit || !publicKey) return;
    setIsConnecting(true);
    try {
      await kit.disconnect();
      setPublicKey(null);
      localStorage.removeItem('flashpool-stellar-pk');
      localStorage.removeItem('flashpool-selected-wallet-id');
      console.log('Wallet disconnected');
    } catch (error: any) {
      console.error('Error disconnecting wallet:', error.message);
    } finally {
      setIsConnecting(false);
    }
  }, [kit, publicKey]);

  const signTransaction = useCallback(async (xdr: string): Promise<string | null> => {
    if (!kit || !publicKey) {
      alert('Please connect your wallet first.');
      return null;
    }
    setIsConnecting(true);
    try {
      const result = await kit.signTransaction(xdr, {
        networkPassphrase: STELLAR_NETWORK_PASSPHRASE,
        address: publicKey,
      });
      console.log('Transaction signed:', result.signedTxXdr);
      return result.signedTxXdr;
    } catch (error: any) {
      console.error('Error signing transaction:', error.message);
      alert(`Failed to sign transaction: ${error.message || 'Unknown error'}`);
      return null;
    } finally {
      setIsConnecting(false);
    }
  }, [kit, publicKey]);

  return (
    <StellarWalletContext.Provider value={{
      publicKey,
      isConnected: !!publicKey,
      isConnecting,
      connect,
      disconnect,
      signTransaction
    }}>
      {children}
    </StellarWalletContext.Provider>
  );
}

export function useStellarWallet() {
  const context = useContext(StellarWalletContext);
  if (context === undefined) {
    throw new Error('useStellarWallet must be used within a StellarWalletsKitProvider');
  }
  return context;
}
 