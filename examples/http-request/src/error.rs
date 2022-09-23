use core::result::Result;
use std::error::Error;

use wasm_bindgen::JsValue;
pub trait OptionToJsResult<T> {
    fn into_js_result<Message: ToString>(self, with_error: Message) -> Result<T, JsValue>;
}

impl<T> OptionToJsResult<T> for Option<T> {
    fn into_js_result<Message: ToString>(self, with_error: Message) -> Result<T, JsValue> {
        if let Some(t) = self {
            Ok(t)
        } else {
            Err(JsValue::from(with_error.to_string()))
        }
    }
}

pub trait ResultToJsResult<T> {
    fn into_js_result(self) -> Result<T, JsValue>;
}

impl<T, E: Error + ToString> ResultToJsResult<T> for Result<T, E> {
    fn into_js_result(self) -> Result<T, JsValue> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(JsValue::from_str(&e.to_string())),
        }
    }
}
