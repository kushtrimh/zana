module.exports = {
    testEnvironment: 'jsdom',
    setupFiles: ['./tests/dukagjinibooks/__mocks__/setup.js'],
    rootDir: '../../',
    testMatch: [
        "<rootDir>/tests/dukagjinibooks/**/*.test.{js,jsx}",
    ],
}
