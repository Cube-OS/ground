extern crate proc_macro2;
extern crate proc_macro;
extern crate syn;
extern crate quote;

use syn::*;
use syn::punctuated::Punctuated;
use syn::parse::Parser;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use std::str::FromStr;
use std::marker::PhantomData;
use std::cell::Cell;
use std::rc::Rc;
use std::fmt::Display;

// struct ItemStruct {
//     struct_token: Token![struct],
//     ident: Ident,
//     brace_token: token::Brace,
//     fields: Punctuated<Field, Token![,]>,
// }

#[proc_macro]
pub fn ground_handle(tokens: TokenStream) -> TokenStream {
    // 
    //     $types, $($cmd),*
    // 
    let input = tokens.clone();
    let parser = Punctuated::<TypePath, Token![,]>::parse_terminated;
    let mut args = parser.parse(tokens).unwrap();
    let mut i: usize = 0;

    let mut stream = TokenStream2::default();
    let mut cmd_stream = TokenStream2::default();
    let mut arg_stream = TokenStream2::default();

    let typ = args.pop().unwrap().into_value();
    let id = &typ.path.segments.first().unwrap().ident;

    let mut msg: Vec<TokenStream2> = Vec::new();
    let length = args.len();
    for l in 0..length/2 {
        let int = args.pop().unwrap().into_value();
        let int2 = &int.path.segments.first().unwrap().ident.to_string();
        // msg.insert(0,quote!{println!("{}",format!("{}",#int2));});
        msg.insert(0,quote!{print!("{} ",#int2);});
    }

    if !args.is_empty() {
        for a in args.iter() {
            let arg_i = TokenStream2::from_str(&format!("arg_{}",i)).unwrap();
            let (stream_ext, cmd_stream_ext, arg_stream_ext) = match_types_func(a,arg_i);
            let m = &msg[i];
            stream.extend::<TokenStream2>(quote!(#m));
            stream.extend::<TokenStream2>(quote!(#stream_ext));
            if i > 0 {
                cmd_stream.extend::<TokenStream2>(quote!(,#cmd_stream_ext));
                arg_stream.extend::<TokenStream2>(quote!(,#arg_stream_ext));
            } else {
                cmd_stream.extend::<TokenStream2>(quote!(#cmd_stream_ext));
                arg_stream.extend::<TokenStream2>(quote!(#arg_stream_ext));
            }            
            i = i+1;
        }    
        stream.extend::<TokenStream2>(quote!(
            let cmd = match Command::<CommandID,(#cmd_stream)>::serialize(CommandID::#id,(#arg_stream)) {
                Ok(cmd) => cmd,
                Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
            };
        ))
    } else {
        stream.extend::<TokenStream2>(quote!{
            let cmd = match Command::<CommandID,()>::serialize(CommandID::#id,()) {
                Ok(cmd) => cmd,
                Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
            };
        });
    }

    let gen = quote!{
        #stream
    };
    println!("{}",gen);
    println!("");
    gen.into()
}

fn type_stream(token: TokenStream2, arg_i: TokenStream2) -> TokenStream2 {
    let t: String = quote!(#token).to_string();
    let r = quote!{
        println!("[{}]:",#t);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let #arg_i = serde_json::from_str::<#token>(&input).unwrap();
    };
    r
}

fn handle_enum(ident: TokenStream2, arg_i: TokenStream2) -> TokenStream2 {
    let stream = quote!{
        let mut items: Vec<#ident> = Vec::new();
        for field in #ident::iter() {
            items.push(field);
        }
        let #arg_i = match Select::new().items(&items).interact_opt().unwrap() {
            Some(index) => items[index].clone(),
            None => items[0].clone(),
        };  
    };
    stream
}

fn match_types_func(
    ftype: &syn::TypePath,
    arg_i: TokenStream2,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let mut stream_extend = TokenStream2::default();
    let mut cmd_stream_extend = TokenStream2::default();
    let mut arg_stream_extend = TokenStream2::default();
    // let mut tuple_stream_extend = TokenStream2::default();

    match ftype.clone().path.segments.first().unwrap().ident.to_string().as_str() {
        "u8" => {
            let token = TokenStream2::from_str("u8").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);                    
        }
        "u16" => {
            let token = TokenStream2::from_str("u16").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "u32" => {
            let token = TokenStream2::from_str("u32").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "u64" => {
            let token = TokenStream2::from_str("u64").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "i8" => {
            let token = TokenStream2::from_str("i8").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "i16" => {
            let token = TokenStream2::from_str("i16").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "i32" => {
            let token = TokenStream2::from_str("i32").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "i64" => {
            let token = TokenStream2::from_str("i64").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "usize" => {
            let token = TokenStream2::from_str("usize").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "isize" => {
            let token = TokenStream2::from_str("isize").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "f32" => {
            let token = TokenStream2::from_str("f32").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "f64" => {
            let token = TokenStream2::from_str("f64").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "bool" => {
            let token = TokenStream2::from_str("bool").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "String" => {
            let token = TokenStream2::from_str("String").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "str" => {
            let token = TokenStream2::from_str("str").unwrap();
            cmd_stream_extend = quote!(#token);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = type_stream(token,arg_i);
        }
        "Vec" => {                   
            let (stream,cmd,arg) = match &ftype.clone().path.segments.first().unwrap().arguments{
                PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                    GenericArgument::Type(t) => match t {
                        Type::Path(f) => match_types_func(f,arg_i.clone()),
                        _ => todo!(),
                    },
                    _ => todo!(),                       
                }                        
                _ => todo!(),
            };
            let ident = TokenStream2::from_str(&format!("Vec<{}>",cmd)).unwrap();
            cmd_stream_extend = ident.clone();
            arg_stream_extend = quote!(#arg_i);
            stream_extend = type_stream(ident,arg_i);
        }
        "Option" => {                   
            let (stream,cmd,arg) = match &ftype.clone().path.segments.first().unwrap().arguments{
                PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                    GenericArgument::Type(t) => match t {
                        Type::Path(f) => match_types_func(f,arg_i.clone()),
                        _ => todo!(),
                    },
                    _ => todo!(),                       
                }                        
                _ => todo!(),
            };
            let ident = TokenStream2::from_str(&format!("Option<{}>",cmd)).unwrap();
            cmd_stream_extend = ident.clone();
            arg_stream_extend = quote!(#arg_i);
            stream_extend = type_stream(ident,arg_i);
        }
        ident => {             
            let ident = TokenStream2::from_str(ident).unwrap();
            cmd_stream_extend = quote!(#ident);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = handle_enum(ident,arg_i);
        },                  
    };
    (stream_extend,cmd_stream_extend,arg_stream_extend)
}