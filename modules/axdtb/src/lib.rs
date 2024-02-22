#![no_std]

extern crate alloc;

// #[macro_use]
// extern crate axlog;

use core::slice;
use core::result::Result;
use core::result::Result::{Ok, Err};
use alloc::string::String;
use alloc::{vec::Vec};

use axio::{Read, Error};

type U32Be = u32;

const FDT_MAGIC: u32 = 0xd00d_feed;

#[repr(C)]
pub struct FdtHeader {
    pub magic: U32Be,
    pub totalsize: U32Be,
    pub off_dt_struct: U32Be,
    pub off_dt_strings: U32Be,
    pub off_mem_rsvmap: U32Be,
    pub version: U32Be,
    pub last_comp_version: U32Be,
    pub boot_cpuid_phys: U32Be,
    pub size_dt_strings: U32Be,
    pub size_dt_struct: U32Be,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevTreeError {
    InvalidParameter(&'static str),
    InvalidMagicNumber,
    InvalidOffset,
    ParseError,
    NotEnoughMemory,
}

struct FdtProperty {
    nameoff: U32Be,
    name: String,
    data: Vec<u8>,
}

struct FdtNode {
    name: String,
    properties: Vec<FdtProperty>,
    children: Vec<FdtNode>,
}

struct DtbParser<R: Read> {
    reader: R,
}

impl<R: Read> DtbParser<R> {
    fn new(reader: R) -> DtbParser<R> {
        DtbParser {
            reader: reader,
        }
    }

    fn read_buf(&mut self, size: usize) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        let mut buf = [0; 1];
        for _i in 0..size {
            self.read_exact(&mut buf).unwrap();
            v.push(buf[0]);
        }
        v
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.reader.read_exact(buf)?;
        Ok(())
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        let num: u32 = u32::from_ne_bytes(buf).to_be();
        Ok(num)
    }
}


pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

fn parse_fdt_node<R: Read>(reader: &mut DtbParser<R>) -> Result<FdtNode, Error> {
    let mut properties = Vec::new();
    let mut children = Vec::new();

    let mut n_name: Vec<u8> = Vec::new();
    loop {
        let tmp = reader.read_u32()?;
        n_name.push((tmp >> 24 & 0xff) as u8);
        n_name.push((tmp >> 16 & 0xff) as u8);
        n_name.push((tmp >> 8 & 0xff) as u8);
        n_name.push((tmp & 0xff) as u8);
        if tmp & 0xff == 0 {
            break;
        }
    }
    let node_name = String::from_utf8(n_name).unwrap();

    let mut property_tag = reader.read_u32()?;
    loop {
        if  property_tag != 0x3 {
            break;
        }
        let mut property_len = reader.read_u32()?;
        property_len = (((property_len - 1) >> 2) + 1) << 2;
        let property_nameoff = reader.read_u32()?;
        let property_data = reader.read_buf(property_len as usize);
        properties.push(
            FdtProperty {
                nameoff: property_nameoff,
                name: String::from("none"),
                data: property_data,
            });

        property_tag = reader.read_u32()?;
    }


    let mut tag = property_tag;
    loop {
        if tag == 0x1 {
            let child = parse_fdt_node(reader)?;
            children.push(child);
        } else if tag == 0x2 || tag == 0x9 {
            break;
        } else if tag != 0x4 {
            break;
        }
        tag = reader.read_u32()?;
    }

    Ok(FdtNode {
        name: node_name,
        properties,
        children,
    })
}

fn get_fdt_string(byte_slice: &[u8], pos: usize) -> Result<String, DevTreeError> {
    let mut parser = DtbParser::new(&byte_slice[pos..]);

    let mut v: Vec<u8> = Vec::new();
    let mut buf = [0; 1];
    loop {
        parser.read_exact(&mut buf).unwrap();
        if buf[0] == 0 {
            return Ok(String::from_utf8_lossy(&v).into_owned());
        }
        v.push(buf[0]);
    }
}

fn fill_property_name(node: &mut FdtNode, byte_slice: &[u8], off_dt_strings: usize) {
    for pro in &mut node.properties {
        let pos = off_dt_strings + pro.nameoff as usize;
        pro.name = get_fdt_string(byte_slice, pos).unwrap();
        // info!("        > {}:{:#?}", pro.name, pro.data);
    }

    for mut child in &mut node.children {
        // info!(">>> node: {}", child.name);
        fill_property_name(&mut child, byte_slice, off_dt_strings);
    }
}

fn get_property_reg(node: & FdtNode, name: & str, mut v: &mut Vec<(usize, usize)>) {
    if node.name.starts_with(&name) {
        for pro in &node.properties {
            if pro.name.starts_with("reg") {
                // info!("        > {}:{:#?}", pro.name, pro.data);
                let buf: [u8; 8] = pro.data[..8].try_into().unwrap();
                let num1 = u64::from_ne_bytes(buf).to_be() as usize;
                let buf: [u8; 8] = pro.data[8..16].try_into().unwrap();
                let num2 = u64::from_ne_bytes(buf).to_be() as usize;
                v.push((num1, num2));
                break;
            }
        }
    }

    for child in &node.children {
        get_property_reg(&child, &name, &mut v);
    }
}


pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo, DevTreeError> {
    let addr: *const FdtHeader = dtb_pa as *const FdtHeader;
    let fdt_hdr: &FdtHeader = unsafe { &*addr };

    if fdt_hdr.magic.to_be() != FDT_MAGIC {
        return Err(DevTreeError::InvalidMagicNumber);
    }

    let address: *const u8 = dtb_pa as *const u8;
    let byte_slice: &[u8] = unsafe {
        slice::from_raw_parts(address, fdt_hdr.totalsize.to_be() as usize)
    };

    let off_dt_struct = fdt_hdr.off_dt_struct.to_be() as usize;
    let mut parser = DtbParser::new(&byte_slice[off_dt_struct..]);

    let tag = parser.read_u32().unwrap();
    if tag != 0x1 {
        return Err(DevTreeError::ParseError);
    }

    let mut root_node = parse_fdt_node(&mut parser).unwrap();

    let off_dt_strings = fdt_hdr.off_dt_strings.to_be() as usize;

    fill_property_name(&mut root_node, byte_slice, off_dt_strings);

    let mut v_mem = Vec::new();
    get_property_reg(&root_node, &"memory", &mut v_mem);

    let mut v_virtio = Vec::new();
    get_property_reg(&root_node, &"virtio_mmio", &mut v_virtio);

    Ok(DtbInfo{
        memory_addr: v_mem[0].0,
        memory_size: v_mem[0].1,
        mmio_regions: v_virtio,
    })

}
