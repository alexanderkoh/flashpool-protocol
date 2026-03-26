'use client';

import { useEffect, useState } from 'react';
import { Badge } from "@/components/ui/badge";
import { FlameIcon, AlertCircleIcon } from "lucide-react";
import { PageHeader } from "../layout/page-header";
import { CampaignDetailSkeleton } from "./campaign-detail-skeleton";
import { getCampaign, getUserPosition } from '@/lib/soroban';
import { useWallet } from '@/hooks/use-wallet';

export function CampaignDetail({ id }: { id: string }) {
  const [campaign, setCampaign] = useState<any>(null);
  const [userPos, setUserPos] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const { publicKey } = useWallet();

  useEffect(() => {
    async function fetch() {
      try {
        const data = await getCampaign(parseInt(id));
        setCampaign(data);
        if (publicKey) {
          try {
            const pos = await getUserPosition(parseInt(id), publicKey);
            setUserPos(pos);
          } catch {
            // User has no position in this campaign
          }
        }
      } catch (error) {
        console.error('Failed to fetch campaign:', error);
      } finally {
        setLoading(false);
      }
    }
    fetch();
  }, [id, publicKey]);

  if (loading) {
    return <CampaignDetailSkeleton />;
  }

  if (!campaign) {
    return (
      <div className="text-center py-12">
        <AlertCircleIcon className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
        <h2 className="text-2xl font-bold">Campaign Not Found</h2>
        <p className="text-muted-foreground mt-2">
          The campaign you&apos;re looking for doesn&apos;t exist or has been removed.
        </p>
      </div>
    );
  }

  // Derive display values from contract Campaign struct fields
  const poolAddr: string = campaign.pool_address ?? '';
  const name = poolAddr
    ? `Pool ${poolAddr.slice(0, 6)}…${poolAddr.slice(-4)}`
    : `Campaign #${id}`;
  const creatorAddr: string = campaign.creator ?? '';
  const creatorDisplay = creatorAddr.length > 12
    ? `${creatorAddr.slice(0, 8)}…${creatorAddr.slice(-4)}`
    : creatorAddr || 'Unknown';
  const bonusFlash: bigint = campaign.bonus_flash ?? BigInt(0);
  const boosted = bonusFlash > BigInt(0);
  const description = `Incentivize liquidity for pool ${poolAddr || id} on FlashPool. Provide LP tokens to earn FLASH rewards.`;

  // Determine status from end_ledger (no live ledger info available here, use placeholder)
  const status = campaign.end_ledger ? 'active' : 'unknown';

  return (
    <div>
      <div className="flex flex-col md:flex-row justify-between gap-4 items-start">
        <PageHeader
          title={name}
          description={description}
        />

        <div className="flex flex-wrap gap-3">
          <Badge className="px-3 py-1">{status}</Badge>
          {boosted && (
            <Badge variant="secondary" className="px-3 py-1 flex items-center gap-1">
              <FlameIcon className="h-3 w-3" />
              Boost Active
            </Badge>
          )}
        </div>
      </div>

      <div className="flex flex-wrap gap-x-8 gap-y-3 mt-6 text-sm">
        <div>
          <span className="text-muted-foreground">Pool:</span>{" "}
          <span className="font-medium font-mono">{poolAddr || '—'}</span>
        </div>
        <div>
          <span className="text-muted-foreground">Created by:</span>{" "}
          <span className="font-medium font-mono">{creatorDisplay}</span>
        </div>
        <div>
          <span className="text-muted-foreground">LP Token:</span>{" "}
          <span className="font-medium font-mono">
            {campaign.lp_token
              ? `${String(campaign.lp_token).slice(0, 6)}…${String(campaign.lp_token).slice(-4)}`
              : '—'}
          </span>
        </div>
        <div>
          <span className="text-muted-foreground">End Ledger:</span>{" "}
          <span className="font-medium">{campaign.end_ledger ?? '—'}</span>
        </div>
        <div>
          <span className="text-muted-foreground">Reward FLASH:</span>{" "}
          <span className="font-medium">
            {campaign.reward_flash !== undefined
              ? (Number(campaign.reward_flash) / 1e7).toLocaleString()
              : '—'}
          </span>
        </div>
        {userPos && (
          <div>
            <span className="text-muted-foreground">Your Position:</span>{" "}
            <span className="font-medium">
              {Number(userPos.lp_amount ?? userPos) / 1e7} LP
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
