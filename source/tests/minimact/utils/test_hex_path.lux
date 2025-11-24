/**
 * Test for utils/hex_path.rsc
 *
 * Tests:
 * - HexPathGenerator creation
 * - next() path generation
 * - build_path()
 * - parse_path()
 * - get_depth()
 * - get_parent_path()
 * - is_ancestor_of()
 * - generate_path_between()
 */

use "../../../reluxscript-plugin-minimact/utils/hex_path.rsc" { HexPathGenerator };

plugin TestHexPath {
    fn test_hex_path_generation() {
        let mut generator = HexPathGenerator::default();

        // Generate root level paths
        let path1 = generator.next("");
        let path2 = generator.next("");
        let path3 = generator.next("");

        // Should generate sequential hex codes
        // path1 should be "10000000"
        // path2 should be "20000000"
        // path3 should be "30000000"

        // Generate child paths
        let child1 = generator.next(&path1);
        let child2 = generator.next(&path1);

        // Build full paths
        let full_child1 = generator.build_path(&path1, &child1);
        let full_child2 = generator.build_path(&path1, &child2);

        // full_child1 should be like "10000000.10000000"
        // full_child2 should be like "10000000.20000000"
    }

    fn test_path_parsing() {
        let generator = HexPathGenerator::default();

        // Parse a simple path
        let segments1 = generator.parse_path("10000000");
        // segments1.len() should be 1

        // Parse a nested path
        let segments2 = generator.parse_path("10000000.20000000.30000000");
        // segments2.len() should be 3
        // segments2[0] should be "10000000"
        // segments2[1] should be "20000000"
        // segments2[2] should be "30000000"
    }

    fn test_depth_calculation() {
        let generator = HexPathGenerator::default();

        let depth1 = generator.get_depth("");
        // depth1 should be 0

        let depth2 = generator.get_depth("10000000");
        // depth2 should be 1

        let depth3 = generator.get_depth("10000000.20000000");
        // depth3 should be 2

        let depth4 = generator.get_depth("10000000.20000000.30000000");
        // depth4 should be 3
    }

    fn test_parent_path() {
        let generator = HexPathGenerator::default();

        // Root has no parent
        let parent1 = generator.get_parent_path("");
        // parent1 should be None

        // Single segment
        let parent2 = generator.get_parent_path("10000000");
        // parent2 should be Some("")

        // Nested path
        let parent3 = generator.get_parent_path("10000000.20000000.30000000");
        // parent3 should be Some("10000000.20000000")
    }

    fn test_ancestry() {
        let generator = HexPathGenerator::default();

        // Direct ancestor
        let is_ancestor1 = generator.is_ancestor_of("10000000", "10000000.20000000");
        // is_ancestor1 should be true

        // Deep ancestor
        let is_ancestor2 = generator.is_ancestor_of("10000000", "10000000.20000000.30000000");
        // is_ancestor2 should be true

        // Not ancestor (sibling)
        let is_ancestor3 = generator.is_ancestor_of("10000000", "20000000");
        // is_ancestor3 should be false

        // Same path
        let is_ancestor4 = generator.is_ancestor_of("10000000", "10000000");
        // is_ancestor4 should be false

        // Empty root is ancestor of all
        let is_ancestor5 = generator.is_ancestor_of("", "10000000.20000000");
        // is_ancestor5 should be true
    }

    fn test_reset() {
        let mut generator = HexPathGenerator::default();

        let path1 = generator.next("");
        let path2 = generator.next("");

        // Reset root counter
        generator.reset("");

        let path3 = generator.next("");
        // path3 should be same as path1 (counter reset)

        // Reset all counters
        generator.reset_all();

        let path4 = generator.next("");
        // path4 should be same as path1
    }

    fn test_path_between() {
        // Generate a path between two existing paths
        let mid_path = HexPathGenerator::generate_path_between("10000000", "20000000");
        // mid_path should be lexicographically between the two

        // mid_path > "10000000" should be true
        // mid_path < "20000000" should be true
    }

    fn test_gap_checking() {
        // Check if there's sufficient gap between paths
        let has_gap1 = HexPathGenerator::has_sufficient_gap("10000000", "20000000", 1000);
        // has_gap1 should be true (large gap)

        let has_gap2 = HexPathGenerator::has_sufficient_gap("10000000", "10000001", 100);
        // has_gap2 should be false (small gap)
    }
}
