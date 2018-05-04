var path = require('path');
var webpack = require('webpack');
var CopyWebpackPlugin = require('copy-webpack-plugin')

module.exports = {
  entry: './src/frontend/static/js/index.jsx',
  output: { path: __dirname + '/www/dist/', filename: 'bundle.js' },
  module: {
    loaders: [
      {
        test: /.jsx?$/,
        loader: 'babel-loader',
        exclude: /node_modules/,
        query: {
          presets: ['env', 'react'],
          plugins: ['transform-class-properties']
        }
      }
    ]
  },
  plugins: [
    new CopyWebpackPlugin([
      { from:  __dirname + '/src/frontend/static/index.html', to: __dirname + '/www/index.html' }
    ])
  ]
};