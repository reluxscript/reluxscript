/// Test: JSON Serialization
///
/// ReluxScript should support JSON serialization and deserialization.

use json;
use fs;

plugin JsonSerializationTest {
    /// Test struct for serialization
    struct Template {
        path: Str,
        template: Str,
        bindings: Vec<Str>,
    }

    /// Test struct for config
    struct Config {
        name: Str,
        version: Str,
        features: Vec<Str>,
    }

    /// Nested struct
    struct Component {
        name: Str,
        templates: HashMap<Str, Template>,
        props: Vec<Prop>,
    }

    struct Prop {
        name: Str,
        prop_type: Str,
        optional: bool,
    }

    /// Test basic serialization
    pub fn test_serialize_basic() -> Str {
        let template = Template {
            path: "0.1.2",
            template: "${count}",
            bindings: vec!["count"],
        };

        let json_str = json::to_string(&template);
        return json_str;
    }

    /// Test pretty serialization
    pub fn test_serialize_pretty() -> Str {
        let config = Config {
            name: "my-plugin",
            version: "1.0.0",
            features: vec!["hooks", "jsx", "typescript"],
        };

        let json_str = json::to_string_pretty(&config);
        return json_str;
    }

    /// Test manual JSON object building
    pub fn test_manual_object() -> Str {
        let mut obj = json::object();

        obj.insert("name", json::string("MyComponent"));
        obj.insert("version", json::number(1));
        obj.insert("enabled", json::boolean(true));

        return json::stringify(&obj);
    }

    /// Test manual JSON array building
    pub fn test_manual_array() -> Str {
        let mut arr = json::array();

        arr.push(json::string("item1"));
        arr.push(json::string("item2"));
        arr.push(json::number(42));

        return json::stringify(&arr);
    }

    /// Test file I/O with JSON
    pub fn save_config(config: &Config, path: &Str) {
        let json_str = json::to_string_pretty(config);
        fs::write(path, &json_str);
    }

    /// Test in visitor context
    pub fn visit_program(node: &Program) {
        let mut names: Vec<Str> = vec![];

        // Process components...
        for item in &node.body {
            if matches!(item, FunctionDeclaration) {
                names.push(item.id.name.clone());
            }
        }

        // Write names as JSON
        let json_str = json::to_string_pretty(&names);
        let path = "output/names.json";
        fs::write(&path, &json_str);
    }
}
