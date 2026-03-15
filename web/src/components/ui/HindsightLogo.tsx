interface HindsightLogoProps {
  size?: number;
  glow?: boolean;
}

/**
 * Hindsight brand mark — a constellation of three connected nodes
 * with the center node glowing. Represents the session graph and
 * the act of observing hidden patterns.
 */
export function HindsightLogo({ size = 28, glow = true }: HindsightLogoProps) {
  const id = `hl-${Math.random().toString(36).slice(2, 6)}`;

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{ flexShrink: 0 }}
    >
      <defs>
        {glow && (
          <radialGradient id={`${id}-glow`} cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#818CF8" stopOpacity="0.4" />
            <stop offset="100%" stopColor="#818CF8" stopOpacity="0" />
          </radialGradient>
        )}
      </defs>

      {/* Glow behind center node */}
      {glow && (
        <circle cx="16" cy="14" r="10" fill={`url(#${id}-glow)`} />
      )}

      {/* Edges — connecting the constellation */}
      <line x1="7" y1="24" x2="16" y2="14" stroke="#818CF8" strokeWidth="1" strokeOpacity="0.3" />
      <line x1="25" y1="24" x2="16" y2="14" stroke="#818CF8" strokeWidth="1" strokeOpacity="0.3" />
      <line x1="7" y1="24" x2="25" y2="24" stroke="#818CF8" strokeWidth="0.7" strokeOpacity="0.15" />
      <line x1="16" y1="14" x2="22" y2="7" stroke="#34D399" strokeWidth="0.8" strokeOpacity="0.25" />
      <line x1="16" y1="14" x2="9" y2="8" stroke="#F59E0B" strokeWidth="0.8" strokeOpacity="0.25" />

      {/* Satellite nodes — smaller, semantic colors */}
      <circle cx="9" cy="8" r="1.8" fill="#F59E0B" opacity="0.7" />   {/* amber — tool */}
      <circle cx="22" cy="7" r="1.5" fill="#34D399" opacity="0.6" />   {/* green — assistant */}
      <circle cx="7" cy="24" r="2.2" fill="#38BDF8" opacity="0.7" />   {/* cyan — user */}
      <circle cx="25" cy="24" r="2" fill="#A78BFA" opacity="0.6" />    {/* purple — thinking */}

      {/* Center node — the observer, indigo accent */}
      <circle cx="16" cy="14" r="3.5" fill="#818CF8" />
      <circle cx="16" cy="14" r="5" fill="none" stroke="#818CF8" strokeWidth="0.6" strokeOpacity="0.3" />

      {/* Inner highlight on center */}
      <circle cx="15" cy="13" r="1.2" fill="white" opacity="0.3" />
    </svg>
  );
}

/**
 * Favicon-sized version — simpler, no glow, bolder strokes
 */
export function HindsightFavicon({ size = 16 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <line x1="7" y1="24" x2="16" y2="14" stroke="#818CF8" strokeWidth="2" strokeOpacity="0.5" />
      <line x1="25" y1="24" x2="16" y2="14" stroke="#818CF8" strokeWidth="2" strokeOpacity="0.5" />

      <circle cx="7" cy="24" r="3" fill="#38BDF8" />
      <circle cx="25" cy="24" r="3" fill="#A78BFA" />
      <circle cx="16" cy="14" r="4.5" fill="#818CF8" />
      <circle cx="15" cy="13" r="1.5" fill="white" opacity="0.4" />
    </svg>
  );
}
