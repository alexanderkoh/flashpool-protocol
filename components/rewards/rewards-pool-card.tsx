import { GlassPanel } from '@/components/ui/glass-panel';
import { Progress } from '@/components/ui/progress';
import { Button } from '@/components/ui/button';
import { formatCurrency } from '@/lib/utils';
import { CheckIcon } from 'lucide-react';

interface RewardsPoolCardProps {
  poolName: string;
  earned: number;
  progress: number;
  daysLeft: number;
  completed?: boolean;
}

export function RewardsPoolCard({
  poolName,
  earned,
  progress,
  daysLeft,
  completed = false,
}: RewardsPoolCardProps) {
  return (
    <GlassPanel className={`p-4 border ${
      completed 
        ? 'border-white/10' 
        : 'border-primary/20'
    }`}>
      <div className="flex justify-between items-start mb-2">
        <h3 className="font-medium">{poolName}</h3>
        <span className={`text-xs px-2 py-0.5 rounded-full ${
          completed 
            ? 'bg-muted text-muted-foreground' 
            : 'bg-primary/20 text-primary'
        }`}>
          {completed ? 'Completed' : 'Active'}
        </span>
      </div>
      
      <div className="mt-4">
        <div className="flex justify-between text-sm mb-1">
          <span className="text-muted-foreground">Campaign Progress</span>
          <span>{progress}%</span>
        </div>
        <Progress value={progress} className="h-1.5" />
      </div>
      
      <div className="grid grid-cols-2 gap-2 mt-4">
        <div>
          <p className="text-xs text-muted-foreground">Earned</p>
          <p className="font-mono font-medium">{formatCurrency(earned)} FLASH</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">
            {completed ? 'Duration' : 'Remaining'}
          </p>
          <p className="font-medium">
            {completed ? 'Ended' : daysLeft > 0 ? `${daysLeft} days` : 'Less than a day'}
          </p>
        </div>
      </div>
      
      <div className="mt-4">
        {completed ? (
          <Button variant="outline" className="w-full" disabled>
            <CheckIcon className="h-4 w-4 mr-2" />
            Rewards Claimed
          </Button>
        ) : (
          <Button variant="outline" className="w-full">
            View Details
          </Button>
        )}
      </div>
    </GlassPanel>
  );
}