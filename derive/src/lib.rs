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
use syn::Field;
use syn::Generics;
use syn::Ident;
use syn::MetaItem;
use syn::NestedMetaItem;
use syn::Path;
use syn::PathParameters;
use syn::PathSegment;
use syn::PolyTraitRef;
use syn::TraitBoundModifier;
use syn::Ty;
use syn::TyParam;
use syn::TyParamBound;
use syn::WhereBoundPredicate;
use syn::WhereClause;
use syn::WherePredicate::BoundPredicate;
use syn::parse_derive_input;


/// An enum representing the various widget types we support to derive from.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Type {
  Container,
  RootWidget,
  Widget,
}

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
///
/// This macro roughly expands to the following code:
///
/// ```rust
/// # extern crate gui;
/// # #[derive(Debug)]
/// # struct TestWidget {
/// #   id: gui::Id,
/// #   parent_id: gui::Id,
/// # }
/// impl<R> gui::Renderable<R> for TestWidget
/// where
///   R: gui::Renderer,
/// {
///   fn render(&self, renderer: &R) {
///     renderer.render(self)
///   }
/// }
///
/// impl gui::Object for TestWidget {
///   fn id(&self) -> gui::Id {
///     self.id
///   }
///   fn parent_id(&self) -> Option<gui::Id> {
///     Some(self.parent_id)
///   }
/// }
///
/// impl<R> gui::Widget<R> for TestWidget
/// where
///   R: gui::Renderer,
/// {
/// }
/// # impl gui::Handleable for TestWidget {}
/// ```
#[proc_macro_derive(GuiWidget, attributes(gui))]
pub fn widget(input: TokenStream) -> TokenStream {
  ui_object(&Type::Widget, input)
}

/// Custom derive functionality for the `gui::Widget` trait for a
/// container variant.
///
/// This macro roughly expands to the following code:
///
/// ```rust
/// # extern crate gui;
/// # #[derive(Debug)]
/// # struct TestContainer {
/// #   id: gui::Id,
/// #   parent_id: gui::Id,
/// #   children: Vec<gui::Id>,
/// # }
/// impl<R> gui::Renderable<R> for TestContainer
/// where
///   R: gui::Renderer,
/// {
///   fn render(&self, renderer: &R) {
///     renderer.render(self)
///   }
/// }
///
/// impl gui::Object for TestContainer {
///   fn id(&self) -> gui::Id {
///     self.id
///   }
///   fn parent_id(&self) -> Option<gui::Id> {
///     Some(self.parent_id)
///   }
///   fn add_child(&mut self, id: gui::Id) {
///     self.children.push(id)
///   }
///   fn iter(&self) -> gui::ChildIter {
///     gui::ChildIter::with_iter(self.children.iter())
///   }
/// }
///
/// impl<R> gui::Widget<R> for TestContainer
/// where
///   R: gui::Renderer,
/// {
/// }
/// # impl gui::Handleable for TestContainer {}
#[proc_macro_derive(GuiContainer, attributes(gui))]
pub fn container(input: TokenStream) -> TokenStream {
  ui_object(&Type::Container, input)
}

/// Custom derive functionality for the `gui::Widget` trait for a root
/// widget variant.
///
/// This macro roughly expands to the following code:
///
/// ```rust
/// # extern crate gui;
/// # #[derive(Debug)]
/// # struct TestRootWidget {
/// #   id: gui::Id,
/// #   children: Vec<gui::Id>,
/// # }
/// impl<R> gui::Renderable<R> for TestRootWidget
/// where
///   R: gui::Renderer,
/// {
///   fn render(&self, renderer: &R) {
///     renderer.render(self)
///   }
/// }
///
/// impl gui::Object for TestRootWidget {
///   fn id(&self) -> gui::Id {
///     self.id
///   }
///   fn parent_id(&self) -> Option<gui::Id> {
///     None
///   }
///   fn add_child(&mut self, id: gui::Id) {
///     self.children.push(id)
///   }
///   fn iter(&self) -> gui::ChildIter {
///     gui::ChildIter::with_iter(self.children.iter())
///   }
/// }
///
/// impl<R> gui::Widget<R> for TestRootWidget
/// where
///   R: gui::Renderer,
/// {
/// }
/// # impl gui::Handleable for TestRootWidget {}
/// ```
#[proc_macro_derive(GuiRootWidget, attributes(gui))]
pub fn root_widget(input: TokenStream) -> TokenStream {
  ui_object(&Type::RootWidget, input)
}

fn ui_object(type_: &Type, input: TokenStream) -> TokenStream {
  match expand_ui_object(type_, input) {
    Ok(tokens) => tokens,
    Err(error) => panic!("{}", error),
  }
}

fn expand_ui_object(type_: &Type, input: TokenStream) -> Result<TokenStream> {
  let string = input.to_string();
  let input = parse_derive_input(&string)?;
  let new = parse_ui_object_attributes(&input.attrs)?;
  let tokens = expand_widget_input(&type_, &new, &input)?.parse()?;
  Ok(tokens)
}

/// Parse the macro's attributes.
fn parse_ui_object_attributes(attributes: &[Attribute]) -> Result<New> {
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
fn parse_gui_attribute(item: &NestedMetaItem) -> Result<New> {
  match *item {
    NestedMetaItem::MetaItem(ref meta_item) => {
      match *meta_item {
        MetaItem::Word(ref ident) if ident == "default_new" => Ok(New::Default),
        _ => Err(Error::from(format!("unsupported attribute: {}", meta_item.name()))),
      }
    },
    NestedMetaItem::Literal(ref literal) => {
      Err(Error::from(format!("unsupported literal: {:?}", literal)))
    },
  }
}

/// Parse a #[gui(list...)] attribute list.
fn parse_gui_attributes(list: &[NestedMetaItem]) -> Result<New> {
  // Right now we only support a single attribute at all (default_new).
  // So strictly speaking if the first item is a match we are good,
  // otherwise we error out. However, we do not simply want to silently
  // ignore other (faulty) attributes, so as to inform the user about
  // any errors early on.
  for item in list {
    parse_gui_attribute(item)?;
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

  match attribute.value {
    MetaItem::List(ref ident, ref list) if ident == "gui" => {
      // Here we have found an attribute of the form #[gui(xxx, yyy,
      // ...)]. Parse the inner list.
      Ok(Some(parse_gui_attributes(list)?))
    },
    _ => Ok(None),
  }
}

/// Expand the input with the implementation of the required traits.
fn expand_widget_input(type_: &Type, new: &New, input: &DeriveInput) -> Result<Tokens> {
  match input.body {
    Body::Struct(ref body) => {
      check_struct_fields(type_, body.fields())?;
      Ok(expand_widget_traits(type_, new, input))
    },
    _ => Err(Error::from("#[derive(GuiWidget)] is only defined for structs")),
  }
}

/// Check the fields of the user's struct for required attributes.
// Note that we only check for the names of the required fields, not for
// the types. Checking types is cumbersome and best-effort anyway as we
// are working on tokens without context (a user could have a field of
// type Id but that could map to ::foo::Id and not ::gui::Id).
fn check_struct_fields(type_: &Type, fields: &[Field]) -> Result<()> {
  let id = ("id", "::gui::Id");
  let par_id = ("parent_id", "::gui::Id");
  let childs = ("children", "::std::vec::Vec<::gui::Id>");

  let req_fields = match *type_ {
    Type::Widget => vec![id, par_id],
    Type::Container => vec![id, par_id, childs],
    Type::RootWidget => vec![id, childs],
  };

  for (req_field, req_type) in req_fields {
    fields
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
fn expand_widget_traits(type_: &Type, new: &New, input: &DeriveInput) -> Tokens {
  let new_impl = expand_new_impl(type_, new, input);
  let renderer = expand_renderer_trait(input);
  let object = expand_object_trait(type_, input);
  let widget = expand_widget_trait(input);

  quote! {
    #new_impl
    #renderer
    #object
    #widget
  }
}


/// Expand an implementation of Type::new() for the struct.
fn expand_new_impl(type_: &Type, new: &New, input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  match *new {
    New::Default => {
      let (args, fields) = match *type_ {
        Type::Widget => {(
          quote! {
            parent_id: ::gui::Id,
            id: ::gui::Id,
          },
          quote! {
            id: id,
            parent_id: parent_id
          }
        )},
        Type::Container => {(
          quote! {
            parent_id: ::gui::Id,
            id: ::gui::Id,
          },
          quote! {
            id: id,
            parent_id: parent_id,
            children: Vec::new(),
          }
        )},
        Type::RootWidget => {(
          quote! {
            id: ::gui::Id,
          },
          quote! {
            id: id,
            children: Vec::new(),
          }
        )},
      };
      quote! {
        #[allow(dead_code)]
        impl #impl_generics #name #ty_generics #where_clause {
          pub fn new(#args) -> Self {
            #name {
              #fields
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
  let (_, ty_generics, _) = input.generics.split_for_impl();
  let impl_generics = extend_generic_impl(&input.generics, Ident::new("__R"));
  let where_clause = extend_where_clause(&input.generics, Ident::new("__R"));

  quote! {
    impl #impl_generics ::gui::Renderable<__R> for #name #ty_generics #where_clause {
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
  let (_, ty_generics, _) = input.generics.split_for_impl();
  let impl_generics = extend_generic_impl(&input.generics, Ident::new("__R"));
  let where_clause = extend_where_clause(&input.generics, Ident::new("__R"));

  quote! {
    impl #impl_generics ::gui::Widget<__R> for #name #ty_generics #where_clause {}
  }
}

/// Extract an extended generic impl of the given Generics object.
fn extend_generic_impl(generics: &Generics, ident: Ident) -> Generics {
  let ty_param = TyParam {
    attrs: Vec::new(),
    ident: ident,
    bounds: Vec::new(),
    default: None,
  };

  let mut impl_generics = generics.clone();
  impl_generics.ty_params.push(ty_param);
  impl_generics
}

/// Extract an extended where clause of the given Generics object.
fn extend_where_clause(generics: &Generics, ident: Ident) -> WhereClause {
  let predicate = BoundPredicate(WhereBoundPredicate {
    bound_lifetimes: Vec::new(),
    bounded_ty: Ty::Path(
      None,
      Path {
        global: false,
        segments: vec![
          PathSegment {
            ident: ident,
            parameters: PathParameters::AngleBracketed(Default::default()),
          },
        ],
      },
    ),
    bounds: vec![
      TyParamBound::Trait(
        PolyTraitRef {
          bound_lifetimes: Vec::new(),
          trait_ref: Path {
            global: true,
            segments: vec![
              PathSegment {
                ident: Ident::new("gui"),
                parameters: PathParameters::AngleBracketed(Default::default()),
              },
              PathSegment {
                ident: Ident::new("Renderer"),
                parameters: PathParameters::AngleBracketed(Default::default()),
              },
            ],
          },
        },
        TraitBoundModifier::None,
      ),
    ],
  });

  let mut where_clause = generics.where_clause.clone();
  where_clause.predicates.push(predicate);
  where_clause
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
/// # extern crate gui;
/// # #[macro_use]
/// # extern crate gui_derive;
/// # #[derive(Debug, GuiWidget)]
/// # struct TestWidget {
/// #   id: gui::Id,
/// #   parent_id: gui::Id,
/// # }
/// impl gui::Handleable for TestWidget {}
/// # fn main() {}
/// ```
#[proc_macro_derive(GuiHandleable)]
pub fn handleable(input: TokenStream) -> TokenStream {
  match expand_handleable(input) {
    Ok(tokens) => tokens,
    Err(error) => panic!("{}", error),
  }
}

fn expand_handleable(input: TokenStream) -> Result<TokenStream> {
  let string = input.to_string();
  let input = parse_derive_input(&string)?;
  let tokens = expand_handleable_input(&input)?.parse()?;
  Ok(tokens)
}

/// Expand the input with the implementation of the required traits.
fn expand_handleable_input(input: &DeriveInput) -> Result<Tokens> {
  match input.body {
    Body::Struct(_) => {
      let name = &input.ident;
      let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

      Ok(quote! {
        impl #impl_generics ::gui::Handleable for #name #ty_generics #where_clause {}
      })
    },
    _ => Err(Error::from("#[derive(GuiHandleable)] is only defined for structs")),
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn impl_extension_no_generics() {
    let string = quote!{
      struct Bar { }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    let impl_generics = extend_generic_impl(&input.generics, Ident::new("Foo"));
    let tokens = quote!{ #impl_generics };

    assert_eq!(tokens.to_string(), "< Foo >");
  }

  #[test]
  fn generic_impl_extension() {
    let string = quote!{
      struct Bar<__T> { }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    let impl_generics = extend_generic_impl(&input.generics, Ident::new("Y"));
    let tokens = quote!{ #impl_generics };

    assert_eq!(tokens.to_string(), "< __T , Y >");
  }

  #[test]
  fn where_clause_extension_no_generics() {
    let string = quote!{
      struct Bar { }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    let where_clause = extend_where_clause(&input.generics, Ident::new("__X"));
    let tokens = quote!{ #where_clause };

    assert_eq!(tokens.to_string(), "where __X : :: gui :: Renderer");
  }

  #[test]
  fn where_clause_extension() {
    let string = quote!{
      struct Bar<'a, T>
      where
        T: Debug,
      {
      }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    let where_clause = extend_where_clause(&input.generics, Ident::new("Test"));
    let tokens = quote!{ #where_clause };

    assert_eq!(tokens.to_string(), "where T : Debug , Test : :: gui :: Renderer");
  }

  #[test]
  fn default_widget_attributes() {
    let string = quote! {
      struct Bar { }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    let new = parse_ui_object_attributes(&input.attrs).unwrap();
    assert_eq!(new, New::None);
  }

  #[test]
  fn default_new() {
    let string = quote! {
      #[gui(default_new)]
      struct Bar { }
    }.to_string();

    let input = parse_derive_input(&string).unwrap();
    assert_eq!(parse_ui_object_attributes(&input.attrs).unwrap(), New::Default);
  }
}