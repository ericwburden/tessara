export default {
  content: ["./crates/tessara-web/src/**/*.rs"],
  theme: {
    extend: {
      borderRadius: {
        DEFAULT: "8px"
      },
      colors: {
        tessara: {
          ink: "#0f172a",
          slate: "#334155",
          teal: "#14b8a6",
          orange: "#f59e0b"
        }
      }
    }
  }
};
