const path = require('path');
const { ProvidePlugin, DefinePlugin } = require('webpack');
const dotenv = require('dotenv');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;
const SwcMinifyWebpackPlugin = require('swc-minify-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

module.exports = (_env, arg) => {
  const plugins = [
    new HtmlWebpackPlugin({
      template: path.join(__dirname, '/src/index.html')
    }),
    new ProvidePlugin({
      process: 'process/browser'
    }),
    new ProvidePlugin({
      Buffer: ['buffer', 'Buffer']
    }),
    new DefinePlugin({
      'process.env': JSON.stringify(
        dotenv.config().parsed || {
          REACT_APP_LOCAL_DATA_API: process.env.REACT_APP_LOCAL_DATA_API,
          REACT_APP_DEV_DATA_API: process.env.REACT_APP_DEV_DATA_API,
          REACT_APP_DATA_API: process.env.REACT_APP_DATA_API,

          REACT_APP_LOCAL_WS_API: process.env.REACT_APP_LOCAL_WS_API,
          REACT_APP_DEV_WS_API: process.env.REACT_APP_DEV_WS_API,
          REACT_APP_WS_API: process.env.REACT_APP_WS_API,

          REACT_APP_RPC_DEV_TOKEN: process.env.REACT_APP_RPC_DEV_TOKEN,
          REACT_APP_RPC_TOKEN: process.env.REACT_APP_RPC_TOKEN,
          REACT_APP_IP_REGISTRY: process.env.REACT_APP_IP_REGISTRY,
          REACT_APP_LOGROCKET_PROJECT: process.env.REACT_APP_LOGROCKET_PROJECT,
          REACT_APP_ALLOWED_WALLETS: process.env.REACT_APP_ALLOWED_WALLETS
        }
      )
    })
  ];

  if (arg.mode === 'production') {
    plugins.push(new MiniCssExtractPlugin());
  }

  if (process.env.ANALYZE) {
    plugins.push(
      new BundleAnalyzerPlugin({
        analyzerMode: 'server'
      })
    );
  }

  return {
    entry: './src/index.tsx',
    output: {
      filename: 'bundle.[name].[contenthash].js',
      path: path.join(__dirname, '/build'),
      clean: true
    },
    optimization: {
      chunkIds: 'named',
      minimize: true,
      minimizer: [new SwcMinifyWebpackPlugin()]
    },
    devtool: arg.mode === 'production' ? 'eval-source-map' : 'eval-source-map',
    devServer: {
      static: {
        directory: path.resolve(__dirname, 'public')
      },
      port: 3000,
      open: true,
      hot: true,
      compress: true,
      historyApiFallback: true,
      client: {
        logging: 'info'
      }
    },
    module: {
      rules: [
        {
          test: /\.m?js/,
          resolve: {
            fullySpecified: false
          }
        },
        {
          test: /\.css$/i,
          include: path.resolve(__dirname, 'src'),
          use: ['style-loader', 'css-loader', 'postcss-loader']
        },
        {
          test: /\.less$/i,
          include: path.resolve(__dirname, 'src/styles'),
          use: [arg.mode === 'production' ? MiniCssExtractPlugin.loader : 'style-loader', 'css-loader', 'less-loader']
        },
        {
          test: /\.jsx?$/,
          exclude: /node_modules/,
          include: path.resolve(__dirname, 'src'),
          loader: 'swc-loader'
        },
        {
          test: /\.tsx?$/,
          exclude: /node_modules/,
          include: [path.resolve(__dirname, 'src'), path.resolve(__dirname, '../../packages')],
          loader: 'swc-loader',
          options: {
            minify: true,
            jsc: {
              parser: {
                syntax: 'typescript',
                tsx: true,
                dynamicImport: true
              },
              transform: {
                react: {
                  runtime: 'automatic'
                }
              },
              target: 'es2020',
              loose: true,
              externalHelpers: true
            }
          }
        },
        {
          test: /\.svg$/i,
          issuer: /\.[jt]sx?$/,
          use: [
            {
              loader: '@svgr/webpack',
              options: {
                typescript: true,
                ext: 'tsx'
              }
            }
          ]
        }
      ]
    },
    resolve: {
      extensions: ['.tsx', '.ts', '.js'],
      fallback: {
        http: false,
        https: false,
        os: false,
        path: false,
        fs: false,
        assert: false,
        util: false,
        url: false,
        stream: require.resolve('stream-browserify'),
        crypto: require.resolve('crypto-browserify')
      },
      alias: {
        '@components': path.resolve(__dirname, 'src/components'),
        '@state': path.resolve(__dirname, 'src/state'),
        '@styles': path.resolve(__dirname, 'src/styles'),
        '@utils': path.resolve(__dirname, 'src/utils'),
        '@assets': path.resolve(__dirname, 'src/assets'),
        '@views': path.resolve(__dirname, 'src/views')
      }
    },
    experiments: {
      asyncWebAssembly: true,
      syncWebAssembly: true
    },
    plugins
  };
};
