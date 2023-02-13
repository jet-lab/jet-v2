//! Read all the IDL files and stip their docs out.
//! This is a temporary measure until we can fix this in anchor-lang.

fn main() {
    let _ = strip();
}

fn strip() -> Option<()> {
    let mut entries = std::fs::read_dir("./target/idl").ok()?;
    while let Some(Ok(entry)) = entries.next() {
        println!("Stripping docs from IDL {:?}", entry.path());
        let file = std::fs::File::open(entry.path()).ok()?;
        let mut idl = serde_json::from_reader::<_, serde_json::Value>(file).ok()?;

        // Recursively strip out all the docs from the IDL
        let val = idl.as_object_mut()?;
        val.remove("docs");

        // Remove in instructions
        let instructions = val.get_mut("instructions")?.as_array_mut()?;
        for ix in instructions {
            let val = ix.as_object_mut()?;
            val.remove("docs");

            // Remove in instructions[].accounts
            let accounts = val.get_mut("accounts")?.as_array_mut()?;
            for account in accounts {
                let val = account.as_object_mut()?;
                val.remove("docs");
            }
        }

        // Remove in accounts, some IDLs might not have top-level accounts
        let Some(Some(accounts)) = val.get_mut("accounts").map(|v| v.as_array_mut()) else {
            continue;
        };

        for account in accounts {
            let val = account.as_object_mut()?;
            val.remove("docs");
            // Remove in type.fields
            let type_ = val.get_mut("type")?;
            let fields = type_.as_object_mut()?.get_mut("fields")?.as_array_mut()?;
            for field in fields {
                let val = field.as_object_mut()?;
                val.remove("docs");
            }
        }

        // Remove in types
        let Some(types) = val.get_mut("types").and_then(|v| v.as_array_mut()) else {
            continue;
        };

        for type_ in types {
            let val = type_.as_object_mut()?;
            val.remove("docs");
            let type_ = val.get_mut("type")?;
            let Some(fields) = type_.as_object_mut()?.get_mut("fields").and_then(|v| v.as_array_mut()) else {
                continue
            };
            for field in fields {
                let val = field.as_object_mut()?;
                val.remove("docs");
            }
        }

        // Write IDL out
        let file = std::fs::File::create(entry.path()).ok()?;
        serde_json::to_writer_pretty(file, &idl).ok()?
    }

    Some(())
}
