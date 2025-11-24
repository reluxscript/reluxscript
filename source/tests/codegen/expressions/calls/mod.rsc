/**
 * Call expression handlers
 *
 * Re-exports all specialized call handlers for method calls and function calls
 */

// Math methods
pub use "./handle_math_calls.rsc" { handle_math_calls };

// Global functions
pub use "./handle_global_functions.rsc" {
    handle_encode_uri_component,
    handle_set_state,
    handle_fetch,
    handle_alert,
    handle_string_constructor
};

// Promise methods
pub use "./handle_promise_calls.rsc" {
    handle_promise_resolve,
    handle_promise_reject
};

// Object/Date/console methods
pub use "./handle_object_calls.rsc" {
    handle_object_keys,
    handle_date_now,
    handle_console_log
};

// String methods
pub use "./handle_string_methods.rsc" {
    handle_to_fixed,
    handle_to_locale_string,
    handle_to_lower_case,
    handle_to_upper_case,
    handle_trim,
    handle_substring,
    handle_pad_start,
    handle_pad_end,
    handle_response_json
};

// Array methods
pub use "./handle_array_methods.rsc" {
    handle_map,
    handle_state_setters,
    ComponentContext,
    UseStateInfo,
    MapContext
};

// Optional map
pub use "./handle_optional_map.rsc" { handle_optional_map };
