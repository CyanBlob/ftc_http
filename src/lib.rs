extern crate reqwest;
extern crate walkdir;

use std::io::{stdout, Read, Write};
use walkdir::{DirEntry, WalkDir};
use std::time::Duration;
use std::process;
use reqwest::*;
use std::fs;

static HOST: &'static str = "http://192.168.49.1:8080";
const TIMEOUT: u64 = 3;

pub fn up(src: &std::path::Path) {
    conn_test();
    let phone = Client::new();

    fn is_src_file(entry: &DirEntry) -> bool {
        entry.file_name().to_str().unwrap().contains(".java")
    }

    let src_tree = WalkDir::new(&src)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(is_src_file);

    for file in src_tree.map(|f| f.path().to_owned()) {
        print!("Pushing {}...", file.display());
        stdout().flush().unwrap();
        let upload = multipart::Form::new()
            .file("file", file)
            .expect("Failed to open file for uploading.");
        phone
            .post(&(HOST.to_string() + "/java/file/upload"))
            .multipart(upload)
            .send()
            .expect("HTTP request failed");
        println!("done");
    }
}

pub fn down(dest: &std::path::Path) {
    conn_test();
    let phone = Client::new();

    let mut tree = String::new();

    phone
        .get(&(HOST.to_string() + "/java/file/tree"))
        .send()
        .expect("HTTP request failed")
        .read_to_string(&mut tree)
        .expect("Couldn't read HTTP response");

    for file in tree.split("\"").filter(|file| file.contains(".java")) {
        print!("Pulling {}...", file);
        stdout().flush().unwrap();
        let filepath = dest.join(&file[1..]);
        fs::DirBuilder::new()
            .recursive(true)
            .create(filepath.parent().unwrap())
            .expect("Creating a new directory failed");
        let mut file_handle = fs::File::create(&filepath).expect("Creating a new file failed");
        let mut file_data = String::new();
        phone
            .get(&(HOST.to_string() + "/java/file/download?f=/src" + file))
            .send()
            .expect("HTTP request failed")
            .read_to_string(&mut file_data)
            .expect("Couldn't read HTTP response");
        file_handle
            .write_all(file_data.as_bytes())
            .expect("Writing to file failed");
        println!("done");
    }
}

pub fn build() {
    conn_test();
    let phone = Client::new();

    phone
        .get(&(HOST.to_string() + "/java/file/tree"))
        .send()
        .expect("HTTP request failed");
    phone
        .get(&(HOST.to_string() + "/java/build/start"))
        .send()
        .expect("HTTP request failed");

    print!("Building...");
    stdout().flush().unwrap();
    let mut status = String::new();

    loop {
        phone
            .get(&(HOST.to_string() + "/java/build/status"))
            .send()
            .expect("HTTP request failed")
            .read_to_string(&mut status)
            .expect("Couldn't read HTTP response");

        if status.contains("\"completed\": true") {
            break;
        } else {
            print!(".");
            stdout().flush().unwrap();
        }
    }

    if status.contains("\"successful\": true") {
        println!("BUILD SUCCESSFUL");
    } else {
        println!("BUILD FAILED");
        let mut error = String::new();
        phone
            .get(&(HOST.to_string() + "/java/build/wait"))
            .send()
            .expect("HTTP request failed")
            .read_to_string(&mut error)
            .expect("Couldn't read HTTP response");
        print!("{}", error);
    }
}

pub fn wipe() {
    conn_test();
    let phone = Client::new();

    print!("Wiping all remote files...");
    stdout().flush().unwrap();
    phone
        .post(&(HOST.to_string() + "/java/file/delete"))
        .form(&[("delete", "[\"src\"]")])
        .send()
        .expect("HTTP request failed");
    println!("done");
}

fn conn_test() {
    let phone = Client::builder()
        .timeout(Duration::from_secs(TIMEOUT))
        .build()
        .unwrap();

    phone.get(HOST).send().unwrap_or_else(|_| {
        println!(
            "Failed to reach the robot controller. Please check that your robot controller\n\
             is in \"Program & Manage\" mode and that your computer is connected to the\n\
             robot controller via wifi-direct."
        );
        process::exit(1)
    });
}
