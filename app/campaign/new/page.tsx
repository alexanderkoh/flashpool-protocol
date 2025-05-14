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

const steps = [
  { id: 'select-pool', title: 'Select Pool' },
  { id: 'set-parameters', title: 'Set Parameters' },
  { id: 'configure-boost', title: 'Configure Boost' },
  { id: 'confirm', title: 'Confirm & Launch' },
];

export default function NewCampaignPage() {
  const [currentStep, setCurrentStep] = useState(0);
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
  });
  const router = useRouter();
  const { toast } = useToast();

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

  const handleSubmit = () => {
    // In a real app, we would submit to the blockchain here
    toast({
      title: "Campaign Created!",
      description: `Your campaign for ${campaignData.poolName} has been launched successfully.`,
    });
    router.push('/explore');
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
            <Button onClick={handleNext}>
              {currentStep === steps.length - 1 ? "Launch Campaign" : "Next"}
            </Button>
          </div>
        </GlassPanel>
      </div>
    </div>
  );
}