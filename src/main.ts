import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { setLocale, useI18n } from "./i18n";
import "highlight.js/styles/tokyo-night-dark.css";
import "./assets/main.css";

// Register ldrs loaders
import { quantum, dotStream, tailspin, waveform } from "ldrs";

quantum.register();
dotStream.register();
tailspin.register();
waveform.register();

const app = createApp(App);
app.use(createPinia());
async function mountApp() {
  try { await setLocale(useI18n().locale.value); }
  catch { await setLocale("ru"); }
  app.mount("#app");
}
void mountApp();
