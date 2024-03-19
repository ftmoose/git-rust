use std::fs;
use std::io::SeekFrom;
use clap::{Parser, Subcommand};
use anyhow::Context;
use flate2::Compression;
use std::ffi::CStr;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Sha1, Digest};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf
    }
}

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        },
        Command::CatFile { pretty_print, object_hash } => {
            anyhow::ensure!(
                pretty_print,
                "mode must be given without -p and we don't support mode"
            );

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

            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("Unknown Kind: '{kind}'")
            };

            let size = size
                .parse::<u64>()
                .context(".git/objects file header has invalid size: {size}")?;

            // NOTE: this won't error if the decompressed file is too long,
            // but at least not vulnerable to zip bombs
            let mut z = z.take(size);
            match kind {
                Kind::Blob => {
                    let stdout = std::io::stdout();
                    let mut stdout = stdout.lock();
                    let n = std::io::copy(&mut z, &mut stdout)
                        .context("write file .git/objects file to stdout")?;
                    anyhow::ensure!(n == size, ".git/object size expected {size} but was {n}");
                }
            }
        },
        Command::HashObject { write, file } => {
            let f = fs::File::open(file)
                .context("open file")?;
            let mut f = BufReader::new(f);
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)
                .context("reading file")?;
            let mut hasher = Sha1::new();
            hasher.update(buf.clone());
            let hash = format!("{:x}", hasher.finalize());

            if write {
                fs::create_dir(format!(".git/objects/{}", &hash[..2]))
                    .context("creating hash dir")?;
                let f = fs::File::create(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("create file")?;

                let mut encoder = ZlibEncoder::new(f, Compression::default());
                encoder.write_all(&buf)
                    .context("writing to file")?;

                encoder.finish().context("closing encoder")?;
            }
        }
    }

    Ok(())
}


