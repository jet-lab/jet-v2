const cypressConfig = {
  e2e: {
    screenshotOnRunFailure: false,
    video: false,
    viewportWidth: 1440,
    viewportHeight: 960,
    env: {
      hideXHR: true
    },
    baseUrl: 'http://localhost:3000'
  }
};

export default cypressConfig;
