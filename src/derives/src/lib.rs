use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// TODO possibly use [darling](https://lib.rs/crates/darling) to make these fields configurable
#[proc_macro_derive(Names)]
pub fn derive_names(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let singular = ident.to_string().to_lowercase();
    let plural = singular.clone() + "s";
    quote! {
        impl Names for #ident {
            const NAME_SINGULAR: &'static str = #singular;
            const NAME_PLURAL: &'static str = #plural;
        }
    }
    .into()
}

#[proc_macro_derive(CRUD)]
pub fn derive_crud(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    quote! {
        impl CRUD for #ident { }
    }
    .into()
}

#[proc_macro_derive(Queryable)]
pub fn derive_queryable(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    quote! {
        impl Queryable for #ident { }
    }
    .into()
}

#[proc_macro_derive(Removeable)]
pub fn derive_removeable(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    quote! {
        impl Removeable for #ident { }
    }
    .into()
}

#[proc_macro_derive(Id)]
pub fn derive_id(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    quote! {
        impl Id for #ident {
            async fn id(&self) -> Uuid {
                self.id.clone()
            }
        }
    }
    .into()
}
