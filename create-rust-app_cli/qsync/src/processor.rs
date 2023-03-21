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
        let meta = attr.parse_meta().unwrap();

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
            return_type: return_type,
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

fn get_api_fn_input_param_type(pat_type: PatType, type_path: TypePath) -> InputType {
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
        let param_type = match last_segment.clone().ident.to_string().as_str() {
            "Path" => ParamType::Path,
            "Json" => ParamType::Body,
            "Form" => ParamType::Body,
            "Query" => ParamType::Query,
            "Auth" => ParamType::Auth,
            _ => ParamType::Unknown,
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
                            path = lit.to_string();
                        }
                    }
                }
            }
        }
    }

    let endpoint_base = input_path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .trim_end_matches('/');

    // the part extracted from the attribute, for example: `#[post("/{id}"]`
    let handler_path = path
        .trim_matches('"')
        .trim_start_matches('/')
        .trim_end_matches('/');
    let handler_path = if !handler_path.is_empty() {
        "/".to_string() + handler_path
    } else {
        "".to_string()
    };

    hook.endpoint_url = format!("/api/{endpoint_base}{handler_path}");

    hook.endpoint_verb = verb;
}

struct BuildState /*<'a>*/ {
    pub types: String,
    pub hooks: Vec<Hook>,
    pub unprocessed_files: Vec<PathBuf>,
    // pub ignore_file_config: Option<gitignore::File<'a>>,
    pub is_debug: bool,
}

fn generate_hook_name(input_path: &Path, fn_name: String) -> String {
    let mut hook_name: Vec<String> = file_path_to_vec_string(input_path);

    hook_name.insert(0, "use".to_string());
    hook_name.push(fn_name.to_pascal_case());
    hook_name.join("")
}

fn process_service_file(input_path: PathBuf, state: &mut BuildState) {
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
                    hook_name: generate_hook_name(&input_path, exported_fn.sig.ident.to_string()),
                    body_params: vec![],
                    path_params: vec![],
                    query_params: vec![],
                };

                extract_endpoint_information(&input_path, &exported_fn.attrs, &mut hook);
                extract_path_params_from_hook_endpoint_url(&mut hook);

                let mut arg_index = 0;
                let num_args = exported_fn.sig.inputs.len() - 1;
                for arg in exported_fn.sig.inputs {
                    if let FnArg::Typed(typed_arg) = arg.clone() {
                        if let Type::Path(type_path) = *typed_arg.clone().ty {
                            let input_type =
                                get_api_fn_input_param_type(typed_arg.clone(), type_path);

                            match input_type.param_type {
                                ParamType::Auth => {
                                    if state.is_debug {
                                        println!("\t> ParamType::AUTH",);
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
                                    // TODO: param type is unknown
                                }
                            }

                            // state.types.push_str(&format!("{}", input_type.ty));
                            if arg_index < num_args {
                                arg_index += 1;
                                // state.types.push_str(", ");
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

pub fn process(input_paths: Vec<PathBuf>, output_path: PathBuf, is_debug: bool) {
    let mut state: BuildState = BuildState {
        types: String::new(),
        hooks: vec![],
        unprocessed_files: Vec::<PathBuf>::new(),
        is_debug,
    };

    state.types.push_str(
        r#"/**
    Hooks in this file were generated by create-rust-app's query-sync feature.

    1 — Generating hooks
    -=-=-=-=-=-=-=-=-=-=-=-
    Execute `create-rust-app` in your project folder and select "query-sync".
    This will generate react-hooks which are missing in this file for all
    functions defined in the `backend/services` folder which have a
    `#[qsync(returns = "<typescript return type>"[, mutate])]` attribute
    as well as one of the following actix_web attributes: `#[post(...)]`,
    `#[get(...)]`, `#[put(...)]`, `#[delete(...)]`, or `#[patch(...)]`.

    2 — Editing hooks
    -=-=-=-=-=-=-=-=-=-
    You may edit the hooks as you see fit, they will not regenerate so long as
    a constant with their name is present. For example:

        const useTodo = ( ... ) => { ... }

    If the "const useTodo" is present, query-sync will not regenerate this hook
    and any changes you make will sustain through subsequent generation. If you
    don't want a particular hook (for whatever reason), you can do this:

        const useTodo = null

    Now, because "const useTodo" is already defined, this hook will not be
    regenerated. As it follows, if you delete "const useTodo = ...", query-sync
    will regenerate that hook, which is useful if you want to start over.
*/
"#,
    );

    // state
    //     .types
    //     .push_str("\nimport type { QueryKey } from 'react-query'\n");

    state
        .types
        .push_str("import { useMutation, useQuery, useQueryClient } from 'react-query'\n");

    state
        .types
        .push_str("\nimport { useAuth } from './hooks/useAuth'\n");

    // state
    //     .types
    //     .push_str("\n/* Placeholder for types which need to be defined */\ntype TODO = unknown\n");

    for input_path in input_paths {
        if !input_path.exists() {
            if is_debug {
                println!("Path `{input_path:#?}` does not exist");
            }

            state.unprocessed_files.push(input_path);
            continue;
        }

        if input_path.is_dir() {
            for entry in WalkDir::new(input_path.clone()).sort_by_file_name() {
                match entry {
                    Ok(dir_entry) => {
                        let path = dir_entry.into_path();

                        // skip dir files because they're going to be recursively crawled by WalkDir
                        if !path.is_dir() {
                            // make sure it is a rust file
                            let extension = path.extension();
                            if extension.is_some() && extension.unwrap().eq_ignore_ascii_case("rs")
                            // && !path
                            //     .file_name()
                            //     .unwrap_or_default()
                            //     .eq_ignore_ascii_case("storage")
                            {
                                process_service_file(path, &mut state);
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
            process_service_file(input_path, &mut state);
        }
    }

    if is_debug {
        println!("======================================");
        println!("FINAL FILE:");
        println!("======================================");
        println!("{}", state.types);
        println!("======================================");
        println!("Note: Nothing is written in debug mode");
        println!("======================================");
    } else {
        // // Verify that the output file either doesn't exists or has been generated by qsync.
        // let original_file_path = Path::new(&output_path);
        // if original_file_path.exists() {
        //     if !original_file_path.is_file() {
        //         panic!("Specified output path is a directory but must be a file.")
        //     }
        //
        //     let mut defined_hooks = Vec::<String>::new();
        //     let file_content =
        //         std::fs::read_to_string(original_file_path).expect("Couldn't open output file");
        //
        //     let hook_regex = regex::Regex::new(r"const\s+([a-zA-Z0-9_]+)").unwrap();
        //     for hook in hook_regex.captures_iter(file_content.as_str()) {
        //         defined_hooks.push(hook[1].to_string());
        //     }
        //     // now we add any hooks that weren't included in the file
        //
        //     let mut file_content = file_content.trim_end().to_string();
        //     file_content.push('\n');
        //
        //     let mut added = 0;
        //     let mut existing = state.hooks.len();
        //
        //     for hook in state
        //         .hooks
        //         .iter()
        //         .filter(|&h| !defined_hooks.contains(&h.hook_name))
        //     {
        //         file_content.push('\n');
        //         file_content.push_str(&hook.to_string());
        //         added += 1;
        //         existing -= 1;
        //         file_content.push('\n');
        //     }
        //
        //     std::fs::write(&output_path, file_content).expect("Unable to write to file");
        //
        //     println!(
        //         "Successfully generated hooks ({} added, {} existing), see {:#?}",
        //         added, existing, &output_path
        //     )
        // } else {
        let mut file: File = File::create(&output_path).expect("Unable to write to file");
        match file.write_all(state.types.as_bytes()) {
            Ok(_) => println!("Successfully generated hooks, see {output_path:#?}"),
            Err(_) => println!("Failed to generate types, an error occurred."),
        }
        // }
    }

    if !state.unprocessed_files.is_empty() {
        println!("Could not parse the following files:");
    }

    for unprocessed_file in state.unprocessed_files {
        println!("• {unprocessed_file:#?}");
    }
}
