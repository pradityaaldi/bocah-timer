import "@fontsource/plus-jakarta-sans/200.css";
import "@fontsource/plus-jakarta-sans/400.css";
import "@fontsource/plus-jakarta-sans/600.css";
import { mount } from "svelte";
import Overlay from "./Overlay.svelte";

const app = mount(Overlay, { target: document.getElementById("overlay") });

export default app;
