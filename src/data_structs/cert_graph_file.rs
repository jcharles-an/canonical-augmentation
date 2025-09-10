use std::collections::BTreeMap;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;
use memmap2::MmapMut;
use crate::data_structs::cert_graph::CertGraph;
use crate::data_structs::consts::{BTree, GraphInfo};
use crate::data_structs::graph::Graph;

pub struct CertGraphFile{}

impl CertGraphFile {

    pub fn get_list( file_name: &str ) -> Result<MmapMut, std::io::Error >{
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_name)?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(mmap)
    }

    pub fn save_list(list: BTree, file_name: &str,save_generators: bool) -> Result<(MmapMut, usize), std::io::Error> {
        // println!("Saving CertGraphFile to {file_name}");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true).truncate(true) // Create the file if it doesn't exist
            .open(file_name);

        if file.is_err() {
            let e = file.unwrap_err();
            println!["{}",e.to_string()];
            return Err(e);
        }
        let file = file?;

        let mut total_nodes = (list.len() as u128).to_le_bytes().to_vec();
        let mut offset = total_nodes.len(); // this is always 16
        let mut buffer = vec![];
        buffer.extend(total_nodes);

        for (cert, (gen, graph)) in list {
            let node = if save_generators {
                (gen, graph)
            } else {
                (vec![], graph)
            };
            let mut bytes = CertGraph::to_bytes((cert,node));
            let bytes_len = (bytes.len() as u16).to_le_bytes().to_vec();
            buffer.extend(bytes_len);
            buffer.extend(bytes);
        }

        let saved_bytes = buffer.len();
        file.set_len(saved_bytes as u64)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap[0..saved_bytes].copy_from_slice(&buffer);

        mmap.flush()?;
        Ok((mmap, saved_bytes))
    }
}

pub struct MmapCertReader{
    mmap: MmapMut,
    offset: usize,
}

impl MmapCertReader {
    pub fn new(mmap: MmapMut) -> MmapCertReader {
        MmapCertReader{
            mmap,
            offset:0,
        }
    }
}
impl Iterator for MmapCertReader{
    type Item = GraphInfo;
    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset + 16;
        if offset >= self.mmap.len() {
            return None;
        }
        //extract 2-byte cert-graph len
        let cert_graph_len = u16::from_le_bytes([self.mmap[offset], self.mmap[offset+1]]) as usize;

        // extract cert-graph node
        let node = CertGraph::from_bytes(&self.mmap[offset + 2..offset + 2 + cert_graph_len]);

        //advance offset
        self.offset += cert_graph_len + 2;

        Some(node)
    }
}

#[test]
fn test_save_and_read(){
    let mut list = BTreeMap::new();
    list.insert(11788363662582650376u128,(vec![], Graph::from_string("1c00:6400:18400:a800:800:13000:9000:0:0:0:7:19:61:2a:2:4c:24:0:0:0")));
    list.insert(7397971259268010437,(vec![], Graph::from_string("1c00:6400:18400:a800:800:13000:1000:8000:0:0:7:19:61:2a:2:8c:24:0:0:0")));
    let (mmap, _) = CertGraphFile::save_list(list, "test_cert.txt", true).unwrap();

    let iter = MmapCertReader::new(mmap);

    let res: Vec<_> = iter.collect();
    println!("{:?}",res);


}