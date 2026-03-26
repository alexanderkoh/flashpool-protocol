'use client';

import { useEffect, useState } from 'react';
import { GlassPanel } from '@/components/ui/glass-panel';
import { cn } from '@/lib/utils';
import { getCampaign } from '@/lib/soroban';

interface TimelineItem {
  title: string;
  description: string;
  completed: boolean;
  current?: boolean;
}

export function CampaignTimeline({ id }: { id: string }) {
  const [items, setItems] = useState<TimelineItem[]>([]);

  useEffect(() => {
    async function fetch() {
      try {
        const campaign = await getCampaign(parseInt(id));
        if (!campaign) return;

        const endLedger = campaign.end_ledger ?? 0;
        const totalLp = Number(campaign.total_lp ?? 0);
        const targetLp = Number(campaign.target_lp ?? 0);
        const bonusFlash = Number(campaign.bonus_flash ?? 0);
        const filled = targetLp > 0 ? Math.min(100, Math.round((totalLp / targetLp) * 100)) : 0;

        const timeline: TimelineItem[] = [
          {
            title: 'Campaign Created',
            description: `Creator set up the campaign targeting ${(targetLp / 1e7).toLocaleString()} LP.`,
            completed: true,
          },
          {
            title: 'Accepting Deposits',
            description: `${filled}% of target LP filled (${(totalLp / 1e7).toLocaleString()} deposited).`,
            completed: true,
            current: true,
          },
        ];

        if (bonusFlash > 0) {
          timeline.push({
            title: 'Bonus FLASH Active',
            description: `${(bonusFlash / 1e7).toLocaleString()} bonus FLASH allocated.`,
            completed: false,
          });
        }

        timeline.push({
          title: 'Campaign Ends',
          description: endLedger > 0
            ? `Ends at ledger ${endLedger.toLocaleString()}.`
            : 'End condition not yet set.',
          completed: false,
        });

        setItems(timeline);
      } catch (error) {
        console.warn('[CampaignTimeline] Failed to load campaign:', error);
        setItems([
          {
            title: 'Campaign Timeline',
            description: 'Unable to load timeline data.',
            completed: false,
            current: true,
          },
        ]);
      }
    }
    fetch();
  }, [id]);

  if (items.length === 0) {
    return (
      <GlassPanel className="p-6 border border-white/10">
        <h2 className="text-xl font-bold mb-6">Campaign Timeline</h2>
        <p className="text-sm text-muted-foreground">Loading...</p>
      </GlassPanel>
    );
  }

  return (
    <GlassPanel className="p-6 border border-white/10">
      <h2 className="text-xl font-bold mb-6">Campaign Timeline</h2>

      <div className="relative">
        <div className="absolute top-4 bottom-4 left-[19px] w-[2px] bg-border" />

        <div className="space-y-6">
          {items.map((item, index) => (
            <div key={index} className="relative flex gap-4">
              <div className="shrink-0 z-10">
                <div
                  className={cn(
                    'w-10 h-10 rounded-full flex items-center justify-center border-2',
                    item.completed
                      ? 'bg-primary/20 border-primary'
                      : 'bg-muted border-border',
                  )}
                >
                  <span className="text-sm">{index + 1}</span>
                </div>
              </div>

              <div
                className={cn(
                  'relative pb-6',
                  index === items.length - 1 ? 'pb-0' : '',
                )}
              >
                <div className="flex flex-col">
                  <div className="flex flex-wrap items-center gap-2">
                    <h3
                      className={cn(
                        'text-base font-medium',
                        item.current ? 'text-primary' : '',
                      )}
                    >
                      {item.title}
                    </h3>

                    {item.current && (
                      <span className="relative flex h-2 w-2">
                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                        <span className="relative inline-flex rounded-full h-2 w-2 bg-primary"></span>
                      </span>
                    )}
                  </div>

                  <p className="text-sm text-muted-foreground mt-1">
                    {item.description}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </GlassPanel>
  );
}
