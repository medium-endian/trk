use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{Local, TimeZone};
use chrono::Duration;

use nom::IResult::Done;
use nom;

/* For running git and html-tidy */
use std::process::Command;

use std::env;

/* For from::utf8 */
use std::str;

pub fn get_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn ts_to_date(timestamp: u64) -> String {
    Local
        .timestamp(timestamp as i64, 0)
        .format("%Y-%m-%d, %H:%M")
        .to_string()
}

pub fn sec_to_hms_string(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds - hours * 3600) / 60;
    let seconds = seconds - minutes * 60 - hours * 3600;
    match (hours, minutes, seconds) {
        (0, 0, 1) => String::from("1 second"),
        (0, 0, s) => format!("{} seconds", s),
        (0, 1, _) => String::from("1 minute"),
        (0, m, _) => format!("{} minutes", m),
        /* Range matching: slightly dubious feature here */
        (1, 0..=4, _)   => String::from("1 hour"),
        (h, 0..=4, _)   => format!("{} hours", h),
        (h, 56..=59, _) => format!("{} hours", h + 1),
        (h, m, _)       => format!("{} hours and {} minutes", h, m),
    }
}

/* For parsing time in HH:MM format. */
named!(duration_hhmm(&[u8]) -> Duration,
    do_parse!(
        hour: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        tag!(":") >>
        min:  map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        /*tag!(":") >>
        sec: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>*/
        (Duration::minutes(hour * 60 + min))
    )
);

pub fn parse_hhmm_to_seconds(timestr: &str) -> Option<u64> {
    match duration_hhmm(timestr.as_bytes()) {
        Done(_, out) => Some(out.num_seconds() as u64),
        _ => None,
    }
}

pub fn set_to_trk_dir() -> bool {
    let mut path = env::current_dir().unwrap();
    loop {
        path.push(".trk");
        if path.exists() {
            path.pop();
            return env::set_current_dir(&path).is_ok()
        } else {
            path.pop();
            if !path.pop() {
                println!("Fatal: not a .trk directory (or subdirectory of one).");
                return false;
            }
        }
    }
}

pub fn git_init_trk() -> bool {
    if !set_to_trk_dir() {
        println!("Could not initialise trk internal git repo!\
                 (couldn't find upper level .trk dir).");
        return false;
    }

    let mut path = env::current_dir().unwrap();
    path.push(".trk");
    if path.exists() {
        if env::set_current_dir(&path).is_err() {
            return false;
        }
    } else {
        println!("Couldn't access .trk sub directory to initialise trk internal git repo.");
        return false;
    }
    let output = Command::new("git")
           .arg("init")
           .output();
    match output {
        Ok(_) => {},
        Err(_) => {
            println!("Could not run git init!");
            return false;
        }
    }
    let output = Command::new("git")
           .arg("add")
           .arg("timesheet.json")
           .output();
    match output {
        Ok(_) => {},
        Err(_) => {
            println!("Could not run git init!");
            return false;
        }
    }

    /* Reset current_dir to previous value */
    path.pop();
    env::set_current_dir(&path).is_ok()
}

pub fn git_commit_trk(message: &str) -> bool {
    if !set_to_trk_dir() {
        println!("Could not commit to trk internal git repo!\
                 (couldn't find upper level .trk dir).");
        return false;
    }

    let mut p = env::current_dir().unwrap();
    p.push(".trk");
    if p.exists() {
        if env::set_current_dir(&p).is_err() {
            return false
        }
    } else {
        println!("Couldn't access .trk sub directory to commit to trk internal git repo.");
        return false;
    }
    let output = Command::new("git")
           .arg("commit")
           .arg("timesheet.json")
           .arg("-m")
           .arg(message)
           .output();
    match output {
        Ok(_) => {},
        Err(_) => {
            println!("Could not run git commit!");
            return false;
        }
    }

    /* Reset current_dir to previous value */
    p.pop();
    env::set_current_dir(&p).is_ok()
}

pub fn git_pull() -> bool {
    if !set_to_trk_dir() {
        println!("Could not pull from git repo!\
                 (couldn't find upper level .trk dir).");
        return false;
    }

    let mut p = env::current_dir().unwrap();
    p.push(".trk");
    if p.exists() {
        if env::set_current_dir(&p).is_err() {
            return false
        }
    } else {
        println!("Couldn't access .trk sub directory to pull from upstream .trk git repo.");
        return false;
    }
    let output = Command::new("git")
           .arg("pull")
           .output();
    match output {
        Ok(_) => {},
        Err(_) => {
            println!("Could not run git pull!");
            return false;
        }
    }

    /* Reset current_dir to previous value */
    p.pop();
    env::set_current_dir(&p).is_ok()
}
pub fn git_push() -> bool {
    if !set_to_trk_dir() {
        println!("Could not push to git repo!\
                 (couldn't find upper level .trk dir).");
        return false;
    }

    let mut p = env::current_dir().unwrap();
    p.push(".trk");
    if p.exists() {
        if env::set_current_dir(&p).is_err() {
            return false
        }
    } else {
        println!("Couldn't access .trk sub directory to push to upstream .trk git repo.");
        return false;
    }
    let output = Command::new("git")
           .arg("push")
           .output();
    match output {
        Ok(_) => {},
        Err(_) => {
            println!("Could not run git push!");
            return false;
        }
    }

    /* Reset current_dir to previous value */
    p.pop();
    env::set_current_dir(&p).is_ok()
}

pub fn git_author() -> Option<String> {
    if let Ok(output) = Command::new("git")
           .arg("config")
           .arg("user.name")
           .output() {
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            /* Remove trailing newline character */
            let mut output = output.to_string();
            output
                .pop()
                .expect("Empty name in git config? Not even a newline?!?");
            Some(output)
        } else {
            let output = String::from_utf8_lossy(&output.stderr);
            println!("git config user.name failed. {}", output);
            None
        }
    } else {
        None
    }
}

pub fn git_commit_message(hash: &str) -> Option<String> {
    if let Ok(output) = Command::new("git")
           .arg("log")
           .arg("--format=%B")
           .arg("-n")
           .arg("1")
           .arg(hash)
           .output() {
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            Some(output.to_string())
        } else {
            let output = String::from_utf8_lossy(&output.stderr);
            println!("git log --format=%B -n 1 <hash> failed. {}", output);
            None
        }
    } else {
        None
    }
}

pub fn format_file(filename: &str) {
    if Command::new("tidy")
           .arg("--tidy-mark")
           .arg("no")
           .arg("-i")
           .arg("-m")
           .arg(filename)
           .output().is_ok() {
    } else {
        println!("tidy-html not found!");
    }
}
