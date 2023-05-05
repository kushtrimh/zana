const fs = require('fs');
const {XMLParser} = require("fast-xml-parser");
const path = require('path');
const TOML = require('@iarna/toml')

const {temporaryFile, temporaryDirectory} = require('tempy');

const {updateProjects, PROJECT_TYPE} = require('../release');

describe('update projects', () => {
    const newVersion = '0.2.0';

    it('should update the version in manifest.json files', () => {
        const tempFile = temporaryFile();

        const testManifest = fs.readFileSync('./tests/files/manifest-test.json', 'utf8');
        fs.writeFileSync(tempFile, testManifest);
        const projects = [
            {
                filename: tempFile,
                type: PROJECT_TYPE.MANIFEST
            }
        ]
        updateProjects(newVersion, projects);
        const changedManifest = fs.readFileSync(tempFile, 'utf8');
        const changedManifestJson = JSON.parse(changedManifest);
        expect(changedManifestJson.version).toBe(newVersion);
    })

    it('should update the version for Node.js projects', () => {
        const tempDir = temporaryDirectory();
        const tempFile = path.join(tempDir, '/package.json');

        const testPackage = fs.readFileSync('./tests/files/package-test.json', 'utf8');
        fs.writeFileSync(tempFile, testPackage);
        const projects = [
            {
                filename: tempFile,
                type: PROJECT_TYPE.NPM
            }
        ]
        updateProjects(newVersion, projects);
        const changedPackage = fs.readFileSync(tempFile, 'utf8');
        const changedPackageJson = JSON.parse(changedPackage);
        expect(changedPackageJson.version).toBe(newVersion);
    })

    it('should update the version for Rust crates', () => {
        const tempDir = temporaryDirectory();
        fs.mkdirSync(path.join(tempDir, '/src'));

        const cargoTempFile = path.join(tempDir, '/Cargo.toml');
        const mainTempFile = path.join(tempDir, '/src/main.rs');

        const testCargo = fs.readFileSync('./tests/files/Cargo-test.toml', 'utf8');
        const testMain = fs.readFileSync('./tests/files/main-test.rs', 'utf8');

        fs.writeFileSync(cargoTempFile, testCargo);
        fs.writeFileSync(mainTempFile, testMain);
        const projects = [
            {
                filename: cargoTempFile,
                type: PROJECT_TYPE.RUST_CRATE
            }
        ];
        updateProjects(newVersion, projects);
        const changedCargo = fs.readFileSync(cargoTempFile, 'utf8');
        const changedCargoToml = TOML.parse(changedCargo);
        expect(changedCargoToml.package.version).toBe(newVersion);
    })

    it('should update the version for Maven projects', () => {
        const tempFile = temporaryFile();

        const tempPom = fs.readFileSync('./tests/files/pom-test.xml', 'utf8');
        fs.writeFileSync(tempFile, tempPom);
        const projects = [
            {
                filename: tempFile,
                type: PROJECT_TYPE.MAVEN
            }
        ]
        updateProjects(newVersion, projects);
        const changedPom = fs.readFileSync(tempFile, 'utf8');
        const parser = new XMLParser();
        const changedPomXml = parser.parse(changedPom);
        expect(changedPomXml.project.version).toBe(newVersion);
    })
})
