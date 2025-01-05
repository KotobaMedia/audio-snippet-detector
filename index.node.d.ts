/// An opaque type representing a native context.
export type Context = unknown;

export function new_ctx(): Context;
export function db_add(ctx: Context, label: string, data: Uint8Array): undefined;
export type StreamValue = { label: string, score: number };
export function stream_next(ctx: Context): Promise<{value: StreamValue, done: boolean}>;
export function stream_write(ctx: Context, data: Uint8Array): undefined;
export function stream_close(ctx: Context): undefined;
