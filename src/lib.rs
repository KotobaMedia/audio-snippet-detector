mod asd;
mod db;
mod mfcc;
mod util;

use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

use asd::AudioSnippetDetector;
use neon::{prelude::*, types::buffer::TypedArray};

type BoxedASD = JsBox<AudioSnippetDetector>;

fn new(mut cx: FunctionContext) -> JsResult<BoxedASD> {
    let asd = AudioSnippetDetector::new();
    Ok(cx.boxed(asd))
}

fn db_add(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let asd = (**cx.argument::<BoxedASD>(0)?).clone();
    let label = cx.argument::<JsString>(1)?.value(&mut cx);
    let fingerprint = cx.argument::<JsUint8Array>(2)?.as_slice(&cx).to_vec();
    let fingerprint_reader = Cursor::new(fingerprint);
    let mfcc_stream = mfcc::MfccIter::new(Arc::new(Mutex::new(fingerprint_reader)));
    let collect = util::collect_to_array2(mfcc_stream);

    asd.db.lock().unwrap().insert(label, collect.view());

    Ok(cx.undefined())
}

fn stream_next(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let stream = (**cx.argument::<BoxedASD>(0)?).clone();
    let promise = cx
        .task(move || {
            let value = stream.next().unwrap();
            value
        })
        .promise(|mut cx, value| {
            let js_value = cx.string(value);
            Ok(js_value)
        });
    Ok(promise)
}

fn stream_write(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let mut stream = (**cx.argument::<BoxedASD>(0)?).clone();
    let bytes = cx.argument::<JsUint8Array>(1)?.as_slice(&cx).to_vec();

    let promise = cx
        .task(move || {
            let _ = stream.write(bytes);
            ()
        })
        .promise(|mut cx, _value| Ok(cx.undefined()));
    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("new", new)?;
    cx.export_function("db_add", db_add)?;

    cx.export_function("stream_next", stream_next)?;
    cx.export_function("stream_write", stream_write)?;

    Ok(())
}
