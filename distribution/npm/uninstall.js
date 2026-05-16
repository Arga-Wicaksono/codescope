const { existsSync, unlinkSync } = require('fs');
const { join, dirname } = require('path');
const { platform } = require('process');

function getBinaryName() {
    return platform() === 'win32' ? 'cs.exe' : 'cs';
}

const binPath = join(__dirname, '..', 'bin', getBinaryName());

if (existsSync(binPath)) {
    unlinkSync(binPath);
    console.log('CodeScope binary removed.');
}
