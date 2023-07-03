use std::{
    io::{self, Write},
    path::Path,
    path::PathBuf,
    process::{exit, Command},
};

use clap::Parser;

/// Streamlining video editing in the commandline
#[derive(Parser)]
#[command(author, about, long_about = None)]
#[command(next_line_help = true)]
struct Cli {
    /// Path to video file
    #[arg(short, long)]
    input: PathBuf,

    //Output file (default is Output)
    #[arg(short, long, default_value_t = String::from("Output."))]
    output: String,

    /// Speed multiplier
    #[arg(short, long, default_value_t = 1.0)]
    speed: f64,

    /// Speed multiplier
    #[arg(short, long, default_value_t = false)]
    mute: bool,

    ///Pitch adjustment in semitones (negative is dropped, positive is raised)
    #[arg(long, default_value_t = -1.0)]
    trim_end: f64,

    /// Trim x seconds from the start. Output will be from x seconds to the end of original file.
    #[arg(long, default_value_t = -1.0)]
    trim_start: f64,

    ///Trim to x seconds. Output will be from 0 to x seconds.
    #[arg(long, default_value_t = -1.0)]
    trim_to: f64,

    ///Pitch up adjustment in semitones
    #[arg(long, default_value_t = 1.0)]
    pitch_up: f64,

    ///Pitch up adjustment in semitones
    #[arg(long, default_value_t = 1.0)]
    pitch_down: f64,
}
fn main() {
    let args = Cli::parse();
    let video_file_ext = args.input.extension().unwrap().to_ascii_lowercase();
    // let tmp_video_file = "tmp".to_owned() + video_file_ext.to_str().unwrap();
    // let mut command_args = ["-i", &args.name.to_str().unwrap(), "-loglevel", "error"];
    let mut command_args = vec![
        "-i".to_owned(),
        args.input.to_str().unwrap().to_owned(),
        "-loglevel".to_owned(),
        "error".to_owned(),
    ];
    if video_file_ext != "mp4" && video_file_ext != "mov" {
        panic!("Not a video file");
    }

    if args.speed != 1.0 {
        speed(&mut command_args, &args.speed);
    }

    if args.pitch_up != 1.0 {
        pitch_up(&mut command_args, &args.pitch_up);
    }
    if args.pitch_down != 1.0 {
        pitch_down(&mut command_args, &args.pitch_down);
    }
    if args.trim_to != -1.0 && args.trim_to != 0.0 {
        trim_to(&mut command_args, args.trim_to);
    }
    if args.trim_start != -1.0 && args.trim_start != 0.0 {
        trim_start(&mut command_args, &args.trim_start);
    }
    if args.mute {
        mute(&mut command_args);
    }
    let mut output_file: PathBuf = PathBuf::from(&args.output);
    if output_file.to_str().unwrap() == "Output." {
        output_file.set_extension(&args.input.as_path().extension().unwrap());
    }

    let mut c: u16 = 0;
    let mut m = output_file.to_string_lossy().to_string();
    let mut changed_name: bool = false;
    while Path::new(&m).exists() {
        c += 1;
        changed_name = true;
        let mut n = String::from(output_file.file_stem().unwrap().to_str().unwrap());
        n.push_str(&c.to_string());
        n.push_str(&".".to_string());
        n.push_str(output_file.extension().unwrap().to_str().unwrap());
        m = n.into();
    }
    if changed_name {
        println!("{:?} already exists, writing to {}", &output_file, &m);
    }
    command_args.push("-c:v".to_string());
    command_args.push("copy".to_string());
    output_file = m.into();
    command_args.push(output_file.to_str().unwrap().to_string());

    println!("{:?}", &command_args);

    let loading_message = "Processing video...";

    print!("{}", loading_message);
    io::stdout().flush().unwrap(); // Ensure the loading message is displayed immediately

    let output = Command::new("ffmpeg")
        .args(&command_args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .expect("Failed to execute ffmpeg command");

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error executing FFmpeg command: {}", error_message);
        exit(1);
    }
    // Clear the loading message
    print!("\r{}", " ".repeat(loading_message.len()));
    io::stdout().flush().unwrap();

    // println!("FFmpeg command executed successfully");
    println!("{:?} has been written to", &output_file);
}

pub fn pitch_down(command_args: &mut Vec<String>, pitch: &f64) {
    let val: f64 = 2_f64.powf(*pitch as f64 / 12.0 * -1_f64);
    let mut rubberband: String = "rubberband=pitch=".to_string();
    rubberband.push_str(&val.to_string());
    command_args.push("-af".to_string());
    command_args.push(rubberband);
}

pub fn pitch_up(command_args: &mut Vec<String>, pitch: &f64) {
    let val: f64 = 2_f64.powf(*pitch as f64 / 12.0);
    let mut rubberband: String = "rubberband=pitch=".to_string();
    rubberband.push_str(&val.to_string());
    command_args.push("-af".to_string());
    command_args.push(rubberband);
}

pub fn trim_end(command_args: &mut Vec<String>) {
    command_args.insert(0, "-ss".to_string());
}
pub fn trim_start(command_args: &mut Vec<String>, seconds: &f64) {
    command_args.insert(0, "-ss".to_string());
    command_args.insert(1, seconds.to_string());
}
pub fn trim_to(command_args: &mut Vec<String>, seconds: f64) {
    command_args.push("-t".to_string());
    command_args.push(seconds.to_string());
}
pub fn mute(command_args: &mut Vec<String>) {
    command_args.push("-an".to_string());
}

pub fn speed(command_args: &mut Vec<String>, speed: &f64) {
    let pts_value = 1 as f64 / speed;
    let mut set_pts = String::from("setpts=");
    set_pts.push_str(&pts_value.to_string());
    set_pts.push_str("*PTS");
    let mut atempo = String::from("atempo=");
    atempo.push_str(&speed.to_string());

    command_args.push("-vf".to_string());
    command_args.push(set_pts);
    command_args.push("-af".to_string());
    command_args.push(atempo);
}
