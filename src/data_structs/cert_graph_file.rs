use std::collections::BTreeMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use memmap2::{Mmap, MmapMut};
use crate::data_structs::bitmaps::BM;
use crate::data_structs::cert_graph::CertGraph;
use crate::data_structs::consts::{BTree, GraphInfo};
use crate::data_structs::graph::Graph;
use crate::data_structs::iterator_merger::IteratorMerger;
use crate::data_structs::nauty_graph::NautyGraph;

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
        let mut buffer = vec![];


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

        buffer.extend(total_nodes);

        let saved_bytes = buffer.len();
        file.set_len(saved_bytes as u64)?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap[0..saved_bytes].copy_from_slice(&buffer);

        mmap.flush()?;
        Ok((mmap, saved_bytes))
    }

    pub fn save_and_merge_iter(iterators: Vec<MmapCertReader>, filename: &str, save_generators: bool ) -> Result<(MmapMut, usize), std::io::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true).truncate(true) // Create the file if it doesn't exist
            .open(filename)?;
        let iterator_merger = IteratorMerger::new(iterators);
        let mut count = 0;
        iterator_merger.for_each(|gi|{
            count += 1;
            let node = if save_generators {
                (gi.gen, gi.graph)
            } else {
                (vec![], gi.graph)
            };
            let mut bytes = CertGraph::to_bytes((gi.cert,node));
            let bytes_len = (bytes.len() as u16).to_le_bytes().to_vec();
            file.write_all(bytes_len.as_slice()).unwrap();
            file.write_all(bytes.as_slice()).unwrap();
        });
        let bytes_len = (count as u128).to_le_bytes().to_vec();
        file.write_all(bytes_len.as_slice())?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        Ok((mmap, count))
    }

    pub fn get_initial_mmap(n: usize) -> MmapMut {
        let ady_mat = vec![BM::new();n];
        let g = Graph {
            ady_matrix: ady_mat,
        };
        let (gen, certificate)= NautyGraph::get_aut_generators_and_cert(&g).unwrap();

        let node = (vec![], g);
        let bytes = CertGraph::to_bytes((certificate,node));
        let mut bytes_len = (bytes.len() as u16).to_le_bytes().to_vec();
        let number_bytes = (1u128).to_le_bytes().to_vec();
        let mut res =unsafe { MmapMut::map_anon(bytes.len() + 18).unwrap()};

        bytes_len.extend_from_slice(bytes.as_slice());
        bytes_len.extend_from_slice(number_bytes.as_slice());
        res.copy_from_slice(bytes_len.as_slice());

        res
    }

    pub fn get_empty_mmap(n: usize) -> MmapMut {
        let number_bytes = (0u128).to_le_bytes().to_vec();
        let mut res =unsafe { MmapMut::map_anon(16).unwrap()};
        res.copy_from_slice(number_bytes.as_slice());
        res
    }



    pub fn get_graphs_number(list: &MmapMut) -> usize {
        let len = list.len();
        let bytes = &list[len-16.. len];
        u128::from_le_bytes(list[len-16.. len].try_into().unwrap()) as usize
    }



    pub fn get_chuck_indexes(list: &MmapMut, chunk_size: usize ) -> Vec<usize> {
        let len = list.len();

        //extract 16-byte number of graphs in file
        let graph_number = Self::get_graphs_number(&list);

        let mut offset = 0;
        let mut indexes = vec![0];
        let mut count = 0;

        while offset < len - 16 {
            //extract 2-byte cert-graph len
            let number_bytes = [list[offset], list[offset+1]];
            let cert_graph_len = u16::from_le_bytes([list[offset], list[offset+1]]) as usize;

            //advance offset
            offset += cert_graph_len + 2;
            count += 1;
            if count == chunk_size {
                indexes.push(offset);
                count = 0;
            }
        }
        indexes.push(len - 16);
        indexes
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
        let offset = self.offset;
        if offset >= self.mmap.len() - 16 { // last 16 bytes are the graph number
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

pub struct SliceReader<'a>{
    bytes: &'a [u8],
    offset: usize,
}
impl SliceReader<'_> {
    pub fn new(bytes: &'_ [u8]) -> SliceReader<'_> {
        SliceReader{
            bytes,
            offset:0
        }
    }
}
impl Iterator for SliceReader<'_>{
    type Item = GraphInfo;
    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        if offset >= self.bytes.len() {
            return None;
        }
        //extract 2-byte cert-graph len
        let cert_graph_len = u16::from_le_bytes([self.bytes[offset], self.bytes[offset+1]]) as usize;

        // extract cert-graph node
        let node = CertGraph::from_bytes(&self.bytes[offset + 2..offset + 2 + cert_graph_len]);

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
#[test]
fn test_get_indexes(){
    let level_graphs_result = CertGraphFile::get_list("results/9_3/g_l_00_c_20.bin").unwrap();
    let indexes = CertGraphFile::get_chuck_indexes(&level_graphs_result, 20);

    println!("{:?}",indexes);

}