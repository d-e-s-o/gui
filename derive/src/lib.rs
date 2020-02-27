// lib.rs

// *************************************************************************
// * Copyright (C) 2018-2020 Daniel Mueller (deso@posteo.net)              *
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

#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  missing_docs,
  rust_2018_compatibility,
  rust_2018_idioms,
  trivial_casts,
  trivial_numeric_casts,
  unsafe_code,
  unstable_features,
  unused_import_braces,
  unused_qualifications,
  unused_results,
)]

//! A crate providing custom derive functionality for the `gui` crate.

extern crate proc_macro;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use proc_macro::LexError;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use syn::Attribute;
use syn::Binding;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::GenericParam;
use syn::Generics;
use syn::parenthesized;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse2;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::token::Eq;
use syn::Type;
use syn::TypeGenerics;
use syn::WhereClause;
use syn::WherePredicate;


/// A type indicating whether or not to create a default implementation of Type::new().
type New = Option<()>;
/// A type representing an event type to parametrize a widget with.
type Event = Option<Type>;


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
/// # type Event = ();
/// # #[derive(Debug)]
/// # struct TestWidget {
/// #   id: gui::Id,
/// # }
/// impl gui::Renderable for TestWidget {
///   fn type_id(&self) -> TypeId {
///     TypeId::of::<TestWidget>()
///   }
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
/// impl gui::Widget<Event> for TestWidget {
///   fn type_id(&self) -> TypeId {
///     TypeId::of::<TestWidget>()
///   }
/// }
/// # impl gui::Handleable<Event> for TestWidget {}
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
  let (new, event) = parse_attributes(&input.attrs)?;
  let tokens = expand_widget_input(new, &event, &input)?;
  Ok(tokens.into())
}

/// Parse the macro's attributes.
fn parse_attributes(attributes: &[Attribute]) -> Result<(New, Event)> {
  let (new, event) = attributes
    .iter()
    .map(|attr| parse_attribute(attr))
    .fold(Ok((None, None)), |result1, result2| {
      match (result1, result2) {
        (Ok((new1, event1)), Ok((new2, event2))) => Ok((new2.or(new1), event2.or(event1))),
        (Err(x), _) | (_, Err(x)) => Err(x),
      }
    })?;

  // If no attribute is given we do not create a default implementation
  // of new().
  Ok((new, event))
}

/// Parse a single item in a #[gui(list...)] attribute list.
fn parse_gui_attribute(item: Attr) -> Result<(New, Event)> {
  match item {
    Attr::Ident(ref ident) if ident == "default_new" => {
      Ok((Some(()), None))
    },
    Attr::Binding(binding) => {
      // Unfortunately we can't use a pattern guard here. See issue
      // https://github.com/rust-lang/rust/issues/15287 for more
      // details.
      if binding.ident == "Event" {
        Ok((None, Some(binding.ty)))
      } else {
        Err(Error::from("encountered unknown binding attribute"))
      }
    },
    _ => Err(Error::from("encountered unknown attribute")),
  }
}

/// Parse a #[gui(list...)] attribute list.
fn parse_gui_attributes(list: AttrList) -> Result<(New, Event)> {
  let mut new = None;
  let mut event = None;

  for item in list.0 {
    let (this_new, this_event) = parse_gui_attribute(item)?;
    new = this_new.or(new);
    event = this_event.or(event);
  }
  Ok((new, event))
}


/// An attribute list representing an syn::Attribute::tts.
struct AttrList(Punctuated<Attr, Comma>);

impl Parse for AttrList {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let content;
    let _ = parenthesized!(content in input);
    let list = content.parse_terminated(Attr::parse)?;

    Ok(Self(list))
  }
}


enum Attr {
  Ident(Ident),
  Binding(Binding),
}

impl Parse for Attr {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    // We need to peek at the second token coming up next first, because
    // attempting to parse it would advance the position in the buffer.
    if input.peek2(Eq) {
      let bind = input.parse::<Binding>()?;
      Ok(Attr::Binding(bind))
    } else {
      input.parse::<Ident>().map(Attr::Ident)
    }
  }
}


/// Parse a single attribute, e.g., #[Event = MyEvent].
fn parse_attribute(attribute: &Attribute) -> Result<(New, Event)> {
  if attribute.path.is_ident("gui") {
    let tokens = attribute.tokens.clone();
    let attr = parse2::<AttrList>(tokens).or_else(|err| {
      Err(format!("unable to parse attributes: {:?}", err))
    })?;

    parse_gui_attributes(attr)
  } else {
    Ok((None, None))
  }
}

/// Expand the input with the implementation of the required traits.
fn expand_widget_input(new: New, event: &Event, input: &DeriveInput) -> Result<Tokens> {
  match input.data {
    Data::Struct(ref data) => {
      check_struct_fields(&data.fields)?;
      Ok(expand_widget_traits(new, event, input))
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
fn expand_widget_traits(new: New, event: &Event, input: &DeriveInput) -> Tokens {
  let new_impl = expand_new_impl(new, input);
  let renderer = expand_renderer_trait(input);
  let object = expand_object_trait(input);
  let widget = expand_widget_trait(event, input);

  quote! {
    #new_impl
    #renderer
    #object
    #widget
  }
}


/// Expand an implementation of Type::new() for the struct.
fn expand_new_impl(new: New, input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  match new {
    Some(..) => {
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
    None => quote! {},
  }
}

/// Expand an implementation for the `gui::Renderer` trait.
fn expand_renderer_trait(input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  quote! {
    impl #impl_generics ::gui::Renderable for #name #ty_generics #where_clause {
      fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<#name #ty_generics>()
      }

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
fn expand_widget_trait(event: &Event, input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let generic = event.is_none();
  let (generics, ty_generics, where_clause) = split_for_impl(&input.generics, generic);

  let widget = if let Some(event) = event {
    quote! { ::gui::Widget<#event> }
  } else {
    let event = Ident::new("__E", Span::call_site());
    quote! { ::gui::Widget<#event> }
  };

  quote! {
    impl #generics #widget for #name #ty_generics #where_clause {
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
/// # type Event = ();
/// # #[derive(Debug, Widget)]
/// # #[gui(Event = Event)]
/// # struct TestWidget {
/// #   id: gui::Id,
/// # }
/// impl gui::Handleable<Event> for TestWidget {}
/// # fn main() {}
/// ```
#[proc_macro_derive(Handleable, attributes(gui))]
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
  let (_, event) = parse_attributes(&input.attrs)?;
  let tokens = expand_handleable_input(&event, &input)?;
  Ok(tokens.into())
}

/// Expand the input with the implementation of the required traits.
fn expand_handleable_input(event: &Event, input: &DeriveInput) -> Result<Tokens> {
  match input.data {
    Data::Struct(_) => Ok(expand_handleable_trait(event, input)),
    _ => Err(Error::from("#[derive(Handleable)] is only defined for structs")),
  }
}

/// Extend generics with a type parameter named as per the given
/// identifier.
fn extend_generics(generics: &Generics, ident: Ident) -> Generics {
  let param = GenericParam::Type(ident.into());
  let mut generics = generics.clone();
  generics.params.push(param);
  generics
}

/// Extract an extended where clause of the given Generics object.
fn extend_where_clause(where_clause: &Option<WhereClause>, ident: &Ident) -> WhereClause {
  if let Some(where_clause) = where_clause {
    let predicate = quote! { #ident: 'static };
    let predicate = parse2::<WherePredicate>(predicate).unwrap();
    let mut where_clause = where_clause.clone();
    where_clause.predicates.push(predicate);
    where_clause
  } else {
    // Strictly speaking we should always have a where clause because
    // Handleable and Widget have additional trait constraints. However,
    // if the user forgets we would hit this code path before the
    // compiler could actually provide a hint (in the form of an error)
    // that clarifies the mistake. So just provide sane behavior here as
    // well.
    let where_clause = quote! { where #ident: 'static };
    parse2::<WhereClause>(where_clause).unwrap()
  }
}

/// Split a type's generics into the pieces required for impl'ing a
/// trait for that type, while correctly handling a potential generic
/// event type.
fn split_for_impl(generics: &Generics,
                  generic: bool) -> (Generics, TypeGenerics<'_>, Option<WhereClause>) {
  let (_, ty_generics, _) = generics.split_for_impl();

  if generic {
    let event = Ident::new("__E", Span::call_site());
    let generics = extend_generics(&generics, event.clone());
    let where_clause = extend_where_clause(&generics.where_clause, &event);

    (generics, ty_generics, Some(where_clause))
  } else {
    let generics = generics.clone();
    let where_clause = generics.where_clause.clone();

    (generics, ty_generics, where_clause)
  }
}

/// Expand an implementation for the `gui::Handleable` trait.
fn expand_handleable_trait(event: &Event, input: &DeriveInput) -> Tokens {
  let name = &input.ident;
  let generic = event.is_none();
  let (generics, ty_generics, where_clause) = split_for_impl(&input.generics, generic);

  let handleable = if let Some(event) = event {
    quote! { ::gui::Handleable<#event> }
  } else {
    let event = Ident::new("__E", Span::call_site());
    quote! { ::gui::Handleable<#event> }
  };

  quote! {
    impl #generics #handleable for #name #ty_generics #where_clause {}
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
    let (new, event) = parse_attributes(&input.attrs).unwrap();
    assert_eq!(new, None);
    assert_eq!(event, None);
  }

  #[test]
  fn default_new() {
    let tokens = quote! {
      #[gui(default_new)]
      struct Bar { }
    };

    let input = parse2::<DeriveInput>(tokens).unwrap();
    let (new, event) = parse_attributes(&input.attrs).unwrap();
    assert_eq!(new, Some(()));
    assert_eq!(event, None);
  }

  #[test]
  fn custom_event() {
    let tokens = quote! {
      #[gui(Event = FooBarBazEvent)]
      struct Bar { }
    };

    let input = parse2::<DeriveInput>(tokens).unwrap();
    let (new, event) = parse_attributes(&input.attrs).unwrap();
    assert_eq!(new, None);

    let tokens = quote! { FooBarBazEvent };
    let foobar = parse2::<Type>(tokens).unwrap();
    assert_eq!(event, Some(foobar));
  }

  #[test]
  fn default_new_and_event_with_ignore() {
    let tokens = quote! {
      #[allow(an_attribute_to_be_ignored)]
      #[gui(default_new, Event = ())]
      struct Baz { }
    };

    let input = parse2::<DeriveInput>(tokens).unwrap();
    let (new, event) = parse_attributes(&input.attrs).unwrap();
    assert_eq!(new, Some(()));

    let tokens = quote! { () };
    let parens = parse2::<Type>(tokens).unwrap();
    assert_eq!(event, Some(parens));
  }

  #[test]
  fn last_event_type_takes_precedence() {
    let tokens = quote! {
      #[gui(Event = Event1)]
      #[gui(Event = Event2)]
      struct Foo { }
    };

    let input = parse2::<DeriveInput>(tokens).unwrap();
    let (_, event) = parse_attributes(&input.attrs).unwrap();

    let tokens = quote! { Event2 };
    let event2 = parse2::<Type>(tokens).unwrap();
    assert_eq!(event, Some(event2));
  }
}
