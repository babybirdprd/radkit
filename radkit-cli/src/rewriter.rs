use anyhow::Result;
use syn::{
    parse_quote, visit_mut::VisitMut, Expr, ExprMethodCall, File, Item, ItemMod, ItemUse, Stmt,
};

pub struct Rewriter {
    syntax_tree: File,
}

impl Rewriter {
    pub fn new(source_code: &str) -> Result<Self> {
        let syntax_tree = syn::parse_file(source_code)?;
        Ok(Self { syntax_tree })
    }

    pub fn to_string(&self) -> String {
        prettyplease::unparse(&self.syntax_tree)
    }

    /// Adds a module declaration `pub mod name;` or `mod name;` if it doesn't exist.
    pub fn add_module_declaration(&mut self, mod_name: &str) {
        let mut exists = false;
        for item in &self.syntax_tree.items {
            if let Item::Mod(item_mod) = item {
                if item_mod.ident == mod_name {
                    exists = true;
                    break;
                }
            }
        }

        if !exists {
            let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
            let new_mod: ItemMod = parse_quote! {
                pub mod #mod_ident;
            };
            // Insert after existing mods or uses, or at top
            // Simplified: Insert at the top of items, or after last mod
            // For now, let's just append to the list of items? No, order matters sometimes.
            // Ideally after last `mod` or `use`.
            let mut insert_idx = 0;
            for (i, item) in self.syntax_tree.items.iter().enumerate() {
                if matches!(item, Item::Mod(_) | Item::Use(_)) {
                    insert_idx = i + 1;
                }
            }
            if insert_idx > self.syntax_tree.items.len() {
                insert_idx = self.syntax_tree.items.len();
            }
            self.syntax_tree
                .items
                .insert(insert_idx, Item::Mod(new_mod));
        }
    }

    /// Removes a module declaration `pub mod name;` or `mod name;`
    pub fn remove_module_declaration(&mut self, mod_name: &str) {
        self.syntax_tree.items.retain(|item| {
            if let Item::Mod(item_mod) = item {
                item_mod.ident != mod_name
            } else {
                true
            }
        });
    }

    /// Adds a `.with_tool(crate::tools::{mod_name}::{tool_name})` call to the agent builder chain.
    pub fn add_tool_wiring(&mut self, mod_name: &str, tool_name: &str) -> Result<()> {
        let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
        let tool_ident = syn::Ident::new(tool_name, proc_macro2::Span::call_site());

        let mut visitor = BuilderVisitor {
            action: BuilderAction::AddTool {
                mod_ident,
                tool_ident,
            },
            found: false,
        };

        visitor.visit_file_mut(&mut self.syntax_tree);

        if !visitor.found {
            anyhow::bail!("Could not find Agent/LlmWorker builder chain to wire tool");
        }
        Ok(())
    }

    /// Adds a `.with_skill(crate::skills::{mod_name}::{skill_struct_name})` call.
    pub fn add_skill_wiring(&mut self, mod_name: &str, skill_struct_name: &str) -> Result<()> {
        let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
        let skill_ident = syn::Ident::new(skill_struct_name, proc_macro2::Span::call_site());

        let mut visitor = BuilderVisitor {
            action: BuilderAction::AddSkill {
                mod_ident,
                skill_ident,
            },
            found: false,
        };

        visitor.visit_file_mut(&mut self.syntax_tree);

        if !visitor.found {
            anyhow::bail!("Could not find Agent/LlmWorker builder chain to wire skill");
        }
        Ok(())
    }

    pub fn remove_tool_wiring(&mut self, mod_name: &str, tool_name: &str) -> Result<()> {
        let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
        let tool_ident = syn::Ident::new(tool_name, proc_macro2::Span::call_site());

        let mut visitor = BuilderVisitor {
            action: BuilderAction::RemoveTool {
                mod_ident,
                tool_ident,
            },
            found: false,
        };
        visitor.visit_file_mut(&mut self.syntax_tree);
        // We don't error if not found for removal
        Ok(())
    }

    pub fn remove_skill_wiring(&mut self, mod_name: &str, skill_struct_name: &str) -> Result<()> {
        let mod_ident = syn::Ident::new(mod_name, proc_macro2::Span::call_site());
        let skill_ident = syn::Ident::new(skill_struct_name, proc_macro2::Span::call_site());

        let mut visitor = BuilderVisitor {
            action: BuilderAction::RemoveSkill {
                mod_ident,
                skill_ident,
            },
            found: false,
        };
        visitor.visit_file_mut(&mut self.syntax_tree);
        Ok(())
    }

    /// Updates the provider instantiation.
    /// Looks for `let llm = Provider::from_env(...)` or similar.
    /// Replaces the provider struct type.
    pub fn update_provider(&mut self, new_provider_struct: &str, _env_var: &str) -> Result<()> {
        // 1. Update use statement
        let mut use_visitor = UseProviderVisitor {
            new_struct: new_provider_struct.to_string(),
        };
        use_visitor.visit_file_mut(&mut self.syntax_tree);

        // 2. Update instantiation
        let mut inst_visitor = InstantiationVisitor {
            new_struct: new_provider_struct.to_string(),
            _env_var: _env_var.to_string(),
            found: false,
        };
        inst_visitor.visit_file_mut(&mut self.syntax_tree);

        if !inst_visitor.found {
            // If not found, try to insert it?
            // That's hard without knowing where to put it in `main`.
            // But usually it's `let llm = ...`.
            // If we didn't find a matching pattern, we might want to warn or fail.
            // But the user might have renamed `llm`.
            // We'll rely on finding `::from_env`.
            anyhow::bail!("Could not find provider instantiation (e.g., `let llm = ...::from_env(...)`) to update.");
        }

        Ok(())
    }

    pub fn get_current_provider(&self) -> Option<String> {
        // Just scan for `use radkit::models::providers::X`
        // Or find the instantiation `let llm = X::from_env`
        // We'll return the struct name if found.
        for item in &self.syntax_tree.items {
            if let Item::Fn(func) = item {
                if func.sig.ident == "main" {
                    for stmt in &func.block.stmts {
                        if let Stmt::Local(local) = stmt {
                            if let Some(init) = &local.init {
                                if let Expr::Call(call) = &*init.expr {
                                    // Check for Path::from_env
                                    if let Expr::Path(path) = &*call.func {
                                        // This handles `Provider::from_env`
                                        // path.path.segments -> [Provider, from_env]
                                        if path.path.segments.len() >= 2 {
                                            let last = path.path.segments.last().unwrap();
                                            if last.ident == "from_env" {
                                                let provider = &path.path.segments
                                                    [path.path.segments.len() - 2]
                                                    .ident;
                                                return Some(provider.to_string());
                                            }
                                        }
                                    }
                                }
                                // Handle `Try` (?) e.g. `Provider::from_env(...)`
                                if let Expr::Try(try_expr) = &*init.expr {
                                    if let Expr::Call(call) = &*try_expr.expr {
                                        if let Expr::Path(path) = &*call.func {
                                            if path.path.segments.len() >= 2 {
                                                let last = path.path.segments.last().unwrap();
                                                if last.ident == "from_env" {
                                                    let provider = &path.path.segments
                                                        [path.path.segments.len() - 2]
                                                        .ident;
                                                    return Some(provider.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

enum BuilderAction {
    AddTool {
        mod_ident: syn::Ident,
        tool_ident: syn::Ident,
    },
    AddSkill {
        mod_ident: syn::Ident,
        skill_ident: syn::Ident,
    },
    RemoveTool {
        mod_ident: syn::Ident,
        tool_ident: syn::Ident,
    },
    RemoveSkill {
        mod_ident: syn::Ident,
        skill_ident: syn::Ident,
    },
}

struct BuilderVisitor {
    action: BuilderAction,
    found: bool,
}

impl VisitMut for BuilderVisitor {
    fn visit_expr_method_call_mut(&mut self, node: &mut ExprMethodCall) {
        // Recurse first (visit children) so we find the deepest `.build()`?
        // Actually, we want to append to the chain.
        // Usually the chain ends with `.build()`.
        // We want to insert `with_tool` before `build` or after other `with_`.

        // If the current node is `.build()`, we want to insert inside the receiver?
        // Wait, `obj.method()` is `ExprMethodCall { receiver: obj, method: method }`.
        // So `x.with_y().build()`:
        // node = build, receiver = x.with_y().
        // We want to change receiver to `x.with_y().with_tool(...)`.

        // Strategy: Look for `.build()` call.
        if node.method == "build" {
            // We found the build call. We inject our call into the receiver.
            // The receiver becomes the new method call.
            self.found = true;

            match &self.action {
                BuilderAction::AddTool {
                    mod_ident,
                    tool_ident,
                } => {
                    // Verify if tool is already added?
                    // Hard to verify fully without traversing the whole chain.
                    // We'll assume check is done elsewhere or duplicate is fine (rust might complain about unused?)
                    // Actually duplicate tool registration is usually runtime error or overwrite.

                    let old_receiver = node.receiver.clone();
                    let new_receiver: Expr = parse_quote! {
                        #old_receiver.with_tool(crate::tools::#mod_ident::#tool_ident)
                    };
                    node.receiver = Box::new(new_receiver);
                }
                BuilderAction::AddSkill {
                    mod_ident,
                    skill_ident,
                } => {
                    let old_receiver = node.receiver.clone();
                    let new_receiver: Expr = parse_quote! {
                        #old_receiver.with_skill(crate::skills::#mod_ident::#skill_ident)
                    };
                    node.receiver = Box::new(new_receiver);
                }
                _ => {} // Removals handled differently
            }
        }

        // For removals, we need to traverse the chain and remove the specific `with_tool` call.
        // A chain is nested `ExprMethodCall`s.
        // `a.b().c()` is `Call(receiver: Call(receiver: a, method: b), method: c)`

        match &self.action {
            BuilderAction::RemoveTool {
                mod_ident,
                tool_ident,
            } => {
                if node.method == "with_tool" {
                    // Check args
                    if let Some(arg) = node.args.first() {
                        // arg should look like `crate::tools::#mod_ident::#tool_ident`
                        // Parsing arg to string is easiest comparison
                        let arg_str = quote::quote!(#arg).to_string();
                        let target_str =
                            format!("crate :: tools :: {} :: {}", mod_ident, tool_ident);
                        // quote output might add spaces around ::

                        // Clean whitespace for comparison
                        let clean_arg = arg_str.replace(" ", "");
                        let clean_target = target_str.replace(" ", "");

                        if clean_arg == clean_target {
                            // Remove this call.
                            // We replace this node with its receiver.
                            // `node` is `receiver.with_tool(...)`.
                            // We want `node` to become `receiver`.
                            // But we can't replace `self` easily in visit_mut.
                            // Actually we can replace `*node` with `*node.receiver`.
                            // But `node.receiver` is `Box<Expr>`. `node` is `ExprMethodCall`.
                            // We need to replace the `Expr` containing this `ExprMethodCall`.
                            // `VisitMut` works on `ExprMethodCall`, not `Expr`.
                            // Wait, visit_mut works on fields.

                            // This is tricky with `visit_expr_method_call_mut`.
                            // Better to use `visit_expr_mut`.
                        }
                    }
                }
            }
            _ => {}
        }

        // Continue visiting children
        syn::visit_mut::visit_expr_method_call_mut(self, node);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Handle removal here where we can swap the expression
        if let Expr::MethodCall(node) = expr {
            let mut remove = false;
            match &self.action {
                BuilderAction::RemoveTool {
                    mod_ident,
                    tool_ident,
                } => {
                    if node.method == "with_tool" {
                        if let Some(arg) = node.args.first() {
                            let arg_str = quote::quote!(#arg).to_string();
                            let target_str =
                                format!("crate :: tools :: {} :: {}", mod_ident, tool_ident);
                            if arg_str.replace(" ", "") == target_str.replace(" ", "") {
                                remove = true;
                            }
                        }
                    }
                }
                BuilderAction::RemoveSkill {
                    mod_ident,
                    skill_ident,
                } => {
                    if node.method == "with_skill" {
                        if let Some(arg) = node.args.first() {
                            let arg_str = quote::quote!(#arg).to_string();
                            let target_str =
                                format!("crate :: skills :: {} :: {}", mod_ident, skill_ident);
                            if arg_str.replace(" ", "") == target_str.replace(" ", "") {
                                remove = true;
                            }
                        }
                    }
                }
                _ => {}
            }

            if remove {
                // Replace expr with receiver
                // We take the receiver out of the node
                let receiver = *node.receiver.clone();
                *expr = receiver;
                // We need to visit the new expr (receiver) in case there are more removals in the chain
                self.visit_expr_mut(expr);
                return;
            }
        }

        syn::visit_mut::visit_expr_mut(self, expr);
    }
}

struct UseProviderVisitor {
    new_struct: String,
}

impl VisitMut for UseProviderVisitor {
    fn visit_item_use_mut(&mut self, item: &mut ItemUse) {
        // Check if use path starts with radkit::models::providers
        // tree is `radkit::models::providers::{X}` or `...::X`
        // Simplified check: stringify
        let use_str = quote::quote!(#item).to_string();
        if use_str.contains("radkit :: models :: providers") {
            // We want to replace the last segment with new_struct
            // Or simpler: replace the whole use statement.
            let _new_use: ItemUse = parse_quote! {
                use radkit::models::providers::{}; // incomplete
            };
            // Construct path manually or parse
            let ns = &self.new_struct;
            let ns_ident = syn::Ident::new(ns, proc_macro2::Span::call_site());
            *item = parse_quote! {
                use radkit::models::providers::#ns_ident;
            };
        }
    }
}

struct InstantiationVisitor {
    new_struct: String,
    _env_var: String,
    found: bool,
}

impl VisitMut for InstantiationVisitor {
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        if let Some(init) = &mut local.init {
            // Check if init is a call to from_env
            let mut is_target = false;
            if let Expr::Call(call) = &*init.expr {
                if let Expr::Path(path) = &*call.func {
                    if let Some(last) = path.path.segments.last() {
                        if last.ident == "from_env" {
                            // Assumed to be the provider instantiation
                            is_target = true;
                        }
                    }
                }
            } else if let Expr::Try(try_expr) = &*init.expr {
                if let Expr::Call(call) = &*try_expr.expr {
                    if let Expr::Path(path) = &*call.func {
                        if let Some(last) = path.path.segments.last() {
                            if last.ident == "from_env" {
                                is_target = true;
                            }
                        }
                    }
                }
            }

            if is_target {
                self.found = true;
                // Replace the whole init expression
                let ns = &self.new_struct;
                let ns_ident = syn::Ident::new(ns, proc_macro2::Span::call_site());
                // Determine if we need to preserve args?
                // usually from_env("model")
                // We'll keep the args from the original call if possible?
                // The user might want to change model.
                // But `add_provider` doesn't take model name yet.
                // We'll assume default or keep existing args.

                // Extract args
                let args = if let Expr::Try(t) = &*init.expr {
                    if let Expr::Call(c) = &*t.expr {
                        c.args.clone()
                    } else {
                        syn::punctuated::Punctuated::new()
                    }
                } else if let Expr::Call(c) = &*init.expr {
                    c.args.clone()
                } else {
                    syn::punctuated::Punctuated::new()
                };

                let new_expr: Expr = parse_quote! {
                    #ns_ident::from_env(#args)?
                };
                // If the original wasn't a try (?), we should check if we need `?`.
                // Assuming `from_env` returns Result.
                // The generated code adds `?`.

                // If original had `?`, we add `?`.
                let original_had_try = matches!(&*init.expr, Expr::Try(_));

                let final_expr = if original_had_try {
                    new_expr
                } else {
                    // strip the `?`
                    if let Expr::Try(t) = new_expr {
                        *t.expr
                    } else {
                        new_expr
                    }
                };

                *init.expr = final_expr;
            }
        }
    }
}
