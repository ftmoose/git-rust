use crate::objects::{Kind, Object};
use anyhow::Context;
use std::io::prelude::*;

pub(crate) fn invoke(name_only: bool, object_hash: &str) -> anyhow::Result<()> {
    let object = Object::read(object_hash).context("making object")?;

    match object.kind {
        Kind::Tree => {
            let mut size = object.expected_size as usize;
            let mut z = object.reader;

            while size > 0 {
                let mut mode = Vec::new();
                let n =
                    z.read_until(b' ', &mut mode).context("reading mode")?;
                let mode = std::str::from_utf8(&mode[..mode.len() - 1])
                    .context("mode not valid utf8")?;
                size -= n;

                let mut name = Vec::new();
                let n = z.read_until(0, &mut name).context("reading name")?;
                let name = std::str::from_utf8(&name[..name.len() - 1])
                    .context("name not valid utf8")?;
                size -= n;

                let mut sha = [0; 20];
                z.read_exact(&mut sha).context("reading sha")?;
                let sha = hex::encode(sha);
                size -= 20;

                let t = match mode {
                    "40000" => "tree",
                    _ => "blob",
                };

                match name_only {
                    true => println!("{}", name),
                    false => println!("{}\t{}\t{}\t{}", mode, t, sha, name),
                }
            }
        }
        _ => anyhow::bail!("object is not a tree"),
    }

    Ok(())
}
