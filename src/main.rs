#![allow(dead_code,unused_imports)]

use std::{self, convert::TryInto, fs::{self, File}, io::{self, Read, Seek, SeekFrom}, path::Path, process::{self, Command}, thread, time::Duration};

fn get_module_base(pid: u32, name: &str) -> usize {
    for maps in get_process_maps(pid) {
        for line in maps {
            if line.filename().as_deref().unwrap_or("") == name
                && line.is_read()
                && line.is_write()
                && !line.is_exec()
            {
                return line.start();
            }
        }
    }
    0
}

fn findpid(name: &str) -> u32 {
    let mut pid: u32 = 0;
    for process in fs::read_dir("/proc").unwrap() {
        let comm = format!("{}/comm", process.unwrap().path().display());
        let file = File::open(Path::new(&comm));
        if let Ok(mut f) = file {
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            if s.trim() == name {
                let split: Vec<&str> = comm.split("/").collect();
                pid = split[2].parse().unwrap();
                break;
            }
        }
    }
    pid
}

#[derive(Debug, Clone, PartialEq)]
struct MapRange {
    range_start: usize,
    range_end: usize,
    offset: usize,
    dev: String,
    flags: String,
    inode: usize,
    pathname: Option<String>,
}

impl MapRange {
    fn size(&self) -> usize {
        self.range_end - self.range_start
    }
    fn start(&self) -> usize {
        self.range_start
    }
    fn filename(&self) -> &Option<String> {
        &self.pathname
    }
    fn is_exec(&self) -> bool {
        &self.flags[2..3] == "x"
    }
    fn is_write(&self) -> bool {
        &self.flags[1..2] == "w"
    }
    fn is_read(&self) -> bool {
        &self.flags[0..1] == "r"
    }
}

fn get_process_maps(pid: u32) -> std::io::Result<Vec<MapRange>> {
    let maps_file = format!("/proc/{}/maps", pid);
    let mut file = File::open(maps_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(parse_proc_maps(&contents))
}

fn parse_proc_maps(contents: &str) -> Vec<MapRange> {
    let mut vec: Vec<MapRange> = Vec::new();
    for line in contents.split("\n") {
        let mut split = line.split_whitespace();
        let range = split.next();
        if range == None {
            break;
        }
        let mut range_split = range.unwrap().split("-");
        let range_start = range_split.next().unwrap();
        let range_end = range_split.next().unwrap();
        let flags = split.next().unwrap();
        let offset = split.next().unwrap();
        let dev = split.next().unwrap();
        let inode = split.next().unwrap();

        vec.push(MapRange {
            range_start: usize::from_str_radix(range_start, 16).unwrap(),
            range_end: usize::from_str_radix(range_end, 16).unwrap(),
            offset: usize::from_str_radix(offset, 16).unwrap(),
            dev: dev.to_string(),
            flags: flags.to_string(),
            inode: usize::from_str_radix(inode, 10).unwrap(),
            pathname: Some(split.collect::<Vec<&str>>().join(" ")).filter(|x| !x.is_empty()),
        });
    }
    vec
}

/* fn main() {
    let pid = findpid("lll");

    thread::spawn(move || -> ! {
        let start = get_module_base(pid, "aa");

        loop {
            thread::sleep(Duration::from_millis(1000));

            let x = read_bytes(pid, start as u64 + 1234, 8);

            match x {
                Ok(mut i) => {
                    i.reverse();
                    println!("{:?}", i);
                    println!("{}", u64::from_be_bytes(vec_to_arr(i)))
                }
                Err(err) => println!("{}",err),
            }
        }
    });

    loop {
        thread::sleep(Duration::from_millis(1000));
        if findpid("lll") == 0 {
            process::exit(0);
        }
    }
}
 */
fn vec_to_arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

fn read_bytes(pid: u32, offset: u64, size: usize) -> Result<Vec<u8>, io::Error> {
    let path = format!("/proc/{}/mem", pid);
    let mut file = File::open(&Path::new(&path))?;
    file.seek(SeekFrom::Start(offset))?;
    let mut buffer = vec![0; size];
    file.read(&mut buffer)?;
    Ok(buffer)
}

fn get_bytes(pid: u32, offset: u64, size: usize) -> Result<Vec<u8>, io::Error> {
    let path = format!("/proc/{}/mem", pid);
    let mut file = File::open(&Path::new(&path))?;
    file.seek(SeekFrom::Start(offset))?;
    let mut buffer = vec![0; size];
    file.read(&mut buffer)?;
    Ok(buffer)
}

fn game_safe(){
    Command::new("sh")
            .arg("-c")
            .arg("echo 0 > /proc/sys/fs/inotify/max_user_watches")
            .output()
            .expect("sh command failed to start");
}

fn main(){

    game_safe();

    let pid = findpid("om.tencent.lolm");

    thread::spawn(move || -> ! {
        let start = get_module_base(pid, "[anon:libc_malloc]");
        println!("pid: {}, startaddr: 0x{:x}",pid,start);
        loop {
            thread::sleep(Duration::from_millis(1000));
            
            let x = read_bytes(pid, 0x4b1d3a4c, 8);

            match x {
                Ok(mut i) => {
                    i.reverse();
                    println!("{:?}", i);
                    println!("{}", u64::from_be_bytes(vec_to_arr(i)))
                }
                Err(err) => println!("{}",err),
            }
        }
    });

loop {
    thread::sleep(Duration::from_millis(1000));
    if findpid("om.tencent.lolm") == 0 {
        process::exit(0);
    }
}
}