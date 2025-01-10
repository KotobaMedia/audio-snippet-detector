import { Writable } from "node:stream";
import * as native from "./load.cjs";

declare module "./load.cjs" {
  /// An opaque type representing a native context.
  type Context = unknown;

  function new_ctx(): Context;
  function db_add(ctx: Context, label: string, data: Uint8Array): undefined;
  type StreamValue = { label: string, score: number };
  function stream_next(ctx: Context): Promise<{value: StreamValue, done: boolean}>;
  function stream_write(ctx: Context, data: Uint8Array): undefined;
  function stream_close(ctx: Context): undefined;
}

export class AudioSnippetDetector extends Writable {
  private _ctx: native.Context;

  constructor() {
    super();
    this._ctx = native.new_ctx();
  }

  _write(chunk: Uint8Array, encoding: string, callback: (error?: Error | null) => void) {
    native.stream_write(this._ctx, chunk);
    callback();
  }

  _final(callback: (error?: Error | null) => void) {
    native.stream_close(this._ctx);
    callback();
  }

  add_database(label: string, data: Uint8Array) {
    native.db_add(this._ctx, label, data);
  }

  [Symbol.asyncIterator]() {
    return {
      next: () => native.stream_next(this._ctx),
    };
  }
}
