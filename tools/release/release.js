const fs = require('fs');
const path = require('path');
const {XMLParser, XMLBuilder} = require("fast-xml-parser");

const { execSync } = require('child_process');

const PROJECT_TYPE = {
    MANIFEST: {
        func: updateManifestFile,
    },
    NPM: {
        func: updateNPMProject,
    },
    RUST_CRATE: {
        func: updateRustCrate,
    },
    MAVEN: {
        func: updateMavenProject,
    }
};

function updateProjects(newVersion, projectsToUpdate) {
    return projectsToUpdate
        .forEach(project => project.type.func(newVersion, project.filename));
}

function updateNPMVersion(newVersion) {
    const npmVersionCommand = `npm version ${newVersion} --no-git-tag-version`;
    execSync(npmVersionCommand);
}

function updateManifestFile(newVersion, filename) {
    let content = fs.readFileSync(filename, 'utf8');
    content = JSON.parse(content);
    content.version = newVersion;
    const updatedContent = JSON.stringify(content, null, 2);
    fs.writeFileSync(filename, updatedContent, 'utf8');
}

function updateNPMProject(newVersion, filename) {
    const absolutePath = path.resolve(filename);
    const directory = path.dirname(absolutePath);
    process.chdir(directory);
    updateNPMVersion(newVersion);
    process.chdir(__dirname);
}

function updateRustCrate(newVersion, filename) {
    let content = fs.readFileSync(filename, 'utf8');
    const updatedContent = content.replace(/version = "(.*)"/, `version = "${newVersion}"`);
    fs.writeFileSync(filename, updatedContent, 'utf8');

    const absolutePath = path.resolve(filename);
    const directory = path.dirname(absolutePath);
    process.chdir(directory);
    execSync('cargo update -p zana');
    process.chdir(__dirname);
}

function updateMavenProject(newVersion, filename) {
    let content = fs.readFileSync(filename, 'utf8');
    const parser = new XMLParser();
    content = parser.parse(content);
    content.project.version = newVersion;

    const builder = new XMLBuilder({
        format: true,
        indentBy: '    '
    });
    const updatedContent = builder.build(content);
    fs.writeFileSync(filename, updatedContent, 'utf8');
}

module.exports = {
    updateProjects,
    updateNPMVersion,
    PROJECT_TYPE
}
