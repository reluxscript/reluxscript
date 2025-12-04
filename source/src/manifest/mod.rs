//! Lux project manifest for declarative builds
//!
//! Defines the structure for `lux.manifest.json` files that specify
//! module dependencies and build order.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Project manifest for building multiple modules and a plugin/writer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxManifest {
    /// Module definitions (libraries)
    #[serde(default)]
    pub modules: Vec<ModuleEntry>,

    /// Plugin to build (optional - either plugin or writer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<TargetEntry>,

    /// Writer to build (optional - either plugin or writer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer: Option<TargetEntry>,
}

/// A module (library) entry in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEntry {
    /// Module name (used for dependency references)
    pub name: String,

    /// Path to the .lux source file
    pub path: String,

    /// Output directory for compiled module
    pub output: String,

    /// Dependencies (names of other modules this depends on)
    #[serde(default)]
    pub deps: Vec<String>,
}

/// Target entry for plugin or writer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetEntry {
    /// Path to the .lux source file
    pub path: String,

    /// Output directory
    pub output: String,

    /// Dependencies (names of modules this depends on)
    #[serde(default)]
    pub deps: Vec<String>,
}

impl LuxManifest {
    /// Load manifest from a JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))
    }

    /// Get modules in topologically sorted order (dependencies first)
    /// Returns an error if there's a cycle
    pub fn sorted_modules(&self) -> Result<Vec<&ModuleEntry>, String> {
        // Build adjacency list and in-degree count
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

        // Initialize all modules with 0 in-degree
        for module in &self.modules {
            in_degree.insert(&module.name, 0);
            dependents.insert(&module.name, vec![]);
        }

        // Build the graph
        for module in &self.modules {
            for dep in &module.deps {
                // Check that dependency exists
                if !in_degree.contains_key(dep.as_str()) {
                    return Err(format!(
                        "Module '{}' depends on '{}' which is not defined in the manifest",
                        module.name, dep
                    ));
                }
                // dep -> module (module depends on dep)
                dependents.get_mut(dep.as_str()).unwrap().push(&module.name);
                *in_degree.get_mut(module.name.as_str()).unwrap() += 1;
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();

        let mut sorted: Vec<&str> = vec![];

        while let Some(node) = queue.pop() {
            sorted.push(node);

            for &dependent in &dependents[node] {
                let deg = in_degree.get_mut(dependent).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dependent);
                }
            }
        }

        // Check for cycles
        if sorted.len() != self.modules.len() {
            let remaining: Vec<_> = in_degree
                .iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(&name, _)| name)
                .collect();
            return Err(format!(
                "Circular dependency detected involving: {}",
                remaining.join(", ")
            ));
        }

        // Map sorted names back to ModuleEntry references
        let module_map: HashMap<&str, &ModuleEntry> = self.modules
            .iter()
            .map(|m| (m.name.as_str(), m))
            .collect();

        Ok(sorted.into_iter().map(|name| module_map[name]).collect())
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), String> {
        // Check for duplicate module names
        let mut seen: HashSet<&str> = HashSet::new();
        for module in &self.modules {
            if !seen.insert(&module.name) {
                return Err(format!("Duplicate module name: '{}'", module.name));
            }
        }

        // Check that plugin/writer deps reference existing modules
        if let Some(ref plugin) = self.plugin {
            for dep in &plugin.deps {
                if !seen.contains(dep.as_str()) {
                    return Err(format!(
                        "Plugin depends on '{}' which is not defined in modules",
                        dep
                    ));
                }
            }
        }

        if let Some(ref writer) = self.writer {
            for dep in &writer.deps {
                if !seen.contains(dep.as_str()) {
                    return Err(format!(
                        "Writer depends on '{}' which is not defined in modules",
                        dep
                    ));
                }
            }
        }

        // Verify topological sort works (no cycles)
        self.sorted_modules()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort() {
        let manifest = LuxManifest {
            modules: vec![
                ModuleEntry {
                    name: "a".to_string(),
                    path: "a.lux".to_string(),
                    output: "dist/a".to_string(),
                    deps: vec![],
                },
                ModuleEntry {
                    name: "b".to_string(),
                    path: "b.lux".to_string(),
                    output: "dist/b".to_string(),
                    deps: vec!["a".to_string()],
                },
                ModuleEntry {
                    name: "c".to_string(),
                    path: "c.lux".to_string(),
                    output: "dist/c".to_string(),
                    deps: vec!["a".to_string(), "b".to_string()],
                },
            ],
            plugin: None,
            writer: None,
        };

        let sorted = manifest.sorted_modules().unwrap();
        let names: Vec<_> = sorted.iter().map(|m| m.name.as_str()).collect();

        // a must come before b and c, b must come before c
        let a_pos = names.iter().position(|&n| n == "a").unwrap();
        let b_pos = names.iter().position(|&n| n == "b").unwrap();
        let c_pos = names.iter().position(|&n| n == "c").unwrap();

        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_cycle_detection() {
        let manifest = LuxManifest {
            modules: vec![
                ModuleEntry {
                    name: "a".to_string(),
                    path: "a.lux".to_string(),
                    output: "dist/a".to_string(),
                    deps: vec!["b".to_string()],
                },
                ModuleEntry {
                    name: "b".to_string(),
                    path: "b.lux".to_string(),
                    output: "dist/b".to_string(),
                    deps: vec!["a".to_string()],
                },
            ],
            plugin: None,
            writer: None,
        };

        let result = manifest.sorted_modules();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }
}
