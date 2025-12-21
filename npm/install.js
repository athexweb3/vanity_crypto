const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const pkg = require('../package.json');
const VERSION = `v${pkg.version}`; // Dynamic version from package.json
const REPO = 'athexweb3/vanity_crypto';
const BIN_NAME = 'vc';

const platform = process.platform;
const arch = process.arch;

let assetName = '';
let binaryExtension = '';

if (platform === 'win32') {
    assetName = `vc-windows-amd64.exe`;
    binaryExtension = '.exe';
} else if (platform === 'darwin') {
    // Assuming arm64 for now as per release.yml, but should support x64 if we build it.
    // RELEASE.yml currently only builds macos-arm64 (apple silicon). 
    // If running on Intel Mac, this might fail unless Rosetta handles it or we add x64 build.
    // For now, strict mapping to what we release.
    if (arch === 'arm64') {
        assetName = `vc-macos-arm64`;
    } else if (arch === 'x64') {
        assetName = `vc-macos-amd64`;
    } else {
        console.error('Unsupported architecture for macOS: ' + arch);
        process.exit(1);
    }
} else if (platform === 'linux') {
    if (arch === 'x64') {
        assetName = `vc-linux-amd64`;
    } else {
        console.error('Unsupported architecture for Linux: ' + arch);
        process.exit(1);
    }
} else {
    console.error('Unsupported platform: ' + platform);
    process.exit(1);
}

const url = `https://github.com/${REPO}/releases/download/${VERSION}/${assetName}`;
const binDir = path.join(__dirname, 'bin');
const binPath = path.join(binDir, BIN_NAME + binaryExtension);

if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir);
}

console.log(`Downloading ${assetName} from ${url}...`);

const file = fs.createWriteStream(binPath);

https.get(url, (response) => {
    if (response.statusCode !== 200) {
        console.error(`Failed to download binary: HTTP ${response.statusCode}`);
        if (response.statusCode === 404) {
            console.error("Release not found. Please ensure the release exists on GitHub.");
        }
        process.exit(1);
    }

    response.pipe(file);

    file.on('finish', () => {
        file.close(() => {
            console.log('Download completed.');
            if (platform !== 'win32') {
                execSync(`chmod +x ${binPath}`);
            }
        });
    });
}).on('error', (err) => {
    fs.unlink(binPath, () => { }); // Delete the file async.
    console.error('Error downloading binary: ' + err.message);
    process.exit(1);
});
