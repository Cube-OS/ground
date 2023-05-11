extern crate proc_macro2;
extern crate proc_macro;
extern crate syn;
extern crate quote;

use syn::{File,Ident,Item,parse2,DeriveInput,ItemStruct,TypePath,Token,PathArguments,Data,GenericArgument,Type,UseTree,UsePath};
// use syn::*;
use std::fs;
use std::path::{Path,PathBuf};
use syn::punctuated::Punctuated;
use syn::parse::Parser;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree;
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
                // Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
                Err(e) => return (),
            };
        ))
    } else {
        stream.extend::<TokenStream2>(quote!{
            let cmd = match Command::<CommandID,()>::serialize(CommandID::#id,()) {
                Ok(cmd) => cmd,
                // Err(e) => return serde_json::to_string_pretty(&e).unwrap(),
                Err(e) => return (),
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

fn struct_stream(name: &Ident, typ: Type) -> TokenStream2 {
    let n: String = quote!(#name).to_string();
    let r = quote!{
        #name: {
            println!("[{}]:",#n);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            serde_json::from_str::<#typ>(&input).unwrap()
        },    
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
// fn handle_struct_enum(name: &Ident, field_type: &Type) -> TokenStream2 {
//     let stream = quote!{
//         #name: {
//             let mut items: Vec<#field_type> = Vec::new();
//             for field in #field_type::iter() {
//                 items.push(field);
//             }
//             match Select::new().items(&items).interact_opt().unwrap() {
//                 Some(index) => items[index].clone(),
//                 None => items[0].clone(),
//             }
//         }          
//     };
//     stream
// }

// fn handle_vec_enum(ident: TokenStream2, arg_i: TokenStream2) -> TokenStream2 {
//     let stream = quote!{
//         let mut items: Vec<#ident> = Vec::new();
//         for field in #ident::iter() {
//             items.push(field);
//         }
//         let mut vec: Vec<#ident> = Vec::new();
//         loop {
//             let mut input = String::new();
//             println!("Add another item? (y/n)");
//             std::io::stdin().read_line(&mut input).unwrap();
//             if input.trim() == "n" {
//                 break;
//             }
//             let item = match Select::new().items(&items).interact_opt().unwrap() {
//                 Some(index) => items[index].clone(),
//                 None => items[0].clone(),
//             };
//             vec.push(item);            
//         }
//         let arg_i = vec;   
//     };
//     stream
// }

// fn handle_struct_vec_enum(name: &Ident, field_type: &Type) -> TokenStream2 {
//     let stream = quote!{
//         #name: {
//             let mut items: Vec<#field_type> = Vec::new();
//             for field in #field_type::iter() {
//                 items.push(field);
//             }
//             let mut vec: Vec<#field_type> = Vec::new();
//             loop {
//                 let mut input = String::new();
//                 println!("Add another item? (y/n)");
//                 std::io::stdin().read_line(&mut input).unwrap();
//                 if input.trim() == "n" {
//                     break;
//                 }
//                 let item = match Select::new().items(&items).interact_opt().unwrap() {
//                     Some(index) => items[index].clone(),
//                     None => items[0].clone(),
//                 };
//                 vec.push(item);                
//             }
//             vec
//         }          
//     };
//     stream
// }

// fn handle_struct_opt_enum(name: &Ident, field_type: &Type) -> TokenStream2 {
//     let stream = quote!{
//         #name: {
//             let mut items: Vec<#field_type> = Vec::new();
//             for field in #field_type::iter() {
//                 items.push(field);
//             }
//             Select::new().items(&items).interact_opt().unwrap()
//         }          
//     };
//     stream
// }

// fn handle_struct_vec_opt_enum(name: &Ident, field_type: &Type) -> TokenStream2 {
//     let stream = quote!{
//         #name: {
//             let mut items: Vec<#field_type> = Vec::new();
//             for field in #field_type::iter() {
//                 items.push(field);
//             }
//             let mut vec: Vec<#field_type> = Vec::new();
//             loop {    
//                 let mut input = String::new();            
//                 println!("Add another item? (y/n)");
//                 std::io::stdin().read_line(&mut input).unwrap();
//                 if input.trim() == "n" {
//                     break;
//                 }
//                 match Select::new().items(&items).interact_opt().unwrap() {
//                     Some(index) => vec.push(items[index].clone()),
//                     None => vec.push(items[0].clone()),
//                 };
//             }
//             vec
//         }          
//     };
//     stream
// }

// fn handle_struct_opt_vec_enum(name: &Ident, field_type: &Type) -> TokenStream2 {
//     let stream = quote!{
//         #name: {
//             let mut items: Vec<#field_type> = Vec::new();
//             for field in #field_type::iter() {
//                 items.push(field);
//             }
//             let mut vec: Vec<#field_type> = Vec::new();
//             loop {
//                 vec.push(Select::new().items(&items).interact_opt().unwrap());                
//                 let mut input = String::new();
//                 println!("Add another item? (y/n)");
//                 std::io::stdin().read_line(&mut input).unwrap();
//                 if input.trim() == "n" {
//                     break;
//                 }
//             }
//             vec
//         }          
//     };
//     stream
// }

// fn handle_vec_struct(name: &Ident, typ: TokenStream2, arg_i: TokenStream2) -> TokenStream2 {
//     find_struct_definition(&parse2::<Ident>(typ.clone()).unwrap()).map(|item| {
//         match item {
//             Item::Struct(item_struct) => {
//                 let stream = quote!{
//                     let #arg_i: Vec<#typ> = Vec::new();
//                     loop {                        
//                         let mut input = String::new();
//                         println!("Add another item? (y/n)");
//                         std::io::stdin().read_line(&mut input).unwrap();
//                         if input.trim() == "n" {
//                             break;
//                         }
//                         #arg_i.push(handle_struct_fields(&item_struct));                        
//                     } 
//                 };   
//                 stream                     
//             },
//             _ => panic!("not a struct"),
//         }
//     }).unwrap()
// }

// fn handle_struct_vec_struct(name: &Ident, typ: TokenStream2) -> TokenStream2 {
    
    
// }

// fn handle_struct_vec_opt_struct(name: &Ident, typ: TokenStream2) -> TokenStream2 {
//     todo!()
// }

// fn handle_struct_opt_struct(name: &Ident, typ: TokenStream2) -> TokenStream2 {
//     todo!()
// }

// fn handle_struct_opt_vec_struct(name: &Ident, typ: TokenStream2) -> TokenStream2 {
//     todo!()
// }

// fn handle_struct_fields(item_struct: &ItemStruct) -> TokenStream2 {
//     let data = &item_struct.fields;
//     let field_streams = data.iter().map(|field| {
//         let field_name = field.ident.as_ref().unwrap();
//         let field_type = &field.ty;

//         let f = quote!(#field_name).to_string();

//         if let Type::Path(type_path) = field_type {
//             let segment_id = &type_path.path.segments.first().unwrap().ident;
//             match segment_id.to_string().as_str() {
//                 "Vec" => {
//                     let field_type_args = &type_path.path.segments.first().unwrap().arguments;
//                     match field_type_args {
//                         PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
//                             GenericArgument::Type(t) => match t {
//                                 Type::Path(f) => {
//                                     let id = &f.path.segments.first().unwrap().ident;
//                                     match id.to_string().as_str() {
//                                         "u8" | "u16" | "u32" | "u64" |
//                                         "i8" | "i16" | "i32" | "i64" |
//                                         "f32" | "f64" | "String" | "bool" => {
//                                             return struct_stream(field_name, field_type.clone())
//                                         }
//                                         "Option" => {
//                                             let field_type_vec_args = &f.path.segments.first().unwrap().arguments;
//                                             match field_type_vec_args {
//                                                 PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
//                                                     GenericArgument::Type(t) => match t {
//                                                         Type::Path(f) => {
//                                                             let id = &f.path.segments.first().unwrap().ident;
//                                                             match id.to_string().as_str() {
//                                                                 "u8" | "u16" | "u32" | "u64" |
//                                                                 "i8" | "i16" | "i32" | "i64" |
//                                                                 "f32" | "f64" | "String" | "bool" => {
//                                                                     return struct_stream(field_name, field_type.clone())
//                                                                 }
//                                                                 _ => {
//                                                                     let type_ident = id.to_string();                                            
                                            
//                                                                     match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", type_ident.to_string())) {
//                                                                         Ok(DeriveInput {
//                                                                             data: Data::Struct(_),
//                                                                             ..
//                                                                         }) => return handle_struct_vec_opt_struct(field_name, type_ident.to_string().parse::<TokenStream2>().unwrap()),
//                                                                         Ok(DeriveInput {
//                                                                             data: Data::Enum(_),
//                                                                             ..
//                                                                         }) => return handle_struct_vec_opt_enum(field_name, field_type),
//                                                                         _ => todo!(),
//                                                                     }
//                                                                 }                                                                
//                                                             }                                                            
//                                                         }
//                                                         _ => todo!(),
//                                                     }
//                                                     _ => todo!(),
//                                                 }
//                                                 _ => todo!(),
//                                             }
//                                         }
//                                         _ => {
//                                             let type_ident = id.to_string();                                            
                                            
//                                             match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", type_ident.to_string())) {
//                                                 Ok(DeriveInput {
//                                                     data: Data::Struct(_),
//                                                     ..
//                                                 }) => return handle_struct_vec_struct(field_name, type_ident.to_string().parse::<TokenStream2>().unwrap()),
//                                                 Ok(DeriveInput {
//                                                     data: Data::Enum(_),
//                                                     ..
//                                                 }) => return handle_struct_vec_enum(field_name, field_type),
//                                                 _ => todo!(),
//                                             }
//                                         }
//                                     }
//                                 }
//                                 _ => todo!(),
//                             }
//                             _ => todo!(),
//                         }
//                         _ => todo!(),
//                     }
//                 }
//                 "Option" => {
//                     let field_type_id = &type_path.path.segments.first().unwrap().arguments;
//                     match field_type_id {
//                         PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
//                             GenericArgument::Type(t) => match t {
//                                 Type::Path(f) => {
//                                     let id = &f.path.segments.first().unwrap().ident;
//                                     match id.to_string().as_str() {
//                                         "u8" | "u16" | "u32" | "u64" |
//                                         "i8" | "i16" | "i32" | "i64" |
//                                         "f32" | "f64" | "String" | "bool" => {
//                                             return struct_stream(field_name, field_type.clone())
//                                         }
//                                         "Vec" => {
//                                             let field_type_vec_args = &f.path.segments.first().unwrap().arguments;
//                                             match field_type_vec_args {
//                                                 PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
//                                                     GenericArgument::Type(t) => match t {
//                                                         Type::Path(f) => {
//                                                             let id = &f.path.segments.first().unwrap().ident;
//                                                             match id.to_string().as_str() {
//                                                                 "u8" | "u16" | "u32" | "u64" |
//                                                                 "i8" | "i16" | "i32" | "i64" |
//                                                                 "f32" | "f64" | "String" | "bool" => {
//                                                                     return struct_stream(field_name, field_type.clone())
//                                                                 }
//                                                                 _ => {
//                                                                     let type_ident = id.to_string();                                            
                                            
//                                                                     match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", type_ident.to_string())) {
//                                                                         Ok(DeriveInput {
//                                                                             data: Data::Struct(_),
//                                                                             ..
//                                                                         }) => return handle_struct_opt_vec_struct(field_name, type_ident.to_string().parse::<TokenStream2>().unwrap()),
//                                                                         Ok(DeriveInput {
//                                                                             data: Data::Enum(_),
//                                                                             ..
//                                                                         }) => return handle_struct_opt_vec_enum(field_name, field_type),
//                                                                         _ => todo!(),
//                                                                     }
//                                                                 }                                                                
//                                                             }                                                            
//                                                         }
//                                                         _ => todo!(),
//                                                     }
//                                                     _ => todo!(),
//                                                 }
//                                                 _ => todo!(),
//                                             }
//                                         }
//                                         _ => {
//                                             let type_ident = id.to_string();                                            
                                            
//                                             match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", type_ident.to_string())) {
//                                                 Ok(DeriveInput {
//                                                     data: Data::Struct(_),
//                                                     ..
//                                                 }) => return handle_struct_opt_struct(field_name, type_ident.to_string().parse::<TokenStream2>().unwrap()),
//                                                 Ok(DeriveInput {
//                                                     data: Data::Enum(_),
//                                                     ..
//                                                 }) => return handle_struct_opt_enum(field_name, field_type),
//                                                 _ => todo!(),
//                                             }
//                                         }
//                                     }
//                                 }
//                                 _ => todo!(),
//                             }
//                             _ => todo!(),
//                         }
//                         _ => todo!(),
//                     }
//                 },
//                 "u8" | "u16" | "u32" | "u64" |
//                 "i8" | "i16" | "i32" | "i64" |
//                 "f32" | "f64" | "String" | "bool" => {
//                     return struct_stream(field_name, field_type.clone())
//                 }
//                 id => {
//                     let type_ident = id.to_string();
//                     // let type_ident = quote!(#id);
                    
//                     match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", type_ident.to_string())) {
//                         Ok(DeriveInput {
//                             data: Data::Struct(_),
//                             ..
//                         }) => return handle_struct_struct(field_name, type_ident.to_string().parse::<TokenStream2>().unwrap()),
//                         Ok(DeriveInput {
//                             data: Data::Enum(_),
//                             ..
//                         }) => return handle_struct_enum(field_name, field_type),
//                         _ => todo!(),
//                     }
//                 }
//             }
//         }
//         match field_type {
//             Type::Path(type_path) => {
//                 let segment_id = &type_path.path.segments.first().unwrap().ident;
                
//                 handle_field_type(segment_id, field_name, field_type, &type_path)                          
//             }
//             _ => todo!(),
//         }

//         struct_stream(field_name, field_type.clone())


//         quote!{
//             #field_name: {
//                 println!("{}:",#f);
//                 let mut input = String::new();
//                 std::io::stdin().read_line(&mut input).unwrap();
//                 serde_json::from_str::<#field_type>(&input).unwrap()
//             },
//         }
//     }).collect::<TokenStream2>();

//     field_streams
// }

// fn handle_struct_struct(field_name: &Ident, ident: TokenStream2) -> TokenStream2 {
//     println!("ident: {}", quote!(#ident).to_string());
//     let ident = parse2::<Ident>(quote!(#ident).to_string().parse::<TokenStream2>().unwrap()).unwrap();
//     find_struct_definition(&ident.clone()).map(|item| {
//         match item {
//             Item::Struct(item_struct) => {
//                 let field_streams = handle_struct_fields(&item_struct);
//                 let stream = quote!{
//                     #field_name: #ident {
//                         #field_streams
//                     },
//                 };
//                 stream
//             },
//             _ => panic!("not a struct"),
//         }
//     }).unwrap()
// }

fn handle_struct_fields(item: ItemStruct) -> TokenStream2 {
    let data = &item.fields;
    let field_streams = data.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let f = quote!(#field_name).to_string();

        if let Type::Path(type_path) = field_type {
            let segment_id = &type_path.path.segments.first().unwrap().ident;
            match segment_id.to_string().as_str() {
                "u8" | "u16" | "u32" | "u64" |
                "i8" | "i16" | "i32" | "i64" |
                "f32" | "f64" | "String" | "bool" => {
                    struct_stream(field_name, field_type.clone())
                },
                "Vec" => {
                    let cmd = match &type_path.clone().path.segments.first().unwrap().arguments{
                        PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                            GenericArgument::Type(t) => match t {
                                Type::Path(f) => f.path.segments.first().unwrap().ident.to_string(),
                                _ => todo!(),
                            },
                            _ => todo!(),                       
                        }                        
                        _ => todo!(),
                    };
                    let ident = syn::parse2::<Ident>(TokenStream2::from_str(&format!("Vec<{}>",cmd)).unwrap()).unwrap();
                    struct_stream(&ident, field_type.clone())
                }
                "Option" => {
                    let cmd = match &type_path.clone().path.segments.first().unwrap().arguments{
                        PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
                            GenericArgument::Type(t) => match t {
                                Type::Path(f) => f.path.segments.first().unwrap().ident.to_string(),
                                _ => todo!(),
                            },
                            _ => todo!(),                       
                        }                        
                        _ => todo!(),
                    };
                    let ident = syn::parse2::<Ident>(TokenStream2::from_str(&format!("Option<{}>",cmd)).unwrap()).unwrap();
                    struct_stream(&ident, field_type.clone())
                }
                _ => TokenStream2::new(),
            }
        } else {
            TokenStream2::new()
        }
    }).collect::<TokenStream2>();

    field_streams
}

fn handle_struct(ident: TokenStream2, arg_i: TokenStream2) -> TokenStream2 {
    find_struct_definition(&parse2::<Ident>(ident.clone()).unwrap()).map(|item| {
        match item {
            Item::Struct(item_struct) => {
                let field_streams = handle_struct_fields(item_struct);
                let stream = quote!{
                    let #arg_i = #ident {
                        #field_streams
                    };
                };
                stream
            },
            _ => panic!("not a struct"),
        }
    }).unwrap()
}

fn recursive_find_path(use_path: &UsePath, ident: &Ident) -> Option<String> {
    println!("{} {}",use_path.ident,ident);
    if use_path.ident == *ident {
        println!("found path: {}",use_path.to_token_stream().to_string());
        Some(use_path.to_token_stream().to_string())
    } else {
        if let use_tree = use_path.tree.as_ref() {
            // println!("tree: {}",use_tree.to_token_stream().to_string());
            // println!("{:?}",use_tree);
            match use_tree {
                UseTree::Path(use_path) => recursive_find_path(use_path, ident),
                UseTree::Name(use_name) => {
                    if use_name.ident == *ident {
                        println!("found path: {}",use_name.to_token_stream().to_string());
                        Some(use_name.to_token_stream().to_string())
                    } else {
                        println!("not found");
                        None
                    }
                }
                _ => None,
            }
        } else {
            println!("no tree");
            None
        }        
    }
}

fn find_path(file_ast: syn::File, ident: &Ident) -> Option<String> {
    for item in file_ast.items.clone() {
        match item {
            Item::Use(item_use) => {
                if let UseTree::Path(use_path) = item_use.tree {
                    match recursive_find_path(&use_path, ident) {
                        Some(_) => return Some(use_path.to_token_stream().to_string()),
                        None => (),
                    }
                }
            },
            _ => (),
        }
    }
    None
}

fn find_struct_definition(ident: &Ident) -> Option<Item> {
    // Get the file path of the current module - fix this to /src/service.rs for now
    let module_path = Path::new(&std::env::current_dir().unwrap()).join("src").join("service.rs");
    let file_content = fs::read_to_string(module_path).unwrap();    
    // Parse the file into a Syn abstract syntax tree (AST)
    let file_ast = syn::parse_file(&file_content).unwrap();
    // println!("file_ast: {:?}",file_ast);

    match find_path(file_ast.clone(), ident) {
        Some(path) => {
            if path.contains("crate ::") {
                let path = path.split("::").collect::<Vec<&str>>();
                let krate = path[path.len()-2];                
                println!("{}",krate);
                let module_path = Path::new(&std::env::current_dir().unwrap()).join("src").join((String::from(krate)+".rs").replace(" ",""));
                println!("{:?}",module_path);
                let file_content = fs::read_to_string(module_path).unwrap();
                let file_ast = syn::parse_file(&file_content).unwrap();
                println!("here");
                
                for item in file_ast.items {
                    match item {
                        Item::Struct(item_struct) => {
                            if item_struct.ident == *ident {
                                return Some(Item::Struct(item_struct));
                            }
                        }
                        _ => (),
                    }
                }
            }
        },
        None => (),
    }
    None
}

fn handle_ident(ident: TokenStream2, arg_i: TokenStream2) -> TokenStream2 {
    match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", ident.to_string())) {
        Ok(DeriveInput {
            data: Data::Struct(_),
            ..
        }) => handle_struct(ident, arg_i),
        Ok(DeriveInput {
            data: Data::Enum(_),
            ..
        }) => handle_enum(ident, arg_i),
        _ => panic!("Unsupported type"),
    }
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
            // let type_args = &type_path.path.segments.first().unwrap().arguments;
            // match type_args {
            //     PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
            //         GenericArgument::Type(t) => match t {
            //             Type::Path(f) => {
            //                 let id = &f.path.segments.first().unwrap().ident;
            //                 match id.to_string().as_str() {
            //                     "u8" | "u16" | "u32" | "u64" |
            //                     "i8" | "i16" | "i32" | "i64" |
            //                     "f32" | "f64" | "String" | "bool" => {
            //                         return match_types_func(f,arg_i.clone()),
            //                     }
            //                     "Option" => {
            //                         let type_vec_args = &f.path.segments.first().unwrap().arguments;
            //                         match type_vec_args {
            //                             PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
            //                                 GenericArgument::Type(t) => match t {
            //                                     Type::Path(f) => {
            //                                         let id = &f.path.segments.first().unwrap().ident;
            //                                         match id.to_string().as_str() {
            //                                             "u8" | "u16" | "u32" | "u64" |
            //                                             "i8" | "i16" | "i32" | "i64" |
            //                                             "f32" | "f64" | "String" | "bool" => {
            //                                                 return match_types_func(f,arg_i.clone()),
            //                                             }
            //                                             _ => {
            //                                                 let type_ident = id.to_string();                                            
                                    
            //                                                 match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", type_ident.to_string())) {
            //                                                     Ok(DeriveInput {
            //                                                         data: Data::Struct(_),
            //                                                         ..
            //                                                     }) => return handle_vec_opt_struct(field_name, type_ident.to_string().parse::<TokenStream2>().unwrap()),
            //                                                     Ok(DeriveInput {
            //                                                         data: Data::Enum(_),
            //                                                         ..
            //                                                     }) => return handle_vec_opt_enum(field_name, field_type),
            //                                                     _ => todo!(),
            //                                                 }
            //                                             }                                                                
            //                                         }                                                            
            //                                     }
            //                                     _ => todo!(),
            //                                 }
            //                                 _ => todo!(),
            //                             }
            //                             _ => todo!(),
            //                         }
            //                     }
            //                     _ => {
            //                         let type_ident = id.to_string();                                            
                                    
            //                         match syn::parse_str::<DeriveInput>(&format!("struct OrEnum{} {{}}", type_ident.to_string())) {
            //                             Ok(DeriveInput {
            //                                 data: Data::Struct(_),
            //                                 ..
            //                             }) => return handle_vec_struct(field_name, type_ident.to_string().parse::<TokenStream2>().unwrap()),
            //                             Ok(DeriveInput {
            //                                 data: Data::Enum(_),
            //                                 ..
            //                             }) => return handle_vec_enum(field_name, field_type),
            //                             _ => todo!(),
            //                         }
            //                     }
            //                 }
            //             }
            //             _ => todo!(),
            //         }
            //         _ => todo!(),
            //     }
            //     _ => todo!(),
            // }           
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
        // ident => {             
        //     let ident = TokenStream2::from_str(ident).unwrap();
        //     cmd_stream_extend = quote!(#ident);
        //     arg_stream_extend = quote!{#arg_i};
        //     stream_extend = handle_enum(ident,arg_i);
        // },                  
        ident => {
            let ident = TokenStream2::from_str(ident).unwrap();
            stream_extend = handle_ident(ident,arg_i);
        }
    };
    (stream_extend,cmd_stream_extend,arg_stream_extend)
}

// #[proc_macro_derive(Ground)]
// pub fn ground_derive(input: TokenStream) -> TokenStream {
//     // Construct a representation of Rust code as a syntax tree
//     // that we can manipulate
//     let output = input.clone();
//     let ast: syn::DeriveInput = syn::parse(input).unwrap();
//     let name = &ast.ident;
//     let strukt = &ast.data;
//     let fields = match strukt {
//         syn::Data::Struct(d) => &d.fields,
//         syn::Data::Enum(e) => {
//             let name2 = format!("Gql{}",name);
//             let gqlname = TokenStream2::from_str(&name2).unwrap();            
//             let mut out_extend = quote!();
//             let mut from_extend = quote!();
//             for variant in e.variants.iter() {
//                 let v = &variant.ident;
//                 out_extend.extend::<TokenStream2>(quote!{
//                     #v,
//                 });
//                 from_extend.extend::<TokenStream2>(quote!{
//                     #name::#v => #gqlname::#v,
//                 });
//             }
//             let out = quote!{
//                 #[derive(GraphQLEnum)]
//                 pub enum #gqlname {
//                     #out_extend
//                 }
//                 impl From<#name> for #gqlname {
//                     fn from(e: #name) -> #gqlname {
//                         match e {
//                             #from_extend
//                         }
//                     }
//                 }
//             };
//             println!("{}",out);
//             return out.into()

//         }
//         _ => return output,
//     };

//     // Build the trait implementation
//     impl_ground(&name,&fields)
// }

// fn impl_ground(name: &syn::Ident, syn_fields: &syn::Fields) -> TokenStream {
//     let name2 = format!("Gql{}",name);

//     let gqlname = TokenStream2::from_str(&name2).unwrap();

//     let mut strukt_stream = TokenStream2::default();
//     let mut from_stream = TokenStream2::default();
//     let mut tuple_stream = TokenStream2::default();

//     if let syn::Fields::Named(FieldsNamed{named, .. }) = syn_fields {
//         let fields = named.iter().map(|f| &f.ident);
//         let ftypes = named.iter().map(|f| &f.ty);

//         // for ft in ftypes.clone().into_iter() {
//         //     println!{"{:?}",ft};
//         // }

//         for (field, ftype) in fields.into_iter().zip(ftypes.into_iter()) {
//             strukt_stream.extend::<TokenStream2>(
//                 quote!{#field: }
//             );
//             from_stream.extend::<TokenStream2>(
//                 quote!{#field: }
//             );
//             let (strukt_extend,from_extend,tuple_extend) = match_types(field,ftype);
//             strukt_stream.extend::<TokenStream2>(
//                 quote!{#strukt_extend,}
//             );
//             from_stream.extend::<TokenStream2>(
//                 quote!{#from_extend,}
//             );
//             tuple_stream.extend::<TokenStream2>(
//                 tuple_extend
//             )
//         }
//     }

//     let gen = quote! {
//         #tuple_stream

//         #[derive(GraphQLObject,Deserialize,Serialize)]
//         pub struct #gqlname {
//             #strukt_stream
//         }
        
//         impl From<#name> for #gqlname {
//             fn from(n: #name) -> #gqlname {
//                 #gqlname {
//                     #from_stream
//                 }
//             }
//         }
//     };

//     println!("{}",gen);
//     gen.into()
// }

// fn match_types(
//     field: &Option<syn::Ident>,
//     ftype: &syn::Type,
// ) -> (TokenStream2, TokenStream2, TokenStream2) {
//     let mut strukt_stream_extend = TokenStream2::default();
//     let mut from_stream_extend = TokenStream2::default();
//     let mut tuple_stream_extend = TokenStream2::default();

//     match ftype {
//         Type::Path(type_path) => {
//             match type_path.clone().path.segments.first().unwrap().ident.to_string().as_str() {
//                 "u8" => {
//                     strukt_stream_extend = quote!{i32};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "u16" => {
//                     strukt_stream_extend = quote!{i32};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "u32" => {
//                     strukt_stream_extend = quote!{f64};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "u64" => {
//                     strukt_stream_extend = quote!{f64};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "i8" => {
//                     strukt_stream_extend = quote!{i32};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "i16" => {
//                     strukt_stream_extend = quote!{i32};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "i32" => {
//                     strukt_stream_extend = quote!{i32};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "i64" => {
//                     strukt_stream_extend = quote!{f64};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "usize" => {
//                     strukt_stream_extend = quote!{f64};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "isize" => {
//                     strukt_stream_extend = quote!{f64};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "f32" => {
//                     strukt_stream_extend = quote!{f64};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "f64" => {
//                     strukt_stream_extend = quote!{f64};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "bool" => {
//                     strukt_stream_extend = quote!{bool};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "String" => {
//                     strukt_stream_extend = quote!{String};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }
//                 "&str" => {
//                     strukt_stream_extend = quote!{String};
//                     from_stream_extend = quote!{n.#field.to_string()};
//                 }
//                 "Vec" => {                   
//                     let (gqltyp,_,_) = match &type_path.clone().path.segments.first().unwrap().arguments{
//                         PathArguments::AngleBracketed(a) => match a.args.first().unwrap() {
//                             GenericArgument::Type(f) => match_types(field,f),     
//                             _ => todo!(),                       
//                         }                        
//                         _ => todo!(),
//                     };
//                     from_stream_extend = quote!{
//                         {
//                             let mut v: Vec<#gqltyp> = Vec::new();
//                             for i in n.#field.iter()  {
//                                 v.push(<#gqltyp>::from(*i));
//                             }
//                             v
//                         }
//                     };
//                     strukt_stream_extend = quote!{Vec<#gqltyp>};
//                 }
//                 _ => {        
//                     let f = type_path.clone().into_token_stream();
//                     let name2 = format!("Gql{}",f);
//                     let gqlname = TokenStream2::from_str(&name2).unwrap();          
//                     strukt_stream_extend = quote!{#gqlname};
//                     from_stream_extend = quote!{n.#field.into()};
//                 }                
//             }      
//         }
//         // Tuple Type not implemented in GraphQL Object
//         // Convert tuple to struct 
//         Type::Tuple(type_tuple) => {
//             from_stream_extend = quote!{n.#field.into()};            
//             let mut gqlfields = TokenStream2::default();
//             let mut elements = TokenStream2::default();
//             let mut from_tuple = TokenStream2::default();
//             let mut i: usize = 0;
//             for elem in type_tuple.elems.iter() {
//                 let (gqlfield,_,_) = match_types(field,elem);
//                 gqlfields.extend::<TokenStream2>(TokenStream2::from_str(&format!("t_{}: {}",i,gqlfield)).unwrap());                                
//                 elements.extend::<TokenStream2>(quote!{#elem});
//                 if Some(elem) != type_tuple.elems.last() {
//                     gqlfields.extend::<TokenStream2>(quote!{,});
//                     elements.extend::<TokenStream2>(quote!{,});
//                 }
//                 // let tunder = TokenStream2::from_str(&format!("t_{}:",i)).unwrap();
//                 // let tdot = TokenStream2::from_str(&format!("t.{}",i)).unwrap();
//                 from_tuple.extend::<TokenStream2>(TokenStream2::from_str(&format!("t_{}: t.{}.into(),",i,i)).unwrap());
//                 i = i+1;
//             }
//             let gqlstruct = TokenStream2::from_str(&format!("Gql{}",field.clone().unwrap().to_string())).unwrap();            
//             strukt_stream_extend = quote!{#gqlstruct};
//             tuple_stream_extend = quote!{
//                 #[derive(GraphQLObject)]
//                 pub struct #gqlstruct {
//                     #gqlfields
//                 }
//                 impl From<(#elements)> for #gqlstruct {
//                     fn from(t: (#elements)) -> #gqlstruct {
//                         #gqlstruct {
//                             #from_tuple
//                         }
//                     }
//                 }
//             };
//         }
//         Type::Array(type_array) => {          
//             let typ = &type_array.elem;
//             let (gqltyp,_,_) = match_types(field,typ);
//             strukt_stream_extend = quote!{Vec<#gqltyp>};
//             if let syn::Expr::Lit(expr_lit) = &type_array.len {
//                 from_stream_extend = quote!{
//                     {
//                         let mut v: Vec<#gqltyp> = Vec::new();
//                         for i in 0..#expr_lit {
//                             v.push(n.#field[i].into());
//                         }
//                         v
//                     }
//                 };
//             }
//         }
//         _ => {}                    
//     };
//     (strukt_stream_extend,from_stream_extend,tuple_stream_extend)
// }