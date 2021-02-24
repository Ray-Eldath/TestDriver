extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};
use syn::{Expr, parse_macro_input, Token, Type};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use crate::assertion::{Assert, Assertion};

mod assertion;

mod keyword {
    syn::custom_keyword!(with);
    syn::custom_keyword!(assert);
    syn::custom_keyword!(eq);
    syn::custom_keyword!(ne);
}

struct Driver {
    subject: Type,
    input: Expr,
    assertions: Punctuated<Assertion, Token![,]>,
}

impl Parse for Driver {
    fn parse(parse: ParseStream) -> syn::Result<Self> {
        let assertions;

        let subject: Type = parse.parse()?;
        parse.parse::<keyword::with>()?;
        let input = Expr::parse(parse)?;
        parse.parse::<Token![,]>()?;
        parse.parse::<keyword::assert>()?;

        syn::braced!(assertions in parse);

        Ok(Driver {
            subject,
            input,
            assertions: assertions.parse_terminated(Assertion::parse)?,
        })
    }
}

macro_rules! append_assertions {
    ($($name:ident),+ of $assertion:ident to $predicate:ident) => {
        match $assertion {
            $(Assertion::$name(a) => $predicate.append_all(a.construct_assertion(Ident::new("constructed", Span::call_site()))),)+
        }
    };
}

/// This macro will first construct a predicate with specified assertions, then
/// pass this predicate, and the input into Vehicle, in which you could designate
/// how to transform the input into output, thus run the predicate on the output.
/// As the example:
///
/// ```plain
/// drive!(Request::PUBLISH with "30 07 00 05 2f 61 62 63 64",
///         assert {
///             eq [field1, expected1],
///             ne [field2, expected2]
///         })
/// ```
///
/// Will be executed as:
///
/// ```plain
/// $subject(Type): Request::PUBLISH
/// $input(Expr): "30 07 00 05 2f 61 62 63 64"
///
/// $converter: impl From<[type of $input, &str]> for <[$subject, Request::PUBLISH]>
/// $predicate: fn([$subject, Request::PUBLISH])   -- constructed from assertions
/// $vehicle: impl Vehicle for <[$subject, Request::PUBLISH]>
/// ---
/// $vehicle.drive($input, $predicate)
/// ```
#[proc_macro]
pub fn drive(input: TokenStream) -> TokenStream {
    let Driver {
        subject,
        input,
        assertions
    } = parse_macro_input!(input as Driver);

    let mut predicate = proc_macro2::TokenStream::new();
    for assertion in assertions {
        append_assertions!(EQ, NE of assertion to predicate);
    }

    let mut expanded = quote! {
        let constructed: #subject = #input.into();
    };
    expanded.append_all(predicate);
    expanded.append_all(
        quote! {
            println!("test {:>10} with {:>10}  --> All assertions passed.", stringify!(#subject), stringify!(#input));
        }
    );

    expanded.into()
}