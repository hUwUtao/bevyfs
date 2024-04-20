use {
    bevy::{
        asset::{
            io::{AssetReader, AssetReaderError, AssetSource, AssetSourceId, PathStream, Reader},
            AsyncReadExt,
        },
        prelude::*,
        utils::BoxedFuture,
    },
    std::{
        collections::HashMap,
        fs::File,
        io::{BufReader, Read, Write},
        path::{Path, PathBuf},
        str::FromStr,
    },
    walkdir::WalkDir,
    zstd::{Decoder, Encoder},
};

pub mod datastruct;
use datastruct::*;
pub mod pakcp;
use pakcp::*;
pub mod err;
use err::*;

struct PakReader(
    Box<dyn AssetReader>,
    (
        MetadataContainer,
        CringeReader<Decoder<'static, BufReader<File>>>,
        Vec<u8>,
    ),
);

impl PakReader {
    fn bundle<'a>() -> Result<
        (
            MetadataContainer,
            CringeReader<Decoder<'a, BufReader<File>>>,
            Vec<u8>,
        ),
        PakReadErr,
    > {
        let mut dict: Vec<u8> = vec![];
        File::options()
            .read(true)
            .write(false)
            .open("target/PAKDICT")?
            .read_to_end(&mut dict)?;

        Ok((
            bincode::deserialize_from::<File, MetadataContainer>(
                File::options()
                    .read(true)
                    .write(false)
                    .open("target/PAKDIR")?,
            )?,
            CringeReader::new(Decoder::new(
                File::options()
                    .read(true)
                    .write(false)
                    .open("target/PAKDAT")?,
            )?),
            dict,
        ))
    }

    fn new(r: Box<dyn AssetReader>) -> PakReader {
        PakReader(
            r,
            // cool
            Self::bundle().unwrap(),
        )
    }
}

impl AssetReader for PakReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        self.0.read(path)
    }
    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        self.0.read_meta(path)
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        self.0.read_directory(path)
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        self.0.is_directory(path)
    }

    fn read_meta_bytes<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Vec<u8>, AssetReaderError>> {
        Box::pin(async move {
            let mut meta_reader = self.read_meta(path).await?;
            let mut meta_bytes = Vec::new();
            meta_reader.read_to_end(&mut meta_bytes).await?;
            Ok(meta_bytes)
        })
    }
}

/// A plugins that registers our new asset reader
struct CustomAssetReaderPlugin();

impl Plugin for CustomAssetReaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSource::build().with_reader(|| {
                Box::new(PakReader::new(AssetSource::get_default_reader(
                    "assets".to_string(),
                )()))
            }),
        );
    }
}

pub fn build() -> Result<(), PakReadErr> {
    let target_dir = "./assets";
    let walker = WalkDir::new(target_dir);

    let metafs = File::options()
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

                let pth = PathBuf::from_str(&format!(
                    "/{}",
                    &entry
                        .path()
                        .iter()
                        .skip(2)
                        .collect::<PathBuf>()
                        .to_string_lossy()
                        .replace("\\", "/")
                ))
                // ?
                .unwrap();
                print!("{:08x} - {}\n", wrst.1, &pth.to_str().unwrap_or_default());
                mtdat.table.insert(
                    pth,
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
            }
        }
    }
    bincode::serialize_into(metafs, &mtdat)?;
    Ok(())
}
