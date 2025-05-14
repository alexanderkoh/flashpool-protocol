import { Suspense } from "react";
import { PageHeader } from "@/components/layout/page-header";
import { CampaignDetailSkeleton } from "@/components/campaign/campaign-detail-skeleton";
import { CampaignDetail } from "@/components/campaign/campaign-detail";
import { DepositPanel } from "@/components/campaign/deposit-panel";
import { CampaignStats } from "@/components/campaign/campaign-stats";
import { CampaignTimeline } from "@/components/campaign/campaign-timeline";

export default function CampaignPage({ params }: { params: { id: string } }) {
  return (
    <div className="container mx-auto max-w-6xl px-4 py-8">
      <Suspense fallback={<CampaignDetailSkeleton />}>
        <CampaignDetail id={params.id} />
      </Suspense>
      
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mt-8">
        <div className="lg:col-span-2 space-y-6">
          <CampaignStats id={params.id} />
          <CampaignTimeline id={params.id} />
        </div>
        <div>
          <DepositPanel id={params.id} />
        </div>
      </div>
    </div>
  );
}