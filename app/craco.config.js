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
    configure: (config, { env, paths }) => {
      const wasmExtensionRegExp = /\.wasm$/;
      config.resolve.extensions.push('.wasm');

      config.module.rules.forEach(rule => {
        (rule.oneOf || []).forEach(oneOf => {
          if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
            oneOf.exclude.push(wasmExtensionRegExp);
          }
        });
      });

      config.module.rules.push({
        test: /\.mjsx?$/,
        include: /node_modules/,
        type: 'javascript/auto'
      });

      config.module.rules.push({
        test: /\.js$/,
        include: /@solana/,
        loader: 'babel-loader',
        options: {}
      });
      return config;
    }
  }
};
