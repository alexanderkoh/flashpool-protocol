import { Progress } from "@/components/ui/progress";

export function RewardsTimeline() {
  // Mock data for rewards unlock timeline
  const timelineData = [
    { date: 'May 18', value: 456.21, unlocked: false, current: true },
  ];
  
  // Calculate progress
  const totalUnlocked = timelineData.filter(item => item.unlocked).reduce((sum, item) => sum + item.value, 0);
  const totalValue = timelineData.reduce((sum, item) => sum + item.value, 0);
  const progressPercentage = (totalUnlocked / totalValue) * 100;
  
  return (
    <div>
      <div className="mb-4">
        <Progress value={progressPercentage} className="h-2" />
        <div className="flex justify-between text-xs text-muted-foreground mt-1">
          <span>0%</span>
          <span>50%</span>
          <span>100%</span>
        </div>
      </div>
      
      <div className="space-y-2">
        {timelineData.map((item, index) => (
          <div 
            key={index} 
            className={`flex justify-between items-center p-2 rounded ${
              item.current ? 'bg-primary/10 border border-primary/30' : ''
            }`}
          >
            <div className="flex items-center">
              <div className={`w-2 h-2 rounded-full mr-2 ${
                item.unlocked ? 'bg-primary' : 'bg-muted'
              }`}></div>
              <span className="text-sm">{item.date}</span>
            </div>
            <div className="flex items-center">
              <span className="font-mono text-sm">
                {item.value > 0 ? item.value.toFixed(2) : '--'} FLASH
              </span>
              {item.unlocked && (
                <span className="ml-2 text-xs text-primary">Unlocked</span>
              )}
              {item.current && (
                <span className="ml-2 text-xs">Current</span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}