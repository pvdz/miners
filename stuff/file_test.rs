// https://docs.rs/regex/1.5.4/regex/
// https://github.com/rust-lang/regex

use std::fs;
use regex::Regex;
use lazy_static::lazy_static;
use std::path::Path;

use std::io;
use std::ffi::OsStr;


/*
scaling this....

there are needles that are regex, multiple at once. and then there are some one-of needles
or are those just too complex to properly regex them up
should probably encode them as groups and then read them like that example
some stuff is conditional to the file path/name. that's just a heuristic and perhaps
that heuristic is not necessary in rust? could lead to noise but we could counter that
with an ignore list or smth.

given a list of files, return any file that matches some abuse related heuristic


 */




// Lazily create regular expressions once
// https://docs.rs/regex/1.5.4/regex/#example-avoid-compiling-the-same-regex-in-a-loop
lazy_static! {
    static ref RULE_MINER_KEYWORDS: Regex = Regex::new(r"miner|pool|stratum").unwrap();
    static ref RULE_MINER_NAMES: Regex = Regex::new(r"xmr|xla").unwrap();
    static ref RULE_MINER_CONFIG_NEEDLES: Regex = Regex::new(r"crypto?knight|swissinfonet|stratum").unwrap();
    static ref RULE_MINER_CONFIG_MATCH: Regex = Regex::new(r".{0,20}(crypto?knight|swissinfonet|stratum).{0,20}").unwrap();
    static ref RULE_MINE_SHORT_NEEDLES: Regex = Regex::new(r"monero|xmr|xla").unwrap();
    static ref RULE_MINE_SHORT_MATCH: Regex = Regex::new(r"[\w\d]*(?:monero|xmr|xla)[\w\d]*").unwrap();
}

pub fn read_test_str(root_path: &str) -> bool {
    println!("Starting from `{}`", root_path);

    // The abs path is irrelevant since the Path instance will already resolve it anyways. shrug
    // let path_abs = match path.canonicalize() {
    //     Ok(ref canon) => { // TODO: I don't know why `ref` is required here but not below
    //         let c = canon.to_string_lossy();
    //         println!("  Absolute: {}", c);
    //         c
    //     },
    //     _ => {
    //         println!("  Is a dir path without file...");
    //         return false;
    //     },
    // };

    let path = Path::new(root_path);
    let b = read_test_path(path);

    if b {
        println!("Some abuse found. Sad!");
    } else {
        println!("No abuse found. Yay!");
    }

    return b;
}

pub fn read_test_path(cur_path: &Path) -> bool {
    // Confirm that this file is not a symlink, it exists, and its filename is valid utf :shrug:
    // If so, scan it.
    println!("read_test_path({})", cur_path.display());


    if !cur_path.exists() {
        // Should this happen? Let's make sure it doesn't crash us.
        println!("  Skipping non-existing file...");
        return false;
    }

    let metadata_maybe = fs::symlink_metadata(cur_path);
    if cur_path.is_dir() {
        println!("  Traversing a dir now...");

        match metadata_maybe {
            Ok(meta) => {
                println!("  ok!");
                let file_type = meta.file_type();
                if file_type.is_symlink() {
                    println!("  Skipping dir path...");
                    return false;
                }
            },
            Err(_) => {
                // Umm. Isn't this bad by default?
                println!("  Skipping path; could not get meta data. Symlink or other problem.");
                return false;
            }
        };

        // Okay, traverse this folder. I don't think this should trigger an error at this point.

        let readdir = fs::read_dir(cur_path);
        match readdir {
            Ok(rd) => {
                for entry in rd {
                    match entry {
                        Ok(dir_entry) => {
                            if read_test_path(&dir_entry.path()) {
                                return true;
                            }
                        },
                        // Ignore errors. I think. (What triggers here that doesn't)
                        Err(_) => {
                            println!("  direntry.error");
                        },
                    };
                }
            },
            Err(_) => {
                println!("  readdir.err");
            },
        };
        println!("  just past it");
        return false;
    } else {
        println!("  Processing a file now...");

        // We skip this file when it is a symlink, a zero file, a too large file, or an error.
        match metadata_maybe {
            Ok(meta) => {
                let file_type = meta.file_type();
                if file_type.is_symlink() {
                    println!("  File is symlink...");
                    return false;
                }

                let len = meta.len();
                if len == 0 || len > 500000 { // arbitrary ceiling. TODO: bench it.
                    println!("  File is empty or too big...");
                    return false;
                }
            },
            Err(_) => {
                // Umm. Isn't this bad by default?
                println!("  Skipping path; could not get meta data. Symlink or other problem.");
                return false;
            }
        };

        if abusive_file(cur_path) {
            println!("Some abuse found. Exiting now.");
            return true;
        }

        return false;
    }
}

pub fn abusive_file(path: &Path) -> bool {
    println!("  The path.display: {}", path.display());

    // Experimental feature..
    // if path.is_symlink() {
    //     println!("  Skipping symlink...");
    //     continue;
    // }

    if !path.exists() {
        // Code wise this can happen because the lstat does not guarantee an exists
        // Semantically I'm not sure if the file should ever not exist at this point but who knows.
        println!("  Skipping non-existing file...");
        return false;
    }

    let _path_abs = match path.canonicalize() {
        Ok(ref canon) => { // TODO: I don't know why `ref` is required here but not below
            let c = canon.to_string_lossy();
            println!("  Absolute: {}", c);
            c
        },
        _ => {
            // Can this happen?
            println!("  Failed to get canonical path");
            return false;
        },
    };

    // The only realistic error here is an invalid utf name and even that is unlikely. Ignore errors
    let file_base = path.file_name().unwrap_or(OsStr::new("<err>")).to_string_lossy();
    let file_stem = path.file_stem().unwrap_or(OsStr::new("<err>")).to_string_lossy();
    let file_ext = path.extension().unwrap_or(OsStr::new("<err>")).to_string_lossy();
    println!("  Base: {}, stem: {}, ext: {}", file_base, file_stem, file_ext);

    let data = fs::read_to_string(path).expect("Unable to read file").to_owned();
    println!("  File size: {} bytes", data.len());

    // At this point it should be a legit file of <=500k and it should've been read into mem.
    // The checks are grouped based on the file/dir/extension

    if file_ext == "jpg" || file_ext == "gif" || file_ext == "png" || file_ext == "svg" || file_ext == "jpeg" || file_ext == "webm" || file_ext == "mpg" || file_ext == "mp3" || file_ext == "mp4" {
        // Note: this heuristic is very shallow as the extension of a file gives zero
        //       guarantees. It's just a name. That said, it's fine until proven otherwise.
        println!("  Skipping media file");
        return false;
    }

    // https://stackoverflow.com/questions/42101070/how-to-match-a-file-extension-represented-as-an-osstr/42101478
    // meh. so either complex match or simple if-elses (but what's more efficient and is it noticable here?)
    if file_base == "package.json" || file_base == "build.sh" || file_base == "Dockerfile" || file_base == "app.js" {
        // CONFIG_FILE_NAMES

        // TODO: benchmark difference in doing the needle prescan before the a.{0,20}abc capture, or no prescan
        if RULE_MINER_CONFIG_NEEDLES.is_match(&data) {
            match RULE_MINER_CONFIG_MATCH.find(&data) {
                Some(range) => {
                    println!("  normal::crypto_names:: {}", range.as_str());
                    return true;
                },
                None => (),
            }
        }

        if RULE_MINE_SHORT_NEEDLES.is_match(&data) {
            match RULE_MINE_SHORT_MATCH.find(&data) {
                Some(range) => {
                    let found = range.as_str();
                    match found {
                        | "spacexlaunches"
                        | "pxlapp"
                        | "Alexlander"
                        | "exlandingpage"
                        | "pxlayer"
                        | "xmrdt"
                        | "flexlayout"
                        | "rexlaunch"
                        | "xlanor"
                        | "maxLambdaSize"
                        => {
                            // Ignore this occurrence
                        }
                        _ => {
                            if file_base == "package.json" {
                                // Only check for xmr/xla on the root package.json (it's very noisy but it has to be this narrow)
                                // There's a font and an innocent monero package that shows up quite frequent
                                println!("  normal::crypto_names:: {}", found);
                            } else {
                                // Trial: consider noisy in other files but who knows
                                println!("  noisy::crypto_names:: {}", found);
                            }
                            return true;
                        }
                    }
                },
                None => (),
            }
        }
    }

    if file_ext == "go" || file_ext == "py" || file_ext == "js" || file_ext == "ts" {
        println!("  Miner keywords:");
        let b = RULE_MINER_KEYWORDS.is_match(&data);
        println!("    Result: {}", b);
    }

    if file_ext == "go" || file_ext == "py" || file_ext == "js" || file_ext == "ts" {
        println!("  Crypto miner names:");
        let b = RULE_MINER_NAMES.is_match(&data);
        println!("    Result: {}", b);
    }

    return false;
}
