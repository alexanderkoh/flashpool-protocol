'use client';

import { GlassPanel } from '@/components/ui/glass-panel';
import {
  Area,
  AreaChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import { formatCurrency } from '@/lib/utils';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

// Mock data - in a real app, this would come from an API call
const tvlData = [
  { date: 'May 01', value: 800000 },
  { date: 'May 02', value: 850000 },
  { date: 'May 03', value: 900000 },
  { date: 'May 04', value: 875000 },
  { date: 'May 05', value: 950000 },
  { date: 'May 06', value: 1100000 },
  { date: 'May 07', value: 1200000 },
  { date: 'May 08', value: 1350000 },
  { date: 'May 09', value: 1500000 },
];

const volumeData = [
  { date: 'May 01', value: 120000 },
  { date: 'May 02', value: 180000 },
  { date: 'May 03', value: 150000 },
  { date: 'May 04', value: 200000 },
  { date: 'May 05', value: 240000 },
  { date: 'May 06', value: 280000 },
  { date: 'May 07', value: 320000 },
  { date: 'May 08', value: 260000 },
  { date: 'May 09', value: 350000 },
];

const apyData = [
  { date: 'May 01', value: 32 },
  { date: 'May 02', value: 31 },
  { date: 'May 03', value: 30 },
  { date: 'May 04', value: 29 },
  { date: 'May 05', value: 28.5 },
  { date: 'May 06', value: 28 },
  { date: 'May 07', value: 29 },
  { date: 'May 08', value: 29.5 },
  { date: 'May 09', value: 28.4 },
];

const CustomTooltip = ({ active, payload, label, valuePrefix }: any) => {
  if (active && payload && payload.length) {
    return (
      <GlassPanel className="p-3 border border-white/20">
        <p className="text-sm font-medium">{label}</p>
        <p className="text-sm font-mono">
          {valuePrefix}{' '}
          {payload[0].value.toLocaleString(undefined, {
            maximumFractionDigits: 2,
          })}
        </p>
      </GlassPanel>
    );
  }

  return null;
};

export function CampaignStats({ id }: { id: string }) {
  return (
    <GlassPanel className="p-6 border border-white/10">
      <Tabs defaultValue="tvl">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold">Campaign Analytics</h2>
          <TabsList>
            <TabsTrigger value="tvl">TVL</TabsTrigger>
            <TabsTrigger value="volume">Volume</TabsTrigger>
            <TabsTrigger value="apy">APY</TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="tvl">
          <div className="h-60">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={tvlData}>
                <defs>
                  <linearGradient id="tvlGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop
                      offset="5%"
                      stopColor="hsl(var(--chart-1))"
                      stopOpacity={0.3}
                    />
                    <stop
                      offset="95%"
                      stopColor="hsl(var(--chart-1))"
                      stopOpacity={0}
                    />
                  </linearGradient>
                </defs>
                <XAxis
                  dataKey="date"
                  stroke="hsl(var(--muted-foreground))"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <YAxis
                  stroke="hsl(var(--muted-foreground))"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={(value) => `$${value / 1000}k`}
                />
                <Tooltip content={<CustomTooltip valuePrefix="$" />} />
                <Area
                  type="monotone"
                  dataKey="value"
                  stroke="hsl(var(--chart-1))"
                  fillOpacity={1}
                  fill="url(#tvlGradient)"
                  isAnimationActive={true}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
          <div className="mt-2 flex justify-between items-center text-sm">
            <span className="text-muted-foreground">Current TVL</span>
            <span className="font-medium">{formatCurrency(1500000, true)}</span>
          </div>
        </TabsContent>

        <TabsContent value="volume">
          <div className="h-60">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={volumeData}>
                <defs>
                  <linearGradient id="volumeGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop
                      offset="5%"
                      stopColor="hsl(var(--chart-2))"
                      stopOpacity={0.3}
                    />
                    <stop
                      offset="95%"
                      stopColor="hsl(var(--chart-2))"
                      stopOpacity={0}
                    />
                  </linearGradient>
                </defs>
                <XAxis
                  dataKey="date"
                  stroke="hsl(var(--muted-foreground))"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <YAxis
                  stroke="hsl(var(--muted-foreground))"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={(value) => `$${value / 1000}k`}
                />
                <Tooltip content={<CustomTooltip valuePrefix="$" />} />
                <Area
                  type="monotone"
                  dataKey="value"
                  stroke="hsl(var(--chart-2))"
                  fillOpacity={1}
                  fill="url(#volumeGradient)"
                  isAnimationActive={true}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
          <div className="mt-2 flex justify-between items-center text-sm">
            <span className="text-muted-foreground">24h Volume</span>
            <span className="font-medium">{formatCurrency(350000, true)}</span>
          </div>
        </TabsContent>

        <TabsContent value="apy">
          <div className="h-60">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={apyData}>
                <defs>
                  <linearGradient id="apyGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop
                      offset="5%"
                      stopColor="hsl(var(--chart-5))"
                      stopOpacity={0.3}
                    />
                    <stop
                      offset="95%"
                      stopColor="hsl(var(--chart-5))"
                      stopOpacity={0}
                    />
                  </linearGradient>
                </defs>
                <XAxis
                  dataKey="date"
                  stroke="hsl(var(--muted-foreground))"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <YAxis
                  stroke="hsl(var(--muted-foreground))"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={(value) => `${value}%`}
                />
                <Tooltip content={<CustomTooltip valuePrefix="" />} />
                <Area
                  type="monotone"
                  dataKey="value"
                  stroke="hsl(var(--chart-5))"
                  fillOpacity={1}
                  fill="url(#apyGradient)"
                  isAnimationActive={true}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
          <div className="mt-2 flex justify-between items-center text-sm">
            <span className="text-muted-foreground">Current APY</span>
            <span className="font-medium">28.4%</span>
          </div>
        </TabsContent>
      </Tabs>
    </GlassPanel>
  );
}