const cypressConfig = {
  e2e: {
    screenshotOnRunFailure: false,
    video: false,
    viewportWidth: 1280,
    viewportHeight: 720,
    env: {
      hideXHR: true
    },
    baseUrl: 'http://localhost:3000?debug-environment=true',
    defaultCommandTimeout: 60000
  }
};

export default cypressConfig;
