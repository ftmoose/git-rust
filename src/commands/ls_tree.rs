use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::BufReader;

pub(crate) fn invoke(name_only: bool, object_hash: &str) -> anyhow::Result<()> {
    // TODO: support shortest unique object hashes
    let f = std::fs::File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..]
    ))
    .context("open in .git/objects")?;

    let z = ZlibDecoder::new(f);
    let mut z = BufReader::new(z);
    let mut buf = Vec::new();
    z.read_until(0, &mut buf)
        .context("read header from .git/objects")?;
    let header = CStr::from_bytes_with_nul(&buf)
        .expect("know there is exactly one nul, and it's at the end");
    let header = header
        .to_str()
        .context(".git/objects file header isn't valid UTF-8")?;
    let Some((kind, size)) = header.split_once(' ') else {
        anyhow::bail!(".git/objects file header did not start with a known type: '{header}'")
    };

    if kind != "tree" {
        anyhow::bail!("object is not a tree");
    }

    let mut size = size
        .parse::<usize>()
        .context(".git/objects file header has invalid size: {size}")?;

    while size > 0 {
        let mut mode = Vec::new();
        let n = z.read_until(b' ', &mut mode).context("reading mode")?;
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
            false => println!("{}\t{}\t{}\t{}", mode, t, sha, name)
        }

        
    }

    Ok(())
}
