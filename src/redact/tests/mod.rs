// SPDX-License-Identifier: GPL-3.0-only

mod debug;

use indoc::indoc;
use std::collections::HashSet;
use strum::{IntoEnumIterator, VariantNames};

use super::{process, PatternEntry};

fn build_nix_test_module() -> (String, HashSet<String>) {
    fn key(name: &str) -> (String, String) {
        (name.into(), format!("\"{name}_key\""))
    }

    fn salt_and_key(name: &str) -> (String, String) {
        (name.into(), format!("\"{name}_salt\" \"{name}_key\""))
    }

    let variants: Vec<(String, String)> = PatternEntry::iter()
        .map(|entry| match entry {
            PatternEntry::Bcrypt => key(entry.as_ref()),
            PatternEntry::Sha512 => key(entry.as_ref()),
            PatternEntry::Psk => salt_and_key(entry.as_ref()),
        })
        .collect();

    let mut keys = HashSet::new();
    let mut node_let_in = vec![];
    let mut node_attr_set = vec![];
    let mut node_list = vec![];

    for (name, value) in variants {
        let node_apply = format!("{name} {value}");

        //function with similar name
        node_let_in.push(format!("{name} = {name}: \"${{{name}}}\";"));

        node_attr_set.push(format!("# {node_apply}"));
        node_attr_set.push(format!("{name} = {node_apply};"));
        node_attr_set.push(format!("{name} = ({node_apply});"));

        // TODO: Critical
        // If node_apply is not enclosed in brackets in lists, this is not a parsing error.
        // This will lead to the disclosure of secrets!
        node_list.push(format!("({node_apply})"));

        keys.insert(value);
        //PatternEntry
        keys.insert(format!("{name},"));
    }

    let module = format!(
        indoc! {r#"
            {{
              {},
              ...
            }} @ input: let
              {}
            in {{
              {}
              list = [
                {} 
              ];
            }}
        "#},
        PatternEntry::VARIANTS.join(",\n  "),
        node_let_in.join("\n  "),
        node_attr_set.join("\n  "),
        node_list.join("\n    ")
    );

    println!("{}", module);

    (module, keys)
}

#[test]
fn process_success() {
    let (module, keys) = build_nix_test_module();

    let redacted = process(&module.as_bytes()).expect("should succeed");
    let redacted_str = std::str::from_utf8(&redacted).expect("should succeed");

    println!("{}", redacted_str);

    for key in keys {
        assert!(!redacted_str.contains(&key))
    }
}

#[test]
#[ignore = "Just for Debug"]
fn build_nix_test_module_debug() {
    let (redacted, _) = build_nix_test_module();

    debug::pretty_print_root(
        &rnix::Root::parse(&redacted).syntax(),
        PatternEntry::VARIANTS,
    );
}
