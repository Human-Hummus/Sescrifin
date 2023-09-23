pub extern crate termion;
pub use termion::color::Fg;
pub use termion::{color};
pub extern crate std;

#[macro_export]
macro_rules! fatal{
    ($text:expr) => {{
        println!("{}{}{}", $crate::termion::color::Fg($crate::termion::color::Red), $text, $crate::termion::color::Fg($crate::termion::color::Reset));$crate::output::std::process::exit(1);}}
}
#[macro_export]
macro_rules! alert{           
    ($text:expr) => {{println!("{}{}{}", $crate::termion::color::Fg($crate::termion::color::Green), $text, $crate::termion::color::Fg($crate::termion::color::Reset))}
}}
#[macro_export]
macro_rules! debug{
    ($text:expr) => {if cfg!(debug_assertions){println!("{}{}{}", $crate::termion::color::Fg($crate::termion::color::Blue), $text, $crate::termion::color::Fg($crate::termion::color::Reset))}}
}
#[macro_export]
macro_rules! error{
    ($text:expr) => {println!("{}{}{}", $crate::termion::color::Fg($crate::termion::color::Red), $text, $crate::termion::color::Fg($crate::termion::color::Reset))}
}
#[macro_export]
macro_rules! warn{
    ($text:expr) => {println!("{}{}{}", $crate::termion::color::Fg($crate::termion::color::Yellow), $text, $crate::termion::color::Fg($crate::termion::color::Reset))}
}
//pub(crate) use fatal;
