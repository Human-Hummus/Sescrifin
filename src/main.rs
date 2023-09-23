use std::env;
mod file_io;
extern crate termion;
#[macro_use]
pub mod output;
use std::fs;
use std::path::Path;
mod file_archive;


const ALPHABETICS:&str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const IS_NOFILE_FATAL:bool = true;
const DEFAULT_CONFIG:&str = "/home/user/.config/sescrifin.conf";



fn main() {
    let mut input_file = String::new();
    let mut output_file = String::new();

    let mut operation = Operation::Nil;

    let args: Vec<String> = env::args().collect();
    let mut x = 1;
    while x < args.len(){
        match args[x].as_str(){
            "--help" | "-h" => {
                alert!(include_str!("help.txt"));
                std::process::exit(0);
            }

            "-i" | "--input" => {
                x+=1;
                if x == args.len(){fatal!("Fatal error: incomplete input argument")} //check if there is a following argument
                input_file = args[x].clone();
            }

            "-o" | "--output" => {
                x+=1;
                if x == args.len(){fatal!("Fatal error: incomplete output argument")} //check if there is a following argument
                output_file = args[x].clone();
            }

            "--generate" | "-g" => operation = Operation::Write,

            "--read" | "-r" | "-u" | "--unpack" => operation = Operation::Read,
            
            _ => {fatal!(format!("Fatal error: unknown argument \"{}\"", args[x]))}
        };x+=1
    }
    if input_file == String::new(){fatal!("fatal error: no input file was specified.")}

    if operation == Operation::Nil{
        error!("Fatal error: undefined operation.");
        error!("Please specify whether you'd like to generate a file, or read a file.");
        error!("If you want to read a file add the flag \"--read\" (or -r).");
        fatal!("If you want to generate a file, add the flag \"--generate\" (or -g).");
    }

    if operation == Operation::Write{
        if output_file == String::new(){fatal!("fatal error: no output file was specified.")}

        let raw_index = file_io::fatal_read_file(&input_file);
        let mut index = file_io::read_index(&raw_index);
        index.full_check(false);
        index = index.expand_dirs().remove_unused_vars();
        file_io::generate_tarball(index, &output_file);

        std::process::exit(0);
    }

    if operation == Operation::Read{
        if output_file != String::new(){warn!("Warning: when reading a file, no output file needs to be specified, as the files to write to are already defined.")}

        let mut data = file_archive::open_archive(fs::File::open(input_file).expect("input file doesn't exist"));
        let mut todist = file_io::read_to_index(&mut data);
        todist.get_vars();todist.full_check(true);

        file_io::dist_files(todist, data);
        
        std::process::exit(0);
    }
    fatal!("If you're reading this, something's gone VERY wrong")
}



#[derive(Debug)]
pub struct Index{
    vars: Vec<Variable>,

    //a list of file paths to be copied, with variables integrated. A file path would look like:
    //<"text", "$var1", "$var2", "more text"> this can be converted to a string with the
    //file_as_string(n) function.
    files: Vec<Vec<String>>,

    //used only in read mode. This list should be as long as the first one, and the contents of the
    //file in this vec match with the name of the file in the files at the same position
    //
    //this is a file # which will be passed to whatever it is file_archive.rs
    file_contents:Vec<u32>, 

    //the config file's path
    config: String
}
impl Index{
    fn file_as_string(&self, file_number:usize) -> String{
        let file_vec = self.files[file_number].clone();
        let mut path = String::new();

        let mut x = 0;
        while x < file_vec.len(){
            if file_vec[x].chars().nth(0).unwrap() == '$'{
                let mut var_content = match self.get_var_dollar(&file_vec[x]){
                    Ok(var) => var,
                    Err(_er) => panic!("variable was not found")
                };
                if x+1 == file_vec.len(){
                    path += &var_content;
                }
                else{
                    if var_content.chars().nth(var_content.len()-1).unwrap() == '/' && file_vec[x+1].chars().nth(0).unwrap() == '/'{
                        var_content.pop();
                        path+=&var_content;
                    }
                    else{
                        path+=&var_content;
                    }
                }
            }
            else{
                path += &file_vec[x];
            }
            x+=1;
        }
        return path;
    }
    
    fn get_var(&self, var_name:&String) -> Result<String, String>{
        let mut x = 0;
        while x < self.vars.len(){
            if &self.vars[x].name == var_name{
                return Ok(self.vars[x].value.clone());
            }
            x+=1;
        }
        return Err(String::from(format!("Error: Var not found: \"{}\"", var_name)))
    }

    fn get_var_dollar(&self, var_name:&String) -> Result<String, String>{return self.get_var(&var_name[1..var_name.len()].to_string())}

    fn are_vars_sane(&self) -> bool{
        for file in (&self.files).iter(){
            for part in file{
                if part.chars().nth(0).unwrap() == '$'{
                    match self.get_var_dollar(part){
                        Ok(_) => continue,
                        Err(_er) => {warn!(format!("var \"{}\" doesn't exist.",part));return false}
                    }
                }
            }
        }
        return true;
    }
    fn get_vars(&mut self){
        let mut x = 0;
        while x < self.vars.len(){
            if self.vars[x].value.len() == 0{
                println!("Please enter the value for \"{}\":", self.vars[x].name);
                std::io::stdin().read_line(&mut self.vars[x].value).expect("illegal response");
                while self.vars[x].value.len() > 0 && "\t\n ".contains(self.vars[x].value.chars().nth(self.vars[x].value.len()-1).unwrap()){self.vars[x].value.pop();}
                if self.vars[x].value.len() == 0{fatal!(format!("fatal error: var \"{}\" has no value", self.vars[x].name))}
                alert!(format!("setting \"{}\" as the value for the variable \"{}\".",self.vars[x].value,self.vars[x].name));
            }
            x+=1;
        }
    }
    fn check_files_exist(&self) -> bool{
        let mut x = 0;
        while x < self.files.len(){
            let filepath = self.file_as_string(x);
            match Path::new(&filepath).exists(){
                true => {x+=1;continue},
                false => {match IS_NOFILE_FATAL{
                            true => fatal!(format!("Fatal error: file \"{}\" does not exist. If you want to make this error non-fatal, change \"IS_NOFILE_FATAL\" in \"src/main.rs\" to \"false\". This may be caused by not adding a / to the end of a directory include, ensure directorys were terminated with a /.", filepath)),
                            false => error!(format!("Non-fatal error: file \"{}\" does not exist. If you want this error to be fatal, change \"IS_NOFILE_FATAL\" in \"src/main.rs\" to \"true\". This may be caused by not adding a / to the end of a directory include, ensure directorys were terminated with a /.", filepath))
                        }x+=1;continue}
            }
        }
        return true;
    }
    fn full_check(&self, is_read:bool){
        alert!("running full sanity check...");
        alert!("testing if undefined vars exist...");
        match self.are_vars_sane(){
            true => alert!("OK..."),
            false => fatal!("Failed...")
        }
        alert!("testing if all files exist...");
        if !is_read{match self.check_files_exist(){
            true => alert!("OK..."),
            false => error!("Failed...")
        }}
        alert!("sanity check concluded; continuing");
    }
    //REQUIRED
    fn expand_dirs(mut self) -> Index{
        let mut new_files:Vec<Vec<String>> = Vec::new();
        let mut x = 0;
        while x < self.files.len(){
            let sfas = self.file_as_string(x).clone();
            let path = Path::new(&sfas);
            if path.is_dir() == false{new_files.push(self.files[x].clone());x+=1;continue}
            for i in get_sub_dirs(&self.file_as_string(x)){
                //this is needed to preserve variables
                let mut toadd = self.files[x].clone();
                toadd.push(i);
                new_files.push(toadd);
            }
            x+=1;

        }
        self.files = new_files;
        debug!(format!("new files: {:?}", self.files));
        return self;

    }

    fn add_var(&mut self, var_name: &String, var_content:&String){
        self.vars.push(Variable {name: var_name.clone(), value: var_content.clone()});
    }
    fn remove_unused_vars(mut self) -> Index{
        let mut used_vars:Vec<String> = Vec::new();
        for file in &self.files{
            for item in file{
                if item.chars().nth(0).unwrap() == '$'{
                    used_vars.push(item.to_string());
                }
            }
        }
        let mut new_vars:Vec<Variable> = Vec::new();
        'allv: for var in self.vars{
            for used_var in &used_vars{
                if used_var == &format!("${}",var.name){
                    new_vars.push(var);
                    continue 'allv;
                }
            }
        }
        self.vars = new_vars;
        return self;

    }
}

pub fn new_index() -> Index{
    Index {vars: Vec::new(), files: Vec::new(), file_contents: Vec::new(), config: DEFAULT_CONFIG.to_string()}
}


fn get_sub_dirs(dir:&String) -> Vec<String>{
    let mut files:Vec<String> = Vec::new();
    if !Path::new(dir).is_dir(){return Vec::new()}
    for i in match fs::read_dir(dir){
        Ok(de) => de,
        Err(_) => fatal!("unknown error occured")
    }{
        if i.as_ref().unwrap().path().is_dir(){
            let new_i = ensure_ends_in_char(i.as_ref().expect("File not found").file_name().into_string().expect("failed to convert os string to string"), '/');
            for f in get_sub_dirs(&i.as_ref().unwrap().path().display().to_string()){
                files.push(format!("{}{}",new_i, f));
            }
        }
        else{

            let iu = i.unwrap().file_name().into_string().expect("failed to convert os string to string");
            files.push(iu.clone());
            debug!(iu)
        }
    }
    debug!(format!("files: {:?}", files));
    return files;
}


//ensure string terminates with given char
fn ensure_ends_in_char(s:String, c:char) -> String{if s.chars().nth(s.len()-1).unwrap()==c{return s}return format!("{}{}",s,c)}
#[derive(PartialEq)]
enum Operation {Nil, Read, Write}

#[derive(Debug)]
struct Variable {
    value: String,
    name: String
}
