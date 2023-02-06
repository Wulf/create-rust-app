pub fn is_primitive_type(ty: String) -> bool {
    matches!(
        ty.as_str(),
        "i8" | "u8"
            | "i16"
            | "u16"
            | "i32"
            | "u32"
            | "i64"
            | "u64"
            | "i128"
            | "u128"
            | "isize"
            | "usize"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "str"
            | "String"
            | "NaiveDateTime"
            | "DateTime"
    )
}

// TODO: leverage TSYNC
pub fn generic_to_typsecript_type(gen_ty: &syn::GenericArgument) -> String {
    match gen_ty {
        syn::GenericArgument::Type(ty) => to_typescript_type(ty),
        _ => "unknown".to_string(),
    }
}

// TODO: leverage TSYNC
pub fn to_typescript_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Reference(p) => to_typescript_type(&p.elem),
        syn::Type::Path(p) => {
            let segment = p.path.segments.last().unwrap();
            let ident = &segment.ident;
            let arguments = &segment.arguments;
            let identifier = ident.to_string();
            match identifier.as_str() {
                "i8" => "number".to_string(),
                "u8" => "number".to_string(),
                "i16" => "number".to_string(),
                "u16" => "number".to_string(),
                "i32" => "number".to_string(),
                "u32" => "number".to_string(),
                "i64" => "number".to_string(),
                "u64" => "number".to_string(),
                "i128" => "number".to_string(),
                "u128" => "number".to_string(),
                "isize" => "number".to_string(),
                "usize" => "number".to_string(),
                "f32" => "number".to_string(),
                "f64" => "number".to_string(),
                "bool" => "boolean".to_string(),
                "char" => "string".to_string(),
                "str" => "string".to_string(),
                "String" => "string".to_string(),
                "NaiveDateTime" => "Date".to_string(),
                "DateTime" => "Date".to_string(),
                "Option" => match arguments {
                    syn::PathArguments::Parenthesized(parenthesized_argument) => {
                        format!("{parenthesized_argument:?}")
                    }
                    syn::PathArguments::AngleBracketed(anglebracketed_argument) => format!(
                        "{} | undefined",
                        generic_to_typsecript_type(anglebracketed_argument.args.first().unwrap())
                    ),
                    _ => "unknown".to_string(),
                },
                "Vec" => match arguments {
                    syn::PathArguments::Parenthesized(parenthesized_argument) => {
                        format!("{parenthesized_argument:?}")
                    }
                    syn::PathArguments::AngleBracketed(anglebracketed_argument) => format!(
                        "Array<{}>",
                        generic_to_typsecript_type(anglebracketed_argument.args.first().unwrap())
                    ),
                    _ => "unknown".to_string(),
                },
                _ => identifier.to_string(),
            }
        }
        _ => "unknown".to_string(),
    }
}
