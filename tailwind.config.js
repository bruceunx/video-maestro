import typography from "@tailwindcss/typography";
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      animation: {
        "slide-in-bottom": "slide-in-bottom 0.3s ease-out",
        "overlay-show": "overlay-show 0.3s cubic-bezier(0.16, 1, 0.3, 1)",
        "content-show": "content-show 0.3s cubic-bezier(0.16, 1, 0.3, 1)",
      },
      keyframes: {
        "slide-in-bottom": {
          "0%": { transform: "translateY(100%)", opacity: "0" },
          "100%": { transform: "translateY(0)", opacity: "1" },
        },
        "overlay-show": {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        "content-show": {
          "0%": { opacity: "0", transform: "translate(-50%, -48%) scale(.96)" },
          "100%": { opacity: "1", transform: "translate(-50%, -50%) scale(1)" },
        },
      },
    },
  },
  plugins: [typography],
};
