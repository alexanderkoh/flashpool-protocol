'use client';

import { useState } from 'react';
import { GlassPanel } from '@/components/ui/glass-panel';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Slider } from '@/components/ui/slider';
import { useWallet } from '@/hooks/use-wallet';
import { useToast } from '@/hooks/use-toast';
import { formatCurrency } from '@/lib/utils';
import { WalletIcon, ArrowRightIcon, LockIcon, AlertCircleIcon } from 'lucide-react';

export function DepositPanel({ id }: { id: string }) {
  const { connected, connect } = useWallet();
  const { toast } = useToast();
  const [amount, setAmount] = useState<number>(500);
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  
  // Mock liquidity data
  const maxLiquidity = 5000;
  const expectedRewards = amount * 0.15; // 15% of deposit amount
  
  const handleMaxClick = () => {
    setAmount(maxLiquidity);
  };
  
  const handleSliderChange = (value: number[]) => {
    setAmount(value[0]);
  };
  
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (isNaN(value)) {
      setAmount(0);
    } else if (value > maxLiquidity) {
      setAmount(maxLiquidity);
    } else {
      setAmount(value);
    }
  };
  
  const handleDeposit = async () => {
    if (!connected) {
      toast({
        title: "Wallet not connected",
        description: "Please connect your Freighter wallet first",
        variant: "destructive",
      });
      return;
    }
    
    if (amount <= 0) {
      toast({
        title: "Invalid amount",
        description: "Please enter an amount greater than 0",
        variant: "destructive",
      });
      return;
    }
    
    setIsSubmitting(true);
    
    // Simulate blockchain transaction
    setTimeout(() => {
      setIsSubmitting(false);
      toast({
        title: "Deposit Successful!",
        description: `You have deposited ${formatCurrency(amount, true)} to the pool.`,
      });
    }, 2000);
  };
  
  return (
    <GlassPanel className="p-6 border border-white/10 sticky top-20">
      <h2 className="text-xl font-bold mb-4">Deposit Liquidity</h2>
      
      {connected ? (
        <>
          <div className="space-y-4 mt-6">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span>Amount (USDC)</span>
                <button 
                  className="text-primary text-xs"
                  onClick={handleMaxClick}
                >
                  MAX
                </button>
              </div>
              
              <div className="relative">
                <Input
                  type="number"
                  value={amount}
                  onChange={handleInputChange}
                  className="pr-16"
                />
                <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                  <span className="text-muted-foreground">USDC</span>
                </div>
              </div>
            </div>
            
            <Slider
              value={[amount]}
              max={maxLiquidity}
              step={100}
              onValueChange={handleSliderChange}
            />
            
            <div className="bg-muted/30 rounded-lg p-4 space-y-3">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">APY</span>
                <span>28.4%</span>
              </div>
              
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Est. FLASH Rewards</span>
                <span>{formatCurrency(expectedRewards)} FLASH</span>
              </div>
              
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Boost Multiplier</span>
                <span>2x (May 19-24)</span>
              </div>
            </div>
            
            <div className="flex items-center gap-2 bg-secondary/5 border border-secondary/20 rounded-lg p-3 text-xs">
              <LockIcon className="h-4 w-4 text-secondary shrink-0" />
              <p>
                Your liquidity will be deposited into the Hoops USDC/XLM pool and can be withdrawn at any time.
              </p>
            </div>
            
            <Button 
              className="w-full" 
              onClick={handleDeposit}
              disabled={isSubmitting || amount <= 0}
            >
              {isSubmitting ? (
                "Processing..."
              ) : (
                <>
                  Deposit Liquidity
                  <ArrowRightIcon className="ml-2 h-4 w-4" />
                </>
              )}
            </Button>
          </div>
        </>
      ) : (
        <div className="space-y-4 mt-6">
          <div className="flex items-center gap-2 bg-muted/30 rounded-lg p-4 text-sm">
            <AlertCircleIcon className="h-5 w-5 text-muted-foreground shrink-0" />
            <p>
              Connect your Freighter wallet to deposit liquidity and start earning FLASH rewards.
            </p>
          </div>
          
          <Button onClick={connect} className="w-full">
            <WalletIcon className="mr-2 h-4 w-4" />
            Connect Wallet
          </Button>
        </div>
      )}
    </GlassPanel>
  );
}