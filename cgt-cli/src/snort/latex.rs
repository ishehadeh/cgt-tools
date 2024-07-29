use anyhow::{bail, Context, Result};
use cgt::numeric::rational::Rational;
use clap::Parser;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

use super::common::Log;

#[derive(Parser, Debug, Clone)]
/// Convert a log file (usually obtained from a genetic algorithm search) to a LaTeX table with images.
///
/// Requires `graphviz` to be installed.
pub struct Args {
    #[arg(long)]
    /// Input file with logs
    in_file: String,

    #[arg(long)]
    /// Output file with LaTeX table
    out_file: String,

    /// See https://graphviz.org/docs/outputs/
    #[arg(long, default_value = "svg")]
    image_format: String,

    /// See https://graphviz.org/docs/layouts/
    #[arg(long, default_value = "fdp")]
    graphviz_engine: String,

    /// Number of columns with images
    #[arg(long, default_value_t = 2)]
    columns: usize,

    /// Width of each individual image, height will be scaled to preserve ratio
    #[arg(long, default_value = "4cm")]
    image_width: String,

    /// Output image file name prefix
    #[arg(long, default_value = "out")]
    image_file_prefix: String,

    /// Fitness lower bound (inclusive)
    #[arg(long, default_value = None)]
    fitness_lower_bound: Option<Rational>,
}

pub fn run(args: Args) -> Result<()> {
    let input = BufReader::new(File::open(&args.in_file).context("Could not open input file")?);
    let mut output =
        BufWriter::new(File::create(&args.out_file).context("Could not open output file")?);

    let input: Result<Vec<Log>> = serde_json::de::Deserializer::from_reader(input)
        .into_iter()
        .map(|log| log.context("Could not decode input"))
        .collect();
    let input = input?;

    let entries = Mutex::new(Vec::new());
    let i = AtomicUsize::new(0);

    input.into_par_iter().try_for_each(|log| -> Result<()> {
        if let Log::HighFitness {
            position,
            temperature,
            degree,
            ..
        } = log
        {
            if args
                .fitness_lower_bound
                .map_or(false, |fitness_lower_bound| {
                    position.score < fitness_lower_bound
                })
            {
                return Ok(());
            }

            // Start engine with output file and no input
            let out_image_name = format!(
                "{}{}.{}",
                &args.image_file_prefix,
                i.fetch_add(1, Ordering::SeqCst),
                &args.image_format
            );
            eprintln!("Generating {}", out_image_name);

            let mut graphviz_proc = Command::new(&args.graphviz_engine)
                .stdin(Stdio::piped())
                .arg(format!("-T{}", &args.image_format))
                .arg(format!("-o{}", &out_image_name))
                .spawn()
                .context("Could not spawn graphviz")?;

            // Pipe dot to the running engine via stdin
            graphviz_proc
                .stdin
                .take()
                .context("Could not open graphviz stdin")?
                .write_all(position.object.to_graphviz().as_bytes())
                .context("Could not write to graphviz stdin")?;

            // Await result and check for errors
            if !graphviz_proc
                .wait()
                .context("Could not wait for graphviz")?
                .success()
            {
                bail!("Graphviz failed");
            };

            let mut es = entries.lock().unwrap();
            es.push((position, temperature, degree, out_image_name));
        }
        Ok(())
    })?;
    let mut entries = entries.lock().unwrap();

    // preamble
    writeln!(output, "{{")?;
    writeln!(output, "%% Auto generated by `cgt-cli`")?;
    writeln!(output, "%% Make sure to include preamble from README.md")?;

    let temperature_column_width = "0.75cm";
    let degree_column_width = "0.75cm";
    let fitness_column_width = "1.25cm";

    // table start
    write!(
        output,
        "\\begin{{longtabu}}{{m{{{}}} m{{{}}} m{{{}}} m{{{}}}",
        args.image_width, temperature_column_width, degree_column_width, fitness_column_width
    )?;
    for _ in 1..args.columns {
        write!(
            output,
            "|m{{{}}} m{{{}}} m{{{}}} m{{{}}}",
            args.image_width, temperature_column_width, degree_column_width, fitness_column_width
        )?;
    }
    write!(output, "}} \n\\hline ")?;

    // header
    for idx in 0..args.columns {
        if idx != 0 {
            write!(output, "& ")?;
        }
        write!(output, "Position & Temp. & Degree & Fitness")?;
    }
    writeln!(output, "\\\\ \\hline \\endhead")?;

    // entries

    // sort by fitness
    entries.sort_by_key(|entry| entry.0.score);
    entries.reverse();

    let mut entries = entries.iter().peekable();
    while entries.peek().is_some() {
        for idx in 0..args.columns {
            if let Some(entry) = entries.next() {
                if idx != 0 {
                    write!(output, "& ")?;
                }
                write!(
                    output,
                    "\\includegraphics[width={}]{{{}}} & ${}$ & ${}$ & ${}$ ",
                    args.image_width, entry.3, entry.1, entry.2, entry.0.score
                )?;
            };
        }
        writeln!(output, "\\\\")?;
    }

    writeln!(output, "\\end{{longtabu}}")?;
    writeln!(output, "}}")?;

    Ok(())
}
