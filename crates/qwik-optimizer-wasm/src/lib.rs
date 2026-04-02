use serde::Serialize;
use serde_wasm_bindgen::from_value;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn transform_modules(config_val: JsValue) -> Result<JsValue, JsValue> {
    let config: qwik_optimizer_oxc::TransformModulesOptions = from_value(config_val)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = qwik_optimizer_oxc::transform_modules(config);
    let serializer = Serializer::new().serialize_maps_as_objects(true);
    result.serialize(&serializer).map_err(|e| JsValue::from(e))
}
