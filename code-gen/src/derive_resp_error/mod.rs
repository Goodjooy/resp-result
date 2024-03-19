use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::spanned::Spanned;
use crate::derive_resp_error::codegen::RespErrorCodeGen;
use crate::derive_resp_error::input::RespErrorDeriveInput;

mod input;
mod structure;
mod codegen;

pub fn gen_resp_error_derive(input:&DeriveInput)->syn::Result<TokenStream>{
    if !input.generics.params.is_empty(){
        Err(syn::Error::new(input.generics.span(),"Not support generic args"))?;
    }
    let input=RespErrorDeriveInput::from_derive_input(input)?;
    let codegen:RespErrorCodeGen = input.try_into()?;
    Ok(quote!(#codegen))
}