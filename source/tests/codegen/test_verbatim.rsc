// Test verbatim code blocks for platform-specific code
plugin TestVerbatim {

fn test_babel_verbatim() {
    babel! {
        const recast = require('recast');
        const fs = require('fs');
        console.log('This is Babel-only code');
    }
}

fn test_swc_verbatim() {
    swc! {
        println!("This is SWC-only Rust code");
        let x: i32 = 42;
    }
}

fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
    // Mix ReluxScript with verbatim blocks
    let tag_name = &node.opening_element.name;

    babel! {
        // Add recast manipulation
        const keyAttr = t.jsxAttribute(
            t.jsxIdentifier('key'),
            t.stringLiteral('generated-key')
        );
        node.openingElement.attributes.push(keyAttr);
    }

    swc! {
        // SWC-specific AST manipulation
        node.opening.attrs.push(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident::new("key".into(), DUMMY_SP)),
            value: Some(JSXAttrValue::Lit(Lit::Str("generated-key".into())))
        });
    }
}

}
