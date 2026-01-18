import { useState, useCallback, useEffect } from "react";

export interface SpeedSchedule {
  id: string;
  name: string;
  enabled: boolean;
  startTime: string; // HH:mm format
  endTime: string; // HH:mm format
  speedLimit: number; // KB/s, 0 = unlimited
  days: number[]; // 0-6 (Sunday-Saturday)
}

const DEFAULT_SCHEDULES: SpeedSchedule[] = [
  {
    id: "night-unlimited",
    name: "Night Unlimited",
    enabled: false,
    startTime: "00:00",
    endTime: "06:00",
    speedLimit: 0,
    days: [0, 1, 2, 3, 4, 5, 6],
  },
  {
    id: "work-hours-limited",
    name: "Work Hours Limited",
    enabled: false,
    startTime: "09:00",
    endTime: "17:00",
    speedLimit: 500,
    days: [1, 2, 3, 4, 5],
  },
];

const STORAGE_KEY = "rongyok-speed-schedules";

function isTimeInRange(current: Date, startTime: string, endTime: string): boolean {
  const [startHour, startMin] = startTime.split(":").map(Number);
  const [endHour, endMin] = endTime.split(":").map(Number);

  const currentMinutes = current.getHours() * 60 + current.getMinutes();
  const startMinutes = startHour * 60 + startMin;
  const endMinutes = endHour * 60 + endMin;

  if (startMinutes <= endMinutes) {
    return currentMinutes >= startMinutes && currentMinutes < endMinutes;
  } else {
    // Overnight schedule (e.g., 22:00 - 06:00)
    return currentMinutes >= startMinutes || currentMinutes < endMinutes;
  }
}

export function useSpeedSchedule(baseSpeedLimit: number) {
  const [schedules, setSchedules] = useState<SpeedSchedule[]>(() => {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      return saved ? JSON.parse(saved) : DEFAULT_SCHEDULES;
    } catch {
      return DEFAULT_SCHEDULES;
    }
  });

  const [activeSchedule, setActiveSchedule] = useState<SpeedSchedule | null>(null);
  const [currentSpeedLimit, setCurrentSpeedLimit] = useState(baseSpeedLimit);

  // Save to localStorage
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(schedules));
  }, [schedules]);

  // Check schedules every minute
  useEffect(() => {
    const checkSchedule = () => {
      const now = new Date();
      const currentDay = now.getDay();

      // Find matching schedule
      const matchingSchedule = schedules.find(
        (schedule) =>
          schedule.enabled &&
          schedule.days.includes(currentDay) &&
          isTimeInRange(now, schedule.startTime, schedule.endTime)
      );

      if (matchingSchedule) {
        setActiveSchedule(matchingSchedule);
        setCurrentSpeedLimit(matchingSchedule.speedLimit);
      } else {
        setActiveSchedule(null);
        setCurrentSpeedLimit(baseSpeedLimit);
      }
    };

    checkSchedule();
    const interval = setInterval(checkSchedule, 60000); // Check every minute

    return () => clearInterval(interval);
  }, [schedules, baseSpeedLimit]);

  const addSchedule = useCallback((schedule: Omit<SpeedSchedule, "id">) => {
    const newSchedule: SpeedSchedule = {
      ...schedule,
      id: `schedule-${Date.now()}`,
    };
    setSchedules((prev) => [...prev, newSchedule]);
  }, []);

  const updateSchedule = useCallback((id: string, updates: Partial<SpeedSchedule>) => {
    setSchedules((prev) =>
      prev.map((s) => (s.id === id ? { ...s, ...updates } : s))
    );
  }, []);

  const deleteSchedule = useCallback((id: string) => {
    setSchedules((prev) => prev.filter((s) => s.id !== id));
  }, []);

  const toggleSchedule = useCallback((id: string) => {
    setSchedules((prev) =>
      prev.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s))
    );
  }, []);

  const resetSchedules = useCallback(() => {
    setSchedules(DEFAULT_SCHEDULES);
  }, []);

  return {
    schedules,
    activeSchedule,
    currentSpeedLimit,
    addSchedule,
    updateSchedule,
    deleteSchedule,
    toggleSchedule,
    resetSchedules,
  };
}
