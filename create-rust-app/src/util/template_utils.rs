use super::workspace_utils::{manifest_path, views_glob};
use lazy_static::lazy_static;
use std::collections::HashMap;
use tera::Tera;

#[derive(Clone)]
/// structure to represent the view (singular) of a single page application
pub struct SinglePageApplication {
    pub view_name: String,
}

lazy_static! {
    /// all the Templates (html files) included in backend/views/..., uses Tera to bundle the frontend into the template
    /// TODO: ensure this is accurate documentation
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new(views_glob()) {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {e}");
                ::std::process::exit(1);
            }
        };
        tera.register_function("bundle", InjectBundle);
        tera.autoescape_on(vec![/*".html", ".sql"*/]);
        tera
    };

    pub static ref VITE_MANIFEST: ViteManifest = {
        load_manifest_entries()
    };
}

pub const DEFAULT_TEMPLATE: &str = "index.html";
pub fn to_template_name(request_path: &str) -> &'_ str {
    let request_path = request_path.strip_prefix('/').unwrap();

    if request_path.eq("") {
        DEFAULT_TEMPLATE
    } else {
        request_path
    }
}

/// This implements the {{ bundle(name="index.tsx") }} function for our templates
struct InjectBundle;
impl tera::Function for InjectBundle {
    fn call(&self, args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        args.get("name").map_or_else(
            || Err("oops".into()),
            |val| {
                tera::from_value::<String>(val.clone()).map_or_else(
                    |_| panic!("No bundle named '{:#?}`", val),
                    |bundle_name| Ok(tera::to_value(create_inject(&bundle_name)).unwrap()),
                )
            },
        )
    }

    fn is_safe(&self) -> bool {
        true
    }
}

fn create_inject(bundle_name: &str) -> String {
    let inject: String;

    #[cfg(not(debug_assertions))]
    {
        let manifest_entry = VITE_MANIFEST
            .get(&format!("bundles/{bundle_name}"))
            .unwrap_or_else(|| panic!("could not get bundle `{}`", bundle_name));
        let entry_file = format!(
            r#"<script type="module" src="/{file}"></script>"#,
            file = manifest_entry.file
        );
        let css_files = manifest_entry
            .css
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|css_file| {
                format!(
                    r#"<link rel="stylesheet" href="/{file}" />"#,
                    file = css_file
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        let dyn_entry_files = manifest_entry
            .dynamicImports
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|dyn_script_file| {
                // TODO: make this deferred or async -- look this up!~
                format!(
                    r#"<script type="module" src="/{file}"></script>"#,
                    file = dyn_script_file
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        inject = format!(
            r#"
    <!-- production mode -->
    {entry_file}
    {css_files}
    {dyn_entry_files}
    "#
        );
    }

    #[cfg(debug_assertions)]
    {
        inject = format!(
            r#"<script>
            // Injecting bundle (dev server mode)
            // {{{{ bundle(name={bundle_name}) }}}}
            ;(() => {{
                const script = document.createElement('script');
                script.type = 'module';
                script.src = `http://${{window.location.hostname}}:21012/bundles/{bundle_name}`;
                document.head.appendChild(script);
            }})();
            </script>"#
        );
    }

    inject
}

#[allow(dead_code, non_snake_case)]
#[derive(serde::Deserialize)]
pub struct ViteManifestEntry {
    /// Script content to load for this entry
    file: String,

    /// Script content to lazy-load for this entry
    dynamicImports: Option<Vec<String>>, // using `import(..)`

    /// Style content to load for this entry
    css: Option<Vec<String>>, // using import '*.css'

    /// If true, eager-load this content
    isEntry: Option<bool>,

    /// If true, lazy-load this content
    isDynamicEntry: Option<bool>, // src: String, /* => not necessary :) */
                                  // assets: Option<Vec<String>>, /* => these will be served by the server! */
}
type ViteManifest = HashMap<String, ViteManifestEntry>;

fn load_manifest_entries() -> ViteManifest {
    use serde_json::Value;
    let mut manifest: ViteManifest = HashMap::new();

    let manifest_json = serde_json::from_str(
        std::fs::read_to_string(std::path::PathBuf::from(manifest_path()))
            .unwrap()
            .as_str(),
    )
    .unwrap();

    match manifest_json {
        Value::Object(obj) => {
            obj.keys().for_each(|manifest_key| {
                let details = obj.get(manifest_key).unwrap();

                let manifest_entry = serde_json::from_value::<ViteManifestEntry>(details.clone())
                    .expect("invalid vite manifest (or perhaps the create-rust-app parser broke!)");

                manifest.insert(manifest_key.to_string(), manifest_entry);
            });
            // done parsing manifest
        }
        _ => {
            panic!("invalid vite manifest (or perhaps the create-rust-app parser broke!)");
        }
    }

    manifest
}
