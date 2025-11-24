/**
 * Test for utils/style_converter.rsc
 *
 * Tests:
 * - camel_to_kebab
 * - convert_style_value
 * - convert_style_object_to_css
 */

use "../../../reluxscript-plugin-minimact/utils/style_converter.rsc" {
    camel_to_kebab,
    convert_style_value,
    convert_style_object_to_css
};

plugin TestStyleConverter {
    fn test_camel_to_kebab() {
        // Simple conversions
        let result1 = camel_to_kebab("marginTop");
        // result1 should be "margin-top"

        let result2 = camel_to_kebab("backgroundColor");
        // result2 should be "background-color"

        let result3 = camel_to_kebab("fontSize");
        // result3 should be "font-size"

        // Already kebab-case
        let result4 = camel_to_kebab("margin-top");
        // result4 should be "margin-top"

        // Single word
        let result5 = camel_to_kebab("color");
        // result5 should be "color"

        // Multiple capitals
        let result6 = camel_to_kebab("MozBorderRadius");
        // result6 should be "-moz-border-radius"

        // Edge case: empty string
        let result7 = camel_to_kebab("");
        // result7 should be ""
    }

    fn test_convert_style_value_string_literal() {
        // String literal: "12px"
        let expr1 = StringLiteral { value: "12px" };
        let result1 = convert_style_value(&expr1);
        // result1 should be "12px"

        // String literal: "red"
        let expr2 = StringLiteral { value: "red" };
        let result2 = convert_style_value(&expr2);
        // result2 should be "red"
    }

    fn test_convert_style_value_numeric_literal() {
        // Numeric literal: 12 -> "12px"
        let expr1 = NumericLiteral { value: 12.0 };
        let result1 = convert_style_value(&expr1);
        // result1 should be "12px"

        // Numeric literal: 0 -> "0"
        let expr2 = NumericLiteral { value: 0.0 };
        let result2 = convert_style_value(&expr2);
        // result2 should be "0"

        // Numeric literal: 3.14 -> "3.14px"
        let expr3 = NumericLiteral { value: 3.14 };
        let result3 = convert_style_value(&expr3);
        // result3 should be "3.14px"
    }

    fn test_convert_style_value_identifier() {
        // Identifier: auto
        let expr1 = Identifier { name: "auto" };
        let result1 = convert_style_value(&expr1);
        // result1 should be "auto"

        // Identifier: inherit
        let expr2 = Identifier { name: "inherit" };
        let result2 = convert_style_value(&expr2);
        // result2 should be "inherit"
    }

    fn test_convert_style_object_basic() {
        // Create a style object: { color: "red", fontSize: "16px" }
        let style_obj = ObjectExpression {
            properties: vec![
                ObjectProperty {
                    key: Identifier { name: "color" },
                    value: StringLiteral { value: "red" },
                },
                ObjectProperty {
                    key: Identifier { name: "fontSize" },
                    value: StringLiteral { value: "16px" },
                },
            ],
        };

        let result = convert_style_object_to_css(&style_obj);
        // result should be Ok("color: red; font-size: 16px")
    }

    fn test_convert_style_object_with_numbers() {
        // Create a style object: { marginTop: 12, padding: 0 }
        let style_obj = ObjectExpression {
            properties: vec![
                ObjectProperty {
                    key: Identifier { name: "marginTop" },
                    value: NumericLiteral { value: 12.0 },
                },
                ObjectProperty {
                    key: Identifier { name: "padding" },
                    value: NumericLiteral { value: 0.0 },
                },
            ],
        };

        let result = convert_style_object_to_css(&style_obj);
        // result should be Ok("margin-top: 12px; padding: 0")
    }

    fn test_convert_style_object_mixed() {
        // Create a style object: {
        //   backgroundColor: "blue",
        //   width: 100,
        //   display: "flex"
        // }
        let style_obj = ObjectExpression {
            properties: vec![
                ObjectProperty {
                    key: Identifier { name: "backgroundColor" },
                    value: StringLiteral { value: "blue" },
                },
                ObjectProperty {
                    key: Identifier { name: "width" },
                    value: NumericLiteral { value: 100.0 },
                },
                ObjectProperty {
                    key: Identifier { name: "display" },
                    value: StringLiteral { value: "flex" },
                },
            ],
        };

        let result = convert_style_object_to_css(&style_obj);
        // result should be Ok("background-color: blue; width: 100px; display: flex")
    }

    fn test_convert_style_object_empty() {
        // Empty style object
        let style_obj = ObjectExpression {
            properties: vec![],
        };

        let result = convert_style_object_to_css(&style_obj);
        // result should be Ok("")
    }
}
