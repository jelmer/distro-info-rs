extern crate chrono;
extern crate clap;
extern crate distro_info;
extern crate failure;

use chrono::Datelike;
use chrono::Utc;
use chrono::naive::NaiveDate;
use clap::{Arg, ArgGroup, App};
use distro_info::UbuntuDistroInfo;
use failure::Error;

fn all(ubuntu_distro_info: UbuntuDistroInfo) {
    for distro_release in ubuntu_distro_info {
        println!("{}", distro_release.series);
    }
}

fn supported(ubuntu_distro_info: UbuntuDistroInfo) {
    let now = Utc::now();
    let today = NaiveDate::from_ymd(now.year(), now.month(), now.day());
    for distro_release in ubuntu_distro_info.supported(today) {
        println!("{}", distro_release.series);
    }
}

fn run() -> Result<(), Error> {
    let matches = App::new("ubuntu-distro-info")
        .version("0.1.2")
        .author("Daniel Watkins <daniel@daniel-watkins.co.uk>")
        .arg(Arg::with_name("all").short("a").long("all"))
        .arg(Arg::with_name("supported").long("supported"))
        .group(ArgGroup::with_name("selector").args(&["all", "supported"]).required(true))
        .get_matches();
    let ubuntu_distro_info = UbuntuDistroInfo::new()?;
    if matches.is_present("all") {
        all(ubuntu_distro_info);
    } else if matches.is_present("supported") {
        supported(ubuntu_distro_info);
    }
    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        writeln!(stderr, "error: {:?}", e).unwrap();
        ::std::process::exit(1);
    }
}
