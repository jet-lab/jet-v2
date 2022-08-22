const cypressConfig = {
  e2e: {
    setupNodeEvents(on, config) {
      // implement node event listeners here
    },
    screenshotOnRunFailure: false,
    video: false,
    viewportWidth: 1440,
    viewportHeight: 960,
    env: {
      hideXHR: true
    }
  }
};

export default cypressConfig;
