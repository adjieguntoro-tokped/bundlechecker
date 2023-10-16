import { checkBundlerSync } from "../index.js";

checkBundlerSync({
  configPath: "./package.json",
  compression: "brotli",
});
