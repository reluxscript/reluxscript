use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "createElement")]
    pub fn createElement(tagName: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "setAttribute")]
    pub fn setAttribute(element: JsValue, name: JsValue, value: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "addEventListener")]
    pub fn addEventListener(element: JsValue, eventType: JsValue, callback: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "fetch")]
    pub fn fetch(url: JsValue, options: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = "log")]
    pub fn log(message: JsValue) -> JsValue;

}
