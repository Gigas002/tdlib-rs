// Copyright 2020 - developers of the `grammers` project.
// Copyright 2022 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` and `tdlib-rs` projects.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::rustifier;
use std::collections::{HashMap, HashSet};
use tdlib_rs_parser::tl::{Category, Definition, Type};

/// Additional metadata required by several parts of the generation.
pub(crate) struct Metadata<'a> {
    recursing_defs: HashSet<&'a String>,
    default_impl_defs: HashSet<&'a String>,
    defs_with_type: HashMap<&'a String, Vec<&'a Definition>>,
}

impl<'a> Metadata<'a> {
    pub fn new(definitions: &'a [Definition]) -> Self {
        let mut metadata = Self {
            recursing_defs: HashSet::new(),
            default_impl_defs: HashSet::new(),
            defs_with_type: HashMap::new(),
        };

        let type_definitions = definitions
            .iter()
            .filter(|d| d.category == Category::Types)
            .collect::<Vec<_>>();

        let type_definition_map = type_definitions
            .iter()
            .map(|d| (&d.name, d))
            .collect::<HashMap<_, _>>();

        type_definitions.iter().for_each(|d| {
            metadata
                .defs_with_type
                .entry(&d.ty.name)
                .or_default()
                .push(d);
        });

        type_definitions.iter().for_each(|d| {
            if def_self_references(
                d,
                d,
                &metadata.defs_with_type,
                &type_definition_map,
                &mut HashSet::new(),
            ) {
                metadata.recursing_defs.insert(&d.name);
            }
        });

        type_definitions.iter().for_each(|d| {
            if def_contains_only_bare_types(d, &type_definition_map) {
                metadata.default_impl_defs.insert(&d.name);
            }
        });

        metadata
    }

    /// Returns `true` if any of the parameters of `Definition` eventually
    /// contains the same type as the `Definition` itself (meaning it recurses).
    pub fn is_recursive_def(&self, def: &Definition) -> bool {
        self.recursing_defs.contains(&def.name)
    }

    /// Returns `true` if the `Definition` can implement the trait `Default`
    pub fn can_def_implement_default(&self, def: &Definition) -> bool {
        self.default_impl_defs.contains(&def.name)
    }

    pub fn defs_with_type(&self, ty: &'a Type) -> &Vec<&Definition> {
        &self.defs_with_type[&ty.name]
    }
}

fn def_self_references<'a>(
    root: &Definition,
    check: &'a Definition,
    defs_with_type: &'a HashMap<&String, Vec<&Definition>>,
    definition_map: &'a HashMap<&String, &&Definition>,
    visited: &mut HashSet<&'a String>,
) -> bool {
    visited.insert(&check.name);
    for param in check.params.iter() {
        if param.ty.name == root.ty.name {
            return true;
        }

        // Bare types refer to a single definition by its name instead of its type
        let defs: Vec<&Definition> = if param.ty.bare {
            definition_map
                .get(&param.ty.name)
                .map(|def| vec![**def])
                .unwrap_or_default()
        } else {
            defs_with_type
                .get(&param.ty.name)
                .cloned()
                .unwrap_or_default()
        };

        for def in defs {
            if visited.contains(&def.name) {
                continue;
            }
            if def_self_references(root, def, defs_with_type, definition_map, visited) {
                return true;
            }
        }
    }

    false
}

fn def_contains_only_bare_types<'a>(
    check: &'a Definition,
    definition_map: &'a HashMap<&String, &&Definition>,
) -> bool {
    for param in check.params.iter() {
        if !rustifier::parameters::is_builtin_type(param) && !param.ty.bare {
            return false;
        }

        if let Some(def) = definition_map.get(&param.ty.name)
            && !def_contains_only_bare_types(def, definition_map)
        {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(defs: &[&str]) -> Vec<Definition> {
        defs.iter().map(|d| d.parse().unwrap()).collect()
    }

    fn find<'a>(defs: &'a [Definition], name: &str) -> &'a Definition {
        defs.iter().find(|d| d.name == name).unwrap()
    }

    #[test]
    fn check_direct_recursion() {
        let defs = parse(&[
            "richTextPlain text:string = RichText",
            "richTextBold text:RichText = RichText",
        ]);
        let metadata = Metadata::new(&defs);
        assert!(!metadata.is_recursive_def(find(&defs, "richTextPlain")));
        assert!(metadata.is_recursive_def(find(&defs, "richTextBold")));
    }

    #[test]
    fn check_indirect_boxed_recursion() {
        let defs = parse(&[
            "pageBlockDetails header:RichText = PageBlock",
            "richTextPlain text:string = RichText",
            "richTextBlock block:PageBlock = RichText",
        ]);
        let metadata = Metadata::new(&defs);
        assert!(metadata.is_recursive_def(find(&defs, "pageBlockDetails")));
        assert!(metadata.is_recursive_def(find(&defs, "richTextBlock")));
        assert!(!metadata.is_recursive_def(find(&defs, "richTextPlain")));
    }

    #[test]
    fn check_indirect_bare_recursion() {
        // A bare type refers to the definition by name, as in TDLib 1.8.68
        let defs = parse(&[
            "inlineButton text:RichText = InlineButton",
            "richTextPlain text:string = RichText",
            "richTextButton button:inlineButton = RichText",
        ]);
        let metadata = Metadata::new(&defs);
        assert!(metadata.is_recursive_def(find(&defs, "richTextButton")));
        // Boxing the `RichText` variant already breaks the cycle
        assert!(!metadata.is_recursive_def(find(&defs, "inlineButton")));
        assert!(!metadata.is_recursive_def(find(&defs, "richTextPlain")));
    }

    #[test]
    fn check_unknown_bare_type_is_not_recursive() {
        let defs = parse(&["richTextButton button:missingType = RichText"]);
        let metadata = Metadata::new(&defs);
        assert!(!metadata.is_recursive_def(find(&defs, "richTextButton")));
    }

    #[test]
    fn check_can_implement_default() {
        let defs = parse(&[
            "point x:double y:double = Point",
            "location point:point = Location",
            "chat type:ChatType = Chat",
            "chatTypePrivate user_id:int53 = ChatType",
        ]);
        let metadata = Metadata::new(&defs);
        assert!(metadata.can_def_implement_default(find(&defs, "point")));
        assert!(metadata.can_def_implement_default(find(&defs, "location")));
        assert!(!metadata.can_def_implement_default(find(&defs, "chat")));
    }

    #[test]
    fn check_defs_with_type() {
        let defs = parse(&[
            "chatTypePrivate user_id:int53 = ChatType",
            "chatTypeSecret secret_chat_id:int32 = ChatType",
            "point x:double y:double = Point",
        ]);
        let metadata = Metadata::new(&defs);
        let ty = "ChatType".parse().unwrap();
        let names: Vec<_> = metadata
            .defs_with_type(&ty)
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, ["chatTypePrivate", "chatTypeSecret"]);
    }
}
