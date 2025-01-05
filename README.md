# audio-snippet-detector
A Node.js module to detect audio features in an streaming audio source.

> [!WARNING]
> This is very, very alpha code. The interfaces may change at any time.

## Building

Building this package requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

To run the build, run:

```sh
$ npm run build
```

This command uses the [@neon-rs/cli](https://www.npmjs.com/package/@neon-rs/cli) utility to assemble the binary Node addon from the output of `cargo`.
