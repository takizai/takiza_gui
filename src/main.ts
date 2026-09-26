import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
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
app.mount("#app");
