import { GlassPanel } from '@/components/ui/glass-panel';
import { cn } from '@/lib/utils';
import { FlameIcon } from 'lucide-react';

// Mock data
const timelineData = [
  {
    date: 'May 01, 2025',
    title: 'Campaign Started',
    description: 'The USDC/XLM pool campaign was launched with 120K FLASH rewards',
    completed: true,
  },
  {
    date: 'May 09, 2025',
    title: 'Current Date',
    description: 'Campaign is ongoing with 18 days remaining',
    completed: true,
    current: true,
  },
  {
    date: 'May 19, 2025',
    title: 'Boost Window Starts',
    description: '5-day boost period with 2x rewards begins',
    completed: false,
    highlight: true,
  },
  {
    date: 'May 24, 2025',
    title: 'Boost Window Ends',
    description: 'End of 5-day boost period',
    completed: false,
  },
  {
    date: 'May 27, 2025',
    title: 'Campaign Ends',
    description: 'The campaign concludes. All rewards are finalized.',
    completed: false,
  },
];

export function CampaignTimeline({ id }: { id: string }) {
  return (
    <GlassPanel className="p-6 border border-white/10">
      <h2 className="text-xl font-bold mb-6">Campaign Timeline</h2>
      
      <div className="relative">
        <div className="absolute top-4 bottom-4 left-[19px] w-[2px] bg-border" />
        
        <div className="space-y-6">
          {timelineData.map((item, index) => (
            <div key={index} className="relative flex gap-4">
              <div className="shrink-0 z-10">
                <div
                  className={cn(
                    "w-10 h-10 rounded-full flex items-center justify-center border-2",
                    item.completed
                      ? "bg-primary/20 border-primary"
                      : item.current
                      ? "bg-primary/10 border-primary"
                      : "bg-muted border-border"
                  )}
                >
                  {item.highlight ? (
                    <FlameIcon className="h-5 w-5 text-secondary" />
                  ) : (
                    <span className="text-sm">
                      {index + 1}
                    </span>
                  )}
                </div>
              </div>
              
              <div className={cn(
                "relative pb-6",
                index === timelineData.length - 1 ? "pb-0" : ""
              )}>
                <div className="flex flex-col">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className={cn(
                      "text-sm font-mono",
                      item.current ? "text-primary" : "text-muted-foreground"
                    )}>
                      {item.date}
                    </p>
                    
                    {item.current && (
                      <span className="relative flex h-2 w-2">
                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                        <span className="relative inline-flex rounded-full h-2 w-2 bg-primary"></span>
                      </span>
                    )}
                  </div>
                  
                  <h3 className={cn(
                    "text-base font-medium",
                    item.highlight ? "text-secondary" : ""
                  )}>
                    {item.title}
                  </h3>
                  
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