/**
 * Hook Import Analyzer
 *
 * Handles cross-file custom hook imports
 * When a component imports a custom hook from another file,
 * we parse that file to understand the hook's signature
 *
 * Uses the RustScript parser module for runtime AST parsing
 */

use fs;
use path;
use parser;
use "./hook_detector.rsc" { is_custom_hook, get_hook_name, get_hook_parameters, get_hook_body, HookParameter };
use "./hook_analyzer.rsc" { analyze_hook, HookAnalysis };

/**
 * Hook metadata from imported file
 */
pub struct ImportedHookMetadata {
    pub hook_name: Str,
    pub original_name: Str,
    pub file_path: Str,
    pub analysis: HookAnalysis,
}

/**
 * Analyze imported hooks from relative imports
 *
 * @param program - Program node to scan for imports
 * @param current_file_path - Path to the current file
 * @returns Map of hook name to metadata
 */
pub fn analyze_imported_hooks(
    program: &Program,
    current_file_path: &Str
) -> HashMap<Str, ImportedHookMetadata> {
    let mut imported_hooks = HashMap::new();
    let current_dir = path::dirname(current_file_path);

    // Find import declarations
    for stmt in &program.body {
        if let Statement::ImportDeclaration(ref import_decl) = stmt {
            let source = &import_decl.source.value;

            // Only process relative imports (potential hook files)
            if !source.starts_with("./") && !source.starts_with("../") {
                continue;
            }

            // Resolve the import path
            match resolve_import_path(source, &current_dir) {
                Some(resolved_path) => {
                    // Check if file exists
                    if fs::exists(&resolved_path) {
                        // Parse the imported file
                        match parser::parse_file(&resolved_path) {
                            Ok(imported_ast) => {
                                // Extract hooks from the imported file
                                let file_hooks = extract_hooks_from_program(&imported_ast, &resolved_path);

                                // Map imported names to hook metadata
                                map_import_specifiers(
                                    import_decl,
                                    &file_hooks,
                                    &mut imported_hooks
                                );
                            }
                            Err(err) => {
                                // Log parse error (in production, you might want to handle this differently)
                                // eprintln!("Failed to parse {}: {}", resolved_path, err);
                            }
                        }
                    }
                }
                None => {
                    // Could not resolve import path
                }
            }
        }
    }

    imported_hooks
}

/**
 * Map import specifiers to hook metadata
 */
fn map_import_specifiers(
    import_decl: &ImportDeclaration,
    file_hooks: &HashMap<Str, ImportedHookMetadata>,
    imported_hooks: &mut HashMap<Str, ImportedHookMetadata>
) {
    for spec in &import_decl.specifiers {
        match spec {
            // Default import: import useHook from './useHook'
            ImportSpecifier::ImportDefaultSpecifier(ref default_spec) => {
                let local_name = &default_spec.local.name;

                // Look for default export
                if let Some(hook) = file_hooks.get("default") {
                    let mut hook_meta = hook.clone();
                    hook_meta.hook_name = local_name.clone();
                    imported_hooks.insert(local_name.clone(), hook_meta);
                }
            }

            // Named import: import { useHook } from './hooks'
            ImportSpecifier::ImportSpecifier(ref named_spec) => {
                let imported_name = &named_spec.imported.name;
                let local_name = &named_spec.local.name;

                if let Some(hook) = file_hooks.get(imported_name) {
                    let mut hook_meta = hook.clone();
                    hook_meta.hook_name = local_name.clone();
                    imported_hooks.insert(local_name.clone(), hook_meta);
                }
            }

            // Namespace import: import * as hooks from './hooks'
            // (Not typically used for hooks, but included for completeness)
            _ => {}
        }
    }
}

/**
 * Extract hook definitions from a parsed AST
 */
fn extract_hooks_from_program(
    program: &Program,
    file_path: &Str
) -> HashMap<Str, ImportedHookMetadata> {
    let mut hooks = HashMap::new();

    for stmt in &program.body {
        match stmt {
            // export function useCounter(...) { ... }
            Statement::ExportNamedDeclaration(ref export) => {
                if let Some(ref decl) = export.declaration {
                    if let Declaration::FunctionDeclaration(ref func) = decl {
                        if let Some(hook_meta) = analyze_hook_if_valid(func, file_path) {
                            hooks.insert(hook_meta.original_name.clone(), hook_meta);
                        }
                    }
                }
            }

            // export default function useCounter(...) { ... }
            Statement::ExportDefaultDeclaration(ref export) => {
                if let Declaration::FunctionDeclaration(ref func) = export.declaration {
                    if let Some(hook_meta) = analyze_hook_if_valid(func, file_path) {
                        hooks.insert("default".to_string(), hook_meta);
                    }
                }
            }

            // Handle: const useCounter = (...) => { ... }; export { useCounter };
            Statement::FunctionDeclaration(ref func) => {
                // Check if this function is exported later
                if is_exported_later(program, func) {
                    if let Some(hook_meta) = analyze_hook_if_valid(func, file_path) {
                        hooks.insert(hook_meta.original_name.clone(), hook_meta);
                    }
                }
            }

            // Handle: const useCounter = (...) => { ... };
            Statement::VariableDeclaration(ref var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(hook_meta) = analyze_var_declarator_hook(declarator, program, file_path) {
                        hooks.insert(hook_meta.original_name.clone(), hook_meta);
                    }
                }
            }

            _ => {}
        }
    }

    hooks
}

/**
 * Analyze a function declaration if it's a valid hook
 */
fn analyze_hook_if_valid(
    func: &FunctionDeclaration,
    file_path: &Str
) -> Option<ImportedHookMetadata> {
    let func_stmt = Statement::FunctionDeclaration(func.clone());

    if !is_custom_hook(&func_stmt) {
        return None;
    }

    let hook_name = get_hook_name(&func_stmt)?;
    let analysis = analyze_hook(&func_stmt);

    Some(ImportedHookMetadata {
        hook_name: hook_name.clone(),
        original_name: hook_name,
        file_path: file_path.clone(),
        analysis,
    })
}

/**
 * Analyze variable declarator if it's a hook (arrow function)
 */
fn analyze_var_declarator_hook(
    declarator: &VariableDeclarator,
    program: &Program,
    file_path: &Str
) -> Option<ImportedHookMetadata> {
    // Check if it's an arrow function or function expression
    if let Some(ref init) = declarator.init {
        let is_function = matches!(init, Expression::ArrowFunctionExpression(_)) ||
                         matches!(init, Expression::FunctionExpression(_));

        if !is_function {
            return None;
        }

        // Get the variable name
        if let Pattern::Identifier(ref id) = declarator.id {
            let var_name = &id.name;

            // Check if it starts with "use" (hook convention)
            if !var_name.starts_with("use") {
                return None;
            }

            // Check if it's exported
            if !is_var_exported_later(program, var_name) {
                return None;
            }

            // Wrap in a statement for analysis
            let var_stmt = Statement::VariableDeclaration(VariableDeclaration {
                kind: VariableDeclarationKind::Const,
                declarations: vec![declarator.clone()],
            });

            let analysis = analyze_hook(&var_stmt);

            return Some(ImportedHookMetadata {
                hook_name: var_name.clone(),
                original_name: var_name.clone(),
                file_path: file_path.clone(),
                analysis,
            });
        }
    }

    None
}

/**
 * Check if a function is exported later in the file
 */
fn is_exported_later(program: &Program, func: &FunctionDeclaration) -> bool {
    let func_name = match &func.id {
        Some(id) => &id.name,
        None => return false,
    };

    for stmt in &program.body {
        match stmt {
            // export { useCounter };
            Statement::ExportNamedDeclaration(ref export) => {
                if export.declaration.is_none() {
                    // Check specifiers
                    for spec in &export.specifiers {
                        if let ExportSpecifier::Named(ref named) = spec {
                            if let ModuleExportName::Identifier(ref id) = named.exported {
                                if id.name == func_name {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }

            // export default useCounter;
            Statement::ExportDefaultDeclaration(ref export) => {
                if let Declaration::Identifier(ref id) = export.declaration {
                    if id.name == func_name {
                        return true;
                    }
                }
            }

            _ => {}
        }
    }

    false
}

/**
 * Check if a variable is exported later
 */
fn is_var_exported_later(program: &Program, var_name: &Str) -> bool {
    for stmt in &program.body {
        if let Statement::ExportNamedDeclaration(ref export) = stmt {
            if export.declaration.is_none() {
                for spec in &export.specifiers {
                    if let ExportSpecifier::Named(ref named) = spec {
                        if let ModuleExportName::Identifier(ref local) = named.local {
                            if local.name == var_name {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/**
 * Resolve relative import path to absolute path
 * Tries common extensions: .tsx, .ts, .jsx, .js, .rsc
 */
fn resolve_import_path(import_source: &Str, current_dir: &Str) -> Option<Str> {
    let extensions = vec![".tsx", ".ts", ".jsx", ".js", ".rsc"];

    for ext in extensions {
        let with_ext = if import_source.ends_with(ext) {
            import_source.clone()
        } else {
            format!("{}{}", import_source, ext)
        };

        let resolved = path::join(vec![current_dir.clone(), with_ext]);

        if fs::exists(&resolved) {
            return Some(resolved);
        }
    }

    None
}

/**
 * Check if a hook is imported in the program
 */
pub fn is_imported_hook(
    hook_name: &Str,
    imported_hooks: &HashMap<Str, ImportedHookMetadata>
) -> bool {
    imported_hooks.contains_key(hook_name)
}

/**
 * Get metadata for an imported hook
 */
pub fn get_imported_hook_metadata(
    hook_name: &Str,
    imported_hooks: &HashMap<Str, ImportedHookMetadata>
) -> Option<ImportedHookMetadata> {
    imported_hooks.get(hook_name).cloned()
}

/**
 * Get all imported hook names
 */
pub fn get_all_imported_hook_names(
    imported_hooks: &HashMap<Str, ImportedHookMetadata>
) -> Vec<Str> {
    imported_hooks.keys().cloned().collect()
}
