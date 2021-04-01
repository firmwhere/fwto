/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use std::fs;
use std::io::prelude::*;
use structopt::StructOpt;
use path_slash::PathBufExt;

use crate::audk;
use crate::subcmd_ovrd;
use crate::libs::ffs;
use crate::libs::git;

#[derive(StructOpt, Debug)]
pub struct Cbup {
    /// Commit to be extract
    #[structopt(short, long, parse(from_str))]
    pub commit          : String,
    #[structopt(flatten)]
    pub flags           : CbupFlags,
}

#[derive(StructOpt, Debug)]
pub struct CbupFlags {
    /// Extract codebase-pure module or package diffs
    #[structopt(long)]
    pub pure            : bool,
}

const CBUP_HOME         : &str = "cbup";
const CBUP_OLD_         : &str = "base.old";
const CBUP_NEW_         : &str = "base.new";
const CBUP_OVRD         : &str = "ovrd";

impl Cbup {
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

        let org = if opt_oemovrd.org.is_some() {
            &opt_oemovrd.org
        } else {
            &cfg_oemovrd.expect("ERR: oemovrd is neither given in cmdline or json").org
        };

        let ibvovrd_dst = if let Some(ibvovrd) = cfg_ibvovrd.as_deref() {
            &ibvovrd.dst
        } else {
            &None
        };
        let ibvovrd_dst = &ibvovrd_dst.as_ref();

        git::reset_hard_and_clean_xfd(&self.commit);
        self.codebase_oemovrd(cif, dst, org, ibvovrd_dst);
        self.codebase_ibvovrd(cif, dst, org, ibvovrd_dst);
        if self.flags.pure {
            git::revert_no_commit(&self.commit);
        }
    }

    fn codebase_oemovrd(&self, cif: &std::path::PathBuf, dst: &std::path::PathBuf, org: &Option<std::path::PathBuf>, ibvovrd_dst: &Option<&std::path::PathBuf>) {
        let find_renames = "75%";
        let show_files   = vec![ibvovrd_dst];

        let not_r_path = std::path::PathBuf::from(audk::FWTO_WS).join(CBUP_HOME).join("!R");
        if self.flags.pure {
            let output = git::show_no_format(&self.commit, true, find_renames, "A", &show_files, true);
            if !output.status.success() {
                println!("codebase_oemovrd.a: {:#?}", output);
            } else {
                let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
                for line in git_show_result.lines() {
                    let fsrc = std::path::PathBuf::from(line);
                    // [0]: light override it
                    subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                }
            }
        }
        let output = git::show_no_format(&self.commit, true, find_renames, "D", &show_files, true);
        if !output.status.success() {
            println!("codebase_oemovrd.d: {:#?}", output);
        } else {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let fsrc = std::path::PathBuf::from(line);
                let fdst = dst.join(&fsrc);
                // [0]: also we have it?
                if fdst.is_file() {
                    // [1]: diff trees for better compare
                    let _old = not_r_path.join(CBUP_OLD_).join(&fsrc);
                    let ovrd = not_r_path.join(CBUP_OVRD).join(&fsrc);
                    // create base.old
                    git::create_file_from(&self.commit, &fsrc, &_old, Some("~"));
                    // create ovrd
                    git::create_file_from(&self.commit, &fdst, &ovrd, None);
                    // [2]: update override
                    subcmd_ovrd::Ovrd::new(&fsrc, true, false).override_del(cif, dst, org);
                }
            }
        }
        let output = git::show_no_format(&self.commit, true, find_renames, "M", &show_files, true);
        if !output.status.success() {
            println!("codebase_oemovrd.m: {:#?}", output);
        } else {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let fsrc = std::path::PathBuf::from(line);
                let fdst = dst.join(&fsrc);
                // [0]: also we have it?
                if fdst.is_file() {
                    // [1]: diff trees for better compare
                    let _old = not_r_path.join(CBUP_OLD_).join(&fsrc);
                    let _new = not_r_path.join(CBUP_NEW_).join(&fsrc);
                    let ovrd = not_r_path.join(CBUP_OVRD).join(&fsrc);
                    // create base.old
                    git::create_file_from(&self.commit, &fsrc, &_old, Some("~"));
                    // create base.new
                    git::create_file_from(&self.commit, &fsrc, &_new, None);
                    // create ovrd
                    git::create_file_from(&self.commit, &fdst, &ovrd, None);
                    // [2]: update override
                    if !self.flags.pure {
                        subcmd_ovrd::Ovrd::new(&fsrc, false, false).override_add(cif, dst, org, ibvovrd_dst);
                    } else {
                        subcmd_ovrd::Ovrd::new(&fsrc, false,  true).override_add(cif, dst, org, ibvovrd_dst);
                    }
                } else if self.flags.pure {
                    subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                }
            }
        }
        let output = git::show_no_format(&self.commit, false, "100%", "R", &show_files, true);
        if output.status.success() {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let mut  part = line.split_whitespace();
                let _rpercent = part.next().unwrap();
                let _old_fsrc = std::path::PathBuf::from(part.next().unwrap());
                let _new_fsrc = std::path::PathBuf::from(part.next().unwrap());
                let fdst = dst.join(&_old_fsrc);
                // [0]: also we have it?
                if fdst.is_file() {
                    // [1]: replace old override with new override
                    if !self.flags.pure {
                        let old_ovrd = subcmd_ovrd::Ovrd::new(&_old_fsrc, false, false);
                        let new_ovrd = subcmd_ovrd::Ovrd::new(&_new_fsrc, false, false);
                        old_ovrd.override_replace_with(&new_ovrd, cif, dst, org, ibvovrd_dst);
                    } else {
                        let old_ovrd = subcmd_ovrd::Ovrd::new(&_old_fsrc, false, true);
                        let new_ovrd = subcmd_ovrd::Ovrd::new(&_new_fsrc, false, true);
                        old_ovrd.override_replace_with(&new_ovrd, cif, dst, org, ibvovrd_dst);
                    }
                } else if self.flags.pure {
                    subcmd_ovrd::Ovrd::new(&_new_fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                }
            }
        } else {
            println!("codebase_oemovrd.r100: {:#?}", output);
        }
        let r_path = std::path::PathBuf::from(audk::FWTO_WS).join(CBUP_HOME).join("R75");
        let output = git::show_no_format(&self.commit, false, find_renames, "R", &show_files, true);
        if output.status.success() {
            let r_path_parent = r_path.parent().unwrap();
            if !r_path_parent.is_dir() {
                fs::create_dir_all(&r_path_parent).unwrap();
            }
            let r_log_path = r_path_parent.join("R75.log");
            let mut r_log = fs::OpenOptions::new().create(true).write(true).open(&r_log_path).unwrap();

            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let mut  part = line.split_whitespace();
                let _rpercent = part.next().unwrap();
                let _old_fsrc = std::path::PathBuf::from(part.next().unwrap());
                let _new_fsrc = std::path::PathBuf::from(part.next().unwrap());
                let fdst = std::path::PathBuf::from(dst).join(&_old_fsrc);
                // [0]: also we have it?
                if fdst.is_file() {
                    if _rpercent == "R100" {
                        continue;
                    } else {
                        // log rename files for R75%
                        let line_with_new_line = String::from(line) + "\r\n";
                        r_log.write(line_with_new_line.as_bytes()).unwrap();
                    }

                    // [1]: build files tree like M, all path align to _new_fsrc for better compare
                    let _old = r_path.join(CBUP_OLD_).join(&_new_fsrc);
                    let _new = r_path.join(CBUP_NEW_).join(&_new_fsrc);
                    let ovrd = r_path.join(CBUP_OVRD).join(&_new_fsrc);
                    // create base.old
                    git::create_file_from(&self.commit, &_old_fsrc, &_old, Some("~"));
                    // create base.new
                    git::create_file_from(&self.commit, &_new_fsrc, &_new, None);
                    // create ovrd
                    git::create_file_from(&self.commit, &fdst, &ovrd, None);
                    // [2]: replace old override with new override like R100%
                    if !self.flags.pure {
                        let old_ovrd = subcmd_ovrd::Ovrd::new(&_old_fsrc, false, false);
                        let new_ovrd = subcmd_ovrd::Ovrd::new(&_new_fsrc, false, false);
                        old_ovrd.override_replace_with(&new_ovrd, cif, dst, org, ibvovrd_dst);
                    } else {
                        let old_ovrd = subcmd_ovrd::Ovrd::new(&_old_fsrc, false, true);
                        let new_ovrd = subcmd_ovrd::Ovrd::new(&_new_fsrc, false, true);
                        old_ovrd.override_replace_with(&new_ovrd, cif, dst, org, ibvovrd_dst);
                    }
                } else if self.flags.pure {
                    subcmd_ovrd::Ovrd::new(&_new_fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                }
            }
            if r_log.metadata().unwrap().len() == 0 {
                ffs::remove_file(&r_log_path).unwrap();
            } else {
                fs::rename(&r_log_path, r_path.join("R75.log")).unwrap();
            }
        } else {
            println!("codebase_oemovrd.r75: {:#?}", output);
        }
    }

    fn codebase_ibvovrd(&self, cif: &std::path::PathBuf, dst: &std::path::PathBuf, org: &Option<std::path::PathBuf>, ibvovrd_dst: &Option<&std::path::PathBuf>) {
        if let Some(ibvovrd_dst) = ibvovrd_dst {
            if !ibvovrd_dst.is_dir() {
                println!("WRN: ibvovrd.dst is set but not a dir: {:?}", &ibvovrd_dst);
                return;
            }
        } else {
            return;
        }
        let find_renames = "75%";
        let show_files   = vec![ibvovrd_dst];
        let not_r_path = std::path::PathBuf::from(audk::FWTO_WS).join(CBUP_HOME).join("!R");
        let output = git::show_no_format(&self.commit, true, find_renames, "A", &show_files, false);
        if !output.status.success() {
            println!("codebase_ibvovrd.a: {:#?}", output);
        } else {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let fibv = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fibv.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                let fdst = dst.join(&fsrc);
                let _new = not_r_path.join(CBUP_NEW_).join(&fsrc);
                if fdst.is_file() || _new.is_file() {
                    // [1]: update diff trees (ibvovrd.A: keep old, update new)
                    git::create_file_from(&self.commit, &fibv, &_new, None);
                } else if self.flags.pure {
                    if fsrc.is_file() {
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                    } else {
                        fs::OpenOptions::new().create(true).open(&fsrc).unwrap();
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                        ffs::remove_file(fsrc).unwrap();
                    }
                }
            }
        }
        let output = git::show_no_format(&self.commit, true, find_renames, "D", &show_files, false);
        if !output.status.success() {
            println!("codebase_ibvovrd.d: {:#?}", output);
        } else {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let fibv = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fibv.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                let fdst = dst.join(&fsrc);
                let _old = not_r_path.join(CBUP_OLD_).join(&fsrc);
                if fdst.is_file() || _old.is_file() {
                    // [1]: update diff trees (ibvovrd.D: keep new, update old)
                    git::create_file_from(&self.commit, &fibv, &_old, Some("~"));
                }
            }
        }
        let output = git::show_no_format(&self.commit, true, find_renames, "M", &show_files, false);
        if !output.status.success() {
            println!("codebase_ibvovrd.m: {:#?}", output);
        } else {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let fibv = std::path::PathBuf::from(line);
                let fsrc = std::path::PathBuf::from(fibv.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                let fdst = dst.join(&fsrc);
                let _old = not_r_path.join(CBUP_OLD_).join(&fsrc);
                let _new = not_r_path.join(CBUP_NEW_).join(&fsrc);
                if fdst.is_file() || _old.is_file() || _new.is_file() {
                    // [1]: update diff trees (ibvovrd.M: update old, update new)
                    git::create_file_from(&self.commit, &fibv, &_old, Some("~"));
                    git::create_file_from(&self.commit, &fibv, &_new, None);
                } else if self.flags.pure {
                    if fsrc.is_file() {
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                    } else {
                        fs::OpenOptions::new().create(true).open(&fsrc).unwrap();
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                        ffs::remove_file(fsrc).unwrap();
                    }
                }
            }
        }
        let output = git::show_no_format(&self.commit, false, "100%", "R", &show_files, false);
        if !output.status.success() {
            println!("codebase_ibvovrd.r100: {:#?}", output);
        } else {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let mut  part = line.split_whitespace();
                let _rpercent = part.next().unwrap();
                let _old_fsrc = std::path::PathBuf::from(part.next().unwrap());
                let _new_fsrc = std::path::PathBuf::from(part.next().unwrap());

                let fsrc = std::path::PathBuf::from(_new_fsrc.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                // [1]: update diff trees (ibvovrd.R: ignore)
                if self.flags.pure {
                    if fsrc.is_file() {
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                    } else {
                        fs::OpenOptions::new().create(true).open(&fsrc).unwrap();
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                        ffs::remove_file(fsrc).unwrap();
                    }
                }
            }
        }

        let r_path = std::path::PathBuf::from(audk::FWTO_WS).join(CBUP_HOME).join("R75");
        let output = git::show_no_format(&self.commit, false, find_renames, "R", &show_files, false);
        if output.status.success() {
            let git_show_result = String::from_utf8(output.clone().stdout).unwrap();
            for line in git_show_result.lines() {
                let mut  part = line.split_whitespace();
                let _rpercent = part.next().unwrap();
                let _old_fsrc = std::path::PathBuf::from(part.next().unwrap());
                let _new_fsrc = std::path::PathBuf::from(part.next().unwrap());

                let fsrc = std::path::PathBuf::from(_new_fsrc.strip_prefix(&ibvovrd_dst.unwrap().to_slash().unwrap()).unwrap());
                let fdst = std::path::PathBuf::from(dst).join(&fsrc);
                let _old = r_path.join(CBUP_OLD_).join(&fsrc);
                let _new = r_path.join(CBUP_NEW_).join(&fsrc);
                if fdst.is_file() {
                    if _rpercent == "R100" {
                        continue;
                    }
                    // [1]: update diff trees (ibvovrd.R: update old, update new)
                    git::create_file_from(&self.commit, &_old_fsrc, &_old, Some("~"));
                    git::create_file_from(&self.commit, &_new_fsrc, &_new, None);
                } else if self.flags.pure {
                    if fsrc.is_file() {
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                    } else {
                        fs::OpenOptions::new().create(true).open(&fsrc).unwrap();
                        subcmd_ovrd::Ovrd::new(&fsrc, false, true).override_add(cif, dst, org, ibvovrd_dst);
                        ffs::remove_file(fsrc).unwrap();
                    }
                }
            }
        } else {
            println!("codebase_oemovrd.r75: {:#?}", output);
        }
    }
}
