/*
The #[qsync] attribute was removed:
• is_mutation can be inferred from the HTTP verb, and,
• return_type can be added in by the user after after the hook is generated.
*/

// extern crate proc_macro;

// use proc_macro::TokenStream;

// // document this attribute
// #[proc_macro_attribute]
// pub fn qsync(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     item
// }
