import { render } from "preact";
import { App } from "./app";

const root = document.getElementById("app");
if (!root) throw new Error("Prosopon glass: #app mount point missing");
render(<App />, root);
