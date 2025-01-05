# audio-snippet-detector
A Node.js module to detect audio features in an streaming audio source. The detector is written in Rust, and provides a Writable / Async Iterator interface to NodeJS code.

> [!WARNING]
> This is very, very alpha code. The interfaces may change at any time. The audio matching logic will also be changing heavily.

At this time, the detector code assumes 16-bit 16kHz signed-integer raw audio data. Providing anything else will probably result in garbage output.

## How to use

```typescript
import { AudioSnippetDetector } from "audio-snippet-detector";
const detector = new AudioSnippetDetector();

// add the sounds you want to detect
detector.add_database('chime', fs.readFileSync('../audio.raw'));

// now, you're ready to detect sounds
const streamingInput = fs.createReadStream('../some-large-file.raw');
streamingInput.pipe(detector);
for await (const item of detector) {
  console.log(`I detected ${item.label} (score: ${item.score})`);
}
```

> [!TIP]
> `score` is an arbitrary value between 0 and 1. Values closer to 1 represent high similarity, but the scale is not linear. Use with caution.

## The matching algorithm

...I will write this later...

## Building

Building this package requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

To run the build, run:

```sh
$ npm run build
```

This command uses the [@neon-rs/cli](https://www.npmjs.com/package/@neon-rs/cli) utility to assemble the binary Node addon from the output of `cargo`.
