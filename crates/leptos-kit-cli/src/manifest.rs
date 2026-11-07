use crate::registry::{COMPONENTS, Component};
use crate::{Result, files};
use std::path::Path;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Value};

const MANIFEST: &str = "Cargo.toml";
const KIT: &str = "leptos-kit";

// the kit's features the copied components need, on its dependency in the app's manifest; the app's own features stay
pub fn write(order: &[&Component]) -> Result {
    let wanted: Vec<&str> = order
        .iter()
        .flat_map(|component| component.features)
        .copied()
        .collect();
    let owned: Vec<&str> = COMPONENTS
        .iter()
        .flat_map(|component| component.features)
        .copied()
        .collect();
    let Some(text) = files::read(Path::new(MANIFEST))? else {
        tell(&wanted);
        return Ok(());
    };
    let mut doc: DocumentMut = text.parse()?;
    let Some(dependency) = doc
        .get_mut("dependencies")
        .and_then(|dependencies| dependencies.get_mut(KIT))
    else {
        tell(&wanted);
        return Ok(());
    };
    if let Item::Value(Value::String(version)) = dependency {
        let mut table = InlineTable::new();
        table.insert("version", Value::String(version.clone()));
        *dependency = Item::Value(Value::InlineTable(table));
    }
    let Some(table) = dependency.as_table_like_mut() else {
        tell(&wanted);
        return Ok(());
    };
    let mut features: Vec<String> = table
        .get("features")
        .and_then(Item::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(|value| value.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let before = features.clone();
    features
        .retain(|feature| !owned.contains(&feature.as_str()) || wanted.contains(&feature.as_str()));
    for feature in &wanted {
        if !features.iter().any(|known| known == feature) {
            features.push((*feature).to_string());
        }
    }
    if features == before {
        return Ok(());
    }
    if features.is_empty() {
        table.remove("features");
    } else {
        let array: Array = features.iter().map(String::as_str).collect();
        table.insert("features", Item::Value(Value::Array(array)));
    }
    files::write(Path::new(MANIFEST), doc.to_string())?;
    println!("{KIT}'s features in {MANIFEST}: {}", features.join(", "));
    Ok(())
}

fn tell(wanted: &[&str]) {
    if !wanted.is_empty() {
        println!(
            "the components need {KIT}'s {} feature, add it to the dependency",
            wanted.join(", ")
        );
    }
}
