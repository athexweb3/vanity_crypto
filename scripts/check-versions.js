const fs = require('fs');
const path = require('path');

const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const RESET = '\x1b[0m';

console.log('🔍 Verifying version consistency...');

// 1. Read package.json
const pkgPath = path.join(__dirname, '..', 'package.json');
const pkg = require(pkgPath);
console.log(`📦 package.json version: ${pkg.version}`);

// 2. Read Cargo.toml
const cargoPath = path.join(__dirname, '..', 'Cargo.toml');
const cargoContent = fs.readFileSync(cargoPath, 'utf8');

// Simple regex to find [workspace.package] version
// Looks for: version = "x.y.z" inside the file. 
// Regex targets workspace version definition
// or we can be more robust. The file has `[workspace.package]` block.
const versionMatch = cargoContent.match(/\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/);

if (!versionMatch) {
    console.error(`${RED}❌ Could not parse version from Cargo.toml${RESET}`);
    process.exit(1);
}

const cargoVersion = versionMatch[1];
console.log(`🦀 Cargo.toml version:  ${cargoVersion}`);

// 3. Read Homebrew Formula
const brewPath = path.join(__dirname, '..', 'Formula', 'vanity_crypto.rb');
const brewContent = fs.readFileSync(brewPath, 'utf8');
const brewMatch = brewContent.match(/version "([^"]+)"/);
if (!brewMatch) {
    console.error(`${RED}❌ Could not parse version from Brew formula${RESET}`);
    process.exit(1);
}
const brewVersion = brewMatch[1];
console.log(`🍺 Brew formula version: ${brewVersion}`);

// 4. Read Scoop Manifest
const scoopPath = path.join(__dirname, '..', 'scoop', 'vanity_crypto.json');
const scoop = require(scoopPath);
console.log(`🍨 Scoop manifest version: ${scoop.version}`);

// 5. Compare All
let hasMismatch = false;

if (pkg.version !== cargoVersion) {
    console.error(`${RED}❌ Mismatch: Cargo.toml (${cargoVersion})${RESET}`);
    hasMismatch = true;
}
if (pkg.version !== brewVersion) {
    console.error(`${RED}❌ Mismatch: Brew formula (${brewVersion})${RESET}`);
    hasMismatch = true;
}
if (pkg.version !== scoop.version) {
    console.error(`${RED}❌ Mismatch: Scoop manifest (${scoop.version})${RESET}`);
    hasMismatch = true;
}

if (hasMismatch) {
    console.error(`${RED}⚠️  Versions do not match package.json (${pkg.version})${RESET}`);
    console.error(`${RED}Publishing aborted. Please sync ALL versions before releasing.${RESET}`);
    process.exit(1);
}

console.log(`${GREEN}✅ All versions match! (${pkg.version})${RESET}`);
process.exit(0);
