const CracoLessPlugin = require('craco-less');

module.exports = {
  plugins: [
    {
      plugin: CracoLessPlugin,
      options: {
        lessLoaderOptions: {
          lessOptions: {
            javascriptEnabled: true
          }
        }
      }
    }
  ],
  webpack: {
    configure: {
      module: {
        rules: [
          {
            test: /\.mjsx?$/,
            include: /node_modules/,
            type: 'javascript/auto'
          }
        ]
      }
    }
  }
};
