'use client';

import { useState, useEffect, useCallback } from 'react';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import { Button } from '@/components/ui/button';
import { formatCurrency } from '@/lib/utils';
import { cn } from '@/lib/utils';
import { InfoIcon, RefreshCwIcon } from 'lucide-react';

// Mock pool data with underlying tokens
const availablePools = [
  { id: 'pool-1', name: 'USDC/XLM', tvl: '$1.5M', volume: '$350K', tokens: ['USDC', 'XLM'] },
  { id: 'pool-2', name: 'ETH/FLASH', tvl: '$750K', volume: '$180K', tokens: ['ETH', 'FLASH'] },
  { id: 'pool-3', name: 'BTC/USDC', tvl: '$1.2M', volume: '$400K', tokens: ['BTC', 'USDC'] },
  { id: 'pool-4', name: 'XRP/FLASH', tvl: '$380K', volume: '$90K', tokens: ['XRP', 'FLASH'] },
  { id: 'pool-5', name: 'TBT/USDC', tvl: '$620K', volume: '$120K', tokens: ['TBT', 'USDC'] },
];

// Mock function to simulate fetching FLASH equivalent
// In a real app, this would involve an API call to an oracle or DEX
const getFlashEquivalent = async (contributionToken: string, contributionAmount: number): Promise<number> => {
  if (!contributionToken || contributionAmount <= 0) return 0;
  console.log(`Fetching FLASH quote for ${contributionAmount} ${contributionToken}...`);
  await new Promise(resolve => setTimeout(resolve, 750)); // Simulate network delay
  let rate = 1;
  switch (contributionToken) {
    case 'USDC': rate = 0.10; break; // 1 USDC = 10 FLASH (example rate)
    case 'XLM': rate = 5; break;    // 1 XLM = 0.2 FLASH
    case 'ETH': rate = 0.005; break; // 1 ETH = 200 FLASH
    case 'BTC': rate = 0.0001; break; // 1 BTC = 10000 FLASH
    case 'XRP': rate = 10; break;    // 1 XRP = 0.1 FLASH
    case 'TBT': rate = 1; break;     // 1 TBT = 1 FLASH (example)
    case 'FLASH': rate = 1; break;   // 1 FLASH = 1 FLASH
    default: rate = 1; // Default if token unknown, though UI should prevent this
  }
  const flashAmount = contributionAmount / rate; // This logic seems inverted, corrected below
  // Corrected logic: if 1 USDC = 10 FLASH, then amountUSDC * 10 = amountFLASH
  // If rate is how many ContribToken per 1 FLASH (e.g. 0.1 USDC for 1 FLASH), then contribAmount / rate = FLASHAmount
  // If rate is how many FLASH per 1 ContribToken (e.g. 1 USDC = 10 FLASH), then contribAmount * rate = FLASHAmount

  // Let's define `rate` as: How many FLASH tokens per 1 unit of contributionToken.
  switch (contributionToken) {
    case 'USDC': rate = 10; break;  // 1 USDC = 10 FLASH
    case 'XLM': rate = 0.2; break; // 1 XLM = 0.2 FLASH
    case 'ETH': rate = 2000; break; // 1 ETH = 2000 FLASH (updated example)
    case 'BTC': rate = 30000; break;// 1 BTC = 30000 FLASH (updated example)
    case 'XRP': rate = 0.5; break;  // 1 XRP = 0.5 FLASH
    case 'TBT': rate = 1; break;    // 1 TBT = 1 FLASH
    case 'FLASH': rate = 1; break;  // Direct contribution of FLASH
    default: rate = 0; // Should not happen if UI is correct
  }
  const calculatedFlashAmount = contributionAmount * rate;
  console.log(`Quoted: ${contributionAmount} ${contributionToken} = ${calculatedFlashAmount} FLASH`);
  return calculatedFlashAmount;
};

interface CampaignFormProps {
  step: number;
  campaignData: any; // Consider defining a more specific type for campaignData
  setCampaignData: (data: any) => void;
}

export function CampaignForm({ step, campaignData, setCampaignData }: CampaignFormProps) {
  const [estimatedApy, setEstimatedApy] = useState<number>(0);
  const [selectedPoolTokens, setSelectedPoolTokens] = useState<string[]>([]);
  const [isQuoting, setIsQuoting] = useState(false);
  const [quotedFlashAmount, setQuotedFlashAmount] = useState<number | null>(null);

  // Renamed for clarity: flashRewardAmount is the actual reward amount in FLASH for the campaign
  // contributionAmount is what the user inputs in their chosen contributionToken

  const fetchAndSetQuote = useCallback(async () => {
    if (campaignData.contributionToken && campaignData.contributionAmount > 0) {
      setIsQuoting(true);
      setQuotedFlashAmount(null); // Clear previous quote while fetching
      try {
        const flashEquivalent = await getFlashEquivalent(campaignData.contributionToken, campaignData.contributionAmount);
        setQuotedFlashAmount(flashEquivalent);
        // Update campaignData with the actual FLASH amount for rewards
        handleChange('flashRewardAmount', flashEquivalent);
      } catch (error) {
        console.error("Error fetching FLASH quote:", error);
        setQuotedFlashAmount(0); // Indicate error or unquotable
        handleChange('flashRewardAmount', 0);
      } finally {
        setIsQuoting(false);
      }
    }
  }, [campaignData.contributionToken, campaignData.contributionAmount, setCampaignData]);

  // Effect to fetch quote when contribution token or amount changes
  useEffect(() => {
    if (campaignData.contributionToken && campaignData.contributionAmount > 0) {
      fetchAndSetQuote();
    } else {
      setQuotedFlashAmount(null);
      handleChange('flashRewardAmount', 0);
    }
  }, [campaignData.contributionToken, campaignData.contributionAmount, fetchAndSetQuote]);

  // Update estimated APY when relevant parameters change
  useEffect(() => {
    // APY is based on flashRewardAmount (actual FLASH tokens for campaign)
    if (campaignData.targetTVL > 0 && campaignData.flashRewardAmount > 0 && campaignData.duration > 0) {
      const annualizedRewards = (campaignData.flashRewardAmount * 24 * 365) / campaignData.duration;
      const apy = (annualizedRewards / campaignData.targetTVL) * 100;
      setEstimatedApy(apy > 1000 ? 1000 : apy);
    } else {
      setEstimatedApy(0);
    }
  }, [campaignData.targetTVL, campaignData.flashRewardAmount, campaignData.duration]);
  
  const handleChange = (field: string, value: any) => {
    setCampaignData((prevData: any) => {
      const newData = {
        ...prevData,
        [field]: value,
      };
      // If contributionToken or contributionAmount changes, flashRewardAmount will be updated by useEffect via fetchAndSetQuote
      // If duration changes, and boostDurationHours is greater than new duration, reset boostDurationHours
      if (field === 'duration' && newData.boostDurationHours > value) {
        newData.boostDurationHours = value;
      }
      if (field === 'duration' && newData.boostStartHour + (newData.boostDurationHours || 1) > value) {
        newData.boostStartHour = Math.max(0, value - (newData.boostDurationHours || 1));
      }
      if (field === 'boostStartHour' && value + (newData.boostDurationHours || 1) > newData.duration) {
        newData.boostDurationHours = Math.max(1, newData.duration - value);
      }
      if (field === 'boostDurationHours' && (newData.boostStartHour || 0) + value > newData.duration) {
        newData.boostStartHour = Math.max(0, newData.duration - value);
      }
      return newData;
    });
  };
  
  const handlePoolChange = (poolId: string) => {
    const pool = availablePools.find(p => p.id === poolId);
    handleChange('poolId', poolId);
    handleChange('poolName', pool?.name || '');
    const currentTokens = pool?.tokens || [];
    setSelectedPoolTokens(currentTokens);
    
    const currentContributionToken = campaignData.contributionToken;
    if (pool && currentContributionToken && !currentTokens.includes(currentContributionToken)) {
      handleChange('contributionToken', currentTokens.length > 0 ? currentTokens[0] : ''); 
    } else if (pool && !currentContributionToken && currentTokens.length > 0) {
      handleChange('contributionToken', currentTokens[0]);
    }
  };

  const formatPoolName = (poolName: string) => {
    return poolName ? `${poolName} Pool` : "Select a pool";
  };

  if (step === 0) {
    // Step 1: Select Pool
    return (
      <div className="space-y-6">
        <div className="space-y-2">
          <Label htmlFor="pool">Select a Liquidity Pool</Label>
          <Select 
            value={campaignData.poolId || ''} 
            onValueChange={handlePoolChange}
          >
            <SelectTrigger id="pool">
              <SelectValue placeholder="Select a pool" />
            </SelectTrigger>
            <SelectContent>
              {availablePools.map((pool) => (
                <SelectItem key={pool.id} value={pool.id}>
                  <div className="flex justify-between items-center w-full">
                    <span>{pool.name}</span>
                    <span className="text-xs text-muted-foreground">TVL: {pool.tvl}</span>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        
        {campaignData.poolId && (
          <>
            <div className="rounded-lg border border-border p-4 space-y-3">
              <h3 className="font-medium">{formatPoolName(campaignData.poolName)} Stats</h3>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-xs text-muted-foreground">Current TVL</p>
                  <p className="font-medium">
                    {availablePools.find(p => p.id === campaignData.poolId)?.tvl}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">24h Volume</p>
                  <p className="font-medium">
                    {availablePools.find(p => p.id === campaignData.poolId)?.volume}
                  </p>
                </div>
              </div>
            </div>
            {selectedPoolTokens.length > 0 && (
              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <Label 
                    htmlFor="contributionToken" 
                    title="This is the token you will contribute. It will be converted to FLASH for the campaign rewards."
                  >
                    Select Contribution Token
                  </Label>
                  <InfoIcon className="h-4 w-4 text-muted-foreground cursor-help"/>
                </div>
                <Select 
                  value={campaignData.contributionToken || ''} 
                  onValueChange={(value) => {
                    handleChange('contributionToken', value);
                    // Reset contribution amount if token changes, as quote will be different
                    handleChange('contributionAmount', 0); 
                    setQuotedFlashAmount(null); // Clear stale quote
                    handleChange('flashRewardAmount', 0);
                  }}
                >
                  <SelectTrigger id="contributionToken">
                    <SelectValue placeholder="Select contribution token" />
                  </SelectTrigger>
                  <SelectContent>
                    {selectedPoolTokens.map((token) => (
                      <SelectItem key={token} value={token}>
                        {token}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}
          </>
        )}
      </div>
    );
  }
  
  if (step === 1) {
    // Step 2: Set Parameters
    return (
      <div className="space-y-6">
        <div className="space-y-2">
          <Label>Selected Pool</Label>
          <div className="rounded-lg border border-border p-3 bg-muted/30">
            {formatPoolName(campaignData.poolName)}
          </div>
        </div>

        {campaignData.contributionToken && (
          <div className="space-y-2">
            <Label>Your Contribution Token</Label>
            <div className="rounded-lg border border-border p-3 bg-muted/30">
              {campaignData.contributionToken}
            </div>
          </div>
        )}
        
        <div className="space-y-2">
          <Label htmlFor="contributionAmount">Your Contribution Amount ({campaignData.contributionToken || 'Token'})</Label>
          <div className="relative">
            <Input
              id="contributionAmount"
              type="number"
              value={campaignData.contributionAmount || ''}
              onChange={(e) => handleChange('contributionAmount', Number(e.target.value))}
              className={cn("pr-20", !campaignData.contributionToken && "pr-3")} 
              placeholder="e.g., 1000"
              disabled={!campaignData.contributionToken}
            />
            {campaignData.contributionToken && (
              <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                <span className="text-muted-foreground">{campaignData.contributionToken}</span>
              </div>
            )}
          </div>
        </div>
        
        <div className="space-y-2 bg-muted/30 p-3 rounded-md">
          <div className="flex justify-between items-center">
            <Label htmlFor="flashQuote">Effective FLASH Rewards</Label>
            {isQuoting && <RefreshCwIcon className="h-4 w-4 animate-spin text-primary"/>}
          </div>
          <div 
            id="flashQuote" 
            className={cn(
              "text-lg font-semibold", 
              isQuoting ? "opacity-50" : "",
              quotedFlashAmount === null && !isQuoting && "text-sm text-muted-foreground italic"
            )}
          >
            {isQuoting 
              ? 'Getting quote...' 
              : quotedFlashAmount !== null 
                ? `${formatCurrency(quotedFlashAmount)} FLASH`
                : 'Enter contribution amount for quote'
            }
          </div>
          { !isQuoting && campaignData.contributionAmount > 0 && (
             <Button variant="link" size="sm" className="p-0 h-auto text-xs" onClick={fetchAndSetQuote}>Refresh quote</Button>
          )}
        </div>
        
        <div className="space-y-2">
          <Label htmlFor="targetTVL">Target TVL (USD)</Label>
          <div className="relative">
            <Input
              id="targetTVL"
              type="number"
              value={campaignData.targetTVL || ''}
              onChange={(e) => handleChange('targetTVL', Number(e.target.value))}
              className="pl-6"
              placeholder="e.g., 100000"
            />
            <div className="absolute inset-y-0 left-0 flex items-center pl-2 pointer-events-none">
              <span className="text-muted-foreground">$</span>
            </div>
          </div>
        </div>
        
        <div className="space-y-2">
          <div className="flex justify-between">
            <Label htmlFor="duration">Campaign Duration (Max 72h, Min 6h)</Label>
            <span className="text-sm text-muted-foreground">{campaignData.duration || 0} hours</span>
          </div>
          <Slider
            id="duration"
            min={6}
            max={72}
            step={1}
            value={[campaignData.duration || 6]}
            onValueChange={(value) => handleChange('duration', value[0])}
          />
        </div>
        
        <div className="rounded-lg border border-border p-4 space-y-3">
          <h3 className="font-medium">Estimated Campaign Stats</h3>
          
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-xs text-muted-foreground">End Date</p>
              <p className="font-medium">
                {campaignData.duration 
                  ? new Date(Date.now() + campaignData.duration * 60 * 60 * 1000).toLocaleString()
                  : 'N/A'
                }
              </p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">Est. APY (based on FLASH rewards)</p>
              <p className="font-medium">
                {campaignData.targetTVL > 0 && campaignData.flashRewardAmount > 0 && campaignData.duration > 0 
                  ? `${estimatedApy.toFixed(1)}%`
                  : 'N/A'
                }
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }
  
  if (step === 2) {
    // Step 3: Configure Boost
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between space-x-2">
          <Label htmlFor="boost-switch" className="flex items-center space-x-2 cursor-pointer">
            <span>Enable Boost Period</span>
          </Label>
          <Switch
            id="boost-switch"
            checked={campaignData.boostEnabled}
            onCheckedChange={(checked) => handleChange('boostEnabled', checked)}
          />
        </div>
        
        {campaignData.boostEnabled && campaignData.duration && (
          <>
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label htmlFor="boostStartHour">Boost Start Hour (Relative to campaign start)</Label>
                <span className="text-sm text-muted-foreground">Hour {campaignData.boostStartHour || 0}</span>
              </div>
              <Slider
                id="boostStartHour"
                min={0} // Can start from the beginning
                max={campaignData.duration - (campaignData.boostDurationHours || 1)} // Max is campaign duration minus boost duration
                step={1}
                value={[campaignData.boostStartHour || 0]}
                onValueChange={(value) => handleChange('boostStartHour', value[0])}
              />
            </div>
            
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label htmlFor="boostDurationHours">Boost Duration (in hours)</Label>
                <span className="text-sm text-muted-foreground">{campaignData.boostDurationHours || 0} hours</span>
              </div>
              <Slider
                id="boostDurationHours"
                min={1} // Minimum 1 hour boost
                max={campaignData.duration - (campaignData.boostStartHour || 0)} // Max is remaining campaign duration
                step={1}
                value={[campaignData.boostDurationHours || 1]}
                onValueChange={(value) => handleChange('boostDurationHours', value[0])}
              />
            </div>
            
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label htmlFor="boostMultiplier">Boost Multiplier</Label>
                <span className="text-sm text-muted-foreground">{campaignData.boostMultiplier || 1}x</span>
              </div>
              <Slider
                id="boostMultiplier"
                min={1.5}
                max={5}
                step={0.5}
                value={[campaignData.boostMultiplier || 1.5]}
                onValueChange={(value) => handleChange('boostMultiplier', value[0])}
              />
            </div>
            
            <div className="rounded-lg border border-secondary/30 bg-secondary/5 p-4 space-y-1">
              <h3 className="font-medium text-secondary">Boost Period</h3>
              <p className="text-sm">
                Hour {campaignData.boostStartHour || 0} - Hour {(campaignData.boostStartHour || 0) + (campaignData.boostDurationHours || 0)} of the campaign.
              </p>
              <p className="text-sm">
                During this period, rewards will be multiplied by {campaignData.boostMultiplier || 1}x
              </p>
            </div>
          </>
        )}
        {!campaignData.duration && campaignData.boostEnabled && (
            <p className="text-sm text-destructive">Please set campaign duration first to configure boost.</p>
        )}
      </div>
    );
  }
  
  // Step 4: Confirm & Launch
  if (step === 3) { // Assuming Step 3 is Confirm & Launch now
    return (
      <div className="space-y-6">
        <h3 className="font-medium">Campaign Summary</h3>
        
        <div className="space-y-4">
          <div className="rounded-lg border border-border p-4 space-y-3">
            <h4 className="text-sm font-medium text-muted-foreground">Pool Information</h4>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-xs text-muted-foreground">Selected Pool</p>
                <p className="font-medium">{formatPoolName(campaignData.poolName)}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">Target TVL</p>
                <p className="font-medium">${campaignData.targetTVL ? campaignData.targetTVL.toLocaleString() : '0'}</p>
              </div>
            </div>
          </div>
          
          <div className="rounded-lg border border-border p-4 space-y-3">
            <h4 className="text-sm font-medium text-muted-foreground">Your Contribution</h4>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-xs text-muted-foreground">Contribution Token</p>
                <p className="font-medium">{campaignData.contributionToken || 'N/A'}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">Contribution Amount</p>
                <p className="font-medium">{formatCurrency(campaignData.contributionAmount || 0)} {campaignData.contributionToken || ''}</p>
              </div>
            </div>
          </div>
          
          <div className="rounded-lg border border-border p-4 space-y-3">
            <h4 className="text-sm font-medium text-muted-foreground">Effective Campaign Rewards</h4>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-xs text-muted-foreground">Reward Token</p>
                <p className="font-medium">FLASH</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">Total FLASH Rewards</p>
                <p className="font-medium">{formatCurrency(campaignData.flashRewardAmount || 0)} FLASH</p>
              </div>
            </div>
          </div>
          
          <div className="rounded-lg border border-border p-4 space-y-3">
            <h4 className="text-sm font-medium text-muted-foreground">Campaign Details</h4>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-xs text-muted-foreground">Duration</p>
                <p className="font-medium">{campaignData.duration || 0} hours</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">End Date</p>
                <p className="font-medium">
                  {campaignData.duration 
                    ? new Date(Date.now() + campaignData.duration * 60 * 60 * 1000).toLocaleString()
                    : 'N/A'
                  }
                </p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground">Est. APY (based on FLASH rewards)</p>
                <p className="font-medium">
                  {campaignData.targetTVL > 0 && campaignData.flashRewardAmount > 0 && campaignData.duration > 0 
                    ? `${estimatedApy.toFixed(1)}%`
                    : 'N/A'
                  }
                </p>
              </div>
            </div>
          </div>
          
          {campaignData.boostEnabled && (
            <div className="rounded-lg border border-secondary/30 bg-secondary/5 p-4 space-y-3">
              <h4 className="text-sm font-medium text-secondary">Boost Configuration</h4>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-xs text-muted-foreground">Boost Period</p>
                  <p className="font-medium">
                    Hour {campaignData.boostStartHour || 0} - Hour { (campaignData.boostStartHour || 0) + (campaignData.boostDurationHours || 0)}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Boost Multiplier</p>
                  <p className="font-medium">{campaignData.boostMultiplier || 1}x</p>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    );
  }

  // Default return or handle other steps if any (e.g. step === 2 for boost config)
  if (step === 2) {
    // Step 3: Configure Boost (Now Step 2)
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between space-x-2">
          <Label htmlFor="boost-switch" className="flex items-center space-x-2 cursor-pointer">
            <span>Enable Boost Period</span>
          </Label>
          <Switch
            id="boost-switch"
            checked={campaignData.boostEnabled}
            onCheckedChange={(checked) => handleChange('boostEnabled', checked)}
          />
        </div>
        
        {campaignData.boostEnabled && campaignData.duration && (
          <>
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label htmlFor="boostStartHour">Boost Start Hour (Relative to campaign start)</Label>
                <span className="text-sm text-muted-foreground">Hour {campaignData.boostStartHour || 0}</span>
              </div>
              <Slider
                id="boostStartHour"
                min={0} // Can start from the beginning
                max={campaignData.duration - (campaignData.boostDurationHours || 1)} // Max is campaign duration minus boost duration
                step={1}
                value={[campaignData.boostStartHour || 0]}
                onValueChange={(value) => handleChange('boostStartHour', value[0])}
              />
            </div>
            
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label htmlFor="boostDurationHours">Boost Duration (in hours)</Label>
                <span className="text-sm text-muted-foreground">{campaignData.boostDurationHours || 0} hours</span>
              </div>
              <Slider
                id="boostDurationHours"
                min={1} // Minimum 1 hour boost
                max={campaignData.duration - (campaignData.boostStartHour || 0)} // Max is remaining campaign duration
                step={1}
                value={[campaignData.boostDurationHours || 1]}
                onValueChange={(value) => handleChange('boostDurationHours', value[0])}
              />
            </div>
            
            <div className="space-y-2">
              <div className="flex justify-between">
                <Label htmlFor="boostMultiplier">Boost Multiplier</Label>
                <span className="text-sm text-muted-foreground">{campaignData.boostMultiplier || 1}x</span>
              </div>
              <Slider
                id="boostMultiplier"
                min={1.5}
                max={5}
                step={0.5}
                value={[campaignData.boostMultiplier || 1.5]}
                onValueChange={(value) => handleChange('boostMultiplier', value[0])}
              />
            </div>
            
            <div className="rounded-lg border border-secondary/30 bg-secondary/5 p-4 space-y-1">
              <h3 className="font-medium text-secondary">Boost Period</h3>
              <p className="text-sm">
                Hour {campaignData.boostStartHour || 0} - Hour {(campaignData.boostStartHour || 0) + (campaignData.boostDurationHours || 0)} of the campaign.
              </p>
              <p className="text-sm">
                During this period, rewards will be multiplied by {campaignData.boostMultiplier || 1}x
              </p>
            </div>
          </>
        )}
        {!campaignData.duration && campaignData.boostEnabled && (
            <p className="text-sm text-destructive">Please set campaign duration first to configure boost.</p>
        )}
      </div>
    );
  }

  // Fallback for unhandled steps, or if step prop is not 0, 1, 2, or 3.
  return <div>Configuration step not found.</div>;
}