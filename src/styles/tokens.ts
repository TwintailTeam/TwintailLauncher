/**
 * Design Tokens - Centralized theme configuration
 *
 * Usage: Import these tokens for consistent styling across components.
 * These values should be used instead of hardcoded colors/sizes.
 */

// =============================================================================
// COLORS
// =============================================================================

export const colors = {
  // Surface colors (darkest to lightest)
  surface: {
    base: '#09090b',        // Main app background
    elevated: '#0c0c0c',    // Cards, modals, popups
    overlay: '#0f0f11',     // Hover states, elevated cards
    input: 'rgb(39 39 42 / 0.6)', // zinc-800/60 - Input backgrounds
  },

  // Primary accent (purple)
  accent: {
    primary: '#9333ea',     // purple-600
    hover: '#a855f7',       // purple-500
    muted: '#7c3aed',       // purple-700
    glow: 'rgba(147, 51, 234, 0.4)',
    subtle: 'rgba(147, 51, 234, 0.2)',
  },

  // Semantic colors
  semantic: {
    success: '#22c55e',     // green-500
    successHover: '#16a34a', // green-600
    warning: '#f59e0b',     // amber-500
    warningHover: '#d97706', // amber-600
    danger: '#ef4444',      // red-500
    dangerHover: '#dc2626', // red-600
    info: '#3b82f6',        // blue-500
    infoHover: '#2563eb',   // blue-600
  },

  // Text colors
  text: {
    primary: 'rgb(255 255 255 / 0.9)',
    secondary: 'rgb(255 255 255 / 0.7)',
    muted: 'rgb(255 255 255 / 0.5)',
    disabled: 'rgb(255 255 255 / 0.3)',
  },

  // Border colors
  border: {
    subtle: 'rgb(255 255 255 / 0.05)',
    default: 'rgb(255 255 255 / 0.1)',
    strong: 'rgb(255 255 255 / 0.2)',
  },
} as const;

// =============================================================================
// TYPOGRAPHY
// =============================================================================

export const typography = {
  // Font sizes with line heights
  size: {
    xs: { fontSize: '0.75rem', lineHeight: '1rem' },      // 12px
    sm: { fontSize: '0.875rem', lineHeight: '1.25rem' },  // 14px
    base: { fontSize: '1rem', lineHeight: '1.5rem' },     // 16px
    lg: { fontSize: '1.125rem', lineHeight: '1.75rem' },  // 18px
    xl: { fontSize: '1.25rem', lineHeight: '1.75rem' },   // 20px
    '2xl': { fontSize: '1.5rem', lineHeight: '2rem' },    // 24px
    '3xl': { fontSize: '1.875rem', lineHeight: '2.25rem' }, // 30px
    '4xl': { fontSize: '2.25rem', lineHeight: '2.5rem' }, // 36px
  },

  // Font weights
  weight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },
} as const;

// =============================================================================
// SPACING
// =============================================================================

export const spacing = {
  0: '0',
  1: '0.25rem',   // 4px
  2: '0.5rem',    // 8px
  3: '0.75rem',   // 12px
  4: '1rem',      // 16px
  5: '1.25rem',   // 20px
  6: '1.5rem',    // 24px
  8: '2rem',      // 32px
  10: '2.5rem',   // 40px
  12: '3rem',     // 48px
  16: '4rem',     // 64px
} as const;

// =============================================================================
// BORDER RADIUS
// =============================================================================

export const radius = {
  none: '0',
  sm: '0.25rem',    // 4px
  md: '0.375rem',   // 6px
  lg: '0.5rem',     // 8px
  xl: '0.75rem',    // 12px
  '2xl': '1rem',    // 16px
  full: '9999px',
} as const;

// =============================================================================
// SHADOWS
// =============================================================================

export const shadows = {
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1)',
  '2xl': '0 25px 50px -12px rgb(0 0 0 / 0.25)',
  glow: {
    purple: '0 0 15px rgba(147, 51, 234, 0.5)',
    green: '0 0 15px rgba(34, 197, 94, 0.5)',
    blue: '0 0 15px rgba(59, 130, 246, 0.5)',
    amber: '0 0 15px rgba(245, 158, 11, 0.5)',
    red: '0 0 15px rgba(239, 68, 68, 0.5)',
  },
  card: '0 8px 32px rgba(0, 0, 0, 0.4)',
} as const;

// =============================================================================
// ANIMATION TIMING
// =============================================================================

export const animation = {
  // Duration
  duration: {
    instant: '0ms',
    fast: '150ms',
    normal: '200ms',
    moderate: '300ms',
    slow: '400ms',
    slower: '600ms',
  },

  // Easing
  easing: {
    default: 'ease-out',
    in: 'ease-in',
    inOut: 'ease-in-out',
    linear: 'linear',
  },

  // Stagger delays for lists
  stagger: {
    fast: 50,    // ms between items
    normal: 100,
    slow: 150,
  },
} as const;

// =============================================================================
// Z-INDEX LAYERS
// =============================================================================

export const zIndex = {
  base: 0,
  dropdown: 10,
  sticky: 20,
  overlay: 30,
  modal: 40,
  popover: 50,
  tooltip: 60,
} as const;

// =============================================================================
// COMMON TAILWIND CLASS COMBINATIONS
// =============================================================================

/**
 * Pre-built class combinations for common UI patterns.
 * Use these for consistency across components.
 */
export const tw = {
  // Card styles
  card: 'bg-zinc-900/85 border border-white/10 rounded-xl p-5 hover:border-white/15 transition-colors',
  cardInteractive: 'bg-zinc-900/85 border border-white/10 rounded-xl p-5 hover:border-white/20 hover:bg-zinc-900/90 transition-all cursor-pointer',
  cardFlat: 'bg-zinc-900/85 border border-white/5 rounded-xl p-5',
  cardElevated: 'bg-[#0c0c0c] border border-white/10 rounded-2xl shadow-2xl',

  // Input field styles
  input: 'bg-zinc-900/90 border border-white/10 rounded-lg px-4 py-2.5 text-white placeholder:text-zinc-500 focus:outline-none focus:ring-2 focus:ring-purple-500/50 focus:border-purple-500/50 transition-all',
  inputError: 'bg-zinc-900/90 border border-red-500/50 rounded-lg px-4 py-2.5 text-white placeholder:text-zinc-500 focus:outline-none focus:ring-2 focus:ring-red-500/50 transition-all',

  // Button styles
  button: 'px-4 py-2.5 rounded-lg font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-black',
  buttonPrimary: 'bg-purple-600 hover:bg-purple-500 text-white shadow-lg hover:shadow-purple-500/25',
  buttonSecondary: 'bg-zinc-800 hover:bg-zinc-700 text-white border border-white/10',
  buttonDanger: 'bg-red-600 hover:bg-red-500 text-white shadow-lg hover:shadow-red-500/25',
  buttonGhost: 'hover:bg-white/5 text-zinc-300 hover:text-white',

  // Icon button (circular)
  iconButton: 'p-2.5 rounded-full border border-white/20 shadow-lg disabled:brightness-75 disabled:saturate-100 transition-colors focus:outline-none focus:ring-2',

  // Gradient divider
  divider: 'w-8 h-px bg-gradient-to-r from-transparent via-white/20 to-transparent',

  // Section header
  sectionHeader: 'text-lg font-semibold text-white/90 mb-4',

  // Label
  label: 'text-sm font-medium text-white/70',

  // Help text / description
  helpText: 'text-sm text-zinc-400',

  // Glass effect
  glass: 'bg-black/85 border border-white/10',

  // Focus ring (for accessibility)
  focusRing: 'focus:ring-2 focus:ring-purple-400/60 focus:outline-none',
} as const;

// =============================================================================
// COMPONENT-SPECIFIC TOKENS
// =============================================================================

export const components = {
  // Sidebar dimensions
  sidebar: {
    width: '4rem',      // 64px / w-16
    iconSize: '3rem',   // 48px / w-12
    padding: '0.5rem',  // 8px / p-2
  },

  // Modal dimensions
  modal: {
    maxWidth: {
      sm: '24rem',      // 384px
      md: '28rem',      // 448px
      lg: '32rem',      // 512px
      xl: '56rem',      // 896px (max-w-4xl)
      '2xl': '72rem',   // 1152px (max-w-6xl)
    },
    maxHeight: '90vh',
  },

  // ActionBar positioning
  actionBar: {
    bottom: '2rem',     // bottom-8
    right: '4rem',      // right-16
  },

  // GameInfoOverlay positioning
  gameInfo: {
    bottom: '2rem',     // bottom-8
    left: '6rem',       // left-24
  },
} as const;
