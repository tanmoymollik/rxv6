use crate::print::println;

pub struct ElfHeader {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

pub struct ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

pub fn parse_elf_header(data: &[u8]) -> Option<ElfHeader> {
    if data.len() < 64 || !data.starts_with(b"\x7fELF") {
        return None;
    }
    let elf_header = ElfHeader {
        e_ident: data[0..16].try_into().unwrap(),
        e_type: u16::from_le_bytes(data[16..18].try_into().unwrap()),
        e_machine: u16::from_le_bytes(data[18..20].try_into().unwrap()),
        e_version: u32::from_le_bytes(data[20..24].try_into().unwrap()),
        e_entry: u64::from_le_bytes(data[24..32].try_into().unwrap()),
        e_phoff: u64::from_le_bytes(data[32..40].try_into().unwrap()),
        e_shoff: u64::from_le_bytes(data[40..48].try_into().unwrap()),
        e_flags: u32::from_le_bytes(data[48..52].try_into().unwrap()),
        e_ehsize: u16::from_le_bytes(data[52..54].try_into().unwrap()),
        e_phentsize: u16::from_le_bytes(data[54..56].try_into().unwrap()),
        e_phnum: u16::from_le_bytes(data[56..58].try_into().unwrap()),
        e_shentsize: u16::from_le_bytes(data[58..60].try_into().unwrap()),
        e_shnum: u16::from_le_bytes(data[60..62].try_into().unwrap()),
        e_shstrndx: u16::from_le_bytes(data[62..64].try_into().unwrap()),
    };
    Some(elf_header)
}

pub fn parse_program_headers<const PHNUM: usize>(
    data: &[u8],
    phoff: usize,
    phentsize: usize,
) -> Option<[ProgramHeader; PHNUM]> {
    if data.len() < phoff + phentsize * PHNUM {
        return None;
    }

    let mut program_headers: [ProgramHeader; PHNUM] = core::array::from_fn(|i| {
        let offset = phoff + i * phentsize;
        parse_program_header(data, offset)
    });
    Some(program_headers)
}

pub fn print_elf_header(header: &ElfHeader) {
    println!("ELF Header:");
    println!("  Type: 0x{:x}", header.e_type);
    println!("  Machine: 0x{:x}", header.e_machine);
    println!("  Version: {}", header.e_version);
    println!("  Entry point address: 0x{:X}", header.e_entry);
    println!("  Program header offset: 0x{:x}", header.e_phoff);
    println!("  Section header offset: 0x{:x}", header.e_shoff);
    println!("  Flags: {}", header.e_flags);
    println!("  ELF header size: 0x{:x}", header.e_ehsize);
    println!("  Program header entry size: 0x{:x}", header.e_phentsize);
    println!("  Number of program headers: {}", header.e_phnum);
    println!("  Section header entry size: 0x{:x}", header.e_shentsize);
    println!("  Number of section headers: {}", header.e_shnum);
    println!("  Section header string table index: {}", header.e_shstrndx);
}

pub fn print_program_headers(program_headers: &[ProgramHeader]) {
    println!("Program Headers:");
    for (i, ph) in program_headers.iter().enumerate() {
        println!("  Program Header {}:", i);
        println!("    Type: 0x{:x}", ph.p_type);
        println!("    Flags: 0x{:x}", ph.p_flags);
        println!("    Offset: 0x{:x}", ph.p_offset);
        println!("    Virtual Address: 0x{:X}", ph.p_vaddr);
        println!("    Physical Address: 0x{:X}", ph.p_paddr);
        println!("    File Size: 0x{:x}", ph.p_filesz);
        println!("    Memory Size: 0x{:x}", ph.p_memsz);
        println!("    Align: 0x{:x}", ph.p_align);
    }
}

fn parse_program_header(data: &[u8], offset: usize) -> ProgramHeader {
    ProgramHeader {
        p_type: u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()),
        p_flags: u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()),
        p_offset: u64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap()),
        p_vaddr: u64::from_le_bytes(data[offset + 16..offset + 24].try_into().unwrap()),
        p_paddr: u64::from_le_bytes(data[offset + 24..offset + 32].try_into().unwrap()),
        p_filesz: u64::from_le_bytes(data[offset + 32..offset + 40].try_into().unwrap()),
        p_memsz: u64::from_le_bytes(data[offset + 40..offset + 48].try_into().unwrap()),
        p_align: u64::from_le_bytes(data[offset + 48..offset + 56].try_into().unwrap()),
    }
}
