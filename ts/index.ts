import { Writable } from "node:stream";
import * as native from "../index.node";

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
