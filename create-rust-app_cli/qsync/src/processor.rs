use super::hook::{Hook, HookBodyParam, HookPathParam, HookQueryParam};
use super::params::generic_to_typsecript_type;
use darling::FromMeta;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::utils::file_path_to_vec_string;
use syn::FnArg;
use syn::Pat;
use syn::PatType;
use syn::PathArguments;
use syn::Type;
use syn::TypePath;
use walkdir::WalkDir;

extern crate inflector;
use super::processor::inflector::Inflector;

struct QsyncAttributeProps {
    return_type: String,
    is_mutation: bool,
}

#[derive(Debug, FromMeta)]
pub struct MacroArgs {
    return_type: Option<String>,
    mutate: Option<bool>,
}

fn has_qsync_attribute(
    is_debug: bool,
    attributes: &[syn::Attribute],
) -> Option<QsyncAttributeProps> {
    let mut is_mutation: Option<bool> = None;
    let mut return_type = "TODO".to_string();

    let mut has_actix_attribute = false;
    let mut has_qsync_attribute = false;

    for attr in attributes.iter() {
        if is_debug {
            let attr_path = attr
                .path
                .segments
                .iter()
                .map(|r| r.ident.to_string())
                .collect::<Vec<String>>()
                .join("::");
            println!("\tFound attribute '{}'", &attr_path);
        }

        let meta = attr.parse_meta();

        if meta.is_err() {
            if is_debug {
                println!("\t> could not parse attribute as `Meta`");
            }
            continue;
        }

        let meta = meta.unwrap();

        // extract and compare against attribute's identifier (#[path::to::identifier])
        let attr_identifier = meta
            .path()
            .segments
            .last()
            .map(|r| r.ident.to_string())
            .unwrap_or_default();

        match attr_identifier.as_str() {
            "qsync" => {
                has_qsync_attribute = true;

                let args = MacroArgs::from_meta(&meta);
                if args.is_err() {
                    if is_debug {
                        println!("qsync error reading attribute args: {:?}", &args.err());
                    }
                    continue;
                }
                let args = args.unwrap();

                if args.mutate.is_some() {
                    is_mutation = args.mutate;
                }
                if args.return_type.is_some() {
                    return_type = args.return_type.unwrap();
                }
            }
            "get" => {
                has_actix_attribute = true;
                if is_mutation.is_none() {
                    is_mutation = Some(false);
                }
            }
            "post" => {
                has_actix_attribute = true;
                if is_mutation.is_none() {
                    is_mutation = Some(true);
                }
            }
            "patch" => {
                has_actix_attribute = true;
                if is_mutation.is_none() {
                    is_mutation = Some(true);
                }
            }
            "put" => {
                has_actix_attribute = true;
                if is_mutation.is_none() {
                    is_mutation = Some(true);
                }
            }
            "delete" => {
                has_actix_attribute = true;
                if is_mutation.is_none() {
                    is_mutation = Some(true);
                }
            }
            _ => {}
        }
    }

    if has_actix_attribute && has_qsync_attribute {
        Some(QsyncAttributeProps {
            is_mutation: is_mutation.unwrap_or_default(),
            return_type,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod processor_tests {
    #[test]
    fn test_hook_printing() {
        println!("test complete!");
    }
}

#[derive(Debug)]
pub enum HttpVerb {
    Get,
    Post,
    Put,
    Delete,
    Unknown,
}
enum ParamType {
    Auth,
    Query,
    Body,
    Path,
    Unknown,
}

struct InputType {
    arg_name: String,
    arg_type: String,
    param_type: ParamType,
}

fn get_api_fn_input_param_type(
    query_input: QsyncInput,
    pat_type: PatType,
    type_path: TypePath,
) -> InputType {
    let mut arg_name: String = "unknown".to_string();
    let mut arg_type: String = "any".to_string();

    if let Pat::TupleStruct(tuple_struct) = *pat_type.pat {
        let ident_elem = tuple_struct.pat.elems.last();
        if let Some(Pat::Ident(ident)) = ident_elem {
            let ident = ident.ident.clone();
            arg_name = ident.to_string();
        }
    }

    let segments: syn::punctuated::Punctuated<syn::PathSegment, syn::token::Colon2> =
        type_path.path.segments;

    let last_segment = segments.last();

    if let Some(last_segment) = last_segment {
        // ```
        // #[qsync]
        // async fn endpoint(path: Path<SomeType>) -> HttpResponse { ... }
        //                         ----
        //                         ^ this is the `endpoint_param_type`
        // ```
        let endpoint_param_type = last_segment.clone().ident.to_string();
        let param_type: ParamType = match endpoint_param_type.as_str() {
            "Path" => ParamType::Path,
            "Json" => ParamType::Body,
            "Form" => ParamType::Body,
            "Query" => ParamType::Query,
            "Auth" => ParamType::Auth,
            _ => {
                if query_input
                    .options
                    .auth_extractors
                    .contains(&endpoint_param_type.to_string())
                {
                    ParamType::Auth
                } else {
                    ParamType::Unknown
                }
            }
        };

        if let PathArguments::AngleBracketed(angled) = last_segment.clone().arguments {
            for arg in angled.args {
                arg_type = generic_to_typsecript_type(&arg);
            }
        }

        InputType {
            param_type,
            arg_name,
            arg_type,
        }
    } else {
        InputType {
            param_type: ParamType::Unknown,
            arg_name,
            arg_type,
        }
    }
}

fn extract_path_params_from_hook_endpoint_url(hook: &mut Hook) {
    if hook.endpoint_url.is_empty() {
        panic!("cannot extract path params because endpoint_url is empty!");
    }

    let re = Regex::new(r"\{[A-Za-z0-9_]+}").unwrap();
    for path_param_text in re.find_iter(hook.endpoint_url.as_str()) {
        let path_param_text = path_param_text
            .as_str()
            .trim_start_matches('{')
            .trim_end_matches('}')
            .to_string();
        hook.path_params.push(HookPathParam {
            hook_arg_name: path_param_text,
            hook_arg_type: "string".to_string(),
        })
    }
}

fn extract_endpoint_information(
    endpoint_prefix: String,
    base_input_path: &Path,
    input_path: &Path,
    attributes: &Vec<syn::Attribute>,
    hook: &mut Hook,
) {
    let mut verb = HttpVerb::Unknown;
    let mut path = "".to_string();

    for attr in attributes {
        let last_segment = attr.path.segments.last();
        if let Some(potential_verb) = last_segment {
            let potential_verb = potential_verb.ident.to_string();

            if potential_verb.eq_ignore_ascii_case("get") {
                verb = HttpVerb::Get;
            } else if potential_verb.eq_ignore_ascii_case("post") {
                verb = HttpVerb::Post;
            } else if potential_verb.eq_ignore_ascii_case("put") {
                verb = HttpVerb::Put;
            } else if potential_verb.eq_ignore_ascii_case("delete") {
                verb = HttpVerb::Delete;
            }
        }

        if !matches!(verb, HttpVerb::Unknown) {
            for token in attr.clone().tokens {
                if let proc_macro2::TokenTree::Group(g) = token {
                    for x in g.stream() {
                        if let proc_macro2::TokenTree::Literal(lit) = x {
                            path = lit.to_string().trim_matches('"').to_string();
                        }
                    }
                }
            }
        }
    }

    let endpoint_base = input_path
        .parent()
        .unwrap()
        .strip_prefix(base_input_path)
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("/")
        .trim_end_matches("/");

    // the part extracted from the attribute, for example: `#[post("/{id}"]`
    let handler_path = path.trim_start_matches('/').trim_end_matches('/');

    hook.endpoint_url = format!("{endpoint_prefix}/{endpoint_base}/{handler_path}")
        .trim_end_matches("/")
        .to_string();

    hook.endpoint_verb = verb;
}

struct BuildState /*<'a>*/ {
    pub types: String,
    pub hooks: Vec<Hook>,
    pub unprocessed_files: Vec<PathBuf>,
    // pub ignore_file_config: Option<gitignore::File<'a>>,
    pub is_debug: bool, // this is a hack, we shouldn't have is_debug in the build state since it's global state rather than build/input-path specific state.
}

fn generate_hook_name(base_input_path: &Path, input_path: &Path, fn_name: String) -> String {
    let relative_file_path = input_path.strip_prefix(base_input_path).unwrap();

    let mut hook_name: Vec<String> = file_path_to_vec_string(relative_file_path);

    hook_name.insert(0, "use".to_string());
    hook_name.push(fn_name.to_pascal_case());
    hook_name.join("")
}

fn process_service_file(
    endpoint_prefix: String,
    base_input_path: PathBuf,
    input_path: PathBuf,
    state: &mut BuildState,
    input: QsyncInput,
) {
    if state.is_debug {
        println!(
            "processing rust file: {:?}",
            input_path.clone().into_os_string().into_string().unwrap()
        );
    }

    let file = File::open(&input_path);

    if file.is_err() {
        state.unprocessed_files.push(input_path);
        return;
    }

    let mut file = file.unwrap();

    let mut src = String::new();
    if file.read_to_string(&mut src).is_err() {
        state.unprocessed_files.push(input_path);
        return;
    }

    let syntax = syn::parse_file(&src);

    if syntax.is_err() {
        state.unprocessed_files.push(input_path);
        return;
    }

    let syntax = syntax.unwrap();

    for item in syntax.items {
        if let syn::Item::Fn(exported_fn) = item {
            let qsync_props = has_qsync_attribute(state.is_debug, &exported_fn.attrs);

            let has_qsync_attribute = qsync_props.is_some();

            if state.is_debug {
                if has_qsync_attribute {
                    println!(
                        "Encountered #[get|post|put|delete] struct: {}",
                        exported_fn.sig.ident
                    );
                } else {
                    println!("Encountered non-query struct: {}", exported_fn.sig.ident);
                }
            }

            if has_qsync_attribute {
                let qsync_props = qsync_props.unwrap();
                let mut hook = Hook {
                    uses_auth: false,
                    endpoint_url: "".to_string(),
                    endpoint_verb: HttpVerb::Unknown,
                    is_mutation: qsync_props.is_mutation,
                    return_type: qsync_props.return_type,
                    hook_name: generate_hook_name(
                        &base_input_path,
                        &input_path,
                        exported_fn.sig.ident.to_string(),
                    ),
                    body_params: vec![],
                    path_params: vec![],
                    query_params: vec![],
                    generated_from: input_path.clone(),
                    generation_options: input.clone(),
                };

                extract_endpoint_information(
                    endpoint_prefix.clone(),
                    &base_input_path,
                    &input_path,
                    &exported_fn.attrs,
                    &mut hook,
                );
                extract_path_params_from_hook_endpoint_url(&mut hook);

                for arg in exported_fn.sig.inputs {
                    if let FnArg::Typed(typed_arg) = arg.clone() {
                        if let Type::Path(type_path) = *typed_arg.clone().ty {
                            let input_type = get_api_fn_input_param_type(
                                input.clone(),
                                typed_arg.clone(),
                                type_path,
                            );

                            match input_type.param_type {
                                ParamType::Auth => {
                                    // TODO: what about custom auth types like API tokens?
                                    if state.is_debug {
                                        println!(
                                            "\t> ParamType::AUTH (create_rust_app::auth::Auth)",
                                        );
                                    }
                                    hook.uses_auth = true;
                                }
                                ParamType::Body => {
                                    if state.is_debug {
                                        println!(
                                            "\t> ParamType::BODY '{}: {}'",
                                            input_type.arg_name, input_type.arg_type
                                        );
                                    }
                                    hook.body_params.push(HookBodyParam {
                                        hook_arg_name: input_type.arg_name,
                                        hook_arg_type: input_type.arg_type,
                                    });
                                }
                                ParamType::Path => {
                                    if state.is_debug {
                                        println!("\t> ParamType::PATH '{}: {}', (ignored; extracted from endpoint url)", input_type.arg_name, input_type.arg_type);
                                    }
                                    // hook.path_params.push(HookPathParam {
                                    //     hook_arg_name: input_type.arg_name,
                                    //     hook_arg_type: input_type.arg_type,
                                    // });
                                }
                                ParamType::Query => {
                                    if state.is_debug {
                                        println!(
                                            "\t> ParamType::QUERY '{}: {}'",
                                            input_type.arg_name, input_type.arg_type
                                        );
                                    }

                                    hook.query_params.push(HookQueryParam {
                                        hook_arg_name: input_type.arg_name,
                                        hook_arg_type: input_type.arg_type,
                                    });
                                }
                                ParamType::Unknown => {
                                    if state.is_debug {
                                        println!(
                                            "\t> ParamType::UNKNOWN '{}: {}'",
                                            input_type.arg_name, input_type.arg_type
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                state.types.push('\n');
                state.types.push_str(&hook.to_string());
                state.hooks.push(hook);
                state.types.push('\n');
            }
        }
    }
}

#[derive(Clone)]
pub struct QsyncInput {
    path: PathBuf,
    options: QsyncOptions,
}

impl QsyncInput {
    pub fn new(path: PathBuf, options: QsyncOptions) -> Self {
        Self { path, options }
    }
}

// #[derive(Clone)]
// /// Currently, only header extractors are supported.
// pub enum Extractor {
//     /// the generated code will require the user to provide a header
//     /// with this name, and will update the request to include it
//     Header(String),
// }

// #[derive(Clone)]
// pub struct ExtractorProperties {
//     /// The extractor's type.
//     ///
//     /// It must match exactly what you write in your endpoint function.
//     ///
//     /// In the example below, we use "ApiAuth" to extract some property from
//     /// the request. "ApiAuth" is the type name.
//     ///
//     /// ```rust
//     /// #[qsync]
//     /// async fn endpoint(auth: path::to::ApiAuth) -> HttpResponse { ... }
//     ///                                   -------
//     ///                                   ^ this is the type_name
//     /// ```
//     pub type_name: String,
//
//     /// This is holds information about what it extracts from the request.
//     /// This property determines how the generated code will change based
//     /// on whether this extractor type is present.
//     pub extracts: Extractor,
// }

#[derive(Clone)]
pub struct QsyncOptions {
    is_debug: bool,
    url_base: String,

    // Discarded this attempt; this is really hard to do correctly
    // /// Qsync detects a set of "extractor" types that are generally
    // /// found in web frameworks like actix_web. It uses these extractors
    // /// to generate code with correct parameters and return types.
    // /// By default, it supports:
    // ///     - Path
    // ///     - Json
    // ///     - Form
    // ///     - Query
    // ///     - Auth
    // ///
    // /// In the case you write your own extractor, you can specify it here.
    // custom_extractors: Vec<ExtractorProperties>,
    auth_extractors: Vec<String>,
}

impl QsyncOptions {
    pub fn new(is_debug: bool, url_base: String, auth_extractors: Vec<String>) -> Self {
        Self {
            is_debug,
            url_base,
            auth_extractors: auth_extractors,
        }
    }
}

pub fn process(input_paths: Vec<QsyncInput>, output_path: PathBuf) {
    let mut state: BuildState = BuildState {
        types: String::new(),
        hooks: vec![],
        unprocessed_files: Vec::<PathBuf>::new(),
        is_debug: false, // we should remove this later on and have a global is_debug state, not BuildState-specific is_debug
    };

    state.types.push_str(
        r#"/**
    Hooks in this file were generated by create-rust-app's query-sync feature.

    You likely have a `qsync.rs` binary in this project which you can use to
    regenerate this file.
    
    To specify a specific a specific return type for a hook, use the
    `#[qsync(return_type = "<typescript return type>")]` attribute above
    your endpoint functions (which are decorated by the actix_web attributes
    (#[get(...)], #[post(...)], etc).
    
    If it doesn't correctly guess whether it's a mutation or query based on
    the actix_web attributes, then you can manually override that by specifying
    the mutate property: `#[qsync(mutate=true)]`.
*/
"#,
    );

    state.types.push_str(
        "import { UseQueryOptions, useMutation, useQuery, useQueryClient } from 'react-query'\n",
    );

    state
        .types
        .push_str("\nimport { useAuth } from './hooks/useAuth'\n");

    for qsync_input in input_paths.clone() {
        let input_path = qsync_input.path.clone();
        let options = qsync_input.clone().options;
        let is_debug = options.is_debug;
        let endpoint_prefix = options.url_base;

        if !input_path.exists() {
            if is_debug {
                println!("Path `{input_path:#?}` does not exist");
            }

            state.unprocessed_files.push(input_path);
            continue;
        }

        let base_input_path = input_path.clone();

        if input_path.clone().is_dir() {
            for entry in WalkDir::new(input_path.clone()).sort_by_file_name() {
                match entry {
                    Ok(dir_entry) => {
                        let path = dir_entry.into_path();

                        // skip dir files because they're going to be recursively crawled by WalkDir
                        if !path.is_dir() {
                            // make sure it is a rust file
                            let extension = path.extension();
                            if extension.is_some() && extension.unwrap().eq_ignore_ascii_case("rs")
                            {
                                state.is_debug = is_debug; // this is a hack, we shouldn't have is_debug in the build state since it's global state rather than build/input-path specific state.
                                process_service_file(
                                    endpoint_prefix.clone(),
                                    base_input_path.clone(),
                                    path,
                                    &mut state,
                                    qsync_input.clone(),
                                );
                            } else if is_debug {
                                println!("Encountered non-service or non-rust file `{path:#?}`");
                            }
                        } else if is_debug {
                            println!("Encountered directory `{path:#?}`");
                        }
                    }
                    Err(_) => {
                        println!(
                            "An error occurred whilst walking directory `{:#?}`...",
                            input_path.clone()
                        );
                        continue;
                    }
                }
            }
        } else {
            state.is_debug = is_debug;
            process_service_file(
                endpoint_prefix.clone(),
                base_input_path,
                input_path,
                &mut state,
                qsync_input.clone(),
            );
        }
    }

    let is_debug = input_paths
        .clone()
        .iter()
        .any(|input| input.options.is_debug);
    if is_debug {
        println!("======================================");
        println!("FINAL FILE:");
        println!("======================================");
        println!("{}", state.types);
        println!("======================================");
        println!("Note: Nothing is written in debug mode");
        println!("======================================");
    } else {
        let mut file: File = File::create(&output_path).expect("Unable to write to file");
        match file.write_all(state.types.as_bytes()) {
            Ok(_) => println!("Successfully generated hooks, see {output_path:#?}"),
            Err(_) => println!("Failed to generate types, an error occurred."),
        }
    }

    if !state.unprocessed_files.is_empty() {
        println!("Could not parse the following files:");
    }

    for unprocessed_file in state.unprocessed_files {
        println!("• {unprocessed_file:#?}");
    }
}
