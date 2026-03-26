'use client';

import { useState, useEffect } from 'react';
import { PageHeader } from "@/components/layout/page-header";
import { GlassPanel } from "@/components/ui/glass-panel";
import { NeonGlow } from "@/components/ui/neon-glow";
import { Button } from "@/components/ui/button";
import { RewardsCounter } from "@/components/rewards/rewards-counter";
import { useToast } from "@/hooks/use-toast";
import { useWallet } from "@/hooks/use-wallet";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RewardsPoolCard } from "@/components/rewards/rewards-pool-card";
import { buildClaim, submitTransaction, getCampaign, getUserPosition } from '@/lib/soroban';

interface UserCampaignInfo {
  campaignId: number;
  poolAddress: string;
  earned: number;
  progress: number;
  completed: boolean;
}

export default function RewardsPage() {
  const { isConnected, publicKey, signTransaction } = useWallet();
  const { toast } = useToast();
  const [isLoading, setIsLoading] = useState(false);
  const [totalRewards, setTotalRewards] = useState(0);
  const [userCampaigns, setUserCampaigns] = useState<UserCampaignInfo[]>([]);
  const [loadingPositions, setLoadingPositions] = useState(false);

  // Fetch user positions across campaigns
  useEffect(() => {
    if (!publicKey) {
      setTotalRewards(0);
      setUserCampaigns([]);
      return;
    }

    async function fetchPositions() {
      setLoadingPositions(true);
      const found: UserCampaignInfo[] = [];
      let total = 0;

      for (let id = 1; id <= 20; id++) {
        try {
          const campaign = await getCampaign(id);
          if (!campaign) continue;

          try {
            const pos = await getUserPosition(id, publicKey!);
            if (pos) {
              const lpAmount = Number(pos.lp_amount ?? pos ?? 0) / 1e7;
              const rewardFlash = Number(campaign.reward_flash ?? 0) / 1e7;
              const totalLp = Number(campaign.total_lp ?? 0) / 1e7;
              const userShare = totalLp > 0 ? lpAmount / totalLp : 0;
              const earned = rewardFlash * userShare;

              const targetLp = Number(campaign.target_lp ?? 0) / 1e7;
              const progress = targetLp > 0 ? Math.min(100, Math.round((totalLp / targetLp) * 100)) : 0;

              const poolAddr = campaign.pool_address ?? '';
              const shortAddr = poolAddr.length > 12
                ? `${poolAddr.slice(0, 6)}...${poolAddr.slice(-4)}`
                : poolAddr || `Campaign #${id}`;

              found.push({
                campaignId: id,
                poolAddress: shortAddr,
                earned,
                progress,
                completed: false, // Would need ledger comparison for real status
              });
              total += earned;
            }
          } catch {
            // User has no position in this campaign — skip
          }
        } catch {
          // Campaign doesn't exist — skip
        }
      }

      setUserCampaigns(found);
      setTotalRewards(total);
      setLoadingPositions(false);
    }

    fetchPositions();
  }, [publicKey]);

  const handleClaim = async (campaignId: number) => {
    if (!publicKey) return;
    setIsLoading(true);
    try {
      const xdr = await buildClaim(campaignId, publicKey);
      const signed = await signTransaction(xdr);
      if (signed) {
        await submitTransaction(signed);
        toast({ title: 'Rewards claimed!' });
      }
    } catch (error: any) {
      toast({ title: 'Claim failed', description: error.message, variant: 'destructive' });
    } finally {
      setIsLoading(false);
    }
  };

  const handleClaimAll = async () => {
    if (!publicKey || userCampaigns.length === 0) return;
    setIsLoading(true);
    try {
      for (const uc of userCampaigns) {
        const xdr = await buildClaim(uc.campaignId, publicKey);
        const signed = await signTransaction(xdr);
        if (signed) {
          await submitTransaction(signed);
        }
      }
      toast({ title: 'All rewards claimed!' });
    } catch (error: any) {
      toast({ title: 'Claim failed', description: error.message, variant: 'destructive' });
    } finally {
      setIsLoading(false);
    }
  };

  const activeCampaigns = userCampaigns.filter(c => !c.completed);
  const completedCampaigns = userCampaigns.filter(c => c.completed);

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

              {!isConnected ? (
                <div className="text-center py-8">
                  <p className="text-muted-foreground">Connect your wallet to view rewards.</p>
                </div>
              ) : loadingPositions ? (
                <div className="text-center py-8">
                  <p className="text-muted-foreground">Loading your positions...</p>
                </div>
              ) : (
                <div className="mb-8">
                  <RewardsCounter value={totalRewards} />

                  <div className="mt-6">
                    <Button
                      size="lg"
                      className="w-full"
                      onClick={handleClaimAll}
                      disabled={isLoading || !isConnected || totalRewards <= 0}
                    >
                      {isLoading ? "Processing..." : "Claim All Rewards"}
                    </Button>
                  </div>
                </div>
              )}
            </GlassPanel>
          </div>

          <div>
            <Tabs defaultValue="active">
              <TabsList className="grid grid-cols-2 mb-4">
                <TabsTrigger value="active">Active</TabsTrigger>
                <TabsTrigger value="completed">Completed</TabsTrigger>
              </TabsList>

              <TabsContent value="active" className="space-y-4">
                {activeCampaigns.length > 0 ? (
                  activeCampaigns.map((uc) => (
                    <RewardsPoolCard
                      key={uc.campaignId}
                      poolName={uc.poolAddress}
                      earned={uc.earned}
                      progress={uc.progress}
                      daysLeft={0}
                    />
                  ))
                ) : (
                  <GlassPanel className="p-4 border border-white/10">
                    <p className="text-sm text-muted-foreground text-center">
                      {isConnected ? 'No active positions found.' : 'Connect wallet to view.'}
                    </p>
                  </GlassPanel>
                )}
              </TabsContent>

              <TabsContent value="completed" className="space-y-4">
                {completedCampaigns.length > 0 ? (
                  completedCampaigns.map((uc) => (
                    <RewardsPoolCard
                      key={uc.campaignId}
                      poolName={uc.poolAddress}
                      earned={uc.earned}
                      progress={100}
                      daysLeft={0}
                      completed
                    />
                  ))
                ) : (
                  <GlassPanel className="p-4 border border-white/10">
                    <p className="text-sm text-muted-foreground text-center">
                      No completed campaigns yet.
                    </p>
                  </GlassPanel>
                )}
              </TabsContent>
            </Tabs>
          </div>
        </div>
      </div>
    </div>
  );
}
