use proc_macro2::{Ident, Punct, Spacing, TokenStream};
use quote::{quote, TokenStreamExt};
use quote::ToTokens;
use syn::{Expr, ExprCall, ExprPath};
use syn::parse::{Parse, ParseStream};
use syn::Token;

use super::keyword;

enum AssertPath {
    Whole,
    Field(ExprPath),
    Call(ExprCall),
}

impl ToTokens for AssertPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            AssertPath::Whole => {}
            AssertPath::Field(f) => f.to_tokens(tokens),
            AssertPath::Call(c) => c.to_tokens(tokens)
        }
    }
}

pub(crate) struct AssertEQ {
    path: AssertPath,
    expected: Expr,
}

pub(crate) trait Assert {
    fn construct_assertion(&self, constructed: Ident) -> TokenStream;
}

impl Assert for AssertEQ {
    fn construct_assertion(&self, constructed: Ident) -> TokenStream {
        let path = &self.path;
        let expected = &self.expected;
        let mut constructed = constructed.to_token_stream();

        match path {
            AssertPath::Whole => {}
            _ => constructed.append(Punct::new('.', Spacing::Alone)),
        }

        quote! {
            assert_eq!(#constructed#path, #expected, "testing eq [{}, {}]", stringify!(#path), #expected);
        }
    }
}

fn is_path_self(path: &ExprPath) -> bool {
    return matches!(path.path.segments.first()
                        .map(|e| e.ident.to_string() == "self" && e.arguments.is_empty()),
                    Some(true));
}

impl Parse for AssertEQ {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: Expr = input.parse()?;

        let path =
            match path {
                Expr::Path(path) => if is_path_self(&path) { AssertPath::Whole } else { AssertPath::Field(path) },
                Expr::Call(call) => AssertPath::Call(call),
                _ => Err(input.error("expected field accessing"))?
            };
        input.parse::<Token![,]>()?;
        let expected: Expr = input.parse()?;
        Ok(AssertEQ { path, expected })
    }
}

pub(crate) struct AssertNE {
    path: Expr,
    expected: Expr,
}

impl Assert for AssertNE {
    fn construct_assertion(&self, constructed: Ident) -> TokenStream {
        let path = &self.path;
        let expected = &self.expected;

        quote! {
            assert_ne!(#constructed.#path, #expected);
        }
    }
}

impl Parse for AssertNE {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let expected: Expr = input.parse()?;
        Ok(AssertNE { path, expected })
    }
}

pub(crate) enum Assertion {
    EQ(AssertEQ),
    NE(AssertNE),
}

impl Parse for Assertion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::eq) {
            keyword::eq::parse(input)?;
            let content;
            syn::bracketed!(content in input);

            Ok(Assertion::EQ(AssertEQ::parse(&content)?))
        } else if lookahead.peek(keyword::ne) {
            keyword::ne::parse(input)?;
            let content;
            syn::bracketed!(content in input);

            Ok(Assertion::NE(AssertNE::parse(&content)?))
        } else {
            Err(lookahead.error())
        }
    }
}
