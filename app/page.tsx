import Image from 'next/image';
import Link from 'next/link';
import { Button } from '@/components/ui/button';
import { NeonGlow } from '@/components/ui/neon-glow';
import { PulseGrid } from '@/components/ui/pulse-grid';
import { GradientTitle } from '@/components/ui/gradient-title';
import { GlassPanel } from '@/components/ui/glass-panel';
import { FlameIcon, RocketIcon, TrendingUpIcon, CoinsIcon } from 'lucide-react';
import { StatsCard } from '@/components/stats-card';

export default function Home() {
  return (
    <div className="relative">
      {/* Background effect */}
      <div className="absolute inset-0 overflow-hidden -z-10">
        <div className="absolute inset-0 bg-black/80"></div>
        <PulseGrid />
      </div>
      
      {/* Hero Section */}
      <section className="relative px-4 pt-28 pb-16 md:pt-32 md:pb-24 overflow-hidden">
        <NeonGlow color="magenta" className="absolute top-1/4 left-1/4 -translate-x-1/2 -translate-y-1/2 opacity-30" />
        <NeonGlow color="yellow" className="absolute bottom-1/4 right-1/4 translate-x-1/2 translate-y-1/2 opacity-20" />
        
        <div className="container mx-auto max-w-6xl relative z-10">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
            <div className="space-y-6">
              <div className="inline-block">
                <span className="inline-flex items-center rounded-full border border-yellow-500/30 bg-yellow-500/10 px-3 py-1 text-sm font-medium text-yellow-400">
                  <span className="mr-1.5 h-2 w-2 rounded-full bg-yellow-400"></span>
                  Mainnet Live
                </span>
              </div>
              
              <GradientTitle className="text-4xl md:text-5xl lg:text-6xl font-bold tracking-tight leading-tight">
                Supercharge Liquidity with <span className="text-primary">FLASH</span> Rewards
              </GradientTitle>
              
              <p className="text-xl text-muted-foreground max-w-xl">
                Create targeted incentive campaigns for any Hoops liquidity pool. Reward LPs with $FLASH and boost your protocol's ecosystem.
              </p>
              
              <div className="flex flex-wrap gap-4 pt-2">
                <Button size="lg" asChild>
                  <Link href="/campaign/new">
                    <RocketIcon className="mr-2 h-5 w-5" />
                    Launch Campaign
                  </Link>
                </Button>
                <Button size="lg" variant="outline" asChild>
                  <Link href="/explore">
                    <TrendingUpIcon className="mr-2 h-5 w-5" />
                    Explore Events
                  </Link>
                </Button>
              </div>
            </div>
            
            <div className="relative">
              <GlassPanel className="p-6 md:p-8 relative overflow-hidden border border-primary/20">
                <div className="absolute top-0 right-0 w-40 h-40 bg-primary/20 blur-3xl rounded-full -translate-y-1/2 translate-x-1/2"></div>
                <Image 
                  src="/clock.png" 
                  alt="FlashPool Dashboard Preview"
                  width={600} 
                  height={400}
                  className="rounded-lg border border-white/10 shadow-xl"
                />
              </GlassPanel>
            </div>
          </div>
        </div>
      </section>
      
      {/* Stats Section */}
      <section className="px-4 py-16 relative">
        <div className="container mx-auto max-w-6xl">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <StatsCard 
              title="Total Campaigns" 
              value="127" 
              icon={<FlameIcon className="h-6 w-6" />} 
              trend="+12% this week"
              trendUp={true}
            />
            <StatsCard 
              title="TVL Across Pools" 
              value="$14.7M" 
              icon={<CoinsIcon className="h-6 w-6" />} 
              trend="+23.5% from last month"
              trendUp={true}
            />
            <StatsCard 
              title="FLASH Rewards" 
              value="1.2M" 
              icon={<TrendingUpIcon className="h-6 w-6" />} 
              trend="Distributed to LPs"
            />
            <StatsCard 
              title="Active LPs" 
              value="842" 
              icon={<RocketIcon className="h-6 w-6" />} 
              trend="Earning rewards"
            />
          </div>
        </div>
      </section>
      
      {/* Features Section */}
      <section className="px-4 py-16 relative">
        <NeonGlow color="magenta" className="absolute bottom-1/3 right-10 opacity-20" />
        <div className="container mx-auto max-w-6xl">
          <h2 className="text-3xl md:text-4xl font-bold mb-12 text-center">How FlashPool Works</h2>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <GlassPanel className="p-6 border border-white/10">
              <div className="rounded-full bg-primary/20 p-3 w-12 h-12 flex items-center justify-center mb-4">
                <span className="text-xl font-bold text-primary">1</span>
              </div>
              <h3 className="text-xl font-bold mb-3">Choose Your Pool</h3>
              <p className="text-muted-foreground">Select any Hoops liquidity pool to incentivize with FLASH rewards.</p>
            </GlassPanel>
            
            <GlassPanel className="p-6 border border-white/10">
              <div className="rounded-full bg-primary/20 p-3 w-12 h-12 flex items-center justify-center mb-4">
                <span className="text-xl font-bold text-primary">2</span>
              </div>
              <h3 className="text-xl font-bold mb-3">Set Parameters</h3>
              <p className="text-muted-foreground">Configure your campaign duration, reward amount, and boost periods.</p>
            </GlassPanel>
            
            <GlassPanel className="p-6 border border-white/10">
              <div className="rounded-full bg-primary/20 p-3 w-12 h-12 flex items-center justify-center mb-4">
                <span className="text-xl font-bold text-primary">3</span>
              </div>
              <h3 className="text-xl font-bold mb-3">Earn Rewards</h3>
              <p className="text-muted-foreground">LPs automatically earn FLASH rewards proportional to their liquidity share.</p>
            </GlassPanel>
          </div>
        </div>
      </section>
    </div>
  );
}