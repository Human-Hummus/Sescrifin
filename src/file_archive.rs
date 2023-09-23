use crate::*;
use std::io::{Seek, SeekFrom, Write, Read};
use std::fs::File;

#[derive(Debug)]
pub struct FileArchive{
    pub files: Vec<FileDes>,
    pub curfile: File, 
    curpos: usize, //unused in read mode
    // 'r' = read, 'w' = write;
    mode: char
}

#[derive(Debug)]
pub struct FileDes{
    start_pos: usize,
    end_pos: usize
}

pub fn new_archive(mut file: File) -> FileArchive{
    file.seek(SeekFrom::Start(0)).unwrap();
    file.write(b"SESCRIFIN").unwrap();
    file.seek(SeekFrom::Start(SESCRIFIN_LEN.into())).unwrap();
    return FileArchive {files: Vec::new(), curfile: file, curpos:"sescrifin".len(), mode:'w'};
}

pub fn open_archive(file: File) -> FileArchive {
    return match open_archive_sub(file){
        Ok(val) => val,
        Err(_) => fatal!("unable to read file/file is invalid")
    }
}

pub fn open_archive_sub(mut file: File) -> Result<FileArchive, std::io::Error>{
    file.seek(SeekFrom::Start(0))?;
    let mut first_9_bytes = vec![0;9];
    file.read_exact(&mut first_9_bytes[0..9])?;
    if first_9_bytes != b"SESCRIFIN"{fatal!("unable to read file");}
    file.seek(SeekFrom::End(-4))?;
    let mut numfiles_vec: Vec<u8> = vec![0;4];
    file.read_exact(&mut numfiles_vec[0..4])?;
    let numfiles:usize = u32::from_be_bytes([numfiles_vec[0], numfiles_vec[1], numfiles_vec[2], numfiles_vec[3]]).try_into().unwrap();
    let mut file_pos_data:Vec<u8> = vec![0;(numfiles*16).try_into().unwrap()];
    file.seek(SeekFrom::End(-i64::try_from(numfiles*16+4).unwrap()))?;
    file.read_exact(&mut file_pos_data[0..numfiles*16])?;

    let mut files:Vec<FileDes> = Vec::new();
    let mut files_to_go = numfiles;
    while files_to_go > 0{
        let pos = numfiles-files_to_go;
        files.push(FileDes{
            start_pos: u64::from_be_bytes(file_pos_data[pos*16..pos*16+8].try_into().unwrap()) as usize,
            end_pos: u64::from_be_bytes(file_pos_data[pos*16+8..pos*16+16].try_into().unwrap()) as usize
        });
        files_to_go-=1;
    }
    return Ok(FileArchive{
        files: files,
        curfile: file,
        curpos: 0, //unused
        mode: 'r'
    })
    
}


//length of the SESCRIFIN name. I know this is unneeded, but I think it helps clarify the function
//of the code.
const SESCRIFIN_LEN:u32 = 9;      //u32::try_from("sescrifin".len()).unwrap();

impl FileArchive{

    //add the data of a file
    pub fn add_file(&mut self, file_content:Vec<u8>){
        if self.mode != 'w' {fatal!("fatal error: tried to add a file while in read mode")}
        self.files.push(FileDes {start_pos: self.curpos, end_pos: self.curpos+file_content.len()});
        self.curfile.seek(SeekFrom::Start(self.curpos.try_into().unwrap())).unwrap();
        self.curpos+=file_content.len();
        self.curfile.write(&file_content.clone()).unwrap();
    }

    //write archive to file
    pub fn closef(&mut self){
        let mut x = 0;
        while x < self.files.len(){
            self.curfile.seek(SeekFrom::Start(self.curpos.try_into().unwrap())).unwrap();
            let mut to_write = u64_to_u8(self.files[x].start_pos.try_into().unwrap());
            to_write.append(&mut u64_to_u8(self.files[x].end_pos.try_into().unwrap()));
            self.curpos+=to_write.len();
            self.curfile.write(&to_write).unwrap();
            x+=1;
        }
        self.curfile.seek(SeekFrom::Start(self.curpos.try_into().unwrap())).unwrap();
        self.curfile.write(&u32_to_u8(self.files.len().try_into().unwrap())).unwrap();
        self.mode = 'w';
    }

    //how many files are there
    pub fn files_number(&self) -> Vec<u32>{
        let mut output:Vec<u32> = Vec::new();
        let mut x:u32 = 0;
        while x < self.files.len().try_into().unwrap(){
            output.push(x);
            x+=1;
        }
        return output;
    }

    //get file of number $number
    pub fn get_file(&mut self, file_num:u32) -> Vec<u8>{
        if self.mode != 'r' {fatal!("Tried to read a file while in write mode")}
        let file_len = self.files[usize::try_from(file_num).unwrap()].end_pos - self.files[usize::try_from(file_num).unwrap()].start_pos;
        let mut toret:Vec<u8> = vec![0;file_len.try_into().unwrap()]; //the buffer to read to
        self.curfile.seek(SeekFrom::Start(self.files[usize::try_from(file_num).unwrap()].start_pos.try_into().unwrap())).unwrap();
        self.curfile.read_exact(&mut toret[0..usize::try_from(file_len).unwrap()]).unwrap();
        return toret;
    }

}


fn u32_to_u8(input: u32) -> Vec<u8>{
    let mut out:Vec<u8> = Vec::with_capacity(4);

    out.push(((input >> 8*3) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*2) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*1) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*0) & 0b11111111).try_into().unwrap());

    return out;

}
fn u64_to_u8(input: u64) -> Vec<u8>{
    let mut out:Vec<u8> = Vec::with_capacity(8);

    out.push(((input >> 8*7) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*6) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*5) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*4) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*3) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*2) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*1) & 0b11111111).try_into().unwrap());
    out.push(((input >> 8*0) & 0b11111111).try_into().unwrap());

    return out;
}


// structure of file
// header: SESCRIFIN (ascii)
//
// {list of all files}
//
// repeat for every file{
//      file_start(in archive): 64 bits
//      file_end(in archive): 64 bits
// }
// len_of_index(in number of files; IE length in bits is this*128): 32 bits
