use crate::*;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum TokenType{
Str,
Variable,
CurlyBracket,
Parenthesis,
Equal,
Other,
EnvVar
}

//each Vec<> is a line.
pub fn tokenizer(text:&String) -> Vec<Vec<(TokenType, String)>>{
    //               type  content
    let mut out:Vec<Vec<(TokenType,   String)>> = Vec::new();

    let mut x = 0;
    let text_as_chars = text.chars().collect::<Vec<char>>();

    let mut cur_line:Vec<(TokenType, String)> = Vec::new();
    while x < text_as_chars.len(){
        match text_as_chars[x]{
            '\n' => {
                x+=1;
                if cur_line.len() > 0{
                    out.push(cur_line)
                }
                cur_line = Vec::new()
            },
            '"' => {
                let mut string_content = String::new();
                x+=1;
                while x < text_as_chars.len(){
                    if text_as_chars[x] == '\\'{
                        x+=1;
                        if x == text_as_chars.len(){fatal!("fatal error: unterminated escape charecter (backslash) in configuration")}
                        if text_as_chars[x] == '"'{
                            string_content.push('"');
                        }
                        else if text_as_chars[x] == '\\'{
                            string_content.push('\\');
                        }
                        else{
                            fatal!(format!("fatal error: unknown escape charecter \"{}\" in configuration file", text_as_chars[x]))
                        }
                    }
                    else if text_as_chars[x] == '"'{
                        x+=1;
                        break;
                    }
                    else{
                        string_content.push(text_as_chars[x]);
                        x+=1;
                    }
                }
                cur_line.push((TokenType::Str, string_content));
            },
            '$' => {
                let mut vname = String::new();
                x+=1;
                while x < text_as_chars.len() && ALPHABETICS.contains(text_as_chars[x]){
                    vname.push(text_as_chars[x]);
                    x+=1;
                }
                cur_line.push((TokenType::Variable, vname));
            },
            '(' | ')' => {
                cur_line.push((TokenType::Parenthesis,text_as_chars[x].to_string()));
                x+=1;
            },
            '=' => {
                x+=1;
                cur_line.push((TokenType::Equal,String::from("=")));
            },
            '{' | '}' => {
                cur_line.push((TokenType::CurlyBracket, text_as_chars[x].to_string()));
                x+=1;
            },
            '#' =>{
                x+=1;
                let mut tkn = String::new();
                while "qwertyuiopaasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM_".contains(text_as_chars[x]){
                    tkn.push(text_as_chars[x]);
                    x+=1;
                }
                cur_line.push((TokenType::EnvVar, tkn));
            }
            _ => {
                let mut token = String::new();
                if !" \n\"&$(){}".contains(text_as_chars[x]){
                    while !" \n\"&$(){}".contains(text_as_chars[x]){
                        token.push(text_as_chars[x]);
                        x+=1;
                    }
                   cur_line.push((TokenType::Other, token));
                }
                else{x+=1}

            }
        }
    }
    if cur_line.len() > 0{
        out.push(cur_line)
    }
    debug!("tokenizer ran");
    debug!(format!("tokens:: {:?}", out));
    return out;
}


//                                                 Variable's name     variable's content
//                                                              |          |
//                                                              V          V
pub fn compute_line(tokens:Vec<(TokenType,String)>, index:&Index) -> (String, String){
    let var_name:String;
    let mut var_content = String::new();
    let mut x = 0;
    if tokens.len() < 3 {fatal!("fatal error: config: Incomplete line within config")}
    if tokens[x].0 != TokenType::Variable {fatal!("fatal error: config: variable decleration isn't a variable. did you forget a dollar sign?")}
    var_name = tokens[x].1.clone();
    x+=1;
    if tokens[x].0!=TokenType::Equal{fatal!(format!("Fatal error: config: missing equal sign on decleration of variable \"{}\".", var_name))}
    x+=1;
    while x<tokens.len(){match tokens[x].0{
        TokenType::Str => {var_content+=&tokens[x].1;x+=1},
        TokenType::EnvVar => {
            var_content += &match std::env::var_os(tokens[x].1.clone()){
                Some(val) => val.into_string().expect("failed to convert os string to string"),
                None => fatal!(format!("Fatal Error: config: in decleration of variable \"${}\", the environment variable \"{}\" wasn't found.", var_name, tokens[x].1))
            };
            x+=1;
        }
        TokenType::Variable => {
            let tknx = tokens[x].1.clone();
            debug!(format!("var name {}",tknx));
            var_content+=&match index.get_var(&tknx){
            Ok(val) => val,
            Err(_) => fatal!(format!("fatal error: config: on decleration of variable \"${}\", the variable \"${}\" wasn't found.",var_name,tokens[x].1))
        };x+=1},
        TokenType::CurlyBracket => {
            if tokens[x].1 != "{"{fatal!(format!("Fatal error: config: in variable decleration \"${}\", there is an out-of-place closing curly bracket.", var_name))}
            x+=1;
            if tokens[x].0 != TokenType::Other && tokens[x].0 != TokenType::Str{fatal!(format!("Fatal Error: config: in variable decleration \"${}\", a shell command's... command isn't a standard token or a string.", var_name))}
            let fn_name = &tokens[x].1;
            let mut is_open_token = false;
            let mut cur_arg = String::new();
            x+=1;
            let mut cmd = std::process::Command::new(fn_name.clone());
            while x < tokens.len(){
                if tokens[x].0 == TokenType::Parenthesis{
                    if is_open_token{
                        if tokens[x].1 != ")"{fatal!(format!("fatal error: config: illegal opening parenthesis in shell command \"{}\" in the decleration of the variable \"${}\".", fn_name, var_name))}
                        is_open_token = false;
                        debug!(format!("cur_arg: {}", cur_arg));
                        cmd.arg(cur_arg);
                        cur_arg = String::new();
                        x+=1;
                    }
                    else{
                        if tokens[x].1 != "("{fatal!(format!("fatal error: config: illegal closing parenthesis in shell command \"{}\" in the decleration of the variable \"${}\".", fn_name, var_name))}
                        is_open_token = true;
                        x+=1;
                    }
                }
                else if tokens[x].0 == TokenType::Str{
                    if is_open_token{
                        cur_arg+=&tokens[x].1;
                    }
                    else{fatal!(format!("fatal error: config: string token in shell command \"{}\" within the decleration of the variable \"${}\".",fn_name,var_name))}
                    x+=1;
                }
                else if tokens[x].0 == TokenType::CurlyBracket{
                    if is_open_token{fatal!(format!("fatal error: config: in the decleration for variable \"${}\", the shell command \"{}\" has an unclosed argument.", var_name, fn_name))}
                    break;
                }
                else if tokens[x].0 == TokenType::Equal{
                    fatal!(format!("fatal error: config: in the decleration of variable \"${}\", in the shell command \"{}\" there's a stray equal sign. If this was intended to be a string, surround it with quotes.",var_name, fn_name));
                }
                else if tokens[x].0 == TokenType::Other{
                    if is_open_token{warn!(format!("warning: config: in the decleration of variable \"${}\" in shell command \"{}\" there's a standard token (\"{}\"), this will be interpreted as a string. If this isn't intended to be a string, that sucks. Otherwise, remember that strings should be surrounded by quotes.",var_name,fn_name,tokens[x].1));
                    cur_arg+=&tokens[x].1;
                    }
                    else{fatal!(format!("fatal error: config in decleration of variable \"${}\", in shell command \"{}\" there's a token outside any argument.", var_name, fn_name))}
                    x+=1;
                }
            }
            if !(x<tokens.len()) || tokens[x].0 != TokenType::CurlyBracket{fatal!(format!("fatal error: config: improperly terminated shell command \"{}\" within variable decleration \"${}\".",fn_name, var_name))}
            x+=1;
            let tmp:Vec<u8>;
            var_content+=&match cmd.output(){
                Ok(val) => {tmp = val.stdout;String::from_utf8_lossy(&tmp)},
                Err(_) => fatal!(format!("in the decleration of the variable \"${}\" the shell command \"{}\" exited with an error",var_name,fn_name))
            }
        },
        TokenType::Parenthesis => {fatal!(format!("fatal error: config: in decleration of variable \"{}\", there's stray parenthesis. Parenthesis are to be used only to surround shell command arguments. If you meant this to be a string, please surround it with quotes.", var_name))},
        TokenType::Equal => {fatal!(format!("Fatal error: config: in variable decleration \"${}\", stray equal sign.", var_name))},
        TokenType::Other => {warn!(format!("warning: config: unknown token \"{}\", this is assumed to be a string. Remember, strings should be surrounded by quotation marks.", tokens[x].1)); var_content+=&tokens[x].1; x+=1;}

    }}
    return (var_name, var_content);
}
