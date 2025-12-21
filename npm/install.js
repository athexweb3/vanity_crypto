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
} else if (platform === 'freebsd') {
    if (arch === 'x64') {
        assetName = `vc-freebsd-amd64`;
    } else {
        console.error(`Unsupported architecture for FreeBSD: ${arch}`);
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

function download(downloadUrl, dest) {
    const file = fs.createWriteStream(dest);

    const request = https.get(downloadUrl, (response) => {
        // Handle Redirects (GitHub Releases -> S3)
        if (response.statusCode === 302 || response.statusCode === 301) {
            console.log(`Following redirect to ${response.headers.location}...`);
            file.close();
            fs.unlinkSync(dest); // Remove empty file from failed attempt
            download(response.headers.location, dest); // Recursive call
            return;
        }

        if (response.statusCode !== 200) {
            console.error(`Failed to download binary: HTTP ${response.statusCode}`);
            if (response.statusCode === 404) {
                console.error(`Release not found for version ${VERSION}. Make sure the GitHub release exists.`);
            }
            file.close();
            fs.unlinkSync(dest);
            process.exit(1);
        }

        response.pipe(file);

        file.on('finish', () => {
            file.close();
            console.log('Download complete!');

            // Make it executable
            if (platform !== 'win32') {
                try {
                    execSync(`chmod +x ${dest}`);
                } catch (e) {
                    console.error('Failed to make binary executable: ' + e.message);
                }
            }
        });
    }).on('error', (err) => {
        fs.unlink(dest, () => { }); // Verify deletion
        console.error('Error downloading binary:', err.message);
        process.exit(1);
    });
}

download(url, binPath);
