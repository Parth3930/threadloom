const puppeteer = require('puppeteer');
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const targetPath = process.argv[2] || '../yew-app/dist';
const targetName = path.basename(path.dirname(targetPath));

async function runBenchmarkIteration() {
    console.log(`Starting server for ${targetName}...`);
    const serverProcess = spawn('cmd.exe', ['/c', 'npx', '-y', 'serve', targetPath, '-p', '8080'], { stdio: 'ignore' });
    
    await new Promise(resolve => setTimeout(resolve, 2000));

    console.log("Launching browser...");
    const browser = await puppeteer.launch();
    const page = await browser.newPage();
    
    const runResults = {};

    async function measure(id, name) {
        console.log(`Measuring: ${name}`);
        const start = Date.now();
        await page.click(`#${id}`);
        await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
        const end = Date.now();
        runResults[name] = end - start;
    }

    try {
        await page.goto('http://localhost:8080');
        await page.waitForSelector('#run');

        await measure('run', 'create_1000');
        await measure('runlots', 'create_10000');
        await measure('add', 'append_1000');
        await measure('update', 'update_every_10th');
        await measure('swaprows', 'swap_rows');
        await measure('clear', 'clear');

    } catch (e) {
        console.error(e);
    } finally {
        await browser.close();
        spawn('taskkill', ['/pid', serverProcess.pid, '/f', '/t']);
        // wait for port release
        await new Promise(resolve => setTimeout(resolve, 1000));
    }
    return runResults;
}

function calculateStats(runs) {
    const categories = Object.keys(runs[0]);
    const stats = {};
    for (const cat of categories) {
        const values = runs.map(r => r[cat]).sort((a, b) => a - b);
        const min = values[0];
        const max = values[values.length - 1];
        const mid = Math.floor(values.length / 2);
        const median = values.length % 2 !== 0 ? values[mid] : (values[mid - 1] + values[mid]) / 2;
        stats[cat] = { min, median, max };
    }
    return stats;
}

async function run() {
    const NUM_RUNS = 10;
    const allRuns = [];
    for (let i = 0; i < NUM_RUNS; i++) {
        console.log(`\n--- RUN ${i + 1}/${NUM_RUNS} ---`);
        const runRes = await runBenchmarkIteration();
        allRuns.push(runRes);
    }
    const finalStats = calculateStats(allRuns);
    const outFile = `${targetName}_results.json`;
    fs.writeFileSync(path.join(__dirname, outFile), JSON.stringify(finalStats, null, 2));
    console.log(`\nFinal Statistics saved to ${outFile}`);
    console.log(finalStats);
}

run();
