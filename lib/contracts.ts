// Contract addresses — update after deployment
export const CONTRACTS = {
  campaignManager: process.env.NEXT_PUBLIC_CAMPAIGN_MANAGER_ID || '',
  treasury: process.env.NEXT_PUBLIC_TREASURY_ID || '',
  tinyamm: process.env.NEXT_PUBLIC_TINYAMM_ID || '',
  flash: process.env.NEXT_PUBLIC_FLASH_TOKEN_ID || '',
  usdc: process.env.NEXT_PUBLIC_USDC_TOKEN_ID || '',
} as const;

export const SOROBAN_RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org';

export const NETWORK_PASSPHRASE =
  process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE || 'Test SDF Network ; September 2015';
