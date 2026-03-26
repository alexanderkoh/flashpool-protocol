'use client';

import { useEffect, useState } from 'react';
import { GlassPanel } from '@/components/ui/glass-panel';
import { formatCurrency } from '@/lib/utils';
import { getCampaign } from '@/lib/soroban';

export function CampaignStats({ id }: { id: string }) {
  const [campaign, setCampaign] = useState<any>(null);

  useEffect(() => {
    async function fetch() {
      try {
        const data = await getCampaign(parseInt(id));
        setCampaign(data);
      } catch (error) {
        console.warn('[CampaignStats] Failed to load campaign data:', error);
      }
    }
    fetch();
  }, [id]);

  const totalLp = campaign ? Number(campaign.total_lp ?? 0) / 1e7 : 0;
  const rewardFlash = campaign ? Number(campaign.reward_flash ?? 0) / 1e7 : 0;
  const bonusFlash = campaign ? Number(campaign.bonus_flash ?? 0) / 1e7 : 0;
  const endLedger = campaign?.end_ledger ?? 0;

  return (
    <GlassPanel className="p-6 border border-white/10">
      <h2 className="text-xl font-bold mb-4">Campaign Stats</h2>

      <div className="grid grid-cols-2 gap-6">
        <div>
          <p className="text-xs text-muted-foreground">Total LP Deposited</p>
          <p className="text-lg font-medium font-mono">
            {campaign ? formatCurrency(totalLp) : '--'}
          </p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">Reward FLASH</p>
          <p className="text-lg font-medium font-mono">
            {campaign ? formatCurrency(rewardFlash) : '--'}
          </p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">Bonus FLASH</p>
          <p className="text-lg font-medium font-mono">
            {campaign ? formatCurrency(bonusFlash) : '--'}
          </p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">End Ledger</p>
          <p className="text-lg font-medium font-mono">
            {endLedger > 0 ? endLedger.toLocaleString() : '--'}
          </p>
        </div>
      </div>

      <div className="mt-6 rounded-lg bg-muted/20 p-4 text-center">
        <p className="text-sm text-muted-foreground">
          Historical charts (TVL, Volume, APY) will be available once indexer data is connected.
        </p>
      </div>
    </GlassPanel>
  );
}
