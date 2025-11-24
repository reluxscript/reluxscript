/**
 * Path Helpers
 *
 * Builds path keys for template maps
 */

/**
 * Build path key for template map
 * Example: div[0].h1[0].text → "div[0].h1[0]"
 *
 * @param tag_name - HTML tag name (e.g., "div", "h1")
 * @param index - Element index at this level
 * @param parent_path - Array of parent indices
 */
pub fn build_path_key(tag_name: &Str, index: i32, parent_path: &Vec<i32>) -> Str {
    let mut parent_keys = vec![];

    // Build parent path from indices
    for i in 0..parent_path.len() {
        parent_keys.push(format!("[{}]", parent_path[i]));
    }

    let parent_str = parent_keys.join(".");
    let full_path = if parent_str.is_empty() {
        format!("{}[{}]", tag_name, index)
    } else {
        format!("{}.{}[{}]", parent_str, tag_name, index)
    };

    full_path
}

/**
 * Build attribute path key
 * Example: div[0].@style or div[1].@className
 *
 * @param tag_name - HTML tag name
 * @param index - Element index at this level
 * @param parent_path - Array of parent indices
 * @param attr_name - Attribute name (e.g., "style", "className")
 */
pub fn build_attribute_path_key(
    tag_name: &Str,
    index: i32,
    parent_path: &Vec<i32>,
    attr_name: &Str
) -> Str {
    let mut parent_keys = vec![];

    for i in 0..parent_path.len() {
        parent_keys.push(format!("[{}]", parent_path[i]));
    }

    let parent_str = parent_keys.join(".");
    let full_path = if parent_str.is_empty() {
        format!("{}[{}].@{}", tag_name, index, attr_name)
    } else {
        format!("{}.{}[{}].@{}", parent_str, tag_name, index, attr_name)
    };

    full_path
}
