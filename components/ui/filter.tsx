'use client';

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuLabel, 
  DropdownMenuSeparator, 
  DropdownMenuCheckboxItem,
  DropdownMenuTrigger
} from "@/components/ui/dropdown-menu";
import { FilterIcon, SearchIcon } from "lucide-react";
import { useState } from "react";

export function Filter() {
  const [filters, setFilters] = useState({
    active: true,
    ended: false,
    boosted: false,
    highAPY: false,
  });

  const toggleFilter = (key: keyof typeof filters) => {
    setFilters(prev => ({
      ...prev,
      [key]: !prev[key]
    }));
  };

  const activeFilters = Object.entries(filters)
    .filter(([_, isActive]) => isActive)
    .map(([key]) => key);

  return (
    <div className="flex flex-col gap-4 w-full md:flex-row">
      <div className="relative flex-grow max-w-full md:max-w-md">
        <SearchIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground h-4 w-4" />
        <Input 
          placeholder="Search campaigns..." 
          className="pl-10"
        />
      </div>
      
      <div className="flex gap-2 flex-wrap mt-2 md:mt-0">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              <FilterIcon className="mr-2 h-4 w-4" />
              Filter
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-40">
            <DropdownMenuLabel>Status</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuCheckboxItem
              checked={filters.active}
              onCheckedChange={() => toggleFilter('active')}
            >
              Active
            </DropdownMenuCheckboxItem>
            <DropdownMenuCheckboxItem
              checked={filters.ended}
              onCheckedChange={() => toggleFilter('ended')}
            >
              Ended
            </DropdownMenuCheckboxItem>
            <DropdownMenuLabel>Features</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuCheckboxItem
              checked={filters.boosted}
              onCheckedChange={() => toggleFilter('boosted')}
            >
              Boosted
            </DropdownMenuCheckboxItem>
            <DropdownMenuCheckboxItem
              checked={filters.highAPY}
              onCheckedChange={() => toggleFilter('highAPY')}
            >
              High APY
            </DropdownMenuCheckboxItem>
          </DropdownMenuContent>
        </DropdownMenu>
        
        {activeFilters.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {activeFilters.map(filter => (
              <Badge 
                key={filter} 
                variant="outline"
                className="px-3 py-1 capitalize flex items-center"
              >
                {filter}
                <button 
                  className="ml-2 h-4 w-4 rounded-full bg-muted-foreground/30 inline-flex items-center justify-center hover:bg-muted-foreground/50"
                  onClick={() => toggleFilter(filter as keyof typeof filters)}
                >
                  <span className="sr-only">Remove</span>
                  <svg width="8" height="8" viewBox="0 0 8 8" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M1 1L7 7M1 7L7 1" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                </button>
              </Badge>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}