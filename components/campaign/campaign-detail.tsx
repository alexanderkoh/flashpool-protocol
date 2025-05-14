import { Badge } from "@/components/ui/badge";
import { FlameIcon, AlertCircleIcon } from "lucide-react";
import { PageHeader } from "../layout/page-header";

// Mock data - in a real app, this would come from an API call
const campaignDetails = {
  1: {
    name: "USDC/STELLAR Pool",
    pair: "USDC/XLM",
    status: "active",
    description: "Incentivize liquidity for the USDC/XLM pair on Hoops Finance. This campaign aims to deepen the pool liquidity to reduce slippage for traders.",
    creator: "FlashDAO Treasury",
    createdAt: "2025-05-01",
    boosted: true,
    currentBoost: true,
  },
  2: {
    name: "ETH/FLASH Pool",
    pair: "ETH/FLASH",
    status: "active",
    description: "Promote trading of the ETH/FLASH pair by providing incentives to liquidity providers. Part of the FLASH token distribution strategy.",
    creator: "Flash Foundation",
    createdAt: "2025-05-10",
    boosted: false,
    currentBoost: false,
  },
  3: {
    name: "BTC/USDC Pool",
    pair: "BTC/USDC",
    status: "active",
    description: "Strengthen the BTC/USDC pairing on Hoops Finance. This high-volume pair is critical for the Stellar ecosystem.",
    creator: "Soroban Ecosystem Fund",
    createdAt: "2025-05-15",
    boosted: true,
    currentBoost: true,
  }
};

export function CampaignDetail({ id }: { id: string }) {
  // In a real app, we would fetch this data using an API call
  const campaign = campaignDetails[id as keyof typeof campaignDetails];
  
  if (!campaign) {
    return (
      <div className="text-center py-12">
        <AlertCircleIcon className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
        <h2 className="text-2xl font-bold">Campaign Not Found</h2>
        <p className="text-muted-foreground mt-2">
          The campaign you're looking for doesn't exist or has been removed.
        </p>
      </div>
    );
  }
  
  return (
    <div>
      <div className="flex flex-col md:flex-row justify-between gap-4 items-start">
        <PageHeader
          title={campaign.name}
          description={campaign.description}
        />
        
        <div className="flex flex-wrap gap-3">
          <Badge className="px-3 py-1">{campaign.status}</Badge>
          {campaign.boosted && (
            <Badge variant="secondary" className="px-3 py-1 flex items-center gap-1">
              <FlameIcon className="h-3 w-3" />
              {campaign.currentBoost ? "Boost Active" : "Boost Scheduled"}
            </Badge>
          )}
        </div>
      </div>
      
      <div className="flex flex-wrap gap-x-8 gap-y-3 mt-6 text-sm">
        <div>
          <span className="text-muted-foreground">Pair:</span>{" "}
          <span className="font-medium">{campaign.pair}</span>
        </div>
        <div>
          <span className="text-muted-foreground">Created by:</span>{" "}
          <span className="font-medium">{campaign.creator}</span>
        </div>
        <div>
          <span className="text-muted-foreground">Start date:</span>{" "}
          <span className="font-medium">{campaign.createdAt}</span>
        </div>
      </div>
    </div>
  );
}