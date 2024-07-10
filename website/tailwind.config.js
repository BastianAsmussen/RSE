import flowbitePlugin from 'flowbite/plugin';
import catppuccinPlugin from '@catppuccin/tailwindcss';

/** @type {import('tailwindcss').Config} */
export default {
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		'./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}'
	],
	theme: {
		extend: {}
	},
	plugins: [
		catppuccinPlugin({
			defaultFlavour: 'mocha'
		}),
		flowbitePlugin
	]
};
