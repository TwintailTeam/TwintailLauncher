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
			}
		}
	},
	plugins: [],
}

