use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub async fn transform_modules(config: serde_json::Value) -> Result<serde_json::Value> {
    let opts: qwik_optimizer_oxc::TransformModulesOptions = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let result = qwik_optimizer_oxc::transform_modules(opts);
    serde_json::to_value(&result).map_err(|e| napi::Error::from_reason(e.to_string()))
}
