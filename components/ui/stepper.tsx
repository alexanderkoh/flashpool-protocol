'use client';

import { CheckIcon } from "lucide-react";
import { cn } from "@/lib/utils";

interface Step {
  id: string;
  title: string;
}

interface StepperProps {
  steps: Step[];
  currentStep: number;
}

export function Stepper({ steps, currentStep }: StepperProps) {
  return (
    <div className="flex items-center justify-center w-full overflow-x-auto pb-2">
      <ol className="flex items-center w-full min-w-max">
        {steps.map((step, index) => (
          <li 
            key={step.id}
            className={cn(
              "flex items-center relative",
              index !== steps.length - 1 ? "w-full" : "",
            )}
          >
            <div className="flex items-center justify-center flex-col">
              <span 
                className={cn(
                  "z-10 flex items-center justify-center w-8 h-8 rounded-full text-sm font-medium transition-all duration-300",
                  index < currentStep 
                    ? "bg-primary text-white"
                    : index === currentStep
                      ? "bg-primary/20 text-primary border border-primary/50"
                      : "bg-muted text-muted-foreground"
                )}
              >
                {index < currentStep ? (
                  <CheckIcon className="w-4 h-4" />
                ) : (
                  index + 1
                )}
              </span>
              <span className="mt-2 text-sm font-medium text-center whitespace-nowrap px-1">
                {step.title}
              </span>
            </div>
            
            {index !== steps.length - 1 && (
              <div 
                className={cn(
                  "w-full bg-muted h-px mx-2",
                  index < currentStep ? "bg-primary" : ""
                )}
              ></div>
            )}
          </li>
        ))}
      </ol>
    </div>
  );
}