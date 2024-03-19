use crate::objects::{Kind, Object};
use anyhow::bail;
use anyhow::Context;

pub(crate) fn invoke(
    pretty_print: bool,
    object_hash: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "mode must be given without -p and we don't support mode"
    );

    let mut object =
        Object::read(object_hash).context("parse out object file")?;

    match object.kind {
        Kind::Blob => {
            let size = object.expected_size;

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("write file .git/objects file to stdout")?;

            anyhow::ensure!(
                n == size,
                ".git/object size expected {size} but was {n}"
            );
        }
        _ => bail!("Cannot print type: {}", object.kind),
    }

    Ok(())
}
