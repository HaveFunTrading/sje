use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Ident, LitInt, LitStr, PathArguments,
    PathSegment, Token, Type,
};

#[derive(Debug, Copy, Clone)]
enum SjeType {
    Object,
    Array,
    Tuple,
    Union,
}

impl FromStr for SjeType {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "object" => Ok(SjeType::Object),
            "array" => Ok(SjeType::Array),
            "tuple" => Ok(SjeType::Tuple),
            "union" => Ok(SjeType::Union),
            _ => Err(syn::Error::new(Span::call_site(), "expected 'object', 'array', 'tuple' or 'union'")),
        }
    }
}

#[derive(Copy, Clone)]
struct SjeAttribute {
    sje_type: SjeType,
}

impl Parse for SjeAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let sje_type = ident.to_string().parse()?;
        Ok(SjeAttribute { sje_type })
    }
}

#[derive(Debug, Clone)]
struct SjeFieldAttribute {
    #[allow(dead_code)]
    /// value length
    len: Option<usize>,
    /// field name override
    name: Option<String>,
    /// json type name override
    ty: Option<String>,
    /// additional conversion method
    also_as: Option<String>,
    /// offset at which value begins
    offset: usize,
}

impl Parse for SjeFieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut len = None;
        let mut name = None;
        let mut ty = None;
        let mut also_as = None;
        let mut offset = 0;

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "len" {
                    input.parse::<Token![=]>()?;
                    let len_lit: LitInt = input.parse()?;
                    len = Some(len_lit.base10_parse()?);
                } else if ident == "rename" {
                    input.parse::<Token![=]>()?;
                    let ref_lit: LitStr = input.parse()?;
                    name = Some(ref_lit.value());
                } else if ident == "ty" {
                    input.parse::<Token![=]>()?;
                    let ty_lit: LitStr = input.parse()?;
                    ty = Some(ty_lit.value());
                } else if ident == "also_as" {
                    input.parse::<Token![=]>()?;
                    let as_lit: LitStr = input.parse()?;
                    also_as = Some(as_lit.value());
                } else if ident == "offset" {
                    input.parse::<Token![=]>()?;
                    let offset_lit: LitInt = input.parse()?;
                    offset = offset_lit.base10_parse()?;
                } else {
                    return Err(syn::Error::new_spanned(ident, "expected ['len' | 'rename' | 'ty']"));
                }
            } else {
                return Err(lookahead.error());
            }

            // Optional comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(SjeFieldAttribute {
            len,
            name,
            ty,
            also_as,
            offset,
        })
    }
}

#[proc_macro_derive(Decoder, attributes(sje))]
pub fn decoder_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let sje_attr = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("sje"))
        .map(|attr| attr.parse_args::<SjeAttribute>())
        .transpose()
        .expect("Failed to parse 'sje' attribute");

    match ast.data {
        Data::Enum(data_enum) => handle_enum(&ast.ident, data_enum),
        Data::Struct(data_struct) => {
            handle_struct(&ast.ident, data_struct, sje_attr.expect("sje attribute must be present"))
        }
        _ => panic!("Decoder can only be derived for enums and structs"),
    }
}

fn handle_enum(name: &syn::Ident, data_enum: DataEnum) -> TokenStream {
    let variants = data_enum.variants.iter().map(|v| &v.ident);
    let gen = quote! {
        impl From<&[u8]> for #name {
            fn from(bytes: &[u8]) -> Self {
                match std::str::from_utf8(bytes).unwrap() {
                    #( stringify!(#variants) => #name::#variants, )*
                    _ => panic!("unrecognized side"),
                }
            }
        }
    };
    gen.into()
}

fn handle_struct(name: &syn::Ident, data_struct: DataStruct, sje_attr: SjeAttribute) -> TokenStream {
    match sje_attr.sje_type {
        SjeType::Object => handle_sje_object(name, data_struct, sje_attr),
        SjeType::Array => unimplemented!("array not supported"),
        SjeType::Tuple => unimplemented!("tuple not supported"),
        SjeType::Union => unimplemented!("union not supported"),
    }
}

fn handle_sje_object(name: &syn::Ident, data_struct: DataStruct, _sje_attr: SjeAttribute) -> TokenStream {
    let struct_name = Ident::new(&format!("{}Decoder", name), name.span());

    let fields = match data_struct.fields {
        Fields::Named(fields) => fields.named,
        _ => return quote! { compile_error!("Decoder can only be derived for structs with named fields."); }.into(),
    };

    let field_initializations = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let mut key_len = field_name.to_string().len();
        let mut val_len = None;
        let mut ty_override = None;
        if let Some(sje_attr) = field.attrs.iter().find(|attr| attr.path().is_ident("sje")) {
            let sje_field = sje_attr.parse_args::<SjeFieldAttribute>().expect("unable to parse");
            if let Some(name) = sje_field.name {
                key_len = name.len();
            }
            if let Some(len) = sje_field.len {
                val_len = Some(len);
            }
            if let Some(ty) = sje_field.ty {
                ty_override = Some(ty);
            }
            key_len += sje_field.offset;
        }

        match resolve_type(&field.ty, ty_override) {
            Ok(type_str) => {
                key_len += 4;
                match val_len {
                    Some(known_len) => {
                        let next = Ident::new(&format!("next_{}_with_known_len", type_str), field_name.span());
                        let field_name_string = field_name.to_string();
                        quote! {
                            scanner.skip(#key_len);
                            let (offset, len) = scanner.#next(#known_len).ok_or_else(|| sje::error::Error::MissingField(#field_name_string))?;
                            let #field_name = sje::LazyField::from_bytes(unsafe { bytes.get_unchecked(offset..offset + len) });
                        }
                    }
                    None => {
                        let next = Ident::new(&format!("next_{}", type_str), field_name.span());
                        let field_name_string = field_name.to_string();
                        if type_str == "array" {
                            quote! {
                                scanner.skip(#key_len);
                                let (offset, len, count) = scanner.#next().ok_or_else(|| sje::error::Error::MissingField(#field_name_string))?;
                                let #field_name = (unsafe { bytes.get_unchecked(offset..offset + len) }, count);
                            }
                        } else {
                            quote! {
                                scanner.skip(#key_len);
                                let (offset, len) = scanner.#next().ok_or_else(|| sje::error::Error::MissingField(#field_name_string))?;
                                let #field_name = sje::LazyField::from_bytes(unsafe { bytes.get_unchecked(offset..offset + len) });
                            }
                        }
                    }
                }
            }
            Err(e) => e.to_compile_error(),
        }
    });

    let field_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name,
        }
    });

    let from_field_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: decoder.#field_name().into(),
        }
    });

    let from_impl = quote! {
        impl From<#struct_name<'_>> for #name {
            fn from(decoder: #struct_name<'_>) -> Self {
                Self {
                    #(#from_field_assignments)*
                }
            }
        }
    };

    let decode_impl = quote! {
        impl <'a> #struct_name<'a> {
            #[inline]
            pub fn decode(bytes: &'a [u8]) -> Result<Self, sje::error::Error> {
                let mut scanner = sje::scanner::JsonScanner::wrap(bytes);
                #(#field_initializations)*
                Ok(Self {
                    #(#field_assignments)*
                })
            }
        }
    };

    let accessor_methods = fields.iter().map(|field| {
        let field_name = &field.ident;
        let as_slice = Ident::new(&format!("{}_as_slice", field_name.as_ref().unwrap()), field_name.span());
        let as_str = Ident::new(&format!("{}_as_str", field_name.as_ref().unwrap()), field_name.span());

        let mut gen = quote! {};

        let field_type = &field.ty;
        if let syn::Type::Path(ref path) = field_type {
            if path.path.segments.last().map(|seg| seg.ident == "Vec").unwrap_or(false) {
                let array_count = Ident::new(&format!("{}_count", field_name.as_ref().unwrap()), field_name.span());
                gen.extend(quote! {
                    #[inline]
                    pub const fn #as_slice(&self) -> &[u8] {
                        self.#field_name.0
                    }
                    #[inline]
                    pub const fn #as_str(&self) -> &str {
                        unsafe { std::str::from_utf8_unchecked(self.#as_slice()) }
                    }
                    #[inline]
                    pub const fn #array_count(&self) -> usize {
                        self.#field_name.1
                    }
                })
            } else {
                let as_lazy_field = Ident::new(&format!("{}_as_lazy_field", field_name.as_ref().unwrap()), field_name.span());
                gen.extend(quote! {
                    #[inline]
                    pub const fn #as_slice(&self) -> &[u8] {
                        self.#field_name.as_slice()
                    }
                    #[inline]
                    pub const fn #as_str(&self) -> &str {
                        self.#field_name.as_str()
                    }
                    #[inline]
                    pub const fn #as_lazy_field(&self) -> &sje::LazyField<'a, #field_type> {
                        &self.#field_name
                    }
                })
            }
        }

        if let Some(sje_attr) = field.attrs.iter().find(|attr| attr.path().is_ident("sje")) {
            let sje_field = sje_attr.parse_args::<SjeFieldAttribute>().expect("unable to parse");
            if let Some(also_as) = sje_field.also_as {
                let type_name = also_as.split("::").last().map(|s| s.to_string()).unwrap();
                let type_name_ident: syn::Path = syn::parse_str(&also_as).unwrap();
                let also_as = Ident::new(
                    &format!("{}_as_{}", field_name.as_ref().unwrap(), type_name.to_snake_case()),
                    field_name.span(),
                );
                gen.extend(quote! {

                    #[inline]
                    pub fn #also_as(&self) -> #type_name_ident {
                        self.#as_str().parse().unwrap()
                    }
                });
            }
        }

        gen
    });

    let new_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        if let syn::Type::Path(ref path) = field_type {
            if path.path.segments.last().map(|seg| seg.ident == "Vec").unwrap_or(false) {
                quote! {
                    #field_name: (&'a [u8], usize),
                }
            } else {
                quote! {
                    #field_name: sje::LazyField<'a, #field_type>,
                }
            }
        } else {
            quote! {}
        }
    });

    let iterators = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        if let syn::Type::Path(ref path) = field_type {
            if path.path.segments.last().map(|seg| seg.ident == "Vec").unwrap_or(false) {
                if let Some(segment) = path.path.segments.last() {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(arg_type)) = args.args.first() {
                            let array_struct_name =
                                format_ident!("{}", field_name.as_ref().unwrap().to_string().to_upper_camel_case());
                            let array_fn_name = format_ident!("{}", field_name.as_ref().unwrap().to_string());
                            let iterator_name =
                                format_ident!("{}Iter", field_name.as_ref().unwrap().to_string().to_upper_camel_case());
                            let next_impl = iterator_next_impl(arg_type);
                            return quote! {
                                #[derive(Debug)]
                                pub struct #array_struct_name<'a> {
                                    bytes: &'a [u8],
                                    remaining: usize,
                                }

                                impl #struct_name<'_> {
                                    #[inline]
                                    pub const fn #array_fn_name(&self) -> #array_struct_name {
                                        #array_struct_name { bytes: self.#array_fn_name.0, remaining: self.#array_fn_name.1 }
                                    }
                                }

                                impl From<#array_struct_name<'_>> for Vec<#arg_type> {
                                    fn from(value: #array_struct_name) -> Self {
                                        value.into_iter().collect()
                                    }
                                }

                                impl<'a> IntoIterator for #array_struct_name<'a> {
                                    type Item = #arg_type;
                                    type IntoIter = #iterator_name<'a>;

                                    fn into_iter(self) -> Self::IntoIter {
                                        #iterator_name {
                                            scanner: sje::scanner::JsonScanner::wrap(self.bytes),
                                            remaining: self.remaining
                                        }
                                    }
                                }

                                pub struct #iterator_name<'a> {
                                    scanner: sje::scanner::JsonScanner<'a>,
                                    remaining: usize,
                                }

                                impl Iterator for #iterator_name<'_> {
                                    type Item = #arg_type;

                                    #[inline]
                                    fn next(&mut self) -> Option<Self::Item> {
                                        #next_impl
                                    }

                                    #[inline]
                                    fn size_hint(&self) -> (usize, Option<usize>) {
                                        (self.remaining, Some(self.remaining))
                                    }
                                }

                                impl ExactSizeIterator for #iterator_name<'_> {

                                    #[inline]
                                    fn len(&self) -> usize {
                                        self.remaining
                                    }
                                }
                            };
                        }
                    }
                }
            } else {
                return quote! {
                    impl #struct_name<'_> {
                        #[inline]
                        pub fn #field_name(&self) -> #field_type {
                            self.#field_name.get().unwrap()
                        }
                    }
                };
            }
        }
        quote! {}
    });

    let generated = quote! {
        #[derive(Debug)]
        pub struct #struct_name<'a> {
            #(#new_fields)*
        }

        #from_impl

        #decode_impl

        impl <'a> #struct_name<'a> {
            #(#accessor_methods)*
        }

        #(#iterators)*
    };

    generated.into()
}

fn resolve_type(ty: &Type, ty_override: Option<String>) -> syn::Result<&'static str> {
    if let Some(ty_override) = ty_override {
        return Ok(ty_override.leak());
    }

    match ty {
        Type::Path(type_path) => {
            let ident = type_path.path.segments.last().unwrap().ident.to_string();

            match ident.as_str() {
                // Primitive number types
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" | "f32" | "f64" => {
                    Ok("number")
                }
                "String" => Ok("string"),
                "bool" => Ok("boolean"),
                "Vec" => Ok("array"),
                _ => Err(Error::new(Span::call_site(), "Only primitives, String, and Vec are allowed")),
            }
        }
        _ => Err(Error::new(Span::call_site(), "Unsupported type: Only primitives, String, and Vec<u8> are allowed")),
    }
}

fn iterator_next_impl(ty: &Type) -> proc_macro2::TokenStream {
    if let Type::Tuple(tuple) = ty {
        // Generate code for processing each element
        let mut code = quote! {};
        let mut tuple_values = Vec::new();

        code.extend(quote! {
            if self.scanner.position() + 1 == self.scanner.bytes().len() {
                return None;
            }
            self.scanner.skip(1);
            let (offset, len) = self.scanner.next_tuple()?;
            let mut tuple_scanner = unsafe { sje::scanner::JsonScanner::wrap(self.scanner.bytes().get_unchecked(offset..offset + len)) };
        });

        // Iterate over the tuple elements and generate code for each element
        for (i, _) in tuple.elems.iter().enumerate() {
            // Dynamically generate a variable name based on the index
            let var_name = format_ident!("val_{i}");

            // Generate the code for processing this element
            code.extend(quote! {
                tuple_scanner.skip(1);
                let (offset, len) = tuple_scanner.next_string()?;
                let str = unsafe { std::str::from_utf8_unchecked(tuple_scanner.bytes().get_unchecked(offset..offset + len)) };
                let #var_name = str.parse().unwrap();
            });

            // Add the variable to the tuple values vector for dynamic construction
            tuple_values.push(quote! { #var_name });
        }

        // Combine the generated code and the `Some(...)` expression
        code.extend(quote! {
            self.remaining -= 1;
            Some((#(#tuple_values),*))
        });

        code
    } else {
        // If it's not a tuple, return an empty TokenStream
        quote! {}
    }
}

#[allow(dead_code)]
fn is_integer_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(PathSegment { ident, .. }) = type_path.path.segments.last() {
            return matches!(
                ident.to_string().as_str(),
                "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
            );
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, parse_str, Attribute};

    use super::*;

    #[test]
    fn should_parse_sje_field_attribute() {
        let attr: Attribute = parse_quote! {
            #[sje(rename = "foo", len = 12)]
        };

        assert!(attr.path().is_ident("sje"));

        let field: SjeFieldAttribute = attr.parse_args().unwrap();
        assert_eq!(Some("foo".to_string()), field.name);
        assert_eq!(Some(12), field.len);
    }

    fn check_type(ty: &str, ty_override: Option<&str>, expected: Result<&'static str, &str>) {
        let parsed_ty: Type = parse_str(ty).expect("Failed to parse type");
        let result = resolve_type(&parsed_ty, ty_override.map(String::from));

        match (result.clone(), expected) {
            (Ok(actual), Ok(expected_str)) => assert_eq!(actual, expected_str),
            (Err(err), Err(expected_err)) => assert!(err.to_string().contains(expected_err)),
            _ => panic!("Unexpected result: {:?}", result),
        }
    }

    #[test]
    fn should_resolve_types() {
        check_type("u64", None, Ok("number"));
        check_type("f64", None, Ok("number"));
        check_type("i32", None, Ok("number"));
        check_type("String", None, Ok("string"));
        check_type("bool", None, Ok("boolean"));
        check_type("String", Some("object"), Ok("object"));
        check_type("Vec<u8>", None, Ok("array"));
        check_type("Vec<Price>", None, Ok("array"));
        check_type("Vec<(Price, Quantity)>", None, Ok("array"));
        check_type("MyStruct", None, Err("Only primitives, String, and Vec are allowed"));
        check_type("Option<u64>", None, Err("Only primitives, String, and Vec are allowed"));
        check_type("Result<String, u8>", None, Err("Only primitives, String, and Vec are allowed"));
    }
}
