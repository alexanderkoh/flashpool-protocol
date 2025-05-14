import { GlassPanel } from '@/components/ui/glass-panel';
import { Skeleton } from '@/components/ui/skeleton';

export function CampaignGridSkeleton() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-6">
      {Array.from({ length: 6 }).map((_, index) => (
        <GlassPanel key={index} className="p-6 border border-white/10 h-full">
          <Skeleton className="h-5 w-3/4 mb-3" />
          <Skeleton className="h-4 w-1/2 mb-6" />
          
          <div className="grid grid-cols-2 gap-4 mb-6">
            <div>
              <Skeleton className="h-3 w-2/3 mb-2" />
              <Skeleton className="h-5 w-full" />
            </div>
            <div>
              <Skeleton className="h-3 w-2/3 mb-2" />
              <Skeleton className="h-5 w-full" />
            </div>
          </div>
          
          <Skeleton className="h-3 w-1/3 mb-2" />
          <Skeleton className="h-4 w-full mb-4" />
          
          <div className="flex justify-between items-center">
            <Skeleton className="h-6 w-16" />
            <Skeleton className="h-8 w-24" />
          </div>
        </GlassPanel>
      ))}
    </div>
  );
}