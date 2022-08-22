const config = {
  clearMocks: true,
  collectCoverage: false,
  coverageThreshold: {
    global: {
      lines: 1.2,
      branches: 2.1,
      functions: 1.7,
      statements: 1.1
    }
  },
  collectCoverageFrom: ['**/*.{js,jsx,ts,tsx}'],
  coverageDirectory: 'coverage',
  setupFilesAfterEnv: ['<rootDir>/setupTests.ts', '<rootDir>/src/__mocks__/localStorage.ts']
};

export default config;
