extern crate toml;

//use std::env;
use std::str;
use std::process::Command;
use std::io::prelude::*;
use std::fs::File;

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
}

fn main () {
    const CONF_FILENAME: &'static str = "superv.conf";
    // TODO having args will come later.
    //let args: Vec<String> = env::args().collect();
    //let progname = args[0].clone();

    // Create a list of processes to manage.
    let mut processes : Vec<Process> = Vec::new();
    // Fill the list of processes from the config file.
    parse_config_file(CONF_FILENAME, &mut processes);

    for p in processes {
        // Launch the program.
        run(&p);
    }
}

// Launch a program, given the program name and arguments.
fn run(p: &Process) {
    println!("starting process {:?}", p);
    let child = Command::new(&p.path)
        .args(&p.args)
        .spawn()
        .expect("failed to run binary");
    println!("returned from spawning {}", p.name);
}

// Parse the config file (first arg), putting processes into the process list
// (second argument).
fn parse_config_file(filename: &str, processes: &mut Vec<Process>) {
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
        processes.push(Process {
            name: a.to_string(),
            path: path.to_string(),
            args: args
        });
    }
}
