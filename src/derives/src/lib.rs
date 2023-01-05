use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

// TODO possibly use [darling](https://lib.rs/crates/darling) to make these fields configurable
#[proc_macro_derive(DbTable)]
pub fn derive_db_table(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let singular = ident.to_string().to_lowercase();
    let plural = singular.clone() + "s";
    quote! {
        impl DbTable for #ident {
            const NAME_SINGULAR: &'static str = #singular;
            const NAME_PLURAL: &'static str = #plural;
        }
    }
    .into()
}
