// Copyright (c) 2016, Randy Westlund. All rights reserved.
// This code is under the BSD-2-Clause license.

// Superv is a process supervisor written in Rust. See README.md for details.

extern crate toml;

//use std::env;
// For reading the config file.
use std::fs::File;
// For appending to log files.
use std::fs::OpenOptions;
// For File traits.
//use std::io::prelude::*;
// The trait for File.as_raw_fd().
use std::os::unix::io::AsRawFd;
// The trait for process::Stdio.from_raw_fd().
use std::os::unix::io::FromRawFd;
// For process management.
use std::process;
// For sharing a Process between threads.
use std::sync::Arc;
// For running multiple threads.
use std::thread;
// For sleeping.
use std::time;
use std::io::Read;

// This describes a process that we're responsible for running. We'll allocate
// one for each one them.
#[derive(Debug)]
struct Process {
    // A user-defined name of the process.
    name:           String,
    // The path to the executable, e.g. /bin/ls.
    path:           String,
    // A list of arguments to pass, e.g. [ "-a", "-l" ] or [].
    args:           Vec<String>,
    // The time to sleep between restarts, in milliseconds. Defaults to 0.
    restart_delay:  u64,
    // The current working directory of the process.
    cwd:            Option<String>,
    // Filenames for where to send stdout and stderr. Defaults to /dev/null.
    stdout:         String,
    stderr:         String,
}

fn main () {
    const CONF_FILENAME: &'static str = "superv.conf";
    // TODO having args will come later.
    //let args: Vec<String> = env::args().collect();
    //let progname = args[0].clone();

    // Create a list of processes to manage.
    let mut processes : Vec<Arc<Process>> = Vec::new();
    // Fill the list of processes from the config file.
    parse_config_file(CONF_FILENAME, &mut processes);

    for p in processes {
        // Launch the program.
        run(p.clone());
    }
    thread::sleep(time::Duration::from_millis(10000));
}

// Keep the given process running. This should be run in a separate thread.
fn run (p: Arc<Process>) {
    thread::Builder::new()
        .name(p.name.clone())
        .spawn(move || {

            //TODO I can't get io redirection to work.
            // Open in append mode.
            //let mut stdout = OpenOptions::new()
                //.create(true)
                //.write(true)
                //.append(true)
                //.open(&p.stdout)
                //.expect(&*format!("Failed to open output file {}.", &p.stdout));

            loop {
                let mut child = launch(&*p);
                //TODO I can't get io redirection to work.
                //match child.stdout {
                    //Some(x) => std::io::copy(&mut x, &mut std::io::stdout()),
                    //None => Ok(0u64),
                //}
                //std::io::copy(&mut child.stdout.unwrap(), &mut std::io::stdout());
                //if p.stdout != "/dev/null" {
                    //std::io::copy(&mut child.stdout.unwrap(), &mut std::io::stdout());
                //}
                let status = child.wait().expect("Child wasn't running!");
                println!("child {} exited with status {}", p.name, status);
                thread::sleep(time::Duration::from_millis(p.restart_delay));
            }
        }).expect("Failed to spawn thread");
}

// Launch a program, given the Process struct, then return the Child.
fn launch(p: &Process) -> process::Child {
    println!("starting process {}", p.name);
    let mut cmd = process::Command::new(&p.path);
    cmd.args(&p.args);
    cmd.stdin(process::Stdio::null());

    if (p.cwd).is_some() {
        cmd.current_dir(p.cwd.as_ref().unwrap());
    }

    // Open output files.
    if p.stdout == "/dev/null" {
        cmd.stdout(process::Stdio::null());
    }
    else {
        cmd.stdout(process::Stdio::piped());
        //TODO I can't get io redirection to work.
        //println!("Sending {} stdout to {}", &p.name, &p.stdout);
        // Open in append mode.
        //let mut stdout = OpenOptions::new()
            //.create(true)
            //.write(true)
            //.append(true)
            //.open(&p.stdout)
            //.expect(&*format!("Failed to open output file {}.", &p.stdout));
        //cmdstdout(unsafe { process::Stdio::from_raw_fd(stdout.as_raw_fd()) });
    }

    cmd.spawn()
        .expect("failed to run binary")
}

// Parse the config file (first arg), putting processes into the process list
// (second argument).
fn parse_config_file(filename: &str, processes: &mut Vec<Arc<Process>>) {
    // Open the config file.
    let mut f = File::open(filename)
        .expect(&*format!("Configuration file {} not found.", filename));

    // Read the config file.
    let mut conf = String::new();
    f.read_to_string(&mut conf)
        .expect(&*format!("Error reading configuration file {}", filename));

    // Parse the config file, returning a toml::Table.
    let conf = toml::Parser::new(&conf)
        .parse()
        .expect(&*format!("Failed to parse file {}", filename));

    // Get the table of defined processes.
    let data = conf.get("process").unwrap();
    // Iterate over all defined processes.
    let procs = data.as_table().unwrap();
    for a in procs.keys() {
        // Get a handle to the process Table.
        let def = procs.get(a)
            .unwrap()
            .as_table()
            .unwrap();
        let path = def.get("path")
            .unwrap()
            .as_str()
            .unwrap();
        let cwd = def.get("cwd")
            .and_then(|x| x.as_str())
            .and_then(|x| Some(x.to_string()));
        let restart_delay = def.get("restart_delay")
            .and_then(|x| x.as_integer())
            .unwrap_or(0);
        let stdout = def.get("stdout")
            .and_then(|x| x.as_str())
            .and_then(|x| Some(x.to_string()))
            .unwrap_or("/dev/null".to_string());
        let stderr = def.get("stderr")
            .and_then(|x| x.as_str())
            .and_then(|x| Some(x.to_string()))
            .unwrap_or("/dev/null".to_string());
        // Getting arguments is more complicated, both because it's optional,
        // and because it needs to be split to prevent the whole string from
        // being delivered as one argument.
        let args = def.get("args")
            .and_then(|x| x.as_str())
            .and_then(|x| Some(
                    x.split(' ')
                    .map(|x| x.to_string())
                    .collect()))
            .unwrap_or(Vec::new());
        // Now that we have all the info, add it to the list.
        processes.push(Arc::new(Process {
            name:           a.to_string(),
            path:           path.to_string(),
            args:           args,
            restart_delay:  restart_delay as u64,
            cwd:            cwd,
            stdout:         stdout,
            stderr:         stderr,
        }));
    }
}
