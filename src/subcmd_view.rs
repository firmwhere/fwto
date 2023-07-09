/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use structopt::StructOpt;
use path_slash::PathBufExt;

use crate::audk;
use crate::libs::git;

#[derive(StructOpt, Debug)]
pub struct View {
    /// New commit for diff
    #[structopt(short, long, parse(from_str), default_value = "HEAD" )]
    pub new             : String,
    /// Old commit for diff
    #[structopt(short, long, parse(from_str) )]
    pub old             : Option<String>,
}

const VIEW_HOME         : &str = "view";
const VIEW_OLD          : &str = "old";
const VIEW_NEW          : &str = "new";

impl View {
    pub fn handler(&self, opt_oemovrd: &audk::StdOvrd, cfg_oemovrd: &Option<&audk::StdOvrd>, cfg_ibvovrd: &Option<&audk::StdOvrd>) {
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
        let dst = &Some(dst);

        let org = if opt_oemovrd.org.is_some() {
            &opt_oemovrd.org
        } else {
            &cfg_oemovrd.expect("ERR: oemovrd is neither given in cmdline or json").org
        };
        let org = &org.as_ref();

        let ibvovrd_dst = if let Some(ibvovrd) = cfg_ibvovrd.as_deref() {
            &ibvovrd.dst
        } else {
            &None
        };
        let ibvovrd_dst = &ibvovrd_dst.as_ref();

        let def = String::new() + &self.new + "~";
        let old = self.old.as_ref().unwrap_or(&def);

        self.review_nonovrd(old, dst, org, ibvovrd_dst);
        self.review_ibvovrd(old,           ibvovrd_dst);
        self.review_oemovrd(old, dst);
    }

    fn review_nonovrd(&self, self_old: &String, dst: &Option<&std::path::PathBuf>, org: &Option<&std::path::PathBuf>, ibvovrd_dst: &Option<&std::path::PathBuf>) {
        let find_renames = "100%";
        let show_files   = vec![dst, org, ibvovrd_dst];

        let v_path = std::path::PathBuf::from(audk::FWTO_WS).join(VIEW_HOME);
        let output = git::diff_no_format(&self_old, &self.new, true, find_renames, "ADM", &show_files, true);
        if !output.status.success() {
            println!("codebase_oemovrd.d: {:#?}", output);
        } else {
            let diff_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in diff_result.lines() {
                let fsrc = std::path::PathBuf::from(line);
                // [1]: diff trees for better compare
                let old = v_path.join(VIEW_OLD).join(&fsrc);
                let new = v_path.join(VIEW_NEW).join(&fsrc);
                // create old
                git::create_file_from(&self_old, &fsrc, &old, None);
                // create new
                git::create_file_from(&self.new, &fsrc, &new, None);
            }
        }
    }

    fn review_ibvovrd(&self, self_old: &String, ibvovrd_dst: &Option<&std::path::PathBuf>) {
        if let Some(ibvovrd_dst) = ibvovrd_dst {
            if !ibvovrd_dst.is_dir() {
                println!("WRN: ibvovrd.dst is set but not a dir: {:?}", &ibvovrd_dst);
                return;
            }
        } else {
            return;
        }
        let find_renames = "100%";
        let show_files   = vec![ibvovrd_dst];

        let v_path = std::path::PathBuf::from(audk::FWTO_WS).join(VIEW_HOME);
        let output = git::diff_no_format(&self_old, &self.new, true, find_renames, "A", &show_files, false);
        if !output.status.success() {
            println!("review_ibvovrd.a: {:#?}", output);
        } else {
            let diff_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in diff_result.lines() {
                let fibv = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fibv.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                // [1]: diff trees for better compare
                let old = v_path.join(VIEW_OLD).join(&fsrc);
                let new = v_path.join(VIEW_NEW).join(&fsrc);
                // create old
                git::create_file_from(&self.new, &fsrc, &old, None);
                // create new
                git::create_file_from(&self.new, &fibv, &new, None);
            }
        }

        let output = git::diff_no_format(&self_old, &self.new, true, find_renames, "D", &show_files, false);
        if !output.status.success() {
            println!("review_ibvovrd.d: {:#?}", output);
        } else {
            let diff_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in diff_result.lines() {
                let fibv = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fibv.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                // [1]: diff trees for better compare
                let old = v_path.join(VIEW_OLD).join(&fsrc);
                let new = v_path.join(VIEW_NEW).join(&fsrc);
                // create old
                git::create_file_from(&self_old, &fibv, &old, None);
                // create new
                git::create_file_from(&self.new, &fsrc, &new, None);
            }
        }

        let output = git::diff_no_format(&self_old, &self.new, true, find_renames, "M", &show_files, false);
        if !output.status.success() {
            println!("review_ibvovrd.m: {:#?}", output);
        } else {
            let diff_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in diff_result.lines() {
                let fibv = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fibv.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                // [1]: diff trees for better compare
                let old = v_path.join(VIEW_OLD).join(&fsrc);
                let new = v_path.join(VIEW_NEW).join(&fsrc);
                // create old
                git::create_file_from(&self_old, &fibv, &old, None);
                // create new
                git::create_file_from(&self.new, &fibv, &new, None);
            }
        }
    }

    fn review_oemovrd(&self, self_old: &String, dst: &Option<&std::path::PathBuf>) {
        let find_renames = "100%";
        let show_files   = vec![dst];

        let v_path = std::path::PathBuf::from(audk::FWTO_WS).join(VIEW_HOME);
        let output = git::diff_no_format(&self_old, &self.new, true, find_renames, "A", &show_files, false);
        if !output.status.success() {
            println!("review_oemovrd.a: {:#?}", output);
        } else {
            let diff_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in diff_result.lines() {
                let fdst = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fdst.strip_prefix(&dst.unwrap().to_slash().unwrap()).unwrap());
                // [1]: diff trees for better compare
                let old = v_path.join(VIEW_OLD).join(&fsrc);
                let new = v_path.join(VIEW_NEW).join(&fsrc);
                // create old
                git::create_file_from(&self.new, &fsrc, &old, None);
                // create new
                git::create_file_from(&self.new, &fdst, &new, None);
            }
        }

        let output = git::diff_no_format(&self_old, &self.new, true, find_renames, "D", &show_files, false);
        if !output.status.success() {
            println!("review_oemovrd.d: {:#?}", output);
        } else {
            let diff_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in diff_result.lines() {
                let fdst = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fdst.strip_prefix(&dst.unwrap().to_slash().unwrap()).unwrap());
                // [1]: diff trees for better compare
                let old = v_path.join(VIEW_OLD).join(&fsrc);
                let new = v_path.join(VIEW_NEW).join(&fsrc);
                // create old
                git::create_file_from(&self_old, &fdst, &old, None);
                // create new
                git::create_file_from(&self.new, &fsrc, &new, None);
                if !new.is_file() {
                    // diff may be   moved to another source
                } else {
                    // diff may be removed
                }
            }
        }

        let output = git::diff_no_format(&self_old, &self.new, true, find_renames, "M", &show_files, false);
        if !output.status.success() {
            println!("review_oemovrd.m: {:#?}", output);
        } else {
            let diff_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in diff_result.lines() {
                let fdst = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fdst.strip_prefix(&dst.unwrap().to_slash().unwrap()).unwrap());
                // [1]: diff trees for better compare
                let old = v_path.join(VIEW_OLD).join(&fsrc);
                let new = v_path.join(VIEW_NEW).join(&fsrc);
                // create old
                git::create_file_from(&self_old, &fdst, &old, None);
                // create new
                git::create_file_from(&self.new, &fdst, &new, None);
            }
        }
    }
}
