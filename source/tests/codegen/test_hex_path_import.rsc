/// Simple test to verify module import works for hex_path.rsc

use "./hex_path.rsc" { HexPathGenerator };

plugin TestHexPathImport {
    fn test_basic_usage() -> Str {
        let mut gen = HexPathGenerator::default();
        let path1 = gen.next("");
        let path2 = gen.next("");

        format!("{}, {}", path1, path2)
    }

    fn test_build_path() -> Str {
        let gen = HexPathGenerator::default();
        gen.build_path("parent", "child")
    }
}
