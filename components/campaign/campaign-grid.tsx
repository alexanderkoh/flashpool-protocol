'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { CampaignCard } from './campaign-card';
import { CampaignGridSkeleton } from './campaign-grid-skeleton';
import { getCampaign } from '@/lib/soroban';

export function CampaignGrid() {
  const [campaigns, setCampaigns] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchCampaigns() {
      try {
        const fetched = [];
        for (let id = 1; id <= 20; id++) {
          try {
            const campaign = await getCampaign(id);
            if (campaign) {
              fetched.push({ id, ...campaign });
            }
          } catch {
            break; // No more campaigns
          }
        }
        setCampaigns(fetched);
      } catch (error) {
        console.error('Failed to fetch campaigns:', error);
      } finally {
        setLoading(false);
      }
    }
    fetchCampaigns();
  }, []);

  if (loading) {
    return <CampaignGridSkeleton />;
  }

  if (campaigns.length === 0) {
    return (
      <div className="text-center py-12 mt-6">
        <p className="text-muted-foreground">No campaigns found.</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-6">
      {campaigns.map((campaign) => {
        // Derive display values from contract Campaign struct fields
        const poolAddr: string = campaign.pool_address ?? '';
        const shortAddr = poolAddr.length > 12
          ? `${poolAddr.slice(0, 6)}…${poolAddr.slice(-4)}`
          : poolAddr || `Campaign #${campaign.id}`;
        const name = `Pool ${shortAddr}`;
        const pair = shortAddr;
        const rewardFlash: bigint = campaign.reward_flash ?? BigInt(0);
        const bonusFlash: bigint = campaign.bonus_flash ?? BigInt(0);
        const totalRewards = Number(rewardFlash + bonusFlash) / 1e7;
        const totalLp: bigint = campaign.total_lp ?? BigInt(0);
        const tvl = Number(totalLp) / 1e7;
        const boosted = bonusFlash > BigInt(0);

        // Estimate duration hours from end_ledger (5s per ledger, ~720 ledgers/hr)
        const endLedger: number = campaign.end_ledger ?? 0;
        const durationHours = Math.max(1, Math.round(endLedger / 720));

        // APY is not directly available — use a placeholder
        const apy = 0;

        return (
          <Link key={campaign.id} href={`/campaign/${campaign.id}`}>
            <CampaignCard
              name={name}
              pair={pair}
              durationHours={durationHours}
              rewardToken="FLASH"
              rewards={totalRewards}
              tvl={tvl}
              boosted={boosted}
              apy={apy}
            />
          </Link>
        );
      })}
    </div>
  );
}
