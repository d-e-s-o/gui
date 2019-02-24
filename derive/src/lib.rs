// lib.rs

// *************************************************************************
// * Copyright (C) 2018-2019 Daniel Mueller (deso@posteo.net)              *
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
  missing_copy_implementations,
  missing_debug_implementations,
  trivial_casts,
  trivial_numeric_casts,
  unsafe_code,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  unused_results,
)]
#![warn(
  future_incompatible,
  rust_2018_compatibility,
  rust_2018_idioms,
)]

//! A crate providing custom derive functionality for the `gui` crate.

extern crate proc_macro;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use proc_macro::LexError;
use proc_macro::TokenStream;
use quote::__rt::TokenStream as Tokens;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::Meta;
use syn::NestedMeta;
use syn::parse2;
use syn::punctuated::Punctuated;
use syn::token::Comma;


/// An enum to decide whether or not to create a default implementation of type::new().
#[derive(Clone, Debug, Eq, PartialEq)]
enum New {
  Default,
  None,
}

/// The error type used internally by this module.
#[derive(Debug)]
enum Error {
  Error(String),
  LexError(LexError),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
///
/// This macro roughly expands to the following code:
///
/// ```rust
/// # use std::any::TypeId;
/// # #[derive(Debug)]
/// # struct TestWidget {
/// #   id: gui::Id,
/// # }
/// impl gui::Renderable for TestWidget {
///   fn render(&self, renderer: &gui::Renderer, bbox: gui::BBox, cap: &gui::Cap) -> gui::BBox {
///     renderer.render(self, bbox, cap)
///   }
/// }
///
/// impl gui::Object for TestWidget {
///   fn id(&self) -> gui::Id {
///     self.id
///   }
/// }
///
/// impl gui::Widget for TestWidget {
///   fn type_id(&self) -> TypeId {
///     TypeId::of::<TestWidget>()
///   }
/// }
/// # impl gui::Handleable for TestWidget {}
/// ```
#[proc_macro_derive(Widget, attributes(gui))]
pub fn widget(input: TokenStream) -> TokenStream {
  match expand_widget(input) {
    Ok(tokens) => tokens,
    Err(error) => panic!("{}", error),
  }
}

fn expand_widget(input: TokenStream) -> Result<TokenStream> {
  let input = parse2::<DeriveInput>(input.into()).or_else(|_| {
    Err("unable to parse input")
  })?;
  let new = parse_widget_attributes(&input.attrs)?;
  let tokens = expand_widget_input(&new, &input)?;
  Ok(tokens.into())
}

/// Parse the macro's attributes.
fn parse_widget_attributes(attributes: &[Attribute]) -> Result<New> {
  let new = attributes
    .iter()
    .map(|attr| parse_widget_attribute(attr))
    .fold(Ok(None), |result1, result2| {
      match (result1, result2) {
        (Ok(new1), Ok(new2)) => Ok(new2.or(new1)),
        (Err(x), _) | (_, Err(x)) => Err(x),
      }
    })?;

  // If no attribute is given we do not create a default implementation
  // of new().
  Ok(new.map_or(New::None, |x| x))
}

/// Parse a single item in a #[gui(list...)] attribute list.
fn parse_gui_attribute(item: &NestedMeta) -> Result<New> {
  match *item {
    NestedMeta::Meta(ref meta_item) => {
      match *meta_item {
        Meta::Word(ref ident) if ident == "default_new" => Ok(New::Default),
        _ => Err(Error::from(format!("unsupported attribute: {}", meta_item.name()))),
      }
    },
    NestedMeta::Literal(_) => Err(Error::from("unsupported literal")),
  }
}

/// Parse a #[gui(list...)] attribute list.
fn parse_gui_attributes(list: &Punctuated<NestedMeta, Comma>) -> Result<New> {
  // Right now we only support a single attribute at all (default_new).
  // So strictly speaking if the first item is a match we are good,
  // otherwise we error out. However, we do not simply want to silently
  // ignore other (faulty) attributes, so as to inform the user about
  // any errors early on.
  for item in list {
    let _ = parse_gui_attribute(item)?;
  }
  if !list.is_empty() {
    Ok(New::Default)
  } else {
    Ok(New::None)
  }
}

/// Parse a single attribute, e.g., #[GuiType = "Widget"].
fn parse_widget_attribute(attribute: &Attribute) -> Result<Option<New>> {
  // We don't care about the other meta data elements, inner/outer,
  // doc/non-doc, it's all fine by us.

  match attribute.interpret_meta() {
    Some(x) => {
      match x {
        Meta::List(ref list) if list.ident == "gui" => {
          // Here we have found an attribute of the form #[gui(xxx, yyy,
          // ...)]. Parse the inner list.
          Ok(Some(parse_gui_attributes(&list.nested)?))
        },
        _ => Ok(None),
      }
    },
    None => Ok(None),
  }
}

/// Expand the input with the implementation of the required traits.
fn expand_widget_input(new: &New, input: &DeriveInput) -> Result<Tokens> {
  match input.data {
    Data::Struct(ref data) => {
      check_struct_fields(&data.fields)?;
      Ok(expand_widget_traits(new, input))
    },
    _ => Err(Error::from("#[derive(Widget)] is only defined for structs")),
  }
}

/// Check the fields of the user's struct for required attributes.
// Note that we only check for the names of the required fields, not for
// the types. Checking types is cumbersome and best-effort anyway as we
// are working on tokens without context (a user could have a field of
// type Id but that could map to ::foo::Id and not ::gui::Id).
fn check_struct_fields(fields: &Fields) -> Result<()> {
  let id = ("id", "::gui::Id");

  for (req_field, req_type) in &[id] {
    let _ = fields
      .iter()
      .find(|field| {
        if let Some(ref ident) = field.ident {
          ident == req_field
        } else {
          false
        }
      })
      .ok_or_else(|| Error::from(format!("struct field {}: {} not found", req_field, req_type)))?;
  }
  Ok(())
}

/// Expand the struct input with the implementation of the required traits.
fn expand_widget_traits(new: &New, input: &DeriveInput) -> Tokens {
  let new_impl = expand_new_impl(new, input);
  let renderer = expand_renderer_trait(input);
  let object = expand_object_trait(input);
  let widget = expand_widget_trait(input);

  quote! {
    #new_impl
    #renderer
    #object
    #widget
  }
}


/// Expand an implementation of Type::new() for the struct.
fn expand_new_impl(new: &New, input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  match *new {
    New::Default => {
      quote! {
        #[allow(dead_code)]
        impl #impl_generics #name #ty_generics #where_clause {
          pub fn new(id: ::gui::Id) -> Self {
            #name {
              id: id,
            }
          }
        }
      }
    },
    New::None => quote! {},
  }
}

/// Expand an implementation for the `gui::Renderer` trait.
fn expand_renderer_trait(input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  quote! {
    impl #impl_generics ::gui::Renderable for #name #ty_generics #where_clause {
      fn render(&self,
                renderer: &::gui::Renderer,
                bbox: ::gui::BBox,
                cap: &::gui::Cap) -> ::gui::BBox {
        renderer.render(self, bbox, cap)
      }
    }
  }
}


/// Expand an implementation for the `gui::Object` trait.
fn expand_object_trait(input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  quote! {
    impl #impl_generics ::gui::Object for #name #ty_generics #where_clause {
      fn id(&self) -> ::gui::Id {
        self.id
      }
    }
  }
}

/// Expand an implementation for the `gui::Widget` trait.
fn expand_widget_trait(input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  quote! {
    impl #impl_generics ::gui::Widget for #name #ty_generics #where_clause {
      fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<#name #ty_generics>()
      }
    }
  }
}

/// Custom derive functionality for the `gui::Handleable` trait.
///
/// Using this macro a default implementation of the `gui::Handleable`
/// trait can be created. This functionality is mostly used in quick
/// prototyping/testing scenarios, because most custom widgets will also
/// need a custom event handler.
///
/// This macro roughly expands to the following code:
///
/// ```rust
/// # use gui_derive::Widget;
/// # #[derive(Debug, Widget)]
/// # struct TestWidget {
/// #   id: gui::Id,
/// # }
/// impl gui::Handleable for TestWidget {}
/// # fn main() {}
/// ```
#[proc_macro_derive(Handleable)]
pub fn handleable(input: TokenStream) -> TokenStream {
  match expand_handleable(input) {
    Ok(tokens) => tokens,
    Err(error) => panic!("{}", error),
  }
}

fn expand_handleable(input: TokenStream) -> Result<TokenStream> {
  let input = parse2::<DeriveInput>(input.into()).or_else(|_| {
    Err("unable to parse input")
  })?;
  let tokens = expand_handleable_input(&input)?;
  Ok(tokens.into())
}

/// Expand the input with the implementation of the required traits.
fn expand_handleable_input(input: &DeriveInput) -> Result<Tokens> {
  match input.data {
    Data::Struct(_) => {
      let name = &input.ident;
      let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

      Ok(quote! {
        impl #impl_generics ::gui::Handleable for #name #ty_generics #where_clause {}
      })
    },
    _ => Err(Error::from("#[derive(Handleable)] is only defined for structs")),
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_widget_attributes() {
    let tokens = quote! {
      struct Bar { }
    };

    let input = parse2::<DeriveInput>(tokens).unwrap();
    let new = parse_widget_attributes(&input.attrs).unwrap();
    assert_eq!(new, New::None);
  }

  #[test]
  fn default_new() {
    let tokens = quote! {
      #[gui(default_new)]
      struct Bar { }
    };

    let input = parse2::<DeriveInput>(tokens).unwrap();
    assert_eq!(parse_widget_attributes(&input.attrs).unwrap(), New::Default);
  }
}
