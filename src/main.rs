use std::fs;
use std::fmt::Write;
use std::path::{PathBuf, Path};
use structopt::*;
use std::str::FromStr;

#[derive(Debug, StructOpt)]
#[structopt(name = "Obrez", about = "Obrez system")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    size: usize,
    #[structopt(default_value = "center")]
    horizontal: HorizontalPosition,
    #[structopt(default_value = "center")]
    vertical: VerticalPosition,
}

#[derive(Debug, Clone, Copy)]
pub enum HorizontalPosition {
    Left,
    Center,
    Right
}

impl FromStr for HorizontalPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "center" => Ok(Self::Center),
            _ => Err("Shoud be one of variant: left, center or right".to_owned())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalPosition {
    Top,
    Center,
    Bottom,
}

impl FromStr for VerticalPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "top" => Ok(Self::Top),
            "center" => Ok(Self::Center),
            "bottom" => Ok(Self::Bottom),
            _ => Err("Shoud be one of variant: top, center or bottom".to_owned())
        }
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub id: i32,
    pub data: Vec<String>,
}

fn load_from_file(filename: impl AsRef<Path>) -> Vec<Row> {
    let data = fs::read_to_string(filename).expect("Error on open file");
    let lines: Vec<_> = data.lines()
        .skip_while(|line| !line.contains("[parts]"))
        .skip(1)
        .collect();
    lines
        .iter()
        .map(|l| l.split_terminator("\t"))
        .map(|mut parts| (parts.next().expect("error").parse::<i32>().expect("Not a number"), parts.map(|s| s.to_owned()).collect::<Vec<_>>()))
        .map(|(id, data)| Row { id, data })
        .collect()
}

fn get_subsystem(system: &Vec<Row>, size: usize, horizontal: HorizontalPosition, vertical: VerticalPosition) -> Vec<Row> {
    let original_size = ((system.len() / 5) as f32).sqrt() as usize;
    assert!(original_size > size);
    let center = original_size / 2;
    let offset = size / 2;

    let (y_start, y_end) = match horizontal {
        HorizontalPosition::Left => (0, size),
        HorizontalPosition::Center => if size % 2 == 0 {
            ((center - offset), (center + offset))
        } else {
            ((center - offset), (center + offset + 1))
        }
        HorizontalPosition::Right => (original_size - size, original_size)
    };

    let (x_start, x_end) = match vertical {
        VerticalPosition::Top => (0, size),
        VerticalPosition::Center => if size % 2 == 0 {
            ((center - offset), (center + offset))
        } else {
            ((center - offset), (center + offset + 1))
        }
        VerticalPosition::Bottom => (original_size - size, original_size),
    };

    let mut out_system = vec![];
    for y in y_start..y_end {
        for x in x_start..x_end {
            let index = (y * original_size + x) * 5;
            let new_index = ((y - y_start) * size + (x - x_start)) * 5;
            dbg!(index);
            dbg!(new_index);
            out_system.extend((0..5)
                .map(|i| Row { id: (new_index + i) as i32, ..system[index + i].clone() })
            );
        }
    }
    out_system
}

fn save_system(filename: impl AsRef<Path>, system: &Vec<Row>) {
    let mut buffer = String::new();
    writeln!(buffer, "[header]").expect("Error");
    writeln!(buffer, "dimensions=2").expect("Error");
    writeln!(buffer, "size={}", system.len()).expect("Error");
    let state = system.iter()
        .map(|r| r.data.last().unwrap())
        .fold(String::new(), |acc, part| acc + part);
    writeln!(buffer, "state={}", state).expect("Error");
    writeln!(buffer, "[parts]").expect("Error");
    for row in system {
        let tail = row.data.join("\t");
        writeln!(buffer, "{}\t{}", row.id, tail).expect("Error");
    }

    fs::write(filename, buffer).expect("Error on write to file");
}

fn main() {
    let opt = Opt::from_args();

    let rows = load_from_file(&opt.input);
    let subsystem = get_subsystem(&rows, opt.size, opt.horizontal, opt.vertical);
    save_system(&opt.output, &subsystem);
}
