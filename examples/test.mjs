import { checkBundlerSync } from "../index.js";

const { result } = checkBundlerSync({
  configPath: "./package.json",
  compression: "",
});

console.log(result);
