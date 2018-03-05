// lib.rs

// *************************************************************************
// * Copyright (C) 2018 Daniel Mueller (deso@posteo.net)                   *
// *                                                                       *
// * This program is free software: you can redistribute it and/or modify  *
// * it under the terms of the GNU General Public License as published by  *
// * the Free Software Foundation, either version 3 of the License, or     *
// * (at your option) any later version.                                   *
// *                                                                       *
// * This program is distributed in the hope that it will be useful,       *
// * but WITHOUT ANY WARRANTY; without even the implied warranty of        *
// * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the         *
// * GNU General Public License for more details.                          *
// *                                                                       *
// * You should have received a copy of the GNU General Public License     *
// * along with this program.  If not, see <http://www.gnu.org/licenses/>. *
// *************************************************************************

// TODO: Figure out how to enable the 'missing_docs' lint. The problem
//       is that this lint seemingly flags a problem with the imported
//       crates, which is not what we want.
#![deny(
  missing_debug_implementations,
  unsafe_code,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  warnings,
)]

//! A crate providing custom derive functionality for the `gui` crate.

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use proc_macro::LexError;
use proc_macro::TokenStream;
use quote::Tokens;
use syn::Attribute;
use syn::Body;
use syn::DeriveInput;
use syn::Lit;
use syn::MetaItem;
use syn::parse_derive_input;
use syn::StrStyle;


/// An enum representing the various widget types we support to derive from.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Type {
  Container,
  RootWidget,
  Widget,
}


/// The error type used internally by this module.
#[derive(Debug)]
enum Error {
  Error(String),
  LexError(LexError),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    match *self {
      Error::Error(ref e) => write!(f, "{}", e),
      Error::LexError(ref e) => write!(f, "{:?}", e),
    }
  }
}

impl From<String> for Error {
  fn from(string: String) -> Error {
    Error::Error(string)
  }
}

impl From<&'static str> for Error {
  fn from(string: &'static str) -> Error {
    Error::Error(string.to_string())
  }
}

impl From<LexError> for Error {
  fn from(error: LexError) -> Error {
    Error::LexError(error)
  }
}

type Result<T> = std::result::Result<T, Error>;


/// Custom derive functionality for the `gui::Widget` trait.
///
/// Using this macro a default implementation of the `gui::Widget`
/// trait can be created. Note that this trait is just a unification of
/// the `gui::Object`, `gui::Renderer`, and `gui::Handleable` traits.
/// Note furthermore that only implementations of the former two will be
/// auto generated. The reason for this behavior is that
/// `gui::Handleable` most likely needs customization to accommodate for
/// custom event handling behavior.
#[proc_macro_derive(GuiWidget, attributes(GuiType))]
pub fn widget(input: TokenStream) -> TokenStream {
  match expand_widget(input) {
    Ok(tokens) => tokens,
    Err(error) => panic!("{}", error),
  }
}

fn expand_widget(input: TokenStream) -> Result<TokenStream> {
  let string = input.to_string();
  let input = parse_derive_input(&string)?;
  let type_ = parse_widget_attributes(&input.attrs)?;
  let tokens = expand_widget_input(&type_, &input)?.parse()?;
  Ok(tokens)
}

/// Parse the macro's attributes.
fn parse_widget_attributes(attributes: &[Attribute]) -> Result<Type> {
  match attributes.len() {
    // If no attribute is given we default to `Widget`.
    0 => Ok(Type::Widget),
    1 => parse_widget_attribute(&attributes[0]),
    x => Err(Error::from(format!("unsupported number of arguments ({})", x))),
  }
}

/// Parse a single attribute, e.g., #[GuiType = "Widget"].
fn parse_widget_attribute(attribute: &Attribute) -> Result<Type> {
  // We don't care about the other meta data elements, inner/outer,
  // doc/non-doc, it's all fine by us.

  match attribute.value {
    MetaItem::NameValue(ref ident, ref literal) if ident == "GuiType" => {
      match *literal {
        Lit::Str(ref string, style) if style == StrStyle::Cooked => {
          match string.as_ref() {
            "Container" => Ok(Type::Container),
            "RootWidget" => Ok(Type::RootWidget),
            "Widget" => Ok(Type::Widget),
            _ => Err(Error::from(format!("unsupported type: {}", string))),
          }
        },
        _ => Err(Error::from(format!("unsupported literal type: {:?}", literal))),
      }
    },
    _ => Err(Error::from(format!("unsupported attribute: {}", attribute.value.name()))),
  }
}

/// Expand the input with the implementation of the required traits.
fn expand_widget_input(type_: &Type, input: &DeriveInput) -> Result<Tokens> {
  match input.body {
    Body::Struct(_) => Ok(expand_widget_traits(type_, input)),
    _ => Err(Error::from("#[derive(GuiWidget)] is only defined for structs")),
  }
}

/// Expand the struct input with the implementation of the required traits.
// TODO: If possible we should detect whether the client struct contains
//       an `id` and `parent_id` field as we require and provide a
//       detailed problem report otherwise.
// TODO: We could provide a default implementation for #name::new, if
//       the client does not define it currently.
// TODO: We could provide a default implementation for gui::Handleable, if
//       the client does not define it currently.
fn expand_widget_traits(type_: &Type, input: &DeriveInput) -> Tokens {
  let renderer = expand_renderer_trait(input);
  let object = expand_object_trait(type_, input);
  let widget = expand_widget_trait(input);

  quote! {
    #renderer
    #object
    #widget
  }
}

/// Expand an implementation for the `gui::Renderer` trait.
fn expand_renderer_trait(input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  quote! {
    impl<__R> #impl_generics ::gui::Renderable<__R> for #name #ty_generics #where_clause
    where
      __R: ::gui::Renderer,
    {
      fn render(&self, renderer: &__R) {
        renderer.render(self)
      }
    }
  }
}


/// Expand an implementation for the `gui::Object` trait.
fn expand_object_trait(type_: &Type, input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  let parent_id = match *type_ {
    Type::Widget | Type::Container => quote!{ Some(self.parent_id) },
    Type::RootWidget => quote!{ None },
  };

  let children = match *type_ {
    Type::Widget => quote!{},
    Type::Container | Type::RootWidget => {
      quote!{
        fn add_child(&mut self, id: ::gui::Id) {
          self.children.push(id)
        }

        fn iter(&self) -> ::gui::ChildIter {
          ::gui::ChildIter::with_iter(self.children.iter())
        }
      }
    },
  };

  quote! {
    impl #impl_generics ::gui::Object for #name #ty_generics #where_clause {
      fn id(&self) -> ::gui::Id {
        self.id
      }

      fn parent_id(&self) -> ::std::option::Option<::gui::Id> {
        #parent_id
      }

      #children
    }
  }
}

/// Expand an implementation for the `gui::Widget` trait.
fn expand_widget_trait(input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  quote! {
    impl<__R> #impl_generics ::gui::Widget<__R> for #name #ty_generics #where_clause
    where
      __R: ::gui::Renderer,
    {
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  fn get_type_attribute(string: &str) -> Result<Type> {
    let string = quote!{
      #[GuiType = #string]
      struct Foo { }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    parse_widget_attributes(&input.attrs)
  }

  #[test]
  fn default_widget_type_attribute() {
    let string = quote!{
      struct Bar { }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    assert_eq!(parse_widget_attributes(&input.attrs).unwrap(), Type::Widget);
  }

  #[test]
  fn widget_type_attribute() {
    let types = vec![
      ("Container", Type::Container),
      ("RootWidget", Type::RootWidget),
      ("Widget", Type::Widget),
    ];
    for (string, expected_type) in types {
      let type_ = get_type_attribute(string).unwrap();
      assert_eq!(type_, expected_type);
    }
  }

  #[test]
  #[should_panic(expected = "unsupported type: Cont")]
  fn unsupported_widget_type_attribute() {
    get_type_attribute("Cont").unwrap();
  }
}
