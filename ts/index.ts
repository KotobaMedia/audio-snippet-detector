import * as native from "../index.node";

export class AudioSnippetDetector {
  private _ctx: native.Context;
  write: WritableStream;

  constructor() {
    this._ctx = native.new_ctx();
    this.write = new WritableStream<Uint8Array>({
      write: (chunk) => {
        native.stream_write(this._ctx, chunk);
      },
      close: () => {
        native.stream_close(this._ctx);
      },
    });
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
