mod asd;
mod db;
mod mfcc;
mod util;

use std::{cell::RefCell, io::Cursor};

use asd::AudioSnippetDetector;
use neon::{prelude::*, types::buffer::TypedArray};

type BoxedASD = JsBox<RefCell<AudioSnippetDetector>>;

fn new(mut cx: FunctionContext) -> JsResult<BoxedASD> {
    let asd = RefCell::new(AudioSnippetDetector::new());
    Ok(cx.boxed(asd))
}

fn db_add(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let asd = (**cx.argument::<BoxedASD>(0)?).clone();
    let asd = asd.borrow_mut();
    let label = cx.argument::<JsString>(1)?.value(&mut cx);

    let fingerprint = cx.argument::<JsUint8Array>(2)?.as_slice(&cx).to_vec();
    let fingerprint_reader = Cursor::new(fingerprint);
    let mfcc_stream = mfcc::MfccIter::new(mfcc::MfccSource::Reader(Box::new(fingerprint_reader)));
    let collect = util::collect_to_array2(mfcc_stream);

    asd.db.lock().unwrap().insert(label, collect.view());

    Ok(cx.undefined())
}

fn stream_next(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let asd = (**cx.argument::<BoxedASD>(0)?).clone();
    let promise = cx
        .task(move || {
            let asd = asd.borrow();
            match asd.next() {
                Ok(value) => Some(value),
                Err(_) => None,
            }
        })
        .promise(|mut cx, next| {
            let obj = cx.empty_object();
            match next {
                Some(value) => {
                    let js_value = cx.string(value);
                    obj.set(&mut cx, "value", js_value)?;
                }
                None => {
                    let true_value = cx.boolean(true);
                    obj.set(&mut cx, "done", true_value)?;
                }
            }
            Ok(obj)
        });
    Ok(promise)
}

fn stream_write(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let stream = (**cx.argument::<BoxedASD>(0)?).clone();
    let bytes = cx.argument::<JsUint8Array>(1)?.as_slice(&cx).to_vec();

    let promise = cx
        .task(move || {
            let stream = stream.borrow();
            let _ = stream.write(bytes);
            ()
        })
        .promise(|mut cx, _value| Ok(cx.undefined()));
    Ok(promise)
}

fn stream_close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let stream = (**cx.argument::<BoxedASD>(0)?).clone();
    let stream = stream.borrow_mut();
    stream.close();
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("new", new)?;
    cx.export_function("db_add", db_add)?;

    cx.export_function("stream_next", stream_next)?;
    cx.export_function("stream_write", stream_write)?;
    cx.export_function("stream_close", stream_close)?;

    Ok(())
}
