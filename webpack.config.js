var path = require('path');
var webpack = require('webpack');

module.exports = {
  entry: './src/frontend/static/js/index.jsx',
  output: { path: __dirname + '/src/frontend/static/dist/', filename: 'bundle.js' },
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
};