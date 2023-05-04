const cypressConfig = {
  projectId: '3g2ars',
  e2e: {
    screenshotOnRunFailure: true,
    video: true,
    viewportWidth: 1_920,
    viewportHeight: 1_280,
    env: {
      hideXHR: true
    },
    baseUrl: 'http://localhost:3000/?cluster=localnet',
    defaultCommandTimeout: 20_000,
    testIsolation: false
  }
};

export default cypressConfig;
