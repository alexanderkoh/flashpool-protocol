import { cn } from "@/lib/utils";
import { GlassPanel } from "./ui/glass-panel";
import { ArrowUpIcon, ArrowDownIcon } from "lucide-react";

interface StatsCardProps {
  title: string;
  value: string;
  icon: React.ReactNode;
  trend?: string;
  trendUp?: boolean;
}

export function StatsCard({ title, value, icon, trend, trendUp }: StatsCardProps) {
  return (
    <GlassPanel className="p-6 border border-white/10">
      <div className="flex justify-between">
        <div>
          <h3 className="text-sm font-medium text-muted-foreground">{title}</h3>
          <p className="text-2xl font-bold mt-1">{value}</p>
          
          {trend && (
            <div className="flex items-center mt-2">
              {trendUp !== undefined && (
                trendUp ? (
                  <ArrowUpIcon className="w-3 h-3 text-green-500 mr-1" />
                ) : (
                  <ArrowDownIcon className="w-3 h-3 text-red-500 mr-1" />
                )
              )}
              <span className={cn(
                "text-xs",
                trendUp === true ? "text-green-500" : 
                trendUp === false ? "text-red-500" : 
                "text-muted-foreground"
              )}>
                {trend}
              </span>
            </div>
          )}
        </div>
        
        <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-primary">
          {icon}
        </div>
      </div>
    </GlassPanel>
  );
}