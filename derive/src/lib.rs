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
)]

//! A crate providing custom derive functionality for the `gui` crate.

extern crate proc_macro;
#[allow(unused_imports)]
#[macro_use]
extern crate quote;
extern crate syn;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use syn::Attribute;
use syn::Lit;
use syn::MetaItem;
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
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    match *self {
      Error::Error(ref e) => write!(f, "{}", e),
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

type Result<T> = std::result::Result<T, Error>;


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

/// Parse the macro's attributes.
fn parse_widget_attributes(attributes: &[Attribute]) -> Result<Type> {
  match attributes.len() {
    // If no attribute is given we default to `Widget`.
    0 => Ok(Type::Widget),
    1 => parse_widget_attribute(&attributes[0]),
    x => Err(Error::from(format!("unsupported number of arguments ({})", x))),
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  use syn::parse_derive_input;


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
