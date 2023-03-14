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
            let cmd = match Command::to_vec(&#id(#arg_stream)) {
                Ok(cmd) => cmd,
                Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
            };
        ))
    } else {
        stream.extend::<TokenStream2>(quote!{
            let cmd = match Command::to_vec(&#id) {
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
        ident => {             
            let ident = TokenStream2::from_str(ident).unwrap();
            cmd_stream_extend = quote!(#ident);
            arg_stream_extend = quote!{#arg_i};
            stream_extend = handle_enum(ident,arg_i);
        },                  
    };
    (stream_extend,cmd_stream_extend,arg_stream_extend)
}

#[proc_macro_derive(Ground)]
pub fn ground_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let output = input.clone();
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let strukt = &ast.data;
    let fields = match strukt {
        syn::Data::Struct(d) => &d.fields,
        syn::Data::Enum(e) => {
            let name2 = format!("Gql{}",name);
            let gqlname = TokenStream2::from_str(&name2).unwrap();            
            let mut out_extend = quote!();
            let mut from_extend = quote!();
            for variant in e.variants.iter() {
                let v = &variant.ident;
                out_extend.extend::<TokenStream2>(quote!{
                    #v,
                });
                from_extend.extend::<TokenStream2>(quote!{
                    #name::#v => #gqlname::#v,
                });
            }
            let out = quote!{
                #[derive(GraphQLEnum)]
                pub enum #gqlname {
                    #out_extend
                }
                impl From<#name> for #gqlname {
                    fn from(e: #name) -> #gqlname {
                        match e {
                            #from_extend
                        }
                    }
                }
            };
            println!("{}",out);
            return out.into()

        }
        _ => return output,
    };

    // Build the trait implementation
    impl_ground(&name,&fields)
}

fn impl_ground(name: &syn::Ident, syn_fields: &syn::Fields) -> TokenStream {
    let name2 = format!("Gql{}",name);

    let gqlname = TokenStream2::from_str(&name2).unwrap();

    let mut strukt_stream = TokenStream2::default();
    let mut from_stream = TokenStream2::default();
    let mut tuple_stream = TokenStream2::default();

    if let syn::Fields::Named(FieldsNamed{named, .. }) = syn_fields {
        let fields = named.iter().map(|f| &f.ident);
        let ftypes = named.iter().map(|f| &f.ty);

        // for ft in ftypes.clone().into_iter() {
        //     println!{"{:?}",ft};
        // }

        for (field, ftype) in fields.into_iter().zip(ftypes.into_iter()) {
            strukt_stream.extend::<TokenStream2>(
                quote!{#field: }
            );
            from_stream.extend::<TokenStream2>(
                quote!{#field: }
            );
            let (strukt_extend,from_extend,tuple_extend) = match_types(field,ftype);
            strukt_stream.extend::<TokenStream2>(
                quote!{#strukt_extend,}
            );
            from_stream.extend::<TokenStream2>(
                quote!{#from_extend,}
            );
            tuple_stream.extend::<TokenStream2>(
                tuple_extend
            )
        }
    }

    let gen = quote! {
        #tuple_stream

        #[derive(GraphQLObject,Deserialize,Serialize)]
        pub struct #gqlname {
            #strukt_stream
        }
        
        impl From<#name> for #gqlname {
            fn from(n: #name) -> #gqlname {
                #gqlname {
                    #from_stream
                }
            }
        }
    };

    println!("{}",gen);
    gen.into()
}

fn match_types(
    field: &Option<syn::Ident>,
    ftype: &syn::Type,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let mut strukt_stream_extend = TokenStream2::default();
    let mut from_stream_extend = TokenStream2::default();
    let mut tuple_stream_extend = TokenStream2::default();

    match ftype {
        Type::Path(type_path) => {
            match type_path.clone().path.segments.first().unwrap().ident.to_string().as_str() {
                "u8" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "u16" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "u32" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "u64" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i8" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i16" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i32" => {
                    strukt_stream_extend = quote!{i32};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "i64" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "usize" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "isize" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "f32" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "f64" => {
                    strukt_stream_extend = quote!{f64};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "bool" => {
                    strukt_stream_extend = quote!{bool};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "String" => {
                    strukt_stream_extend = quote!{String};
                    from_stream_extend = quote!{n.#field.into()};
                }
                "&str" => {
                    strukt_stream_extend = quote!{String};
                    from_stream_extend = quote!{n.#field.to_string()};
                }
                "Vec" => {                   
                    let (gqltyp,_,_) = match &type_path.clone().path.segments.first().unwrap().arguments{
                        PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                            GenericArgument::Type(f) => match_types(field,f),     
                            _ => todo!(),                       
                        }                        
                        _ => todo!(),
                    };
                    from_stream_extend = quote!{
                        {
                            let mut v: Vec<#gqltyp> = Vec::new();
                            for i in n.#field.iter()  {
                                v.push(<#gqltyp>::from(*i));
                            }
                            v
                        }
                    };
                    strukt_stream_extend = quote!{Vec<#gqltyp>};
                }
                _ => {        
                    let f = type_path.clone().into_token_stream();
                    let name2 = format!("Gql{}",f);
                    let gqlname = TokenStream2::from_str(&name2).unwrap();          
                    strukt_stream_extend = quote!{#gqlname};
                    from_stream_extend = quote!{n.#field.into()};
                }                
            }      
        }
        // Tuple Type not implemented in GraphQL Object
        // Convert tuple to struct 
        Type::Tuple(type_tuple) => {
            from_stream_extend = quote!{n.#field.into()};            
            let mut gqlfields = TokenStream2::default();
            let mut elements = TokenStream2::default();
            let mut from_tuple = TokenStream2::default();
            let mut i: usize = 0;
            for elem in type_tuple.elems.iter() {
                let (gqlfield,_,_) = match_types(field,elem);
                gqlfields.extend::<TokenStream2>(TokenStream2::from_str(&format!("t_{}: {}",i,gqlfield)).unwrap());                                
                elements.extend::<TokenStream2>(quote!{#elem});
                if Some(elem) != type_tuple.elems.last() {
                    gqlfields.extend::<TokenStream2>(quote!{,});
                    elements.extend::<TokenStream2>(quote!{,});
                }
                // let tunder = TokenStream2::from_str(&format!("t_{}:",i)).unwrap();
                // let tdot = TokenStream2::from_str(&format!("t.{}",i)).unwrap();
                from_tuple.extend::<TokenStream2>(TokenStream2::from_str(&format!("t_{}: t.{}.into(),",i,i)).unwrap());
                i = i+1;
            }
            let gqlstruct = TokenStream2::from_str(&format!("Gql{}",field.clone().unwrap().to_string())).unwrap();            
            strukt_stream_extend = quote!{#gqlstruct};
            tuple_stream_extend = quote!{
                #[derive(GraphQLObject)]
                pub struct #gqlstruct {
                    #gqlfields
                }
                impl From<(#elements)> for #gqlstruct {
                    fn from(t: (#elements)) -> #gqlstruct {
                        #gqlstruct {
                            #from_tuple
                        }
                    }
                }
            };
        }
        Type::Array(type_array) => {          
            let typ = &type_array.elem;
            let (gqltyp,_,_) = match_types(field,typ);
            strukt_stream_extend = quote!{Vec<#gqltyp>};
            if let syn::Expr::Lit(expr_lit) = &type_array.len {
                from_stream_extend = quote!{
                    {
                        let mut v: Vec<#gqltyp> = Vec::new();
                        for i in 0..#expr_lit {
                            v.push(n.#field[i].into());
                        }
                        v
                    }
                };
            }
        }
        _ => {}                    
    };
    (strukt_stream_extend,from_stream_extend,tuple_stream_extend)
}