const fs = require('fs');
const { updateProjects, updateNPMVersion, PROJECT_TYPE } = require('./release');
const packageJson = require("./package.json");

const projects = [
    {
        filename: '../../extension/addon/manifest.json',
        type: PROJECT_TYPE.MANIFEST
    },
    {
        filename: '../../extension/platform/chrome/manifest.json',
        type: PROJECT_TYPE.MANIFEST
    },
    {
        filename: '../../extension/package.json',
        type: PROJECT_TYPE.NPM
    },
    {
        filename: '../../services/zana/Cargo.toml',
        type: PROJECT_TYPE.RUST_CRATE
    },
    {
        filename: '../../services/zana_lambda/Cargo.toml',
        type: PROJECT_TYPE.RUST_CRATE
    },
    {
        filename: '../../deployment/zana_aws/pom.xml',
        type: PROJECT_TYPE.MAVEN
    }
];

const [,, ...args] = process.argv;

// Get the current and new version
const version = packageJson.version;
const releaseType = args[0];

let newVersion;
const [major, minor, patch] = version.split('.');
if (releaseType === 'minor') {
    newVersion = `${major}.${parseInt(minor) + 1}.${patch}`;
} else {
    newVersion = `${major}.${minor}.${parseInt(patch) + 1}`;
}

// Update the version file
fs.writeFileSync('../../VERSION', newVersion, 'utf8');

// Update the version for the release package
updateNPMVersion(newVersion);

updateProjects(newVersion, projects);
console.log('All projects updated');
console.log(`New version: ${newVersion}`);
