import { createRoot } from "react-dom/client";
import { GameRenderer } from "./renderer/GameRenderer";
import { App } from "./ui/App";
import "./ui/styles/theme.css";

async function main() {
  const renderer = new GameRenderer();
  await renderer.init();

  const uiRoot = document.getElementById("ui-root");
  if (!uiRoot) {
    throw new Error("UI root element not found");
  }

  const root = createRoot(uiRoot);
  root.render(<App />);

  console.log("Deterrence initialized - click or press ENTER to start wave");
}

main();
