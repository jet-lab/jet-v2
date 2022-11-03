const path = require('path');
const { ProvidePlugin, DefinePlugin } = require('webpack');
const dotenv = require('dotenv');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;

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
    'process.env': JSON.stringify(dotenv.config().parsed || {})
  })
];

if (process.env.ANALYZE) {
  plugins.push(
    new BundleAnalyzerPlugin({
      analyzerMode: 'server'
    })
  );
}

module.exports = {
  entry: './src/index.tsx',
  output: {
    filename: 'bundle.[name].[contenthash].js',
    path: path.join(__dirname, '/build'),
    clean: true
  },
  optimization: {
    chunkIds: 'named'
  },
  devtool: 'source-map',
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
        test: /\.css$/,
        use: ['style-loader', 'css-loader']
      },
      {
        test: /\.less$/i,
        use: [
          {
            loader: 'style-loader'
          },
          {
            loader: 'css-loader'
          },
          {
            loader: 'less-loader',
            options: {
              lessOptions: {
                javascriptEnabled: true,
                relativeUrls: true
              }
            }
          }
        ]
      },
      {
        test: /\.jsx?$/,
        exclude: /node_modules/,
        loader: 'babel-loader'
      },
      {
        test: /\.tsx?$/,
        exclude: /node_modules/,
        loader: 'ts-loader'
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
      stream: require.resolve("stream-browserify"),
      crypto: require.resolve("crypto-browserify")
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
    asyncWebAssembly: true
  },
  plugins
};
