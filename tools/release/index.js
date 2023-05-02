const fsp = require('fs').promises;
const { exec } = require('child_process');
const packageJson = require('./package.json');

const [,, ...args] = process.argv;

const version = packageJson.version;
const releaseType = args[0];

let newVersion;
const [major, minor, patch] = version.split('.');
if (releaseType === 'minor') {
    newVersion = `${major}.${parseInt(minor) + 1}.${patch}`;
} else {
    newVersion = `${major}.${minor}.${parseInt(patch) + 1}`;
}

const npmVersionCommand = `npm version ${newVersion} --no-git-tag-version`;
exec(npmVersionCommand, (err, stdout, stderr) => {
    if (err) {
        console.error(err);
        return;
    }
    console.log(`stdout: ${stdout}`);
    if (stderr) {
        console.error(`stderr: ${stderr}`);
    }
});

/*
- Update the version in package.json for extension package
- Update the version in the addon firefox manifest
- Update the version for the chrome manifest
- Update the version for the zana rust crate
- Update the version for the zana lambda rust crate
- Update the version for the zana aws maven project
 */
