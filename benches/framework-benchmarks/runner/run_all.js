const { execSync } = require('child_process');
console.log("Running React...");
execSync('node bench.js ../react-app/dist', { stdio: 'inherit' });
console.log("Running Next.js...");
execSync('node bench.js ../nextjs-app/out', { stdio: 'inherit' });
console.log("Running Dioxus...");
execSync('node bench.js ../dioxus-app/dist', { stdio: 'inherit' });
console.log("Done!");
