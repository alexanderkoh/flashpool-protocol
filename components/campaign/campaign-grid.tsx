import Link from 'next/link';
import { CampaignCard } from './campaign-card';

// Mock data for campaigns - UPDATED STRUCTURE
const campaigns = [
  {
    id: '1',
    name: 'USDC/STELLAR Liquidity Boost',
    pair: 'USDC/XLM',
    durationHours: 72, // Changed from timeLeft
    rewardToken: 'USDC', // Added
    rewards: 120000, // This is now rewardAmount
    tvl: 1500000,
    boosted: true,
    boostStartHour: 24, // Added
    boostDurationHours: 12, // Added
    boostMultiplier: 2, // Added for potential future display
    apy: 28.4,
  },
  {
    id: '2',
    name: 'ETH/FLASH Yield Farm',
    pair: 'ETH/FLASH',
    durationHours: 48,
    rewardToken: 'ETH',
    rewards: 50, // 50 ETH
    tvl: 750000,
    boosted: false,
    apy: 21.2,
  },
  {
    id: '3',
    name: 'Bitcoin Stability Campaign',
    pair: 'BTC/USDC',
    durationHours: 24,
    rewardToken: 'BTC',
    rewards: 0.5, // 0.5 BTC
    tvl: 1200000,
    boosted: true,
    boostStartHour: 6,
    boostDurationHours: 6,
    boostMultiplier: 1.5,
    apy: 15.8,
  },
  {
    id: '4',
    name: 'Ripple Flash Drop',
    pair: 'FLASH/XRP',
    durationHours: 60,
    rewardToken: 'XRP',
    rewards: 75000,
    tvl: 380000,
    boosted: false,
    apy: 31.5,
  },
  {
    id: '5',
    name: 'Treasury Bond Token Incentives',
    pair: 'TBT/USDC',
    durationHours: 36,
    rewardToken: 'TBT',
    rewards: 10000,
    tvl: 620000,
    boosted: false,
    apy: 19.3,
  },
  {
    id: '6',
    name: 'Stellar Lumens Growth Fund',
    pair: 'XLM/USDT',
    durationHours: 72,
    rewardToken: 'XLM',
    rewards: 250000,
    tvl: 480000,
    boosted: true,
    boostStartHour: 0,
    boostDurationHours: 24,
    boostMultiplier: 3,
    apy: 24.7,
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