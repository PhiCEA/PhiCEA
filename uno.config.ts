import {
  defineConfig,
  presetWind3,
  presetAttributify,
  presetIcons,
} from "unocss";
import { presetAnimations } from "unocss-preset-animations";
import type { PresetWind3Theme as Theme } from "unocss";

export default defineConfig<Theme>({
  presets: [
    presetWind3(),
    presetAttributify(),
    presetIcons({}),
    presetAnimations(),
  ],
});
