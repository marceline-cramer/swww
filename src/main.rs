use fork;
use nix::{sys::signal, unistd::Pid};
use structopt::StructOpt;
mod daemon;

const PID_FILE: &str = "/tmp/fswww/pid";

#[derive(Debug, StructOpt)]
#[structopt(
    name = "fswww",
    about = "The Final Solution to your Wayland Wallpaper Woes"
)]
enum Fswww {
    ///Initialize the daemon. Exits if there is already a daemon running
    Init {
        ///Don't fork the daemon. This will keep it running in the current
        ///terminal, so you can track its log, for example
        #[structopt(long)]
        no_daemon: bool,
    },

    ///Kills the daemon
    Kill,

    /// Send an img for the daemon to display
    Img { path: String },
}

fn main() {
    let opts = Fswww::from_args();
    match opts {
        Fswww::Init { no_daemon } => {
            if !no_daemon {
                if let Ok(fork::Fork::Child) = fork::daemon(false, false) {
                    daemon::main();
                } else {
                    eprintln!("Couldn't fork process!");
                }
            } else {
                daemon::main();
            }
        }
        Fswww::Kill => kill(),
        Fswww::Img { path } => send_img(&path),
    }
}

fn send_img(path: &str) {
    let pid = get_daemon_pid();
    let mut img_path = path.to_string();
    img_path.push('\n');
    std::fs::write("/tmp/fswww/in", img_path)
        .expect("Couldn't write to /tmp/fswww/in. Did you delete the file?");

    signal::kill(Pid::from_raw(pid), signal::SIGUSR1).expect("Failed to send signal.");
}

fn kill() {
    let pid = get_daemon_pid();

    signal::kill(Pid::from_raw(pid), signal::SIGKILL).expect("Failed to kill daemon...");

    std::fs::remove_dir_all("/tmp/fswww").expect("Failed to remove /tmp/fswww directory.");

    println!("Successfully killed fswww daemon and removed /tmp/fswww directory!");
}

fn get_daemon_pid() -> i32 {
    let pid_file_path = std::path::Path::new(PID_FILE);
    if !pid_file_path.exists() {
        panic!(
            "pid file {} doesn't exist. Are you sure the daemon is running?",
            PID_FILE
        );
    }
    std::fs::read_to_string(pid_file_path)
        .expect("Failed to read pid file")
        .parse()
        .unwrap()
}
