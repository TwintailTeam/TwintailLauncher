/** @type {import('tailwindcss').Config} */
export default {
	content: [
		"./index.html",
		"./src/**/*.{js,ts,jsx,tsx}",
	],
	theme: {
		extend: {
			// Design token colors
			colors: {
				surface: {
					base: '#09090b',
					elevated: '#0c0c0c',
					overlay: '#0f0f11',
				},
			},
			// Standardized border radius
			borderRadius: {
				'card': '1rem',      // 16px - for cards and modals
				'input': '0.75rem',  // 12px - for inputs and buttons
				'button': '0.75rem', // 12px - for buttons
				'pill': '9999px',    // Full rounded for pills/tags
			},
			// Box shadows including glow effects
			boxShadow: {
				'card': '0 8px 32px rgba(0, 0, 0, 0.4)',
				'glow-purple': '0 0 15px rgba(147, 51, 234, 0.5)',
				'glow-green': '0 0 15px rgba(34, 197, 94, 0.5)',
				'glow-blue': '0 0 15px rgba(59, 130, 246, 0.5)',
				'glow-amber': '0 0 15px rgba(245, 158, 11, 0.5)',
			},
			// Standardized transition durations
			transitionDuration: {
				'fast': '150ms',
				'normal': '200ms',
				'moderate': '300ms',
				'slow': '400ms',
				'slower': '600ms',
			},
			// Typography scale for consistent text sizing
			fontSize: {
				// Labels, captions
				'label': ['0.75rem', { lineHeight: '1rem', fontWeight: '500' }],
				// Small body text, descriptions
				'body-sm': ['0.875rem', { lineHeight: '1.25rem' }],
				// Default body text
				'body': ['1rem', { lineHeight: '1.5rem' }],
				// Section headers, card titles
				'heading-sm': ['1.125rem', { lineHeight: '1.75rem', fontWeight: '600' }],
				// Page section headers
				'heading': ['1.25rem', { lineHeight: '1.75rem', fontWeight: '600' }],
				// Modal titles
				'heading-lg': ['1.5rem', { lineHeight: '2rem', fontWeight: '700' }],
				// Page titles, hero text
				'title': ['1.875rem', { lineHeight: '2.25rem', fontWeight: '700' }],
				// Large hero titles
				'title-lg': ['2.25rem', { lineHeight: '2.5rem', fontWeight: '700' }],
			},
			transitionProperty: {
				'height': 'height',
				'padding': 'padding',
				'max-height': 'max-height',
				'border-bottom-right-radius': 'border-bottom-right-radius',
				'border-bottom-left-radius': 'border-bottom-left-radius',
			},
			animation: {
				'fadeIn': 'fadeInOpacity 300ms ease-out',
				'slideUp': 'slideUp 600ms ease-out',
				'slideInLeft': 'slideInLeft 300ms ease-out',
				'slideInRight': 'slideInRight 400ms ease-out',
				'slideOutLeft': 'slideOutLeft 200ms ease-in',
				'slideUpToPosition': 'slideUpToPosition 300ms ease-in',
				'slideDownToPosition': 'slideDownToPosition 300ms ease-out',
				'shimmer': 'shimmer 2s infinite',
				'bg-fade-in': 'bgFadeIn 300ms ease-out',
				'bg-fade-out': 'bgFadeOut 300ms ease-out forwards',
				'zoom-in': 'zoomIn 200ms ease-out both',
				'zoom-out': 'zoomOut 200ms ease-in forwards',
				'scaleIn': 'scaleIn 200ms ease-out both',
				'bounce-slow': 'bounceSlow 3s ease-in-out infinite',
				'backdrop-in': 'backdropIn 200ms ease-out both',
			},
			keyframes: {
				fadeInOpacity: {
					'0%': { opacity: '0' },
					'100%': { opacity: '1' }
				},
				slideUp: {
					'0%': { opacity: '0', transform: 'translateY(20px)' },
					'100%': { opacity: '1', transform: 'translateY(0)' }
				},
				slideInLeft: {
					'0%': { opacity: '0', transform: 'translateX(-20px)' },
					'100%': { opacity: '1', transform: 'translateX(0)' }
				},
				slideInRight: {
					'0%': { opacity: '0', transform: 'translateX(20px)' },
					'100%': { opacity: '1', transform: 'translateX(0)' }
				},
				slideOutLeft: {
					'0%': { opacity: '1', transform: 'translateX(0)' },
					'100%': { opacity: '0', transform: 'translateX(-20px)' }
				},
				slideUpToPosition: {
					'0%': { transform: 'translateY(0)' },
					'100%': { transform: 'translateY(var(--target-y))' }
				},
				slideDownToPosition: {
					'0%': { transform: 'translateY(var(--target-y))' },
					'100%': { transform: 'translateY(0)' }
				},
				shimmer: {
					'0%': { transform: 'translateX(-100%)' },
					'100%': { transform: 'translateX(100%)' }
				},
				bgFadeIn: {
					'0%': { opacity: '0.01', transform: 'scale(1.015)' },
					'100%': { opacity: '1', transform: 'scale(1)' }
				},
				bgFadeOut: {
					'0%': { opacity: '1' },
					'100%': { opacity: '0.01' }
				},
				zoomIn: {
					'0%': { opacity: '0.01', transform: 'scale(0.95)' },
					'100%': { opacity: '1', transform: 'scale(1)' }
				},
				zoomOut: {
					'0%': { opacity: '1', transform: 'scale(1)' },
					'100%': { opacity: '0.01', transform: 'scale(0.95)' }
				},
				scaleIn: {
					'0%': { opacity: '0.01', transform: 'scale(0.95)' },
					'100%': { opacity: '1', transform: 'scale(1)' }
				},
				bounceSlow: {
					'0%, 100%': { transform: 'translateY(0)' },
					'50%': { transform: 'translateY(-10px)' }
				},
				backdropIn: {
					'0%': { opacity: '0.01' },
					'100%': { opacity: '1' }
				}
			}
		}
	},
	plugins: [
		require('tailwind-scrollbar'),
	],
}
