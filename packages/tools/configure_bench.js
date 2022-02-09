// Configure benchmarks for sycamore and sycamore-baseline

const { execSync } = require("child_process");
const { writeFileSync } = require("fs");

const exec = (command) => {
    execSync(command, { stdio: "inherit" });
}

console.log("Configuring benchmarks");
// ---

console.log("Building sycamore...");
exec(`cd ./sycamore/examples/js-framework-benchmark && trunk build --release --public-url "/frameworks/keyed/sycamore"`);
console.log("Done building. Copying files...")
exec(`cp -r ./sycamore/examples/js-framework-benchmark/dist ./js-framework-benchmark/frameworks/keyed/sycamore`);

// ---
console.log("Building sycamore-baseline...");
exec(`cd ./sycamore-baseline/examples/js-framework-benchmark && trunk build --release --public-url "/frameworks/keyed/sycamore-baseline"`);
console.log("Done building. Copying files...");
exec(`cp -r ./sycamore/examples/js-framework-benchmark/dist ./js-framework-benchmark/frameworks/keyed/sycamore`);

console.log("Creating package.json files...");

const PACKAGE_JSON = (name, version) =>
    `{
        "name": "js-framework-benchmark-keyed-${name}",
        "version": "1.0.0",
        "description": "Benchmark for Sycamore",
        "license": "ISC",
        "js-framework-benchmark": {
            "frameworkVersion": "${version}"
        },
        "scripts": {},
        "repository": {
            "type": "git",
            "url": "https://github.com/krausest/js-framework-benchmark.git"
        },
        "devDependencies": {}
    }`;

writeFileSync("./js-framework-benchmark/frameworks/keyed/sycamore/package.json", PACKAGE_JSON("sycamore", "head"))
writeFileSync("./js-framework-benchmark/frameworks/keyed/sycamore-baseline/package.json", PACKAGE_JSON("sycamore-baseline", "baseline"))
