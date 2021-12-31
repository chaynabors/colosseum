// Copyright 2021 Chay Nabors.

use std::path::Path;

use codegen::Scope;
use codegen::Variant;
use heck::ToUpperCamelCase;

fn generate_common(scope: &mut Scope, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let identifier = scope
        .new_enum(&format!("{}Identifier", content.to_upper_camel_case()))
        .derive("Copy")
        .derive("Clone")
        .derive("Debug")
        .derive("Deserialize")
        .derive("EnumString")
        .derive("Eq")
        .derive("Hash")
        .derive("PartialEq")
        .derive("Serialize")
        .repr("u8")
        .vis("pub");

    for read_dir in std::fs::read_dir(&format!("content/{}/", content))? {
        let name = read_dir?.path().file_stem().unwrap().to_owned().into_string().unwrap();
        identifier.push_variant(Variant::new(&format!("#[strum(serialize = \"{}\")] {}", name, name.to_upper_camel_case())));
    }

    scope
        .new_impl(&format!("&{}", content.to_upper_camel_case()))
        .impl_trait(&format!("From<{}Identifier>", content.to_upper_camel_case()))
        .new_fn("from")
        .arg("from", &format!("{}Identifier", content.to_upper_camel_case()))
        .ret("Self")
        .line("&STORE[&from]");

    scope
        .new_impl(&format!("{}Identifier", content.to_upper_camel_case()))
        .impl_trait("fmt::Display")
        .new_fn("fmt")
        .arg_ref_self()
        .arg("f", "&mut fmt::Formatter")
        .ret("fmt::Result")
        .line("write!(f, \"{:?}\", self)");

    let store_type = format!("HashMap<{}Identifier, {}>", content.to_upper_camel_case(), content.to_upper_camel_case());
    let mut store = "lazy_static::lazy_static! {\n".to_owned();
    store.push_str(&format!("    static ref STORE: {} = {{\n", store_type));
    store.push_str("        let mut hashmap = HashMap::new();\n");
    store.push_str("        for file in DIR.files() {\n");
    store.push_str("            let file_name = file.path().file_stem().unwrap().to_str().unwrap();\n");
    store.push_str(
        "            let stored = serde_json::from_str(file.contents_utf8().unwrap()).expect(&format!(\"failed to \
         deserialize {}\", file_name));\n",
    );
    store.push_str(&format!(
        "            hashmap.insert({}Identifier::from_str(file_name).unwrap(), stored);\n",
        content.to_upper_camel_case()
    ));
    store.push_str("        }\n\n");
    store.push_str("        hashmap\n");
    store.push_str("    };\n");
    store.push_str("}\n");

    scope.raw(&format!("const DIR: include_dir::Dir = include_dir::include_dir!(\"$CARGO_MANIFEST_DIR/content/{}\");", content));
    scope.raw(&store);

    Ok(())
}

fn generate_wearable(wearable: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut scope = Scope::new();
    scope.import("crate::aspect", "Aspect");
    scope.import("serde", "Deserialize");
    scope.import("serde", "Serialize");
    scope.import("std::collections", "HashMap");
    scope.import("std", "fmt");
    scope.import("std::str", "FromStr");
    scope.import("strum", "EnumString");

    scope
        .new_struct(&wearable.to_upper_camel_case())
        .vis("pub")
        .derive("Debug")
        .derive("Default")
        .derive("Deserialize")
        .derive("PartialEq")
        .derive("Serialize")
        .field("pub display_name", "String")
        .field("pub description", "String")
        .field("pub fire_defense", "f64")
        .field("pub frost_defense", "f64")
        .field("pub lightning_defense", "f64")
        .field("pub physical_defense", "f64");

    scope
        .new_impl(&wearable.to_upper_camel_case())
        .new_fn("defense")
        .vis("pub")
        .arg_ref_self()
        .arg("aspect", "Aspect")
        .ret("f64")
        .line("match aspect {")
        .line("    Aspect::Fire => self.fire_defense,")
        .line("    Aspect::Frost => self.frost_defense,")
        .line("    Aspect::Lightning => self.lightning_defense,")
        .line("    Aspect::Physical => self.physical_defense,")
        .line("}");

    generate_common(&mut scope, wearable)?;

    std::fs::write(
        &format!("src/generated/{}.rs", wearable),
        &format!("// Copyright 2021 Chay Nabors.\n// THIS IS A GENERATED FILE AND NOT INTENDED FOR EDITING\n\n{}", scope.to_string()),
    )?;

    Ok(())
}

fn generate_wearables() -> Result<(), Box<dyn std::error::Error>> {
    generate_wearable("bodywear")?;
    generate_wearable("footwear")?;
    generate_wearable("handwear")?;
    generate_wearable("headwear")?;
    generate_wearable("legwear")?;

    Ok(())
}

fn generate_content(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut scope = Scope::new();
    scope.import("super", &format!("{}", content.to_upper_camel_case()));
    scope.import("serde", "Deserialize");
    scope.import("serde", "Serialize");
    scope.import("std::collections", "HashMap");
    scope.import("std", "fmt");
    scope.import("std::str", "FromStr");
    scope.import("strum", "EnumString");
    scope.raw("// THIS IS A GENERATED FILE AND NOT INTENDED FOR EDITING");

    generate_common(&mut scope, content)?;

    std::fs::write(
        &format!("src/generated/{}.rs", content),
        &format!("// Copyright 2021 Chay Nabors.\n\n{}", scope.to_string()),
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generated = Path::new("src/generated/");
    if !generated.exists() {
        std::fs::create_dir("src/generated")?;
    }

    generate_wearables()?;
    generate_content("consumable")?;
    generate_content("skill")?;
    generate_content("weapon")?;
    Ok(())
}
