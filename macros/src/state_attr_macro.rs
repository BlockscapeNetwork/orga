use quote::quote;
use syn::*;
use super::*;

pub fn state_attr(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let mut item = parse_macro_input!(item as DeriveInput);

    let store_param: GenericParam = parse_quote!(S: orga::Store);
    item.generics.params.insert(0, store_param);

    struct_fields_mut(&mut item)
        .for_each(add_store_param_to_field);

    let name = &item.ident;
    let field_names = struct_fields(&item)
        .map(|field| &field.ident);

    (quote! {
        #item

        impl<S: orga::Store> orga::WrapStore<S> for #name<S> {
            fn wrap_store(store: S) -> ed::Result<Self> {
                let mut splitter = orga::Splitter::new(store);
                Ok(Self {
                    #(
                        #field_names: orga::WrapStore::wrap_store(splitter.split())?,
                    )*
                })
            }
        }
    }).into()
}