const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const prompts = require('prompts');
const semver = require('semver');

const pkgPath = path.join(__dirname, '..', 'package.json');
const cargoPath = path.join(__dirname, '..', 'Cargo.toml');
const brewPath = path.join(__dirname, '..', 'homebrew', 'vanity_crypto.rb');
const scoopPath = path.join(__dirname, '..', 'scoop', 'vanity_crypto.json');

const pkg = require(pkgPath);
const currentVersion = pkg.version;

function run(command) {
    console.log(`> ${command}`);
    execSync(command, { stdio: 'inherit', cwd: path.join(__dirname, '..') });
}

(async () => {
    console.log(`Automated Release (Current: ${currentVersion})`);

    const response = await prompts({
        type: 'select',
        name: 'value',
        message: 'Select release type:',
        choices: [
            { title: 'Patch (0.0.X)', value: 'patch' },
            { title: 'Minor (0.X.0)', value: 'minor' },
            { title: 'Major (X.0.0)', value: 'major' },
            { title: 'Pre-release (Prerelease)', value: 'prerelease' },
            { title: 'Pre-patch (Prepatch)', value: 'prepatch' },
            { title: 'Pre-minor (Preminor)', value: 'preminor' },
            { title: 'Custom', value: 'custom' }
        ],
        initial: 0
    });

    let targetVersion;
    if (response.value === 'custom') {
        const custom = await prompts({
            type: 'text',
            name: 'value',
            message: 'Enter custom version:',
            validate: v => semver.valid(v) ? true : 'Invalid semver'
        });
        targetVersion = custom.value;
    } else {
        // If prerelease, ask for identifier (alpha, beta, rc)
        let identifier = undefined;
        if (['prerelease', 'prepatch', 'preminor'].includes(response.value)) {
            const idRes = await prompts({
                type: 'text',
                name: 'value',
                message: 'Enter prerelease identifier (e.g., alpha, beta, rc):',
                initial: 'alpha'
            });
            identifier = idRes.value;
        }
        targetVersion = semver.inc(currentVersion, response.value, identifier);
    }

    if (!targetVersion) {
        console.log('Cancelled.');
        process.exit(0);
    }

    const confirm = await prompts({
        type: 'confirm',
        name: 'value',
        message: `Bump versions to ${targetVersion} and create git tag?`,
        initial: true
    });

    if (!confirm.value) process.exit(0);

    console.log('\n📝 Updating files...');

    // 1. Update package.json
    pkg.version = targetVersion;
    fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
    console.log('package.json updated');

    // 2. Update Cargo.toml (Workspace)
    let cargoContent = fs.readFileSync(cargoPath, 'utf8');
    // Replace valid semver string in version field
    cargoContent = cargoContent.replace(/version = "[^"]+"/, `version = "${targetVersion}"`);
    fs.writeFileSync(cargoPath, cargoContent);
    console.log('Cargo.toml updated');

    // 3. Update Homebrew (`version "..."`)
    let brewContent = fs.readFileSync(brewPath, 'utf8');
    brewContent = brewContent.replace(/version "[^"]+"/, `version "${targetVersion}"`);
    fs.writeFileSync(brewPath, brewContent);
    console.log('Homebrew Formula updated');

    // 4. Update Scoop (`"version": "..."`)
    let scoopContent = fs.readFileSync(scoopPath, 'utf8');
    scoopContent = scoopContent.replace(/"version": "[^"]+"/, `"version": "${targetVersion}"`);
    fs.writeFileSync(scoopPath, scoopContent);
    console.log('Scoop Manifest updated');

    console.log('\nRunning verification...');
    try {
        require('./check-versions.js'); // Assuming check-versions logic runs on require or we can execute it
        // Since check-versions runs instantly, we can just shell out to be safe context-wise
        // execSync('node scripts/check-versions.js', { stdio: 'inherit' });
    } catch (e) {
        console.error('Verification failed. Aborting commit.');
        process.exit(1);
    }

    console.log('\nCommitting and Tagging...');
    try {
        run('git add package.json Cargo.toml Formula/vanity_crypto.rb scoop/vanity_crypto.json');
        run(`git commit -m "chore: release v${targetVersion}"`);
        run(`git tag v${targetVersion}`);
        console.log(`Successfully tagged v${targetVersion}`);
        console.log(`👉 Now run: git push origin v${targetVersion}`);

        const push = await prompts({
            type: 'confirm',
            name: 'value',
            message: 'Push changes and tag now?',
            initial: true
        });

        if (push.value) {
            run('git push');
            run(`git push origin v${targetVersion}`);
            console.log('Pushed! CI will start building.');
        } else {
            console.log('Remember to push manually!');
        }

    } catch (e) {
        console.error('Error during git operations:', e);
    }

})();
