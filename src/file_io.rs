use crate::*;
//#[path = "file_archive.rs"] mod file_archive;
#[path = "config.rs"] mod config;
use std::fs::File;
use std::io::Write;
use std::str;

//read a file and kill the program if it doesn't exist
pub fn fatal_read_file(file_path:&String) -> String{
    return fs::read_to_string(file_path).unwrap_or_else(|_|fatal!(format!("fatal error: file \"{}\" doesn't exist or isn't readable.", file_path)))
}


pub fn read_index(content: &String) -> Index{
    let mut index = read_config(new_index());
    let mut lines = get_lines(content);
    debug!("got lines");
    //get file names
        {let tmp = process_file_names(index, lines); index = tmp.0; lines = tmp.1};
    debug!("got file names");

    // run get_vars.
        index = get_vars(lines, index);
    debug!("read vars");

    //sanity check
    index.are_vars_sane();
    index.get_vars();
    return index
}

fn process_file_names(mut index:Index, lines:Vec<Vec<char>>) -> (Index, Vec<Vec<char>>){
    let mut other_lines:Vec<Vec<char>> = Vec::new();for line in lines{

        //skip variable declerations
        if is_var(&line){ other_lines.push(line)                               ;continue};
        debug!("got var");

        index.files.push(process_file_path(&line))}return (index, other_lines)}



fn get_vars(lines:Vec<Vec<char>>, mut index:Index) -> Index{
    for line in lines{if is_var(&line){index = process_var(&line, index)}}
    return index}

fn process_var(line:&Vec<char>, mut index: Index) -> Index{
    let mut x = 0;
    let mut var_name = String::new();

    //skip to variable:
    while line[x] != '$'{x+=1}x+=1;

    //get var's name:
    while x< line.len() && ALPHABETICS.contains(line[x]){ var_name.push(line[x]); x+=1 }
    index.vars.push(Variable {name: var_name, value: String::new()});
    return index
}

fn process_file_path(line:&Vec<char>) -> Vec<String>{
    let mut x = 0;
    let mut file:Vec<String> = Vec::new();
    let mut buffer = String::new();
    while x < line.len(){
        if line[x] == '$'{
            if buffer.len() > 0 {file.push(buffer)}
            let mut var = String::from("$");
            x+=1;
            while x < line.len() && ALPHABETICS.contains(line[x]){
                var+=&line[x].to_string();x+=1
            }
            file.push(format!("{}", var));buffer = String::new();
            //debug!(format!("linex: {}, var: {}",line[x], var));
        }
        else {
            buffer.push(line[x]);x+=1
        }
    }
    if buffer.len() > 0 {file.push(buffer)}
    return file;
}

fn is_var(line:&Vec<char>) -> bool{
    let mut first_word = String::new();

    //get first word:
    for i in line.iter(){ if "\t ".contains(*i){break} first_word.push(*i) }

    return match first_word.as_str(){
        "var" => true,
        _ => false
    }
}

fn get_lines(text:&String) -> Vec<Vec<char>>{
    let mut output:Vec<Vec<char>> = Vec::new();
    let text_vec:Vec<char> = text.chars().collect::<Vec<char>>();
    let mut whitespace = 0;
    let mut buffer:Vec<char> = Vec::new();

    for chr in text_vec.iter(){

        //skip initial whitespace of a given line:
        if chr == &' ' && buffer.len() > 0{whitespace+=1}

        //line termination:
        else if chr == &'\n'{whitespace = 0; //push line ONLY if it has content:
                                            if buffer.len() != 0{output.push(buffer); buffer = Vec::new()}}

        //if a non-whitespace char is found, push all previous whitespace to the line, along with the char:
        else {while whitespace != 0{buffer.push(' ');whitespace-=1}buffer.push(chr.to_owned())}
    }

    //push remaining line:
    if buffer.len() != 0{output.push(buffer)}

    return output;
}

pub fn generate_tarball(index:Index, outfile:&String){
    let mut raw_index:Vec<u8> = Vec::new();
    
    let mut archive = file_archive::new_archive(fs::File::create(outfile).unwrap_or_else(|_|{fatal!(format!("Fatal Error: file \"{}\" can't be written to!", outfile))}));
    for i in &index.vars{
        raw_index.append(&mut i.name.clone().into_bytes());
        raw_index.push(0);
    }
    archive.add_file(raw_index);
    let mut x = 0;

    while x < index.files.len(){
        let mut content = file_as_string_vars(&index.files[x]).as_bytes().to_vec();
        content.push(0);
        content.extend(match fs::read(index.file_as_string(x)){
            Ok(val) => val,
            Err(_) => {warn!(format!("warning: file \"{}\" does not exist", index.file_as_string(x)));x+=1;continue}
        });
        archive.add_file(content);
        x+=1;
    }
    archive.closef();
}


fn file_as_string_vars(text:&Vec<String>) -> String{
    let mut out = String::new();
    for x in text{
        out+=&x;
    }
    return out;

}

pub fn read_config(mut index:Index) -> Index{
    let raw_config = match fs::read_to_string(index.config.clone()) {
        Ok(contents) => contents,
        Err(_) => { warn!("Warning: config file is non-existent or unreadable."); return index;}
    };
    let tokens = config::tokenizer(&raw_config);
    for line in tokens{
        let new_var = config::compute_line(line, &index);
        index.add_var(&new_var.0, &new_var.1);
        debug!(format!("vars: {:?}",index.vars));
    }
    return index

}



pub fn read_to_index(data: &mut file_archive::FileArchive) -> Index{
    let mut toret = read_config(new_index());
    let num_files = data.files_number();
    if num_files.len() < 1{fatal!("Fatal error: unable to read file")}
    let vars:Vec<char> = str::from_utf8(&data.get_file(0)).unwrap().chars().collect();
    let mut pos = 0;
    let mut buffer = String::new();
    debug!(format!("filez: {:?}", data));
    while pos < vars.len(){
        if vars[pos] == '\0'{
            let mut x = 0;
            let mut add = true;
            while x < toret.vars.len(){
                if toret.vars[x].name == buffer{
                    add = false;x=toret.vars.len();
                }
                x+=1
            }
            if add{toret.add_var(&buffer, &String::new())}
            buffer.clear();
        }
        else{
            buffer.push(vars[pos]);
        }
        pos+=1;
    }
    if buffer.len() > 0{toret.add_var(&buffer, &String::new())}
    let mut x = 1;
    while x < num_files.len(){
        let mut filename_u8:Vec<u8> = Vec::new();
        let mut pos = 0;
        let curfile:Vec<u8> = data.get_file(x.try_into().unwrap());
        while pos < curfile.len() && curfile[pos] != b'\0'{
            filename_u8.push(curfile[pos]);
            pos+=1;
        }
        debug!(format!("filename_u8: {:?}", filename_u8));
        toret.files.push(process_file_path(&str::from_utf8(&filename_u8).unwrap().chars().collect()));
        toret.file_contents.push(x.try_into().unwrap());
        x+=1;
    }
    if toret.files.len() != toret.file_contents.len(){
        fatal!("\"files\" & \"file_contents\" variables must be the same length");
    }
    debug!(format!("{:?}", toret.vars));
    return toret;
}


pub fn dist_files(index:Index, mut filearch: file_archive::FileArchive){
    if index.files.len() != index.file_contents.len(){
        fatal!("\"files\" & \"file_contents\" variables must be the same length");
    }
    let mut x = 0;
    while x < index.files.len(){
        match fs::create_dir_all(std::path::PathBuf::from(index.file_as_string(x)).parent().unwrap()){
            Ok(_) => {},
            Err(_) => fatal!(format!("Fatal Error: unable to create directory for \"{}\"",index.file_as_string(x)))
            
        };
        let data = filearch.get_file(index.file_contents[x]);
        let mut x1 = 1;
        while x1 < data.len() && data[x1-1] != 0{x1+=1}
        let mut filetow = File::create(index.file_as_string(x)).unwrap_or_else(|_|fatal!(format!("fatal error: unable to write file \"{}\"", index.file_as_string(x))));
        filetow.write_all(&data[x1..data.len()]).unwrap_or_else(|_| fatal!(format!("fatal error: unable to write file \"{}\"", index.file_as_string(x))));
        x+=1
    }

}
