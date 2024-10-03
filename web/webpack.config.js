import { resolve } from 'path';
import HtmlWebpackPlugin from 'html-webpack-plugin';
import webpack from 'webpack';
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";
import process from 'process';

const isDev = process.env.NODE_ENV === 'development';
const isProd = !isDev;

export default {
    entry: './index.js',
    output: {
        path: resolve(import.meta.dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: resolve(import.meta.dirname, 'template.html'),
            inject: true,
            minify: {
                removeComments: false,
                minifyJS: isProd,
                minifyURLs: isProd,
                collapseWhitespace: isProd,
            }
        }),
        new WasmPackPlugin({
            crateDirectory: resolve(import.meta.dirname, ".")
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
    },
}

