use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TestManifest {
    test: TestInfo,
    plugin: PluginInfo,
    input: InputInfo,
    expected: ExpectedInfo,
}

#[derive(Debug, Deserialize)]
struct TestInfo {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct PluginInfo {
    source: String,
}

#[derive(Debug, Deserialize)]
struct InputInfo {
    source: String,
}

#[derive(Debug, Deserialize)]
struct ExpectedInfo {
    source: String,
}

struct IntegrationTest {
    name: String,
    test_dir: PathBuf,
    manifest: TestManifest,
}

impl IntegrationTest {
    fn load(test_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let manifest_path = test_dir.join("test.toml");
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: TestManifest = toml::from_str(&manifest_content)?;

        let name = manifest.test.name.clone();

        Ok(Self {
            name,
            test_dir,
            manifest,
        })
    }

    fn plugin_path(&self) -> PathBuf {
        self.test_dir.join(&self.manifest.plugin.source)
    }

    fn input_path(&self) -> PathBuf {
        self.test_dir.join(&self.manifest.input.source)
    }

    fn expected_path(&self) -> PathBuf {
        self.test_dir.join(&self.manifest.expected.source)
    }

    fn compile_to_babel(&self) -> Result<String, Box<dyn std::error::Error>> {
        let plugin_path = self.plugin_path();
        let output_dir = std::env::temp_dir().join(format!("relux_babel_{}", self.name));
        fs::create_dir_all(&output_dir)?;

        println!("Compiling {} to Babel...", self.name);

        // Run relux compiler to generate Babel plugin
        let output = Command::new("cargo")
            .args(&["run", "--bin", "relux", "--", "build", "--target", "babel"])
            .arg(&plugin_path)
            .arg("-o")
            .arg(&output_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Babel compilation failed: {}", stderr).into());
        }

        let babel_plugin_path = output_dir.join("index.js");
        Ok(babel_plugin_path.to_string_lossy().to_string())
    }

    fn compile_to_swc(&self) -> Result<String, Box<dyn std::error::Error>> {
        let plugin_path = self.plugin_path();
        let output_dir = std::env::temp_dir().join(format!("relux_swc_{}", self.name));
        fs::create_dir_all(&output_dir)?;

        println!("Compiling {} to SWC...", self.name);

        // Run relux compiler to generate SWC plugin
        let output = Command::new("cargo")
            .args(&["run", "--bin", "relux", "--", "build", "--target", "swc"])
            .arg(&plugin_path)
            .arg("-o")
            .arg(&output_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("SWC compilation failed: {}", stderr).into());
        }

        let swc_plugin_path = output_dir.join("lib.rs");
        Ok(swc_plugin_path.to_string_lossy().to_string())
    }

    fn execute_babel_plugin(&self, plugin_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        println!("Executing Babel plugin for {}...", self.name);

        let input_js = fs::read_to_string(self.input_path())?;
        let runner_script = create_babel_runner(plugin_path, &input_js)?;

        // Execute the runner script with Node.js
        let output = Command::new("node")
            .arg(&runner_script)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Babel execution failed: {}", stderr).into());
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    }

    fn execute_swc_plugin(&self, _plugin_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        println!("Executing SWC plugin for {}...", self.name);

        // TODO: Implement SWC execution
        // For now, return a placeholder
        Err("SWC execution not yet implemented".into())
    }

    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Running integration test: {} ===", self.manifest.test.name);
        println!("Description: {}", self.manifest.test.description);

        // Compile to both targets
        let babel_plugin = self.compile_to_babel()?;
        let _swc_plugin = self.compile_to_swc()?;

        // Execute Babel plugin
        let babel_output = self.execute_babel_plugin(&babel_plugin)?;

        // Execute SWC plugin
        // let swc_output = self.execute_swc_plugin(&swc_plugin)?;

        // For now, just validate Babel output against expected
        let expected = fs::read_to_string(self.expected_path())?;

        if normalize_js(&babel_output) == normalize_js(&expected) {
            println!("✓ Babel output matches expected");
        } else {
            println!("✗ Babel output does not match expected");
            println!("\nExpected:\n{}", expected);
            println!("\nGot:\n{}", babel_output);
            return Err("Output mismatch".into());
        }

        // TODO: Compare Babel vs SWC outputs
        // assert_eq!(normalize_js(&babel_output), normalize_js(&swc_output));

        println!("✓ Test passed!\n");
        Ok(())
    }
}

fn create_babel_runner(plugin_path: &str, input_js: &str) -> Result<String, Box<dyn std::error::Error>> {
    let runner_dir = std::env::temp_dir().join("relux_babel_runner");
    fs::create_dir_all(&runner_dir)?;

    // Create a Node.js script that runs the Babel plugin
    let runner_script = format!(r#"
const babel = require('@babel/core');
const plugin = require('{}');

const code = `{}`;

const result = babel.transformSync(code, {{
    plugins: [plugin],
    parserOpts: {{
        sourceType: 'module'
    }}
}});

console.log(result.code);
"#, plugin_path.replace('\\', "\\\\"), input_js.replace('`', "\\`"));

    let script_path = runner_dir.join("run.js");
    fs::write(&script_path, runner_script)?;

    Ok(script_path.to_string_lossy().to_string())
}

fn normalize_js(code: &str) -> String {
    // Normalize whitespace and newlines for comparison
    code.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn discover_integration_tests() -> Result<Vec<IntegrationTest>, Box<dyn std::error::Error>> {
    let integration_dir = Path::new("tests/integration");

    if !integration_dir.exists() {
        return Ok(Vec::new());
    }

    let mut tests = Vec::new();

    for entry in fs::read_dir(integration_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("test.toml");
            if manifest_path.exists() {
                match IntegrationTest::load(path) {
                    Ok(test) => tests.push(test),
                    Err(e) => eprintln!("Failed to load test: {}", e),
                }
            }
        }
    }

    Ok(tests)
}

#[test]
fn run_all_integration_tests() {
    let tests = discover_integration_tests().expect("Failed to discover tests");

    if tests.is_empty() {
        println!("No integration tests found");
        return;
    }

    println!("\nDiscovered {} integration test(s)", tests.len());

    let mut failed = Vec::new();

    for test in tests {
        if let Err(e) = test.run() {
            eprintln!("Test {} failed: {}", test.name, e);
            failed.push(test.name);
        }
    }

    if !failed.is_empty() {
        panic!("{} test(s) failed: {:?}", failed.len(), failed);
    }
}
