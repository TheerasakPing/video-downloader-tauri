import React from "react";

interface LogoProps {
  size?: number;
  className?: string;
}

export const Logo: React.FC<LogoProps> = ({ size = 40, className = "" }) => {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      {/* Background circle with gradient */}
      <defs>
        <linearGradient id="bgGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#8B5CF6" />
          <stop offset="50%" stopColor="#A855F7" />
          <stop offset="100%" stopColor="#D946EF" />
        </linearGradient>
        <linearGradient id="playGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#FFFFFF" />
          <stop offset="100%" stopColor="#E0E7FF" />
        </linearGradient>
        <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="3" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* Main circle */}
      <circle cx="50" cy="50" r="45" fill="url(#bgGradient)" />

      {/* Inner ring */}
      <circle cx="50" cy="50" r="38" fill="none" stroke="rgba(255,255,255,0.2)" strokeWidth="2" />

      {/* Play button triangle */}
      <path
        d="M40 30 L40 70 L72 50 Z"
        fill="url(#playGradient)"
        filter="url(#glow)"
      />

      {/* Download arrow overlay */}
      <path
        d="M50 58 L50 75 M42 67 L50 75 L58 67"
        stroke="rgba(255,255,255,0.9)"
        strokeWidth="4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />

      {/* Decorative dots */}
      <circle cx="20" cy="50" r="3" fill="rgba(255,255,255,0.5)" />
      <circle cx="80" cy="50" r="3" fill="rgba(255,255,255,0.5)" />
      <circle cx="50" cy="20" r="3" fill="rgba(255,255,255,0.5)" />
    </svg>
  );
};

export default Logo;
