import { useState, useCallback, useRef } from "react";

interface SpeedDataPoint {
  time: number;
  speed: number;
}

const MAX_POINTS = 60; // 60 seconds of data

export function useSpeedGraph() {
  const [speedData, setSpeedData] = useState<SpeedDataPoint[]>([]);
  const [currentSpeed, setCurrentSpeed] = useState(0);
  const [avgSpeed, setAvgSpeed] = useState(0);
  const [peakSpeed, setPeakSpeed] = useState(0);
  const startTimeRef = useRef<number>(Date.now());

  const addDataPoint = useCallback((speed: number) => {
    const now = Date.now();
    const time = (now - startTimeRef.current) / 1000; // seconds since start

    setCurrentSpeed(speed);
    setSpeedData((prev) => {
      const newData = [...prev, { time, speed }].slice(-MAX_POINTS);

      // Calculate avg and peak
      if (newData.length > 0) {
        const avg = newData.reduce((sum, p) => sum + p.speed, 0) / newData.length;
        const peak = Math.max(...newData.map((p) => p.speed));
        setAvgSpeed(avg);
        setPeakSpeed(peak);
      }

      return newData;
    });
  }, []);

  const reset = useCallback(() => {
    setSpeedData([]);
    setCurrentSpeed(0);
    setAvgSpeed(0);
    setPeakSpeed(0);
    startTimeRef.current = Date.now();
  }, []);

  return {
    speedData,
    currentSpeed,
    avgSpeed,
    peakSpeed,
    addDataPoint,
    reset,
  };
}
