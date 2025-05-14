'use client';

import { useState } from 'react';
import { PageHeader } from "@/components/layout/page-header";
import { GlassPanel } from "@/components/ui/glass-panel";
import { Button } from "@/components/ui/button";
import { useToast } from "@/hooks/use-toast";
import { 
  Table, 
  TableHeader, 
  TableBody, 
  TableRow, 
  TableHead, 
  TableCell 
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter } from "@/components/ui/dialog";
import { AlertCircleIcon, RefreshCwIcon, CheckCircleIcon } from "lucide-react";

// Mock data for admin dashboard
const campaigns = [
  { id: 'camp-1', name: 'USDC/BTC Pool', status: 'active', tvl: '$1.2M', rewards: '120,000 FLASH', timeLeft: '12 days' },
  { id: 'camp-2', name: 'ETH/USDC Pool', status: 'active', tvl: '$895K', rewards: '80,000 FLASH', timeLeft: '5 days' },
  { id: 'camp-3', name: 'FLASH/XLM Pool', status: 'completed', tvl: '$1.5M', rewards: '150,000 FLASH', timeLeft: '0 days' },
  { id: 'camp-4', name: 'TBT/USDC Pool', status: 'pending', tvl: '$0', rewards: '50,000 FLASH', timeLeft: 'Not started' },
];

export default function AdminPage() {
  const { toast } = useToast();
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [selectedCampaign, setSelectedCampaign] = useState<any>(null);

  const handleRefresh = () => {
    setIsRefreshing(true);
    setTimeout(() => {
      setIsRefreshing(false);
      toast({
        title: "Cache Refreshed",
        description: "All campaign data has been updated from the blockchain.",
      });
    }, 2000);
  };

  const handleFinalize = () => {
    toast({
      title: "Campaign Finalized",
      description: `${selectedCampaign.name} has been successfully finalized.`,
    });
  };

  return (
    <div className="container mx-auto max-w-6xl px-4 py-8">
      <PageHeader
        title="Admin Dashboard"
        description="Manage active campaigns and system settings."
      />
      
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">Campaign Management</h2>
        <Button variant="outline" onClick={handleRefresh} disabled={isRefreshing}>
          <RefreshCwIcon className="mr-2 h-4 w-4" />
          {isRefreshing ? "Refreshing..." : "Refresh Cache"}
        </Button>
      </div>
      
      <GlassPanel className="p-0 border border-white/10 relative overflow-hidden">
        <div className="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow className="bg-white/5">
                <TableHead>Campaign</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>TVL</TableHead>
                <TableHead>Rewards</TableHead>
                <TableHead>Time Left</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {campaigns.map((campaign) => (
                <TableRow key={campaign.id}>
                  <TableCell className="font-medium">{campaign.name}</TableCell>
                  <TableCell>
                    <Badge variant={
                      campaign.status === 'active' ? 'default' :
                      campaign.status === 'completed' ? 'outline' : 'secondary'
                    }>
                      {campaign.status}
                    </Badge>
                  </TableCell>
                  <TableCell>{campaign.tvl}</TableCell>
                  <TableCell>{campaign.rewards}</TableCell>
                  <TableCell>{campaign.timeLeft}</TableCell>
                  <TableCell className="text-right">
                    {campaign.status === 'active' && (
                      <Dialog>
                        <DialogTrigger asChild>
                          <Button 
                            variant="outline" 
                            size="sm"
                            onClick={() => setSelectedCampaign(campaign)}
                          >
                            Finalize
                          </Button>
                        </DialogTrigger>
                        <DialogContent>
                          <DialogHeader>
                            <DialogTitle>Finalize Campaign</DialogTitle>
                          </DialogHeader>
                          <div className="py-4">
                            <div className="flex items-center gap-2 text-amber-400 mb-4">
                              <AlertCircleIcon className="h-5 w-5" />
                              <p className="font-medium">Campaign still has time remaining</p>
                            </div>
                            <p>
                              Are you sure you want to finalize {campaign.name} early? 
                              This will end the campaign and distribute any remaining rewards.
                            </p>
                          </div>
                          <DialogFooter>
                            <Button variant="outline" onClick={() => {}}>Cancel</Button>
                            <Button onClick={handleFinalize}>Confirm Finalize</Button>
                          </DialogFooter>
                        </DialogContent>
                      </Dialog>
                    )}
                    {campaign.status === 'completed' && (
                      <span className="flex items-center justify-end text-muted-foreground">
                        <CheckCircleIcon className="h-4 w-4 mr-1" />
                        Finalized
                      </span>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </GlassPanel>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
        <GlassPanel className="p-6 border border-white/10">
          <h3 className="text-xl font-bold mb-4">System Stats</h3>
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total Active Campaigns:</span>
              <span className="font-medium">2</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Pending Campaigns:</span>
              <span className="font-medium">1</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total FLASH in Campaigns:</span>
              <span className="font-medium">400,000</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Recycled Rewards:</span>
              <span className="font-medium">24,321 FLASH</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">API Cache Last Updated:</span>
              <span className="font-medium">5 minutes ago</span>
            </div>
          </div>
        </GlassPanel>
        
        <GlassPanel className="p-6 border border-white/10">
          <h3 className="text-xl font-bold mb-4">Quick Actions</h3>
          <div className="space-y-3">
            <Button variant="outline" className="w-full justify-start">
              <RefreshCwIcon className="mr-2 h-4 w-4" />
              Refresh Hoops API Cache
            </Button>
            <Button variant="outline" className="w-full justify-start">
              Update Oracle Price Feeds
            </Button>
            <Button variant="outline" className="w-full justify-start">
              View System Logs
            </Button>
            <Button variant="outline" className="w-full justify-start">
              Export Campaign Data
            </Button>
          </div>
        </GlassPanel>
      </div>
    </div>
  );
}