/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{vue,js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                gray: {
                    900: '#111111', // Darker background for Obsidian feel
                    800: '#1a1a1a',
                    700: '#2a2a2a',
                    600: '#404040',
                },
                // Linear-like accents
                blue: {
                    500: '#3b82f6',
                    600: '#2563eb',
                },
                orange: {
                    500: '#f97316',
                }
            },
        },
    },
    plugins: [],
}
