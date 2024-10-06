use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use syn::punctuated::Punctuated;
use syn::*;
use template_quote::quote;

fn random() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
}

fn internal(input: ItemEnum) -> TokenStream {
    let krate: Path = input
        .attrs
        .iter()
        .filter_map(|a| match &a.meta {
            Meta::List(MetaList { path, tokens, .. }) => {
                if let (true, krate) = (path.is_ident("discriminant"), parse_quote!(#tokens)) {
                    Some(krate)
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
        .unwrap_or(parse_quote!(::discriminant));
    let discriminant_attrs = input
        .attrs
        .iter()
        .filter_map(|a| match &a.meta {
            Meta::NameValue(MetaNameValue { path, value, .. })
                if path.is_ident("discriminant_attr") =>
            {
                let s: LitStr = parse2(quote! {#value}).unwrap();
                Some(s.value())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let discriminant_attrs = core::convert::identity::<ItemStruct>(
        parse_str(&format!("{} struct S {{}}", discriminant_attrs.join(""))).unwrap(),
    )
    .attrs;
    let specified_repr = discriminant_attrs
        .iter()
        .chain(&input.attrs)
        .filter_map(|a| match &a.meta {
            Meta::List(MetaList { path, tokens, .. }) if path.is_ident("repr") => {
                if let Ok(reprs) = parse::Parser::parse2(
                    Punctuated::<Meta, Token![,]>::parse_terminated,
                    tokens.clone(),
                ) {
                    reprs
                        .iter()
                        .filter_map(|r| Some(r.path().get_ident()?.to_string()))
                        .filter_map(|r| match r.as_str() {
                            "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32"
                            | "i64" | "isize" => Some(Ident::new(&r, Span::call_site())),
                            _ => None,
                        })
                        .next()
                } else {
                    None
                }
            }
            _ => None,
        })
        .next();
    let repr = specified_repr
        .clone()
        .unwrap_or(Ident::new("isize", Span::call_site()));
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let discriminant_enum_ident = Ident::new(
        &format!("__Discriminant_{}_{}", &input.ident, random() % 1000),
        Span::call_site(),
    );
    let disc_indices = input
        .variants
        .iter()
        .scan(parse_quote!(0), |acc, variant| {
            if let Some((_, expr)) = &variant.discriminant {
                *acc = expr.clone();
            }
            let ret = acc.clone();
            *acc = parse_quote!(#ret + 1);
            Some(ret)
        })
        .collect::<Vec<Expr>>();
    quote! {
        #[repr(#repr)]
        #(#discriminant_attrs)*
        #[derive(
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::core::fmt::Debug,
            ::core::hash::Hash,
            ::core::cmp::PartialEq,
            ::core::cmp::Eq,
        )]
        #{&input.vis} enum #discriminant_enum_ident {
            #(for variant in &input.variants) {
                #{
                    variant.attrs.iter().filter_map(|a| match &a.meta {
                        Meta::NameValue(MetaNameValue{path, value, ..}) if path.is_ident("discriminant_attr") => {
                            let s: LitStr = parse2(quote! {#value}).unwrap();
                            let discriminant_attrs = core::convert::identity::<ItemStruct>(
                                parse_str(&format!("{} struct S {{}}", s.value())).unwrap()
                            ).attrs;
                            Some(quote!{#(#discriminant_attrs)*})
                        },
                        _ => None,
                    }).next()
                }
                #{&variant.ident}
                #(if let Some((eq_token, expr)) = &variant.discriminant) {
                    #eq_token #expr
                },
            }
        }

        impl ::core::fmt::Display for #discriminant_enum_ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                <Self as ::core::fmt::Debug>::fmt(self, f)
            }
        }

        impl ::core::cmp::PartialOrd for #discriminant_enum_ident {
            fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::core::cmp::Ordering> {
                (*self as #repr).partial_cmp(&(*other as #repr))
            }
        }

        impl ::core::cmp::Ord for #discriminant_enum_ident {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                (*self as #repr).cmp(&(*other as #repr))
            }
        }

        #[automatically_derived]
        unsafe impl #impl_generics #krate::Enum for #{&input.ident}
        #ty_generics #where_clause
        {
            type Discriminant = #discriminant_enum_ident;

            fn discriminant(&self) -> Self::Discriminant {
                match self {
                    #(for Variant{ident, fields, ..} in &input.variants) {
                        Self::#ident
                        #(if let Fields::Unnamed(_) = fields) { (..) }
                        #(if let Fields::Named(_) = fields) { {..} }
                        => #discriminant_enum_ident::#ident,
                    }
                }
            }
        }

        impl ::core::convert::TryFrom<#repr> for #discriminant_enum_ident {
            type Error = ();
            fn try_from(value: #repr) -> ::core::result::Result<Self, Self::Error> {
                #(for (variant, disc) in input.variants.iter().zip(&disc_indices)) {
                    if value == #disc { ::core::result::Result::Ok(Self::#{&variant.ident}) } else
                }
                { ::core::result::Result::Err(()) }
            }
        }

        impl ::core::convert::Into<#repr> for #discriminant_enum_ident {
            fn into(self) -> #repr {
                self as #repr
            }
        }

        unsafe impl #krate::Discriminant for #discriminant_enum_ident {
            type Repr = #repr;
            fn all() -> impl ::core::iter::Iterator<Item = Self> {
                struct Iter(::core::option::Option<#discriminant_enum_ident>);
                impl ::core::iter::Iterator for Iter {
                    type Item = #discriminant_enum_ident;
                    fn next(&mut self) -> Option<Self::Item> {
                        match self.0 {
                            #(for (curr, next) in input.variants.iter().zip(
                                    input.variants.iter().skip(1).map(Some).chain(core::iter::once(None))
                            )) {
                                ::core::option::Option::Some(#discriminant_enum_ident::#{&curr.ident}) => {
                                    let ret = self.0;
                                    self.0 = #(if let Some(next) = next) {
                                        Some(#discriminant_enum_ident::#{&next.ident})
                                    } #(else) { None };
                                    ret
                                }
                            }
                            ::core::option::Option::None => ::core::option::Option::None,
                        }
                    }
                    fn size_hint(&self) -> (
                        ::core::primitive::usize,
                        ::core::option::Option<::core::primitive::usize>
                    ) {
                        let n = Self(self.0).count();
                        (n, ::core::option::Option::Some(n))
                    }
                    fn count(self) -> usize {
                        match self.0 {
                            #(for (n, variant) in input.variants.iter().enumerate()) {
                                ::core::option::Option::Some(#discriminant_enum_ident::#{&variant.ident}) => #{disc_indices.len() - n},
                            }
                            ::core::option::Option::None => 0,
                        }
                    }
                    fn last(self) -> Option<Self::Item> {
                        #(if let Some(last) = &input.variants.iter().last()) {
                            self.0.map(|_| #discriminant_enum_ident::#{&last.ident})
                        } #(else) {
                            ::core::option::Option::None
                        }
                    }
                }
                #(if let Some(item) = input.variants.iter().next()) {
                    Iter(::core::option::Option::Some(#discriminant_enum_ident::#{&item.ident}))
                } #(else) {
                    Iter(::core::option::Option::None)
                }
            }
        }
    }
}

#[proc_macro_derive(Enum, attributes(discriminant, discriminant_attr))]
#[proc_macro_error]
pub fn derive_enum(input: TokenStream1) -> TokenStream1 {
    internal(parse(input).unwrap_or_else(|_| {
        abort!(
            Span::call_site(),
            "#[derive(Enum)] is only applicative on enums."
        )
    }))
    .into()
}
