// tailwind.config.js
module.exports = {
    darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: '#2563eb', // blue-600
        success: '#10b981', // emerald-500
        danger: '#ef4444',  // red-500
      },
    },
  },
  plugins: [],
}