/**
 * Hex Path Generator for Minimact
 *
 * Generates lexicographically sortable, insertion-friendly paths using hex codes.
 *
 * Benefits:
 * - No renumbering needed when inserting elements
 * - String comparison works for sorting
 * - Billions of slots between any two elements
 * - Easy to visualize tree structure
 *
 * Example:
 *   div [10000000]
 *     span [10000000.10000000]
 *     span [10000000.20000000]
 *     p [10000000.30000000]
 *   section [20000000]
 */

/**
 * Hex Path Generator
 *
 * Tracks counters per parent path to generate sequential hex codes
 */
pub struct HexPathGenerator {
    pub gap: i32,
    pub counters: HashMap<Str, i32>,
}

impl HexPathGenerator {
    /**
     * Create a new HexPathGenerator
     *
     * @param gap - Spacing between elements (default: 0x10000000 = 268,435,456)
     */
    pub fn new(gap: i32) -> HexPathGenerator {
        HexPathGenerator {
            gap,
            counters: HashMap::new(),
        }
    }

    /**
     * Create with default gap
     */
    pub fn default() -> HexPathGenerator {
        HexPathGenerator::new(0x10000000)
    }

    /**
     * Generate next hex code for a given parent path
     *
     * @param parent_path - Parent path (e.g., "10000000" or "10000000.1")
     * @returns Next hex code (compact: 1, 2, 3...a, b, c...10, 11...)
     */
    pub fn next(&mut self, parent_path: &Str) -> Str {
        // Get or initialize counter for this parent path
        let counter = self.counters.get(parent_path).unwrap_or(&0);
        let new_counter = counter + 1;
        self.counters.insert(parent_path.clone(), new_counter);

        // For root level (empty parent), use gap-based spacing for components
        // For child elements, use simple sequential hex (1, 2, 3...a, b, c...)
        let hex_value = if parent_path.is_empty() {
            (new_counter * self.gap).to_string(16)
        } else {
            new_counter.to_string(16)
        };

        // Truncate trailing zeroes to keep paths compact (1 instead of 10000000)
        Self::trim_trailing_zeros(&hex_value)
    }

    /**
     * Build full path by joining parent and child
     *
     * @param parent_path - Parent path
     * @param child_hex - Child hex code
     * @returns Full path (e.g., "10000000.20000000")
     */
    pub fn build_path(&self, parent_path: &Str, child_hex: &Str) -> Str {
        if parent_path.is_empty() {
            child_hex.clone()
        } else {
            format!("{}.{}", parent_path, child_hex)
        }
    }

    /**
     * Parse path into segments
     *
     * @param path - Full path (e.g., "10000000.20000000.30000000")
     * @returns Array of hex segments
     */
    pub fn parse_path(&self, path: &Str) -> Vec<Str> {
        path.split(".").map(|s| s.to_string()).collect()
    }

    /**
     * Get depth of a path (number of segments)
     *
     * @param path - Full path
     * @returns Depth (0 for root, 1 for first level, etc.)
     */
    pub fn get_depth(&self, path: &Str) -> i32 {
        if path.is_empty() {
            0
        } else {
            self.parse_path(path).len() as i32
        }
    }

    /**
     * Get parent path
     *
     * @param path - Full path
     * @returns Parent path or None if root
     */
    pub fn get_parent_path(&self, path: &Str) -> Option<Str> {
        if let Some(last_dot_index) = path.rfind('.') {
            if last_dot_index > 0 {
                Some(path[0..last_dot_index].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /**
     * Check if path1 is ancestor of path2
     *
     * @param ancestor_path - Potential ancestor
     * @param descendant_path - Potential descendant
     * @returns true if ancestor_path is an ancestor of descendant_path
     */
    pub fn is_ancestor_of(&self, ancestor_path: &Str, descendant_path: &Str) -> bool {
        let prefix = format!("{}.", ancestor_path);
        descendant_path.starts_with(&prefix)
    }

    /**
     * Reset counter for a specific parent (useful for testing)
     *
     * @param parent_path - Parent path to reset
     */
    pub fn reset(&mut self, parent_path: &Str) {
        self.counters.remove(parent_path);
    }

    /**
     * Reset all counters (useful for testing)
     */
    pub fn reset_all(&mut self) {
        self.counters.clear();
    }

    /**
     * Generate a path between two existing paths (for future insertion)
     *
     * @param path1 - First path
     * @param path2 - Second path
     * @returns Midpoint path
     */
    pub fn generate_path_between(path1: &Str, path2: &Str) -> Str {
        let segments1: Vec<Str> = path1.split(".").map(|s| s.to_string()).collect();
        let segments2: Vec<Str> = path2.split(".").map(|s| s.to_string()).collect();

        // Find common prefix length
        let mut common_length = 0;
        let min_len = segments1.len().min(segments2.len());

        while common_length < min_len && segments1[common_length] == segments2[common_length] {
            common_length += 1;
        }

        // Get the differing segments
        let seg1 = if common_length < segments1.len() {
            i32::from_str_radix(&segments1[common_length], 16).unwrap_or(0)
        } else {
            0
        };

        let seg2 = if common_length < segments2.len() {
            i32::from_str_radix(&segments2[common_length], 16).unwrap_or(0)
        } else {
            0
        };

        // Generate midpoint
        let midpoint = (seg1 + seg2) / 2;
        let new_segment = format!("{:08x}", midpoint);

        // Build new path
        if common_length > 0 {
            let prefix = segments1[0..common_length].join(".");
            format!("{}.{}", prefix, new_segment)
        } else {
            new_segment
        }
    }

    /**
     * Check if there's sufficient gap between two paths
     *
     * @param path1 - First path
     * @param path2 - Second path
     * @param min_gap - Minimum required gap (default: 0x00100000)
     * @returns true if gap is sufficient
     */
    pub fn has_sufficient_gap(path1: &Str, path2: &Str, min_gap: i32) -> bool {
        let seg1_str = path1.split(".").last().unwrap_or("0");
        let seg2_str = path2.split(".").last().unwrap_or("0");

        let seg1 = i32::from_str_radix(seg1_str, 16).unwrap_or(0);
        let seg2 = i32::from_str_radix(seg2_str, 16).unwrap_or(0);

        (seg2 - seg1).abs() > min_gap
    }

    /**
     * Helper: Trim trailing zeros from hex string
     * "10000000" -> "1"
     * "20000000" -> "2"
     * "0" -> "0"
     */
    fn trim_trailing_zeros(hex_str: &Str) -> Str {
        let trimmed = hex_str.trim_end_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    }
}
