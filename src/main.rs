extern crate toml;

//use std::env;
use std::str;
use std::process::Command;
use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::sync::Arc;

// This describes a process that we're responsible for running. We'll allocate
// one for each one them.
#[derive(Debug)]
struct Process {
    // A user-defined name of the process.
    name: String,
    // The path to the executable, e.g. /bin/ls.
    path: String,
    // A list of arguments to pass, e.g. [ "-a", "-l" ] or [].
    args: Vec<String>,
    // The time to sleep between restarts, in milliseconds. Defaults to 0.
    restart_delay: u64,
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
    thread::sleep(std::time::Duration::from_millis(10000));
}

// Keep the given process running. This should be run in a separate thread.
fn run (p: Arc<Process>) {
    thread::spawn(move || {
        loop {
            let mut child = launch(&*p);
            let status = child.wait().expect("Child wasn't running!");
            println!("child {} exited with status {}", p.name, status);
            thread::sleep(std::time::Duration::from_millis(p.restart_delay));
        }
    });
}

// Launch a program, given the Process struct, then return the Child.
fn launch(p: &Process) -> std::process::Child {
    println!("starting process {}", p.name);
    Command::new(&p.path)
        .args(&p.args)
        .spawn()
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
        let restart_delay = def.get("restart_delay")
            .and_then(|x| x.as_integer())
            .unwrap_or(0);
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
        }));
    }
}
