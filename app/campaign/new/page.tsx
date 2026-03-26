'use client';

import { useState } from 'react';
import { PageHeader } from "@/components/layout/page-header";
import { CampaignForm } from "@/components/campaign/campaign-form";
import { GlassPanel } from "@/components/ui/glass-panel";
import { NeonGlow } from "@/components/ui/neon-glow";
import { Stepper } from "@/components/ui/stepper";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/navigation";
import { useToast } from "@/hooks/use-toast";
import { buildCreateCampaign, submitTransaction } from '@/lib/soroban';
import { useWallet } from '@/hooks/use-wallet';

const steps = [
  { id: 'select-pool', title: 'Select Pool' },
  { id: 'set-parameters', title: 'Set Parameters' },
  { id: 'configure-boost', title: 'Configure Boost' },
  { id: 'confirm', title: 'Confirm & Launch' },
];

export default function NewCampaignPage() {
  const [currentStep, setCurrentStep] = useState(0);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [campaignData, setCampaignData] = useState({
    poolId: '',
    poolName: '',
    targetTVL: 0,
    duration: 30,
    flashRewards: 0,
    boostEnabled: false,
    boostStartDay: 15,
    boostDuration: 5,
    boostMultiplier: 2,
    contributionToken: '',
    contributionAmount: 0,
    flashRewardAmount: 0,
    selectedAdapterId: null as number | null,
    selectedPoolAddress: null as string | null,
    selectedLpToken: null as string | null,
    durationLedgers: 100000,
    bonusFlash: 0,
  });
  const router = useRouter();
  const { toast } = useToast();
  const { publicKey, signTransaction } = useWallet();

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleSubmit();
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSubmit = async () => {
    if (!publicKey) {
      toast({ title: 'Connect wallet first', variant: 'destructive' });
      return;
    }
    setIsSubmitting(true);
    try {
      const xdr = await buildCreateCampaign(
        BigInt(Math.floor(campaignData.contributionAmount * 1e7)),
        BigInt(campaignData.selectedAdapterId ?? 3),
        campaignData.selectedPoolAddress ?? '',
        campaignData.selectedLpToken ?? '',
        campaignData.durationLedgers ?? 100000,
        BigInt(Math.floor(campaignData.targetTVL * 1e7)),
        BigInt(Math.floor((campaignData.bonusFlash ?? 0) * 1e7)),
        BigInt(0),
        publicKey,
      );
      const signed = await signTransaction(xdr);
      if (signed) {
        const result = await submitTransaction(signed);
        toast({ title: 'Campaign created!' });
        router.push('/explore');
      }
    } catch (error: any) {
      toast({ title: 'Failed to create campaign', description: error.message, variant: 'destructive' });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="container mx-auto max-w-6xl px-4 py-8">
      <PageHeader
        title="Launch Campaign"
        description="Create a new liquidity incentive campaign with FLASH rewards."
      />
      
      <div className="relative">
        <NeonGlow color="magenta" className="absolute -top-20 -left-20 opacity-10" />
        <NeonGlow color="yellow" className="absolute -bottom-20 -right-20 opacity-10" />
        
        <GlassPanel className="mt-8 p-6 md:p-8 border border-white/10 relative overflow-hidden">
          <Stepper steps={steps} currentStep={currentStep} />
          
          <div className="mt-8">
            <CampaignForm 
              step={currentStep} 
              campaignData={campaignData}
              setCampaignData={setCampaignData}
            />
          </div>
          
          <div className="mt-8 flex justify-between">
            <Button 
              variant="outline" 
              onClick={handleBack}
              disabled={currentStep === 0}
            >
              Back
            </Button>
            <Button onClick={handleNext} disabled={isSubmitting}>
              {currentStep === steps.length - 1 ? (isSubmitting ? "Launching..." : "Launch Campaign") : "Next"}
            </Button>
          </div>
        </GlassPanel>
      </div>
    </div>
  );
}