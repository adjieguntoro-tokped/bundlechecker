import { checkBundlerSync } from "../index.js";

try {
  const { result, summary } = checkBundlerSync({
    configPath: "./package.json",
    compression: "brotli", // can be "brotli"
    silent: true,
  });
  console.log({ result, summary });
} catch (err) {
  console.error(err);
}
