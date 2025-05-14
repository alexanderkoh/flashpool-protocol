import { Suspense } from 'react';
import { CampaignGrid } from '@/components/campaign/campaign-grid';
import { PageHeader } from '@/components/layout/page-header';
import { CampaignGridSkeleton } from '@/components/campaign/campaign-grid-skeleton';
import { Filter } from '@/components/ui/filter';

export default function ExplorePage() {
  return (
    <div className="container mx-auto max-w-6xl px-4 py-8">
      <PageHeader
        title="Explore Campaigns"
        description="Discover active liquidity incentive campaigns and start earning FLASH rewards."
      />
      
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 my-6">
        <Filter />
      </div>
      
      <Suspense fallback={<CampaignGridSkeleton />}>
        <CampaignGrid />
      </Suspense>
    </div>
  );
}