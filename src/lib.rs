use syn::parse_macro_input;
use syn::parse_quote;
use syn::PatIdent;
use syn::PatType;
use syn::ItemFn;
use syn::FnArg;
use syn::Error;
use syn::Meta;
use syn::Pat;

use quote::quote;
use quote::ToTokens;

use std::env::var;
use std::ops::Deref;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn smartlink(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let var_name = if !attrs.is_empty() {
        let attrs = parse_macro_input!(attrs as Meta);
        if let Meta::Path(path) = attrs {
            path.get_ident().unwrap().to_string()
        } else {
            let error = Error::new_spanned(attrs, "bruh");
            return TokenStream::from(error.into_compile_error());
        }
    } else {
        "SMARTLINK_NO_IMPL".into()
    };
    
    let mut input = parse_macro_input!(input as ItemFn);
    
    if let Ok(obj) = var(var_name) {
        let sig = input.sig.clone();
        let name = input.sig.ident.clone();
        
        let params = input.sig.inputs.iter().map(|arg| match arg {
            FnArg::Receiver(_) => quote!(self),
            FnArg::Typed(PatType { pat, .. }) => match pat.deref() {
                Pat::Ident(PatIdent { ident, .. }) => quote!(#ident),
                _ => return Error::new_spanned(arg, "unsupported arg notation").into_compile_error(),
            }
        });
        
        let body = quote! {{
            #[link(name = stringify!(#obj), kind = "dylib")]
            extern "Rust" {
                #sig;
            }
            unsafe { #name(#(#params),*) }
        }};

        input.block = parse_quote! { #body };
        TokenStream::from(input.to_token_stream())
    } else {
        let name = input.sig.ident.clone();
        TokenStream::from(quote! {
           #[export_name = stringify!(#name)]
           #input
        })
    }
}
