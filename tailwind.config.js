/** @type {import('tailwindcss').Config} */
export default {
	content: [
		"./index.html",
		"./src/**/*.{js,ts,jsx,tsx}",
	],
	theme: {
		extend: {
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
					'0%': { opacity: '0', transform: 'scale(1.015)' },
					'100%': { opacity: '1', transform: 'scale(1)' }
				},
				bgFadeOut: {
					'0%': { opacity: '1' },
					'100%': { opacity: '0' }
				}
			}
		}
	},
	plugins: [
		require('tailwind-scrollbar'),
	],
}

