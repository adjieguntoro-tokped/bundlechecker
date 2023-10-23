import { checkBundlerSync } from "../index.js";

try {
  const { result, summary } = checkBundlerSync({
    configPath: "./package.json",
    compression: "", // can be "brotli"
    reporter: "",
    silent: true,
  });
  console.log({ result, summary });
} catch (err) {
  console.error(err);
}
