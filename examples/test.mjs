import { checkBundlerSync } from "../index.js";

const { result, summary } = checkBundlerSync({
    configPath: "./package.json",
    compression: "", // can be "brotli"
    reporter: "",
});

console.log({ result, summary });
