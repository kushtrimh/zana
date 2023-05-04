module.exports = {
    testMatch: [
        '<rootDir>/tests/**/*.test.js',
    ],
    transform: {
        '\\.[jt]sx?$': 'babel-jest'
    },
    transformIgnorePatterns: [
        'node_modules/(?!(tempy|unique-string|crypto-random-string|is-stream)/)',
    ]
}
