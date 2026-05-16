// CodeScope (cs) — npm binary wrapper
// Downloads the appropriate binary on install

const { createReadStream } = require('fs');
const { createInterface } = require('readline');
const { createWriteStream, existsSync, mkdirSync, chmodSync } = require('fs');
const { join, dirname } = require('path');
const { get } = require('https');
const { pipeline } = require('stream/promises');
const { execSync } = require('child_process');
const { tmpdir } = require('os');
const { platform, arch } = require('process');

const GITHUB_REPO = 'Arga-Wicaksono/codescope';
const VERSION = require('./package.json').version;
const BIN_DIR = join(__dirname, 'bin');

function getPlatform() {
    const p = platform();
    const a = arch();
    if (p === 'darwin' && a === 'arm64') return 'cs-aarch64-macos';
    if (p === 'darwin' && a === 'x64') return 'cs-x86_64-macos';
    if (p === 'linux' && a === 'arm64') return 'cs-aarch64-linux-musl';
    if (p === 'linux' && a === 'x64') return 'cs-x86_64-linux-musl';
    if (p === 'win32' && a === 'x64') return 'cs-x86_64-windows.zip';
    throw new Error(`Unsupported platform: ${p} ${a}`);
}

function getExtension(platform) {
    return platform.endsWith('.zip') ? '.zip' : '.tar.gz';
}

function getBinaryName(platform) {
    return platform.endsWith('.zip') ? 'cs.exe' : 'cs';
}

async function download(url, dest) {
    return new Promise((resolve, reject) => {
        get(url, (res) => {
            if (res.statusCode === 302 || res.statusCode === 301) {
                download(res.headers.location, dest).then(resolve).catch(reject);
                return;
            }
            if (res.statusCode !== 200) {
                reject(new Error(`HTTP ${res.statusCode}`));
                return;
            }
            const file = createWriteStream(dest);
            res.pipe(file);
            file.on('finish', () => { file.close(); resolve(); });
        }).on('error', reject);
    });
}

async function main() {
    const platformName = getPlatform();
    const ext = getExtension(platformName);
    const binaryName = getBinaryName(platformName);
    const url = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${platformName}`;
    const archivePath = join(tmpdir(), `codescope-${VERSION}${ext}`);

    console.log(`Downloading CodeScope v${VERSION} for ${platformName}...`);

    if (!existsSync(BIN_DIR)) mkdirSync(BIN_DIR, { recursive: true });

    await download(url, archivePath);

    if (ext === '.tar.gz') {
        execSync(`tar xzf "${archivePath}" -C "${BIN_DIR}"`, { stdio: 'pipe' });
    } else {
        execSync(`powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${BIN_DIR}' -Force"`, { stdio: 'pipe' });
    }

    chmodSync(join(BIN_DIR, binaryName), 0o755);
    console.log('CodeScope installed successfully!');
}

main().catch(err => {
    console.error('Installation failed:', err.message);
    process.exit(1);
});
