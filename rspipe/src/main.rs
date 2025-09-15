use clap::{Arg, ArgAction, Command};
use rustsynth::{
    core::{CoreCreationFlags, CoreRef},
    map::OwnedMap,
    vsscript::Environment,
};
use std::collections::HashMap;
use std::io::{self, BufWriter, Write};
use std::process;
use std::sync::{Arc, Mutex};

mod output;
mod progress;

use output::OutputWriter;
use progress::ProgressTracker;

fn main() {
    let matches = Command::new("rspipe")
        .version("0.1.0")
        .disable_version_flag(true)
        .about("VapourSynth script processor using rustsynth")
        .arg(
            Arg::new("script")
                .help("VapourSynth script file (.vpy)")
                .required_unless_present("version")
                .index(1),
        )
        .arg(
            Arg::new("outfile")
                .help("Output file (use '-' for stdout, '--' for no output)")
                .required_unless_present("version")
                .required_unless_present("info")
                .index(2),
        )
        .arg(
            Arg::new("arg")
                .short('a')
                .long("arg")
                .help("Argument to pass to the script environment")
                .value_name("key=value")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("start")
                .short('s')
                .long("start")
                .help("Set output frame/sample range start")
                .value_name("N")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("end")
                .short('e')
                .long("end")
                .help("Set output frame/sample range end (inclusive)")
                .value_name("N")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("outputindex")
                .short('o')
                .long("outputindex")
                .help("Select output index")
                .value_name("N")
                .value_parser(clap::value_parser!(i32))
                .default_value("0"),
        )
        .arg(
            Arg::new("requests")
                .short('r')
                .long("requests")
                .help("Set number of concurrent frame requests")
                .value_name("N")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("container")
                .short('c')
                .long("container")
                .help("Add headers for the specified format to the output")
                .value_name("FORMAT")
                .value_parser(["y4m", "wav", "w64"]),
        )
        .arg(
            Arg::new("progress")
                .short('p')
                .long("progress")
                .help("Print progress to stderr")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("info")
                .short('i')
                .long("info")
                .help("Print all set output node info and exit")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .help("Show version info and exit")
                .action(ArgAction::SetTrue)
                .global(true),
        )
        .get_matches();

    if matches.get_flag("version") {
        println!("rspipe 0.1.0 (rustsynth {})", rustsynth::api_version());
        return;
    }

    let script_path = matches.get_one::<String>("script").unwrap();
    let binding = "--".to_string();
    let outfile = matches.get_one::<String>("outfile").unwrap_or(&binding);

    // Initialize VapourSynth
    let core = CoreRef::new(CoreCreationFlags::NONE);
    let mut environment = match Environment::new(&core) {
        Ok(env) => env,
        Err(e) => {
            eprintln!("Failed to initialize VapourSynth environment: {}", e);
            process::exit(1);
        }
    };

    Environment::load_api(core.info().api_version);
    // Set script arguments
    if let Some(args) = matches.get_many::<String>("arg") {
        let mut script_args = HashMap::new();
        for arg in args {
            if let Some((key, value)) = arg.split_once('=') {
                script_args.insert(key.to_string(), value.to_string());
            } else {
                eprintln!("Invalid argument format: {}. Use key=value", arg);
                process::exit(1);
            }
        }

        let mut vars_map = OwnedMap::new();
        for (key, value) in script_args {
            if let Err(e) = vars_map.set(&key, &value) {
                eprintln!("Failed to set script variable {}: {}", key, e);
                process::exit(1);
            }
        }
        if let Err(e) = environment.set_variables(&vars_map) {
            eprintln!("Failed to set script variables: {}", e);
            process::exit(1);
        }
    }

    // Evaluate script
    if let Err(e) = environment.eval_file(script_path) {
        eprintln!("Script evaluation failed: {}", e);
        process::exit(1);
    }

    let output_index = *matches.get_one::<i32>("outputindex").unwrap();
    let node = match environment.get_output(output_index) {
        Some(node) => node,
        None => {
            eprintln!("No output node found at index {}", output_index);
            process::exit(1);
        }
    };

    // Handle info mode
    if matches.get_flag("info") {
        print_node_info(&node);
        return;
    }

    let video_info = match node.video_info() {
        Some(info) => info,
        None => {
            eprintln!("Node has no video info (audio nodes not yet supported)");
            process::exit(1);
        }
    };

    // Determine frame range
    let start_frame = matches.get_one::<usize>("start").copied().unwrap_or(0);
    let end_frame = matches
        .get_one::<usize>("end")
        .copied()
        .unwrap_or((video_info.num_frames - 1) as usize);

    if start_frame > end_frame {
        eprintln!("Start frame cannot be greater than end frame");
        process::exit(1);
    }

    let total_frames = end_frame - start_frame + 1;

    // Set up output writer
    let mut writer = match OutputWriter::new(outfile, matches.get_one::<String>("container")) {
        Ok(writer) => writer,
        Err(e) => {
            eprintln!("Failed to create output writer: {}", e);
            process::exit(1);
        }
    };

    // Write container header if needed
    if let Err(e) = writer.write_header(&video_info) {
        eprintln!("Failed to write container header: {}", e);
        process::exit(1);
    }

    // Set up progress tracking
    let mut progress = ProgressTracker::new(total_frames, matches.get_flag("progress"));

    // Process frames concurrently
    let num_requests = *matches
        .get_one::<usize>("requests")
        .unwrap_or(&environment.get_core().info().num_threads);
    process_frames_concurrent(
        &node,
        &mut writer,
        start_frame,
        end_frame,
        num_requests,
        &mut progress,
    );

    progress.finish();

    if let Err(e) = writer.finish() {
        eprintln!("Failed to finish output: {}", e);
        process::exit(1);
    }
}

fn process_frames_concurrent(
    node: &rustsynth::node::Node,
    writer: &mut OutputWriter,
    start_frame: usize,
    end_frame: usize,
    num_requests: usize,
    progress: &mut ProgressTracker,
) {
    use std::sync::mpsc;

    let total_frames = end_frame - start_frame + 1;
    let (tx, rx) = mpsc::channel::<(usize, Result<rustsynth::frame::Frame, String>)>();
    let node_clone = node.clone();

    // Track pending requests
    let pending_requests = Arc::new(Mutex::new(0));

    // Start initial batch of async frame requests
    let mut next_request = start_frame;

    // Request initial batch
    for _ in 0..num_requests.min(total_frames) {
        *pending_requests.lock().unwrap() += 1;
        let tx_clone = tx.clone();
        let node_clone = node_clone.clone();
        let pending_clone = Arc::clone(&pending_requests);
        let frame_num = next_request;
        next_request += 1;

        node_clone.get_frame_async(frame_num, move |result, n, _| {
            let result_owned = match result {
                Ok(frame) => Ok(frame),
                Err(e) => Err(format!("Frame error: {}", e)),
            };
            tx_clone.send((n as usize, result_owned)).unwrap();
            *pending_clone.lock().unwrap() -= 1;
        });
    }

    // Collect and write frames in order
    let mut frames_received = HashMap::new();
    let mut next_frame = start_frame;
    let mut frames_written = 0;

    while frames_written < total_frames {
        if let Ok((frame_num, result)) = rx.recv() {
            match result {
                Ok(frame) => {
                    frames_received.insert(frame_num, frame);

                    // Request next frame if we haven't requested all frames yet
                    if next_request <= end_frame {
                        *pending_requests.lock().unwrap() += 1;
                        let tx_clone = tx.clone();
                        let node_clone = node_clone.clone();
                        let pending_clone = Arc::clone(&pending_requests);
                        let frame_num_to_request = next_request;
                        next_request += 1;

                        node_clone.get_frame_async(frame_num_to_request, move |result, n, _| {
                            let result_owned = match result {
                                Ok(frame) => Ok(frame),
                                Err(e) => Err(format!("Frame error: {}", e)),
                            };
                            tx_clone.send((n as usize, result_owned)).unwrap();
                            *pending_clone.lock().unwrap() -= 1;
                        });
                    }

                    // Write frames in sequential order
                    while let Some(frame) = frames_received.remove(&next_frame) {
                        if let Err(e) = writer.write_frame(&frame) {
                            eprintln!("Failed to write frame {}: {}", next_frame, e);
                            process::exit(1);
                        }

                        frames_written += 1;
                        next_frame += 1;

                        progress.update(frames_written);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get frame {}: {}", frame_num, e);
                    process::exit(1);
                }
            }
        }
    }
}

fn print_node_info(node: &rustsynth::node::Node) {
    let mut writer = BufWriter::new(io::stderr());

    if let Some(video_info) = node.video_info() {
        writeln!(writer, "Width: {}", video_info.width).unwrap();
        writeln!(writer, "Height: {}", video_info.height).unwrap();
        writeln!(writer, "Frames: {}", video_info.num_frames).unwrap();
        writeln!(writer, "FPS: {}/{}", video_info.fps_num, video_info.fps_den).unwrap();
        writeln!(
            writer,
            "Format Name: {}",
            video_info
                .format
                .get_name()
                .unwrap_or("Unknown".to_string())
        )
        .unwrap();
        writeln!(writer, "Color Family: {:?}", video_info.format.color_family).unwrap();
        writeln!(writer, "Sample Type: {:?}", video_info.format.sample_type).unwrap();
        writeln!(
            writer,
            "Bits Per Sample: {}",
            video_info.format.bits_per_sample
        )
        .unwrap();
        writeln!(
            writer,
            "Bytes Per Sample: {}",
            video_info.format.bytes_per_sample
        )
        .unwrap();
        writeln!(
            writer,
            "Subsampling W: {}",
            video_info.format.sub_sampling_w
        )
        .unwrap();
        writeln!(
            writer,
            "Subsampling H: {}",
            video_info.format.sub_sampling_h
        )
        .unwrap();
        writeln!(writer, "Num Planes: {}", video_info.format.num_planes).unwrap();
    } else if let Some(audio_info) = node.audio_info() {
        writeln!(writer, "Sample Rate: {}", audio_info.sample_rate).unwrap();
        writeln!(writer, "Num Samples: {}", audio_info.num_samples).unwrap();
        writeln!(writer, "Num Channels: {}", audio_info.format.num_channels).unwrap();
        writeln!(
            writer,
            "Channel Layout: {}",
            audio_info.format.channel_layout
        )
        .unwrap();
        writeln!(
            writer,
            "Format Name: {}",
            audio_info
                .format
                .get_name()
                .unwrap_or("Unknown".to_string())
        )
        .unwrap();
        writeln!(writer, "Sample Type: {:?}", audio_info.format.sample_type).unwrap();
        writeln!(
            writer,
            "Bits Per Sample: {}",
            audio_info.format.bits_per_sample
        )
        .unwrap();
        writeln!(
            writer,
            "Bytes Per Sample: {}",
            audio_info.format.bytes_per_sample
        )
        .unwrap();
    }

    writer.flush().unwrap();
}
