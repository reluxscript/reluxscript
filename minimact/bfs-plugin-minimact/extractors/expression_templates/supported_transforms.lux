/**
 * Supported Transforms
 *
 * Defines supported transformation types for expressions
 */

/**
 * Transform metadata
 */
pub struct TransformInfo {
    pub transform_type: Str,
    pub safe: bool,
}

/**
 * Get transform info for a method name
 */
pub fn get_transform_info(method_name: &Str) -> Option<TransformInfo> {
    match method_name.as_str() {
        // Number formatting
        "toFixed" | "toPrecision" | "toExponential" => {
            Some(TransformInfo {
                transform_type: String::from("numberFormat"),
                safe: true,
            })
        }

        // String operations
        "toUpperCase" | "toLowerCase" | "trim" | "substring" | "substr" | "slice" => {
            Some(TransformInfo {
                transform_type: String::from("stringTransform"),
                safe: true,
            })
        }

        // Array operations
        "length" => {
            Some(TransformInfo {
                transform_type: String::from("property"),
                safe: true,
            })
        }

        "join" => {
            Some(TransformInfo {
                transform_type: String::from("arrayTransform"),
                safe: true,
            })
        }

        _ => None
    }
}

/**
 * Check if a method is a supported transform
 */
pub fn is_supported_transform(method_name: &Str) -> bool {
    get_transform_info(method_name).is_some()
}

/**
 * Check if a transform is safe (can be executed without side effects)
 */
pub fn is_safe_transform(method_name: &Str) -> bool {
    if let Some(info) = get_transform_info(method_name) {
        info.safe
    } else {
        false
    }
}
