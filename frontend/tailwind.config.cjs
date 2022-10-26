/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        "./index.html",
        "./src/**/*.{vue,js,ts,jsx,tsx}",
    ],
    theme: {
        colors: {
            transparent: "transparent",
            white: "#ffffff",
            black: "#000000",
            green: "#57F287",
            yellow: "#FEE75C",
            fuchsia: "#EB459E",
            red: "#E53E3E",
            redlight: "#f16767",
            gray: "#a3a3a3",
            graylight: "#cccbcb",
            bg: "#0C0D0F",
            bglight: "#191B1F",
            bglighter: "#303030",
            bglightest: "#464545",
            blurple: "#715DF2",
            blurplelight: "#8B7AF4",
        },
        extend: {},
    },
    plugins: [
        function ({ addVariant }) {
            addVariant("child", "& > *");
        },
    ],
}
