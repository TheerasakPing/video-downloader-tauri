import React from "react";

interface LogoProps {
  size?: number;
  className?: string;
}

export const Logo: React.FC<LogoProps> = ({ size = 40, className = "" }) => {
  return (
    <div
      className={`icon-glow icon-glow-lg icon-glow-fuchsia icon-glow-animated ${className}`}
      style={{ padding: size * 0.1 }}
    >
      <svg
        width={size}
        height={size}
        viewBox="0 0 100 100"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* Definitions for gradients and effects */}
        <defs>
          {/* Main background gradient */}
          <linearGradient id="logoGradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#8B5CF6" />
            <stop offset="40%" stopColor="#A855F7" />
            <stop offset="70%" stopColor="#D946EF" />
            <stop offset="100%" stopColor="#EC4899" />
          </linearGradient>

          {/* Inner glow gradient */}
          <radialGradient id="innerGlow" cx="30%" cy="30%" r="70%">
            <stop offset="0%" stopColor="rgba(255,255,255,0.4)" />
            <stop offset="100%" stopColor="rgba(255,255,255,0)" />
          </radialGradient>

          {/* Play button gradient */}
          <linearGradient id="playBtnGradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#FFFFFF" />
            <stop offset="100%" stopColor="#E0E7FF" />
          </linearGradient>

          {/* Outer glow filter */}
          <filter id="outerGlow" x="-100%" y="-100%" width="300%" height="300%">
            <feGaussianBlur stdDeviation="4" result="blur1" />
            <feFlood floodColor="#D946EF" floodOpacity="0.6" />
            <feComposite in2="blur1" operator="in" />
            <feMerge>
              <feMergeNode />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>

          {/* Inner element glow */}
          <filter id="elementGlow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="2" result="blur2" />
            <feMerge>
              <feMergeNode in="blur2" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>

          {/* Pulse animation glow */}
          <filter id="pulseGlow" x="-100%" y="-100%" width="300%" height="300%">
            <feGaussianBlur stdDeviation="6" result="blur3" />
            <feFlood floodColor="#A855F7" floodOpacity="0.4" />
            <feComposite in2="blur3" operator="in" />
            <feMerge>
              <feMergeNode />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Outer glow circle */}
        <circle
          cx="50"
          cy="50"
          r="48"
          fill="none"
          stroke="url(#logoGradient)"
          strokeWidth="2"
          opacity="0.5"
          filter="url(#pulseGlow)"
        />

        {/* Main circle with gradient */}
        <circle
          cx="50"
          cy="50"
          r="44"
          fill="url(#logoGradient)"
          filter="url(#outerGlow)"
        />

        {/* Inner highlight */}
        <circle
          cx="50"
          cy="50"
          r="44"
          fill="url(#innerGlow)"
        />

        {/* Decorative ring */}
        <circle
          cx="50"
          cy="50"
          r="36"
          fill="none"
          stroke="rgba(255,255,255,0.25)"
          strokeWidth="1.5"
          strokeDasharray="8 4"
        />

        {/* Play button triangle */}
        <path
          d="M42 32 L42 68 L70 50 Z"
          fill="url(#playBtnGradient)"
          filter="url(#elementGlow)"
        />

        {/* Download arrow */}
        <g filter="url(#elementGlow)">
          <path
            d="M50 56 L50 74"
            stroke="rgba(255,255,255,0.95)"
            strokeWidth="3.5"
            strokeLinecap="round"
          />
          <path
            d="M43 67 L50 74 L57 67"
            stroke="rgba(255,255,255,0.95)"
            strokeWidth="3.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </g>

        {/* Sparkle effects */}
        <g opacity="0.8">
          <circle cx="22" cy="35" r="2.5" fill="white">
            <animate
              attributeName="opacity"
              values="0.4;1;0.4"
              dur="2s"
              repeatCount="indefinite"
            />
          </circle>
          <circle cx="78" cy="35" r="2" fill="white">
            <animate
              attributeName="opacity"
              values="0.6;1;0.6"
              dur="1.5s"
              repeatCount="indefinite"
            />
          </circle>
          <circle cx="75" cy="70" r="2.5" fill="white">
            <animate
              attributeName="opacity"
              values="0.5;1;0.5"
              dur="1.8s"
              repeatCount="indefinite"
            />
          </circle>
          <circle cx="25" cy="68" r="2" fill="white">
            <animate
              attributeName="opacity"
              values="0.7;1;0.7"
              dur="2.2s"
              repeatCount="indefinite"
            />
          </circle>
        </g>

        {/* Top sparkle star */}
        <g transform="translate(50, 12)" opacity="0.9">
          <path
            d="M0 -4 L1 -1 L4 0 L1 1 L0 4 L-1 1 L-4 0 L-1 -1 Z"
            fill="white"
          >
            <animate
              attributeName="opacity"
              values="0.5;1;0.5"
              dur="1.2s"
              repeatCount="indefinite"
            />
            <animateTransform
              attributeName="transform"
              type="rotate"
              from="0"
              to="360"
              dur="8s"
              repeatCount="indefinite"
            />
          </path>
        </g>
      </svg>
    </div>
  );
};

export default Logo;
