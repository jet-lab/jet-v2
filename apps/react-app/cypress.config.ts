const cypressConfig = {
  e2e: {
    screenshotOnRunFailure: false,
    video: false,
    viewportWidth: 1_920,
    viewportHeight: 1_080,
    env: {
      hideXHR: true
    },
    baseUrl: 'http://localhost:3000',
    defaultCommandTimeout: 60_000,
    testIsolation: false
  }
};

export default cypressConfig;
