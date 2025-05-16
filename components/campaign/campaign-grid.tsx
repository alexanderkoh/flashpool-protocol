import Link from 'next/link';
import { CampaignCard } from './campaign-card';

// Mock data for campaigns - UPDATED STRUCTURE
const campaigns = [
  {
    id: '1',
    name: 'XLM/USDC Liquidity Boost',
    pair: 'XLM/USDC',
    platform: 'Soroswap',
    durationHours: 72,
    rewardToken: 'XLM',
    rewards: 150000,
    tvl: 1000000,
    boosted: true,
    boostStartHour: 24,
    boostDurationHours: 12,
    boostMultiplier: 2,
    apy: 25.0,
  },
  {
    id: '2',
    name: 'FLASH/USDC Yield Farm',
    pair: 'FLASH/USDC',
    platform: 'Soroswap',
    durationHours: 48,
    rewardToken: 'FLASH',
    rewards: 200000,
    tvl: 800000,
    boosted: false,
    apy: 30.0,
  },
];

export function CampaignGrid() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-6">
      {campaigns.map((campaign) => (
        <Link key={campaign.id} href={`/campaign/${campaign.id}`}>
          <CampaignCard
            name={campaign.name}
            pair={campaign.pair}
            platform={campaign.platform}
            durationHours={campaign.durationHours} // Updated
            rewardToken={campaign.rewardToken} // Added
            rewards={campaign.rewards}
            tvl={campaign.tvl}
            boosted={campaign.boosted}
            boostStartHour={campaign.boostStartHour} // Added (optional)
            boostDurationHours={campaign.boostDurationHours} // Added (optional)
            // boostMultiplier={campaign.boostMultiplier} // Pass if you want to display on card
            apy={campaign.apy}
          />
        </Link>
      ))}
    </div>
  );
}