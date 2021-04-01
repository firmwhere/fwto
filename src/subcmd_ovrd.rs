/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use std::fs;
use std::io::prelude::*;
use std::result::Result;
use structopt::StructOpt;

use crate::audk;
use crate::libs::ffs;

#[derive(StructOpt, Debug)]
pub struct Ovrd {
    /// File to be override
    #[structopt(short, long, parse(from_os_str))]
    pub src             : std::path::PathBuf,
    #[structopt(flatten)]
    pub flags           : OvrdFlags,
}

#[derive(StructOpt, Debug)]
pub struct OvrdFlags {
    /// Clean files from override
    #[structopt(long)]
    pub clean               : bool,
    /// Skip original of override
    #[structopt(long)]
    pub skip_org            : bool,
}

impl Ovrd {
    pub fn new(src: &std::path::PathBuf, clean: bool, skip_org: bool) -> Self {
        Ovrd {
            src: std::path::PathBuf::from(src), flags: OvrdFlags { clean, skip_org }
        }
    }

    pub fn handler(&self, opt_oemovrd: &audk::StdOvrd, cfg_oemovrd: &Option<&audk::StdOvrd>, cfg_ibvovrd: &Option<&audk::StdOvrd>) {
        let ws = std::env::current_dir().unwrap();
        // Skip *.cif, *.sdl
        if self.src.extension().unwrap() == "cif" ||
           self.src.extension().unwrap() == "sdl" {
            println!("ERR: want override file, which's unsupport {:?}", ws.join(&self.src));
            return;
        }
        let cif = if let Some(cif) = &opt_oemovrd.cif {
            cif
        } else {
            cfg_oemovrd.expect("ERR: oemovrd is neither given in cmdline or json").cif.as_ref().expect("ERR: cif is None in json.")
        };
        if !cif.is_file() {
            println!("ERR: want override.cif, but not a file: {:?}", cif);
            return;
        }
        let dst = if let Some(dst) = &opt_oemovrd.dst {
            dst
        } else {
            cfg_oemovrd.expect("ERR: oemovrd is neither given in cmdline or json").dst.as_ref().expect("ERR: dst is None in json.")
        };
        if !dst.is_dir() {
            println!("ERR: want override.dst, but not a dir: {:?}", dst);
            return;
        }
        let org = if opt_oemovrd.org.is_some() {
            &opt_oemovrd.org
        } else {
            &cfg_oemovrd.expect("ERR: oemovrd is neither given in cmdline or json").org
        };

        if self.flags.clean {
            self.override_del(cif, dst, org);
        } else {
            let ibvovrd_dst = if let Some(ibvovrd) = cfg_ibvovrd.as_deref() {
                &ibvovrd.dst
            } else {
                &None
            };
            self.override_add(cif, dst, org, &ibvovrd_dst.as_ref());
        }
    }

    fn build_cif_override_line(&self, dst: &std::path::PathBuf) -> String {
        let ovrd_src = &self.src;
        let ovrd_dst = std::path::PathBuf::from(dst.file_name().unwrap()).join(&self.src);

        use path_slash::PathBufExt;
        let ovrd_src = std::path::PathBuf::from_slash(ovrd_src.to_str().unwrap());
        let ovrd_dst = std::path::PathBuf::from_slash(ovrd_dst.to_str().unwrap());

        String::new() + r#"""# + ovrd_dst.to_str().unwrap() + r#"";""# + ovrd_src.to_str().unwrap() + r#"""# + "\r\n"
    }
    
    fn add_override_files(&self, dst: &std::path::PathBuf, org: &Option<std::path::PathBuf>, ibvovrd_dst: &Option<&std::path::PathBuf>) -> Result<bool, String> {
        if !self.src.is_file() {
            return Err(format!("ERR: want override file, but no file found {:?}", &self.src));
        }

        let mut is_1st_time_ovrd = true;

        let fdst = std::path::PathBuf::from(dst).join(&self.src);
        // keep fdst as is if not 1st time override
        if !fdst.is_file() {
            let fdst_parent = fdst.parent().unwrap();
            if !fdst_parent.is_dir() {
                fs::create_dir_all(fdst_parent).unwrap();
            }
            ffs::copy(&self.src, &fdst).unwrap();
            // override if there is ibvovrd.fsrc
            if let Some(ibvovrd_dst) = ibvovrd_dst {
                let ibvovrd_fsrc = ibvovrd_dst.join(&self.src);
                if  ibvovrd_fsrc.is_file() {
                    ffs::copy(&ibvovrd_fsrc, &fdst).unwrap();
                }
            }
        } else {
            is_1st_time_ovrd = false;
        }
        // always overrides forg if it is available
        if !self.flags.skip_org {
            if let Some(org) = org {
                if !org.is_dir() {
                    self.del_override_files(dst, &None);
                    return Err(format!("ERR: want override.org, but not a dir: {:?}", org));
                }
                let forg = std::path::PathBuf::from(org).join(&self.src);
                let forg_parent = forg.parent().unwrap();
                if !forg_parent.is_dir() {
                    fs::create_dir_all(forg_parent).unwrap();
                }
                ffs::copy(&self.src, &forg).unwrap();
            }
        }

        return Ok(is_1st_time_ovrd);
    }

    fn del_override_files(&self, dst: &std::path::PathBuf, org: &Option<std::path::PathBuf>) {
        let fdst = std::path::PathBuf::from(dst).join(&self.src);
        if fdst.is_file() {
            ffs::remove_file(fdst).unwrap();
        }
        if !self.flags.skip_org {
            if let Some(org) = org {
                let forg = std::path::PathBuf::from(org).join(&self.src);
                if forg.is_file() {
                    ffs::remove_file(forg).unwrap();
                }
            }
        }
    }

    pub fn override_add(&self, cif: &std::path::PathBuf, dst: &std::path::PathBuf, org: &Option<std::path::PathBuf>, ibvovrd_dst: &Option<&std::path::PathBuf>) {
        let result = self.add_override_files(dst, org, ibvovrd_dst);
        if let Err(error) = result { println!("{}", error);return; }

        if let Ok(is_1st_time_ovrd) = result {
            if !is_1st_time_ovrd { return; }
        }
        // build [file] override statement and add to cif
        let cif_override_line = self.build_cif_override_line(dst);

        let cifbf = fs::read_to_string(cif).unwrap();
        let lines = cifbf.lines();
        let mut fcif = fs::OpenOptions::new().write(true).truncate(true).open(cif).unwrap();
        for line in lines {
            if line.to_ascii_lowercase().starts_with("<endcomponent>") {
                fcif.write(cif_override_line.as_bytes()).unwrap();
            }
            let line_with_new_line = String::from(line) + "\r\n";
            fcif.write(line_with_new_line.as_bytes()).unwrap();
        }
    }

    pub fn override_del(&self, cif: &std::path::PathBuf, dst: &std::path::PathBuf, org: &Option<std::path::PathBuf>) {
        use path_slash::PathBufExt;
        let old_dst_file = std::path::PathBuf::from_slash(std::path::PathBuf::from(dst.file_name().unwrap()).join(&self.src).to_str().unwrap());
        let old_dst_line = String::new() + r#"""# + old_dst_file.to_str().unwrap() + r#"""#;

        let cifbf = fs::read_to_string(cif).unwrap();
        let lines = cifbf.lines();
        let mut fcif = fs::OpenOptions::new().write(true).truncate(true).open(cif).unwrap();
        for line in lines {
            if line.to_ascii_lowercase().starts_with(&old_dst_line.to_ascii_lowercase()) {
                self.del_override_files(dst, org);
            } else {
                let line_with_new_line = String::from(line) + "\r\n";
                fcif.write(line_with_new_line.as_bytes()).unwrap();
            }
        }
    }

    pub fn override_replace_with(&self, new: &Self, cif: &std::path::PathBuf, dst: &std::path::PathBuf, org: &Option<std::path::PathBuf>, ibvovrd_dst: &Option<&std::path::PathBuf>) {
        use path_slash::PathBufExt;
        let old_dst_file = std::path::PathBuf::from_slash(std::path::PathBuf::from(dst.file_name().unwrap()).join(&self.src).to_str().unwrap());
        let old_dst_line = String::new() + r#"""# + old_dst_file.to_str().unwrap() + r#"""#;

        let cifbf = fs::read_to_string(cif).unwrap();
        let lines = cifbf.lines();
        let mut fcif = fs::OpenOptions::new().write(true).truncate(true).open(cif).unwrap();
        for line in lines {
            if line.to_ascii_lowercase().starts_with(&old_dst_line.to_ascii_lowercase()) {
                // replace old override with new override
                let result = new.add_override_files(dst, org, ibvovrd_dst);
                if let Err(error) = result { println!("{}", error); return; }

                let old_fdst = dst.join(&self.src);
                let new_fdst = dst.join(& new.src);
                ffs::copy(&old_fdst, &new_fdst).unwrap();

                self.del_override_files(dst, org);

                fcif.write(new.build_cif_override_line(dst).as_bytes()).unwrap();
            } else {
                let line_with_new_line = String::from(line) + "\r\n";
                fcif.write(line_with_new_line.as_bytes()).unwrap();
            }
        }
    }
}
