use super::params::is_primitive_type;
use super::processor::HttpVerb;

pub struct HookPathParam {
    pub hook_arg_name: String,
    pub hook_arg_type: String,
}

pub struct HookQueryParam {
    pub hook_arg_name: String,
    pub hook_arg_type: String,
}

pub struct HookBodyParam {
    pub hook_arg_name: String,
    pub hook_arg_type: String,
}

pub struct Hook {
    pub hook_name: String,
    pub endpoint_url: String,
    pub endpoint_verb: HttpVerb,
    pub uses_auth: bool,
    pub return_type: String,
    pub is_mutation: bool,

    pub query_key_base: String,

    // params
    pub query_params: Vec<HookQueryParam>,
    pub body_params: Vec<HookBodyParam>,
    pub path_params: Vec<HookPathParam>,
}

///
/// A react hook which uses react-query to pull data from some endpoint. Here's an example:
///
/// ```
/// export const useCenters = (params: PaginationParams) => {
///   return useQuery<Centers[]>(
///    ['centers', params],
///    async () => await (await fetch(`/api/centers?page=${params.page}&page_size=${params.page_size}`)).json()
///   )
/// }
/// ```
impl Hook {
    fn build_args_string(&self) -> String {
        //=============================== Build Hook Args

        let mut hook_args = String::new();

        for (index, arg) in self.path_params.iter().enumerate() {
            hook_args.push_str(&format!("{}: {}", arg.hook_arg_name, arg.hook_arg_type));
            if index != self.path_params.len() - 1 {
                hook_args.push_str(", ");
            }
        }

        if self.path_params.len() > 0 && self.query_params.len() > 0 {
            hook_args.push_str(", ");
        }

        for (index, arg) in self.query_params.iter().enumerate() {
            hook_args.push_str(&format!("{}: {}", arg.hook_arg_name, arg.hook_arg_type));
            if index != self.query_params.len() - 1 {
                hook_args.push_str(", ");
            }
        }

        if (self.path_params.len() > 0 || self.query_params.len() > 0) && self.body_params.len() > 0
        {
            hook_args.push_str(", ");
        }

        for (index, arg) in self.body_params.iter().enumerate() {
            hook_args.push_str(&format!("{}: {}", arg.hook_arg_name, arg.hook_arg_type));
            if index != self.body_params.len() - 1 {
                hook_args.push_str(", ");
            }
        }

        hook_args
    }

    fn build_vars_string(&self) -> String {
        //=============================== Build Variables

        let mut variables = String::new();

        if self.uses_auth {
            variables.push_str("  const auth = useAuth()\n");
        }

        if self.is_mutation {
            variables.push_str("  const queryClient = useQueryClient()\n");
        }

        if self.query_params.len() > 0 {
            variables.push_str("  const queryParams: Record<string, any> = Object.assign({}, ");
            let query_param_iter = self.query_params.iter();
            for (index, param) in query_param_iter.clone().enumerate() {
                let is_primitive_type = is_primitive_type(param.hook_arg_type.clone());

                if is_primitive_type {
                    variables.push_str("{ ");
                }
                if self.is_mutation && !is_primitive_type {
                    variables.push_str("params.");
                }
                if self.is_mutation && is_primitive_type {
                    variables.push_str(param.hook_arg_name.as_str());
                    variables.push_str(": params.");
                }
                variables.push_str(param.hook_arg_name.as_str());
                if is_primitive_type {
                    variables.push_str(" }");
                }
                if index != query_param_iter.len() - 1 {
                    variables.push_str(", ");
                }
            }
            variables.push_str(")\n");
        }

        if self.path_params.len() > 0 {
            variables.push_str("  const pathParams = Object.assign({}, ");
            let path_param_iter = self.path_params.iter();
            for (index, param) in path_param_iter.clone().enumerate() {
                variables.push_str("{ ");
                if self.is_mutation {
                    variables.push_str(param.hook_arg_name.as_str());
                    variables.push_str(": params.");
                }
                variables.push_str(param.hook_arg_name.as_str());
                variables.push_str(" }");
                if index != path_param_iter.len() - 1 {
                    variables.push_str(", ");
                }
            }
            variables.push_str(")\n");
        }

        if self.body_params.len() > 0 {
            variables.push_str("  const bodyParams = Object.assign({}, ");
            let body_param_iter = self.body_params.iter();
            for (index, param) in body_param_iter.clone().enumerate() {
                let is_primitive_type = is_primitive_type(param.hook_arg_type.clone());
                if is_primitive_type {
                    variables.push_str("{ ");
                }
                if self.is_mutation && !is_primitive_type {
                    variables.push_str("params.");
                }
                if self.is_mutation && is_primitive_type {
                    variables.push_str(param.hook_arg_name.as_str());
                    variables.push_str(": params.");
                }
                variables.push_str(param.hook_arg_name.as_str());
                if is_primitive_type {
                    variables.push_str(" }");
                }

                if index != body_param_iter.len() - 1 {
                    variables.push_str(", ");
                }
            }
            variables.push_str(")\n");
        }

        variables
    }

    fn build_query_key(&self) -> String {
        //=============================== Build Query Key

        let mut query_key = String::new();

        for (index, arg) in self.path_params.iter().enumerate() {
            if self.is_mutation {
                query_key.push_str("params.");
            }
            query_key.push_str(&arg.hook_arg_name);
            if index != self.path_params.len() - 1 {
                query_key.push_str(", ");
            }
        }

        if self.path_params.len() > 0 && self.query_params.len() > 0 {
            query_key.push_str(", ");
        }

        for (index, arg) in self.query_params.iter().enumerate() {
            if self.is_mutation {
                query_key.push_str("params.");
            }
            query_key.push_str(&arg.hook_arg_name);
            if index != self.query_params.len() - 1 {
                query_key.push_str(", ");
            }
        }

        if (self.path_params.len() > 0 || self.query_params.len() > 0) && self.body_params.len() > 0
        {
            query_key.push_str(", ");
        }

        for (index, arg) in self.body_params.iter().enumerate() {
            if self.is_mutation {
                query_key.push_str("params.");
            }
            query_key.push_str(&arg.hook_arg_name);
            if index != self.body_params.len() - 1 {
                query_key.push_str(", ");
            }
        }

        if query_key.len() > 0 {
            query_key.insert_str(0, ", ")
        }
        query_key.insert_str(0, &format!("{:?}", self.query_key_base));

        query_key
    }

    pub fn to_string(&self) -> String {
        if self.is_mutation {
            format!(
                r#"export const {hook_name} = (params: {{{hook_args}}}) => {{
{variables}  return useMutation<{return_type}>(
        async () => await (await fetch(`{endpoint_url}{query_string}`, {{
            method: '{endpoint_verb}',
            {query_body}headers: {{
                {authorization_header}'Content-Type': 'application/json',
            }},
        }})).json(),
        {{
            onSuccess: () => queryClient.invalidateQueries([{query_key}]),
        }}
    )
}}"#,
                variables = self.build_vars_string(),
                authorization_header = if self.uses_auth {
                    "'Authorization': `${auth.accessToken}`,\n              "
                } else {
                    ""
                },
                query_body = if self.body_params.len() > 0 {
                    "body: JSON.stringify(bodyParams),\n"
                } else {
                    ""
                },
                query_string = if self.query_params.len() > 0 {
                    "?${new URLSearchParams(queryParams).toString()}"
                } else {
                    ""
                },
                endpoint_url = self.endpoint_url.replace("{", "${pathParams."),
                endpoint_verb = &format!("{:?}", self.endpoint_verb),
                hook_name = self.hook_name,
                hook_args = self.build_args_string(),
                return_type = self.return_type.trim_matches('"'),
                query_key = self.build_query_key()
            )
            .to_string()
        } else {
            format!(
                r#"export const {hook_name} = ({hook_args}) => {{
{variables}  return useQuery<{return_type}>(
        [{query_key}],
        async () => await (await fetch(`{endpoint_url}{query_string}`, {{
            method: '{endpoint_verb}',
            {query_body}headers: {{
                {authorization_header}'Content-Type': 'application/json',
            }},
        }})).json()
    )
}}"#,
                variables = self.build_vars_string(),
                authorization_header = if self.uses_auth {
                    "'Authorization': `${auth.accessToken}`,\n              "
                } else {
                    ""
                },
                query_body = if self.body_params.len() > 0 {
                    "body: JSON.stringify(bodyParams),\n"
                } else {
                    ""
                },
                query_string = if self.query_params.len() > 0 {
                    "?${new URLSearchParams(queryParams).toString()}"
                } else {
                    ""
                },
                endpoint_url = self.endpoint_url.replace("{", "${pathParams."),
                endpoint_verb = &format!("{:?}", self.endpoint_verb),
                hook_name = self.hook_name,
                hook_args = self.build_args_string(),
                return_type = self.return_type.trim_matches('"'),
                query_key = self.build_query_key()
            )
            .to_string()
        }
    }
}
