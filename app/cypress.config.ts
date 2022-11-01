const cypressConfig = {
  e2e: {
    screenshotOnRunFailure: true,
    video: true,
    viewportWidth: 1280,
    viewportHeight: 720,
    env: {
      hideXHR: true
    },
    baseUrl: 'http://localhost:3000?debug-environment=true',
    defaultCommandTimeout: 60000
  },
};

export default cypressConfig;
