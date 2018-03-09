// mod.rs

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

extern crate gui;

use std::any::Any;

use gui::Id;
use gui::Renderer;


#[derive(Debug, GuiRootWidget, GuiHandleable)]
#[gui(default_new)]
pub struct TestRootWidget {
  id: Id,
  children: Vec<Id>,
}


#[derive(Debug, GuiWidget, GuiHandleable)]
#[gui(default_new)]
pub struct TestWidget {
  id: Id,
  parent_id: Id,
}


#[derive(Debug, GuiContainer, GuiHandleable)]
#[gui(default_new)]
pub struct TestContainer {
  id: Id,
  parent_id: Id,
  children: Vec<Id>,
}


#[allow(unused)]
#[derive(Debug)]
pub struct TestRenderer {}

impl Renderer for TestRenderer {
  fn render(&self, _object: &Any) {}
}
