use anyhow::Result;
use async_zip::{tokio::write::ZipFileWriter, Compression};
use std::{env::args, path::PathBuf, process::Command};
use tokio::fs::{create_dir, remove_dir_all, File};
use tokio_util::compat::TokioAsyncReadCompatExt;

#[tokio::main]
async fn main() -> Result<()> {
    let file_paths = args().skip(1).map(PathBuf::from).collect::<Vec<_>>();

    for file_path in file_paths {
        println!("{}", file_path.file_stem().unwrap().to_string_lossy());

        eprint!("  Decompressing... ");
        {
            let out_path = file_path.with_extension("");
            let _ = remove_dir_all(&out_path).await;
            create_dir(&out_path).await?;
            let file = File::open(&file_path).await?;
            libium::modpack::extract_zip(file, &out_path).await?;
        }
        println!("✓");

        eprint!("  Compressing... ");
        let out_path = file_path.with_file_name(
            file_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
                + ".libium.zip",
        );
        {
            let mut zip_writer = ZipFileWriter::new(File::create(&out_path).await?.compat());
            libium::modpack::compress_dir(
                &mut zip_writer,
                &file_path.with_extension(""),
                "",
                Compression::Deflate,
            )
            .await?;
            zip_writer.close().await?;
        }
        println!("✓");

        assert!(Command::new("unzip")
            .arg("-qq")
            .arg("-o")
            .arg(&out_path)
            .arg("-d")
            .arg(&out_path.with_extension(""))
            .spawn()?
            .wait()?
            .success());

        if Command::new("diff")
            .arg("-qr")
            .arg(file_path.with_extension(""))
            .arg(out_path.with_extension(""))
            .spawn()?
            .wait()?
            .success()
        {
            println!("  Files match");
        } else {
            panic!("  File comparison failed!")
        }
    }

    Ok(())
}
