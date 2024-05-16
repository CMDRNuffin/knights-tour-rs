use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parenthesized, parse::Parse, parse_macro_input, token::Paren, Error, Ident, LitStr, Token};

struct SvgInput {
    writer: Ident,
    doc: XmlDoc,
}

impl Parse for SvgInput{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let writer: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let doc: XmlDoc = input.parse()?;
        Ok(SvgInput { writer, doc })
    }
}

struct XmlDoc {
    name: String,
    is_ref: bool,
    is_self_closed: bool,
    attributes: Vec<XmlAttribute>,
    children: Vec<XmlDocChild>,
}

enum XmlDocChild {
    Repeat(String),
    Raw{
        value: String,
        is_ref: bool
    },
    Doc(XmlDoc),
}

#[derive(PartialEq, Eq)]
enum XmlDocStyle {
    Ref,
    Name,
}

impl Parse for XmlDoc {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let is_ref = input.peek(Token![#]) && { input.parse::<Token![#]>()?; true };

        let (name, style) = if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            (ident.to_string(), if is_ref { XmlDocStyle::Ref } else { XmlDocStyle::Name })
        } else if !is_ref && input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            (lit.value(), XmlDocStyle::Name)
        } else {
            return Err(input.error("Expected a #variable, an identifier or a string literal"));
        };

        let attributes = input.call(parse_attributes)?;

        let is_self_closed = input.peek(Token![/]) && { input.parse::<Token![/]>()?; true };
        input.parse::<Token![>]>()?;

        let children =  if !is_self_closed {
            let children = input.call(parse_children)?;
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            if matches!(style, XmlDocStyle::Ref) {
                input.parse::<Token![#]>().map_err(|e|Error::new(e.span(), format!("Invalid closing tag, expected '#{}'", name)))?;
            }

            let (closing_tag_name, span) = if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                (ident.to_string(), ident.span())
            } else if !is_ref && input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                (lit.value(), lit.span())
            } else {
                return Err(input.error(format!("Invalid closing tag, expected '{}'", name)));
            };

            if closing_tag_name != name {
                return Err(Error::new(span, format!("Invalid closing tag, expected '{}'", name)));
            }

            input.parse::<Token![>]>()?;

            children
        } else {
            Vec::new()
        };

        Ok(XmlDoc {
            name,
            is_ref,
            is_self_closed,
            attributes,
            children
        })
    }
}

fn parse_attributes(input: syn::parse::ParseStream) -> syn::Result<Vec<XmlAttribute>> {
    let mut res = Vec::new();
    while !input.peek(Token![/]) && !input.peek(Token![>]) {
        res.push(input.call(parse_attribute)?);
    }

    Ok(res)
}

fn parse_attribute(input: syn::parse::ParseStream) -> syn::Result<XmlAttribute> {
    let (value, value_is_ref);
    let (name, mut name_is_ref, _) = input.call(|s|parse_str(s, None))?;

    if !input.peek(Token![=]) {
        value = name.clone();
        value_is_ref = name_is_ref;
        name_is_ref = false;
    } else {
        input.parse::<Token![=]>()?;
        (value, value_is_ref, _) = input.call(|s|parse_str(s, None))?;
    }

    Ok(XmlAttribute {
        name,
        name_is_ref,
        value,
        value_is_ref,
    })
}

fn parse_str(input: syn::parse::ParseStream, expected_style: Option<XmlDocStyle>) -> syn::Result<(String, bool, Span)> {
    if input.peek(Token![#]) && (expected_style.is_none() || expected_style == Some(XmlDocStyle::Ref)) {
        let pound = input.parse::<Token![#]>()?;
        let name: Ident = input.parse()?;
        Ok((name.to_string(), true, pound.span))
    } else if input.peek(Ident) && (expected_style.is_none() || expected_style == Some(XmlDocStyle::Name)) {
        let ident: Ident = input.parse()?;
        Ok((ident.to_string(), false, ident.span()))
    } else if input.peek(LitStr) && (expected_style.is_none() || expected_style == Some(XmlDocStyle::Name)) {
        let lit: LitStr = input.parse()?;
        Ok((lit.value(), false, lit.span()))
    } else {
        if let Some(style) = expected_style {
            return match style {
                XmlDocStyle::Name => Err(input.error("Expected an identifier or a string literal")),
                XmlDocStyle::Ref => Err(input.error("Expected a #variable")),
            };
        } else {
            return Err(input.error("Expected a #variable, an identifier or a string literal"));
        }
    }
}

fn parse_children(input: syn::parse::ParseStream) -> syn::Result<Vec<XmlDocChild>> {
    let mut res = Vec::new();
    loop {
        if (input.peek(Token![<]) && input.peek2(Token![/])) || input.is_empty() {
            return Ok(res);
        } else if input.peek(Token![<]) {
            res.push(XmlDocChild::Doc(input.parse::<XmlDoc>()?));
        } else if input.peek(Token![#]) {
            if input.peek2(Paren) {
                input.parse::<Token![#]>()?;
                let content;
                parenthesized!(content in input);
                let (iter, _, _) = parse_str(&content, Some(XmlDocStyle::Ref))?;
                res.push(XmlDocChild::Repeat(iter));
                input.parse::<Token![*]>()?;
            } else {
                let (item, is_ref, _) = parse_str(input, None)?;
                res.push(XmlDocChild::Raw{ value: item, is_ref });
            }
        } else {
            return Err(input.error("Expected a child element"));
        }
    }
}

struct XmlAttribute {
    name: String,
    name_is_ref: bool,
    value: String,
    value_is_ref: bool,
}

#[proc_macro]
pub fn svg(tt: TokenStream) -> TokenStream {
    let res = parse_macro_input!(tt as SvgInput);
    let writer_ident = res.writer;
    let doc: XmlDoc = res.doc;
    let doc_tree = quote_doc(&doc, &writer_ident);

    quote! {
        let indent = 0;
        #doc_tree
    }.into()
}


fn quote_doc(doc: &XmlDoc, writer: &Ident) -> proc_macro2::TokenStream {
    let title: proc_macro2::TokenStream = if doc.is_ref {
        let ident = Ident::new(&doc.name, Span::call_site());
        quote! { #ident }
    } else {
        let lit = LitStr::new(&doc.name, Span::call_site());
        quote! { #lit }
    };

    let attributes;
    let attributes_fmt = if doc.attributes.len() == 0 {
        attributes = quote!{ };
        String::new()
    } else {
        let attr_iter: Vec<_> = doc.attributes.iter().map(map_attribute).collect();
        let (attr_fmts, attr_values): (Vec<_>, Vec<_>) = attr_iter.into_iter().unzip();
        let attr_values = attr_values.into_iter().filter(|v|v.is_some()).map(|v|v.unwrap());
        attributes = quote! { #(, #attr_values)* };
        attr_fmts.join("")
    };

    let open_tag_format = format!("{}{}{}>", "{: >indent$}<{}", attributes_fmt, if doc.is_self_closed { "/" } else { "" });

    let mut res = Vec::new();
    res.push(quote! { writeln!(#writer, #open_tag_format, ' ', #title #attributes)?; });
    if !doc.is_self_closed {
        let children: Vec<_> = doc.children.iter().map(|child|quote_child(child, writer)).collect();
        res.push(quote!{
            {
                let indent =  indent + 4;
                #(#children)*
            }
            writeln!(#writer, "{: >indent$}</{}>", ' ', #title)?;
        });
    }

    quote!{
        #(#res)*
    }
}

fn quote_child(child: &XmlDocChild, writer: &Ident) -> proc_macro2::TokenStream {
    match child {
        XmlDocChild::Repeat(iter) => {
            let ident = Ident::new(iter, Span::call_site());
            quote!{
                for #ident in #ident {
                    writeln!(#writer, "{: >indent$}{}", ' ', #ident)?;
                }
            }
        },
        XmlDocChild::Raw{ value, is_ref } => {
            if *is_ref {
                let ident = Ident::new(value, Span::call_site());
                quote!{ writeln!(#writer, "{: >indent$}{}", ' ', #ident)?; }
            } else {
                let lit = LitStr::new(value, Span::call_site());
                quote!{ writeln!(#writer, "{: >indent$}{}", ' ', #lit)?; }
            }
        },
        XmlDocChild::Doc(doc) => quote_doc(doc, writer),
    }
}

fn map_attribute(attr: &XmlAttribute) -> (String, Option<proc_macro2::TokenStream>) {
    let (name_format, name_args) = if attr.name_is_ref {
        let ident = Ident::new(&attr.name, Span::call_site());
        (String::from(" {}="), Some(quote! { #ident }))
    } else {
        (format!(" {}=", attr.name), None)
    };

    let (value_format, value_args) = if attr.value_is_ref {
        let ident = Ident::new(&attr.value, Span::call_site());
        (String::from("\"{}\""), Some(quote!{ #ident }))
    } else {
        (format!("\"{}\"", attr.value), None)
    };

    let tt = match (name_args, value_args) {
        (Some(name), Some(val)) => Some(quote!{ #name, #val }),
        (Some(name), None) => Some(quote!{ #name }),
        (None, Some(val)) => Some(quote!{ #val }),
        (None, None) => None,
    };

    (format!("{}{}", name_format, value_format), tt)
}