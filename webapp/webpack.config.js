const path = require("path")
const { ProvidePlugin, DefinePlugin } = require("webpack")
const dotenv = require("dotenv")

const HtmlWebpackPlugin = require("html-webpack-plugin")
module.exports = {
  entry: "./src/index.tsx",
  output: {
    path: path.join(__dirname, "/build"),
    filename: "bundle.js"
  },
  devtool: "source-map",
  devServer: {
    port: 3000,
    historyApiFallback: true
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: ["style-loader", "css-loader"]
      },
      {
        test: /\.less$/i,
        use: [
          {
            loader: "style-loader"
          },
          {
            loader: "css-loader"
          },
          {
            loader: "less-loader",
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
        loader: "babel-loader"
      },
      {
        test: /\.tsx?$/,
        exclude: /node_modules/,
        loader: "ts-loader"
      },
      {
        test: /\.svg$/i,
        issuer: /\.[jt]sx?$/,
        use: [
          {
            loader: "@svgr/webpack",
            options: {
              typescript: true,
              ext: "tsx"
            }
          }
        ]
      }
    ]
  },
  resolve: {
    extensions: [".tsx", ".ts", ".js"],
    fallback: {
      http: false,
      https: false,
      os: false,
      path: false,
      fs: false,
      assert: false,
      util: false,
      url: false
    }
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: path.join(__dirname, "/src/index.html")
    }),
    new ProvidePlugin({
      process: "process/browser"
    }),
    new ProvidePlugin({
      Buffer: ["buffer", "Buffer"]
    }),
    new DefinePlugin({
      "process.env": JSON.stringify(dotenv.config().parsed)
    })
  ]
}
