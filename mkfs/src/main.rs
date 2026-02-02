use kernelapi::fs;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: mkfs <fs.img> <UserManifest.toml>");
    }

    let toml_str = std::fs::read_to_string(&args[2])?;
    let progs: BTreeMap<String, String> = toml::from_str(&toml_str)?;
    let bsize = fs::BSIZE as u64;

    let mut fs_img = File::create(&args[1])?;
    let mut prog_blocks: Vec<fs::ProgBlock> = vec![];
    let mut current_block = 2u64;
    for (name, path) in progs {
        println!(
            "Placing {} (from {}) at block {}",
            name, path, current_block
        );
        let mut src = File::open(path)?;
        fs_img.seek(SeekFrom::Start(current_block * bsize))?;
        let bytes_written = io::copy(&mut src, &mut fs_img)?;

        let blocks_used = (bytes_written + bsize - 1) / bsize;
        prog_blocks.push(fs::ProgBlock {
            nblocks: blocks_used,
            start_block: current_block,
        });
        current_block += blocks_used;
    }

    let sp_block = fs::SuperBlock::new(current_block);
    fs_img.seek(SeekFrom::Start(bsize))?;
    fs_img.write_all(sp_block.as_u8_slice())?;
    prog_blocks
        .iter()
        .try_for_each(|prog| fs_img.write_all(prog.as_u8_slice()))?;
    fs_img.set_len(current_block * bsize)?;

    Ok(())
}
