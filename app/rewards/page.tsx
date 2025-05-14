'use client';

import { useState } from 'react';
import { PageHeader } from "@/components/layout/page-header";
import { GlassPanel } from "@/components/ui/glass-panel";
import { NeonGlow } from "@/components/ui/neon-glow";
import { Button } from "@/components/ui/button";
import { RewardsCounter } from "@/components/rewards/rewards-counter";
import { RewardsTimeline } from "@/components/rewards/rewards-timeline";
import { useToast } from "@/hooks/use-toast";
import { useWallet } from "@/hooks/use-wallet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RewardsPoolCard } from "@/components/rewards/rewards-pool-card";

export default function RewardsPage() {
  const { connected } = useWallet();
  const { toast } = useToast();
  const [isLoading, setIsLoading] = useState(false);

  const handleClaim = async () => {
    if (!connected) {
      toast({
        title: "Wallet not connected",
        description: "Please connect your Freighter wallet first",
        variant: "destructive",
      });
      return;
    }

    setIsLoading(true);
    // Simulate blockchain transaction
    setTimeout(() => {
      setIsLoading(false);
      toast({
        title: "Rewards Claimed!",
        description: "456.21 FLASH has been transferred to your wallet",
      });
    }, 2000);
  };

  return (
    <div className="container mx-auto max-w-6xl px-4 py-8">
      <PageHeader
        title="Claim Rewards"
        description="View and claim your earned FLASH rewards from liquidity campaigns."
      />
      
      <div className="relative">
        <NeonGlow color="yellow" className="absolute -top-20 -right-20 opacity-10" />
        
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mt-8">
          <div className="lg:col-span-2">
            <GlassPanel className="p-6 md:p-8 border border-white/10 relative overflow-hidden h-full">
              <h2 className="text-2xl font-bold mb-6">Your Rewards</h2>
              
              <div className="mb-8">
                <RewardsCounter value={456.21} />
                
                <div className="mt-6">
                  <Button 
                    size="lg" 
                    className="w-full" 
                    onClick={handleClaim} 
                    disabled={isLoading || !connected}
                  >
                    {isLoading ? "Processing..." : "Claim All Rewards"}
                  </Button>
                </div>
              </div>
              
              <div>
                <h3 className="text-lg font-medium mb-4">Unlock Schedule</h3>
                <RewardsTimeline />
              </div>
            </GlassPanel>
          </div>
          
          <div>
            <Tabs defaultValue="active">
              <TabsList className="grid grid-cols-2 mb-4">
                <TabsTrigger value="active">Active</TabsTrigger>
                <TabsTrigger value="completed">Completed</TabsTrigger>
              </TabsList>
              
              <TabsContent value="active" className="space-y-4">
                <RewardsPoolCard 
                  poolName="USDC/TBT"
                  earned={245.21}
                  progress={65}
                  daysLeft={12}
                />
                <RewardsPoolCard 
                  poolName="ETH/FLASH"
                  earned={211.00}
                  progress={38}
                  daysLeft={19}
                />
              </TabsContent>
              
              <TabsContent value="completed" className="space-y-4">
                <RewardsPoolCard 
                  poolName="BTC/USDC"
                  earned={189.45}
                  progress={100}
                  daysLeft={0}
                  completed
                />
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>
    </div>
  );
}