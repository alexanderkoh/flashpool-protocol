'use client';

import React, { useEffect, useRef } from 'react';

export function PulseGrid() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resizeCanvas = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };

    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    // Define grid points with a density factor based on device performance and screen size
    const isMobile = window.innerWidth < 768;
    const gridSpacing = isMobile ? 100 : 70; // Larger spacing (fewer points) on mobile
    let points: { x: number; y: number; size: number; speed: number; hue: number }[] = [];

    const initGrid = () => {
      points = [];
      // Limit total points on lower-end devices
      const maxPoints = isMobile ? 500 : 1000;
      const columns = Math.min(Math.ceil(canvas.width / gridSpacing) + 1, Math.sqrt(maxPoints));
      const rows = Math.min(Math.ceil(canvas.height / gridSpacing) + 1, maxPoints / columns);
      
      for (let i = 0; i < columns; i++) {
        for (let j = 0; j < rows; j++) {
          points.push({
            x: i * gridSpacing,
            y: j * gridSpacing,
            size: Math.random() * 1.5 + 0.5,
            speed: Math.random() * 0.05 + 0.01,
            hue: Math.random() < 0.5 ? 328 : 47, // Magenta or Yellow
          });
        }
      }
    };

    initGrid();
    window.addEventListener('resize', initGrid);

    // Animation
    let animationFrame: number;
    let time = 0;
    let lastTime = 0;
    
    // Reduce animation frame rate on mobile
    const animate = (currentTime: number) => {
      // Skip frames on mobile for better performance
      if (isMobile && currentTime - lastTime < 32) { // ~ 30fps instead of 60fps
        animationFrame = requestAnimationFrame(animate);
        return;
      }
      
      lastTime = currentTime;
      time += 0.01;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      
      // Draw grid points
      points.forEach(point => {
        const pulseFactor = Math.sin(time + point.x * 0.01 + point.y * 0.01) * 0.5 + 0.5;
        const size = point.size * pulseFactor;
        
        ctx.beginPath();
        
        // Determine color based on point's hue value
        const color = point.hue === 328 
          ? `rgba(199, 29, 151, ${0.1 * pulseFactor})` 
          : `rgba(223, 171, 1, ${0.1 * pulseFactor})`;
          
        ctx.fillStyle = color;
        ctx.arc(point.x, point.y, size, 0, Math.PI * 2);
        ctx.fill();
      });
      
      animationFrame = requestAnimationFrame(animate);
    };

    animationFrame = requestAnimationFrame(animate);

    return () => {
      window.removeEventListener('resize', resizeCanvas);
      window.removeEventListener('resize', initGrid);
      cancelAnimationFrame(animationFrame);
    };
  }, []);

  return (
    <canvas 
      ref={canvasRef} 
      className="w-full h-full absolute top-0 left-0"
    />
  );
}