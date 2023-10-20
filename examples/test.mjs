import { checkBundlerSync } from "../index.js";

checkBundlerSync({
  configPath: "./package.json",
  compression: "", // can be "brotli"
  reporter: "",
});
