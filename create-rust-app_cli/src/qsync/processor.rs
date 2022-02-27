use super::hook::{Hook, HookBodyParam, HookPathParam, HookQueryParam};
use super::params::generic_to_typsecript_type;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
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

fn has_qsync_attribute(
    _is_debug: bool,
    attributes: &Vec<syn::Attribute>,
) -> Option<QsyncAttributeProps> {
    let mut qsync_props = QsyncAttributeProps {
        is_mutation: false,
        return_type: "TODO".to_string(),
    };
    let mut has_actix_attribute = false;
    /*
        The #[qsync] attribute was removed:
        • is_mutation can be inferred from the HTTP verb, and,
        • return_type can be added in by the user after after the hook is generated.
    */
    let has_qsync_attribute = true;

    for attr in attributes.iter() {
        let last_segment = attr.path.segments.last();
        if last_segment.is_some() {
            let last_segment = last_segment.unwrap();

            // if last_segment
            //     .clone()
            //     .ident
            //     .to_string()
            //     .as_str()
            //     .eq_ignore_ascii_case("qsync")
            // {
            //     has_qsync_attribute = true;

            //     for token in attr.tokens.clone() {
            //         match token {
            //             TokenTree::Group(g) => {
            //                 for i in g.stream() {
            //                     match i {
            //                         TokenTree::Ident(ident) => {
            //                             if ident.to_string().eq_ignore_ascii_case("mutate") {
            //                                 qsync_props.is_mutation = true;
            //                             }
            //                         }
            //                         TokenTree::Literal(literal) => {
            //                             qsync_props.return_type = literal.to_string();
            //                         }
            //                         _ => {}
            //                     }
            //                 }
            //             }
            //             _ => {
            //                 if is_debug {
            //                     println!("Could not parse attribute");
            //                 }
            //                 continue;
            //             }
            //         }
            //     }
            // }

            match last_segment.ident.to_string().as_str() {
                "get" => {
                    has_actix_attribute = true;
                    qsync_props.is_mutation = false;
                }
                "post" => {
                    has_actix_attribute = true;
                    qsync_props.is_mutation = true;
                }
                "patch" => {
                    has_actix_attribute = true;
                    qsync_props.is_mutation = true;
                }
                "put" => {
                    has_actix_attribute = true;
                    qsync_props.is_mutation = true;
                }
                "delete" => {
                    has_actix_attribute = true;
                    qsync_props.is_mutation = true;
                }
                _ => {}
            }
        }
    }

    if has_actix_attribute && has_qsync_attribute {
        Some(qsync_props)
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
    GET,
    POST,
    PUT,
    DELETE,
    UNKNOWN,
}

enum ParamType {
    AUTH,
    QUERY,
    BODY,
    PATH,
    UNKNOWN,
}

struct InputType {
    arg_name: String,
    arg_type: String,
    param_type: ParamType,
}

fn get_api_fn_input_param_type(pat_type: PatType, type_path: TypePath) -> InputType {
    let mut arg_name: String = "unknown".to_string();
    let mut arg_type: String = "any".to_string();

    match *pat_type.pat {
        Pat::TupleStruct(tuple_struct) => {
            let ident_elem = tuple_struct.pat.elems.last();
            if ident_elem.is_some() {
                let ident_elem = ident_elem.unwrap().clone();

                match ident_elem {
                    Pat::Ident(ident) => {
                        let ident = ident.ident;
                        arg_name = ident.to_string();
                    }
                    _ => { /* failed to capture identifier for hook argument */ }
                }
            }
        }
        _ => { /* failed to determine name of arg */ }
    }

    let segments: syn::punctuated::Punctuated<syn::PathSegment, syn::token::Colon2> =
        type_path.path.segments;

    let last_segment = segments.last();

    if last_segment.is_some() {
        let last_segment = last_segment.unwrap();

        let param_type = match last_segment.clone().ident.to_string().as_str() {
            "Path" => ParamType::PATH,
            "Json" => ParamType::BODY,
            "Form" => ParamType::BODY,
            "Query" => ParamType::QUERY,
            "Auth" => ParamType::AUTH,
            _ => ParamType::UNKNOWN,
        };

        match last_segment.clone().arguments {
            PathArguments::AngleBracketed(angled) => {
                for arg in angled.args {
                    arg_type = generic_to_typsecript_type(&arg);
                }
            }
            _ => { /* could not find type for param */ }
        }

        InputType {
            param_type: param_type,
            arg_name: arg_name.to_string(),
            arg_type: arg_type.to_string(),
        }
    } else {
        InputType {
            param_type: ParamType::UNKNOWN,
            arg_name: arg_name.to_string(),
            arg_type: arg_type.to_string(),
        }
    }
}

fn extract_path_params_from_hook_endpoint_url(hook: &mut Hook) {
    if hook.endpoint_url.is_empty() {
        panic!("cannot extract path params because endpoint_url is empty!");
    }

    let re = Regex::new(r"\{[A-Za-z0-9_]+\}").unwrap();
    for path_param_text in re.find_iter(hook.endpoint_url.as_str()) {
        let path_param_text = path_param_text
            .as_str()
            .trim_start_matches("{")
            .trim_end_matches("}")
            .to_string();
        hook.path_params.push(HookPathParam {
            hook_arg_name: path_param_text,
            hook_arg_type: "string".to_string(),
        })
    }
}

fn extract_endpoint_information(
    input_path: &PathBuf,
    attributes: &Vec<syn::Attribute>,
    hook: &mut Hook,
) {
    let mut verb = HttpVerb::UNKNOWN;
    let mut path = "".to_string();

    for attr in attributes {
        let last_segment = attr.path.segments.last();
        if last_segment.is_some() {
            let potential_verb = last_segment.unwrap().ident.to_string();

            if potential_verb.eq_ignore_ascii_case("get") {
                verb = HttpVerb::GET;
            } else if potential_verb.eq_ignore_ascii_case("post") {
                verb = HttpVerb::POST;
            } else if potential_verb.eq_ignore_ascii_case("put") {
                verb = HttpVerb::PUT;
            } else if potential_verb.eq_ignore_ascii_case("delete") {
                verb = HttpVerb::DELETE;
            }
        }

        if !matches!(verb, HttpVerb::UNKNOWN) {
            for token in attr.clone().tokens {
                match token {
                    syn::__private::quote::__private::TokenTree::Group(g) => {
                        for x in g.stream() {
                            match x {
                                syn::__private::quote::__private::TokenTree::Literal(lit) => {
                                    path = lit.to_string();
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let endpoint_base = input_path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .trim_end_matches("/");

    // the part extracted from the attribute, for example: `#[post("/{id}"]`
    let handler_path = path.trim_matches('"').trim_start_matches('/').trim_end_matches("/");
    let handler_path = if !handler_path.is_empty() { "/".to_string() + handler_path } else { "".to_string() };

    hook.endpoint_url = format!(
        "/api/{}{}",
        endpoint_base,
        handler_path
    );

    hook.endpoint_verb = verb;
}

struct BuildState /*<'a>*/ {
    pub types: String,
    pub hooks: Vec<Hook>,
    pub unprocessed_files: Vec<PathBuf>,
    // pub ignore_file_config: Option<gitignore::File<'a>>,
    pub is_debug: bool,
}

fn generate_hook_name(input_path: &PathBuf, fn_name: String) -> String {
    let mut s: Vec<String> = vec![];

    let mut copy = false;
    for path in input_path.components() {
        let path_as_string = path.as_os_str().to_str().unwrap_or_default().to_string();
        let path_as_string = path_as_string.trim_end_matches(".rs").to_string();
        if copy {
            s.push(path_as_string.clone());
        }
        if path_as_string.eq_ignore_ascii_case("services") {
            copy = true;
        }
    }

    let mut two: Vec<String> = s.iter().map(|s| s.to_pascal_case()).collect();

    two.insert(0, "use".to_string());
    two.insert(two.len(), fn_name.to_pascal_case());
    two.join("")
}

fn generate_query_key_base(input_path: &PathBuf) -> String {
    let mut s: Vec<String> = vec![];

    let mut copy = false;
    for path in input_path.components() {
        let path_as_string = path.as_os_str().to_str().unwrap_or_default().to_string();
        let path_as_string = path_as_string.trim_end_matches(".rs").to_string();
        if copy {
            s.push(path_as_string.clone());
        }
        if path_as_string.eq_ignore_ascii_case("services") {
            copy = true;
        }
    }

    let two: Vec<String> = s.iter().map(|s| s.to_pascal_case()).collect();

    two.join("")
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
        match item {
            syn::Item::Fn(exported_fn) => {
                let qsync_props = has_qsync_attribute(state.is_debug, &exported_fn.attrs);

                let has_qsync_attribute = qsync_props.is_some();

                if state.is_debug {
                    if has_qsync_attribute {
                        println!(
                            "Encountered #[get|post|put|delete] struct: {}",
                            exported_fn.sig.ident.to_string()
                        );
                    } else {
                        println!(
                            "Encountered non-query struct: {}",
                            exported_fn.sig.ident.to_string()
                        );
                    }
                }

                if has_qsync_attribute {
                    let qsync_props = qsync_props.unwrap();
                    let mut hook = Hook {
                        uses_auth: false,
                        endpoint_url: "".to_string(),
                        endpoint_verb: HttpVerb::UNKNOWN,
                        is_mutation: qsync_props.is_mutation,
                        return_type: qsync_props.return_type,
                        hook_name: generate_hook_name(
                            &input_path,
                            exported_fn.sig.ident.to_string(),
                        ),
                        query_key_base: generate_query_key_base(&input_path),
                        body_params: vec![],
                        path_params: vec![],
                        query_params: vec![],
                    };

                    extract_endpoint_information(&input_path, &exported_fn.attrs, &mut hook);
                    extract_path_params_from_hook_endpoint_url(&mut hook);

                    let mut arg_index = 0;
                    let num_args = exported_fn.sig.inputs.len() - 1;
                    for arg in exported_fn.sig.inputs {
                        match arg.clone() {
                            FnArg::Typed(typed_arg) => match *typed_arg.clone().ty {
                                Type::Path(type_path) => {
                                    let input_type =
                                        get_api_fn_input_param_type(typed_arg.clone(), type_path);

                                    match input_type.param_type {
                                        ParamType::AUTH => {
                                            if state.is_debug {
                                                println!("\t> ParamType::AUTH",);
                                            }
                                            hook.uses_auth = true;
                                        }
                                        ParamType::BODY => {
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
                                        ParamType::PATH => {
                                            if state.is_debug {
                                                println!("\t> ParamType::PATH '{}: {}', (ignored; extracted from endpoint url)", input_type.arg_name, input_type.arg_type);
                                            }
                                            // hook.path_params.push(HookPathParam {
                                            //     hook_arg_name: input_type.arg_name,
                                            //     hook_arg_type: input_type.arg_type,
                                            // });
                                        }
                                        ParamType::QUERY => {
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
                                        ParamType::UNKNOWN => {
                                            // TODO: param type is unknown
                                        }
                                    }

                                    // state.types.push_str(&format!("{}", input_type.ty));
                                    if arg_index < num_args {
                                        arg_index += 1;
                                        // state.types.push_str(", ");
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    state.types.push('\n');
                    state.types.push_str(&hook.to_string());
                    state.hooks.push(hook);
                    state.types.push('\n');
                }
            }
            _ => {}
        }
    }
}

pub fn process(input_paths: Vec<PathBuf>, output_path: PathBuf, is_debug: bool) {
    let mut state: BuildState = BuildState {
        types: String::new(),
        hooks: vec![],
        unprocessed_files: Vec::<PathBuf>::new(),
        is_debug: is_debug,
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

    state
        .types
        .push_str("\n/* Placeholder for types which need to be defined */\ntype TODO = unknown\n");

    for input_path in input_paths {
        if !input_path.exists() {
            if is_debug {
                println!("Path `{:#?}` does not exist", input_path);
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
                            //     .eq_ignore_ascii_case("mod.rs")
                            {
                                process_service_file(path, &mut state);
                            } else if is_debug {
                                println!("Encountered non-service or non-rust file `{:#?}`", path);
                            }
                        } else if is_debug {
                            println!("Encountered directory `{:#?}`", path);
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
        // Verify that the output file either doesn't exists or has been generated by qsync.
        let original_file_path = Path::new(&output_path);
        if original_file_path.exists() {
            if !original_file_path.is_file() {
                panic!("Specified output path is a directory but must be a file.")
            }

            let mut defined_hooks = Vec::<String>::new();
            let file_content =
                std::fs::read_to_string(original_file_path).expect("Couldn't open output file");

            let hook_regex = regex::Regex::new(r"const\s+([a-zA-Z0-9_]+)").unwrap();
            for hook in hook_regex.captures_iter(file_content.as_str()) {
                defined_hooks.push(hook[1].to_string());
            }
            // now we add any hooks that weren't included in the file

            let mut file_content = file_content.trim_end().to_string();
            file_content.push('\n');

            let mut added = 0;
            let mut existing = state.hooks.len();

            for hook in state
                .hooks
                .iter()
                .filter(|&h| !defined_hooks.contains(&h.hook_name))
            {
                file_content.push('\n');
                file_content.push_str(&hook.to_string());
                added = added + 1;
                existing = existing - 1;
                file_content.push('\n');
            }

            std::fs::write(&output_path, file_content).expect("Unable to write to file");

            println!(
                "Successfully generated hooks ({} added, {} existing), see {:#?}",
                added, existing, &output_path
            )
        } else {
            let mut file: File = File::create(&output_path).expect("Unable to write to file");
            match file.write_all(state.types.as_bytes()) {
                Ok(_) => println!("Successfully generated hooks, see {:#?}", output_path),
                Err(_) => println!("Failed to generate types, an error occurred."),
            }
        }
    }

    if state.unprocessed_files.len() > 0 {
        println!("Could not parse the following files:");
    }

    for unprocessed_file in state.unprocessed_files {
        println!("• {:#?}", file = unprocessed_file);
    }
}
