import { GlassPanel } from '@/components/ui/glass-panel';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { formatCurrency } from '@/lib/utils';
import { FlameIcon, ClockIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface CampaignCardProps {
  name: string;
  pair: string;
  platform?: string;
  durationHours: number;
  rewardToken: string;
  rewards: number;
  tvl: number;
  boosted: boolean;
  boostStartHour?: number;
  boostDurationHours?: number;
  apy: number;
}

export function CampaignCard({
  name,
  pair,
  platform,
  durationHours,
  rewardToken,
  rewards,
  tvl,
  boosted,
  apy,
}: CampaignCardProps) {

  const MOCK_HOURS_PASSED = Math.floor(Math.random() * durationHours / 2);
  const hoursLeft = durationHours - MOCK_HOURS_PASSED;
  const progressValue = (MOCK_HOURS_PASSED / durationHours) * 100;

  const displayTimeLeft = () => {
    if (hoursLeft <= 0) return "Ended";
    if (hoursLeft < 24) return `${hoursLeft} hour${hoursLeft > 1 ? 's' : ''} left`;
    const days = Math.floor(hoursLeft / 24);
    const remainingHours = hoursLeft % 24;
    if (remainingHours === 0) return `${days} day${days > 1 ? 's' : ''} left`;
    return `${days}d ${remainingHours}h left`;
  };

  return (
    <GlassPanel className="p-6 border border-white/10 h-full transition-all duration-300 hover:border-primary/40 hover:shadow-[0_0_15px_rgba(199,29,151,0.15)] flex flex-col justify-between">
      <div>
        <div className="flex justify-between items-start mb-1">
          <h3 className="text-lg font-bold">{name}</h3>
          {boosted && (
            <Badge variant="secondary" className="flex items-center gap-1">
              <FlameIcon className="h-3 w-3" />
              Boosted
            </Badge>
          )}
        </div>
        <p className="text-sm text-muted-foreground mb-1">{pair} Pair</p>
        {platform && <p className="text-xs text-muted-foreground mb-6">Platform: {platform}</p>}
        
        <div className="grid grid-cols-2 gap-4 mb-6">
          <div>
            <p className="text-xs text-muted-foreground">Contribution Pool</p>
            <p className="text-lg font-bold">{formatCurrency(rewards)} {rewardToken}</p>
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Current TVL</p>
            <p className="text-lg font-bold">{formatCurrency(tvl, true)}</p>
          </div>
        </div>
      </div>
      
      <div>
        <div className="mb-4">
          <div className="flex justify-between text-xs mb-1 items-center">
            <span className="text-muted-foreground flex items-center"><ClockIcon className="h-3 w-3 mr-1" />Time Remaining</span>
            <span>{displayTimeLeft()}</span>
          </div>
          <Progress value={progressValue} className="h-1" />
        </div>
        
        <div className="flex justify-between items-center">
          <div className="text-xs text-muted-foreground">
            <span className="font-mono text-white text-sm">{apy.toFixed(1)}% APY</span>
          </div>
          <Button size="sm">
            Join Pool →
          </Button>
        </div>
      </div>
    </GlassPanel>
  );
}