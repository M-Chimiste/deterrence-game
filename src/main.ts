import { GameRenderer } from "./renderer/GameRenderer";

async function main() {
  const renderer = new GameRenderer();
  await renderer.init();

  console.log("Deterrence initialized \u2014 click or press ENTER to start wave");
}

main();
