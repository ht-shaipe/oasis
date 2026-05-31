import { defineConfig, presetUno, presetAttributify, presetIcons, transformerVariantGroup } from "unocss";

export default defineConfig({
  presets: [
    presetUno(),
    presetAttributify(),
    presetIcons({
      scale: 1.2,
      warn: true,
    }),
  ],
  transformers: [
    transformerVariantGroup(),
  ],
  shortcuts: {
    "flex-center": "flex items-center justify-center",
    "flex-col-center": "flex flex-col items-center justify-center",
  },
  theme: {
    colors: {
      primary: {
        DEFAULT: "#42b883",
        hover: "#369a6d",
      },
    },
    spacing: {
      35: "140px",
    },
  },
});