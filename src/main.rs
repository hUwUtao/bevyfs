use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use walkdir::WalkDir;

mod datastruct;
use datastruct::*;

mod pakcp;
use pakcp::*;
use zstd::Encoder;

fn main() -> std::io::Result<()> {
    let target_dir = "./assets";
    let walker = WalkDir::new(target_dir);

    let mut metafs = File::options()
        .write(true)
        .create(true)
        .open("target/PAKDIR")?;
    let mut datafs = File::options()
        .write(true)
        .create(true)
        .open("target/PAKDAT")?;

    let mut mtdat = MetadataContainer {
        magic: *b"bevypakfsdir\0\x07\x02\x07",
        paksize: 16,
        table: HashMap::new(),
    };

    let mut dict: Vec<u8> = vec![];
    if let Ok(mut dictfs) = File::open("target/PAKDICT") {
        dictfs.read_to_end(&mut dict)?;
        dictfs.flush()?;
    };
    datafs.write(b"bevypakfsdat\0\x07\x02\x07")?;
    let mut encw = Encoder::with_dictionary(datafs, 21, &dict)?;

    for entry in walker {
        if let Ok(entry) = entry {
            if let Ok(stat) = entry.metadata() {
                let kind_treat = if stat.is_dir() {
                    NodeKind::BRANCH
                } else {
                    NodeKind::LEAF
                };

                let wrst = match kind_treat {
                    NodeKind::LEAF => {
                        let mut cpfs = File::open(entry.path())?;
                        fscow(&mut cpfs, &mut encw)?
                    }
                    NodeKind::BRANCH => (0, 0),
                };

                let pth = format!(
                    "/{}",
                    entry
                        .path()
                        .iter()
                        .skip(2)
                        .collect::<PathBuf>()
                        .to_string_lossy()
                        .replace("\\", "/")
                );
                mtdat.table.insert(
                    pth.clone(),
                    NodeMetadata {
                        name: entry
                            .path()
                            .file_name()
                            .expect("oops can't get file name")
                            .to_string_lossy()
                            .to_string(),
                        kind: kind_treat,
                        hash: wrst.1, // TODO
                        start: mtdat.paksize,
                        end: mtdat.paksize + wrst.0,
                    },
                );

                print!("{:08x} - {}\n", wrst.1, pth);
            }
        }
    }
    if let Ok(mtbuf) = bincode::serialize(&mtdat) {
        metafs.write_all(&mtbuf)?;
    }
    Ok(())
}
