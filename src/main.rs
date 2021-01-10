use std::fs;
use std::fmt::Write;
use std::path::{PathBuf, Path};
use structopt::*;
use std::str::FromStr;
use std::fs::File;
use std::io::Write as _;

type Vec3 = vek::Vec3<f64>;

#[derive(Debug, StructOpt)]
#[structopt(name = "Obrez", about = "Obrez system")]
enum Opt {
    Corner {
        #[structopt(parse(from_os_str))]
        input: PathBuf,
        #[structopt(parse(from_os_str))]
        output: PathBuf,
        size: usize,
        #[structopt(default_value = "center-center")]
        corner: Corner
    },
    Energy {
        #[structopt(parse(from_os_str))]
        input: PathBuf,
        #[structopt(parse(from_os_str))]
        output: PathBuf,
        #[structopt(short, long)]
        sizes: Vec<usize>,
        #[structopt(short, long)]
        corners: Vec<Corner>,
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Corner {
    pub horizontal: HorizontalPosition,
    pub vertical: VerticalPosition,
}

impl FromStr for Corner {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split_terminator("-").collect();
        if parts.len() != 2 {
            return Err("Should be horizontal-vertical".to_owned())
        };
        Ok(Self {
            horizontal: HorizontalPosition::from_str(parts[0])?,
            vertical: VerticalPosition::from_str(parts[1])?
        })
    }
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

pub struct System(pub Vec<Element>);

impl System {
    pub fn energy(&self) -> f64 {
        self.0.iter()
            .fold(0.0, |acc, elem| acc + self.0.iter()
                .fold(0.0, |acc, elem2| acc + elem.energy_with(elem2))
            ) / 2.0
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub id: i32,
    pub pos: Vec3,
    pub m: Vec3,
    pub state: bool,
}

impl Element {
    pub fn from_raw_row(row: &[&str]) -> Element {
        Element {
            id: row[0].parse().expect("Error on parse"),
            pos: Vec3::new(row[1].parse().expect("Error on parse"), row[2].parse().expect("Error on parse"), row[3].parse().expect("Error on parse")),
            m: Vec3::new(row[4].parse().expect("Error on parse"), row[5].parse().expect("Error on parse"), row[6].parse().expect("Error on parse")),
            state: row[7] == "1",
        }
    }

    pub fn energy_with(&self, element: &Element) -> f64 {
        let pij = self.pos - element.pos;

        let mi = self.m;
        let mj = element.m;

        let r = pij.magnitude();
        let r3 = r * r * r;
        let r5 = r3 * r * r;

        let result = (mi.dot(mj) / r3) - 3.0 * ((mi.dot(pij) * mj.dot(pij)) / r5);
        if result.is_nan() {
            0.0
        } else {
            result
        }
    }
}

fn load_from_file(filename: impl AsRef<Path>) -> System {
    let data = fs::read_to_string(filename).expect("Error on open file");
    let lines: Vec<_> = data.lines()
        .skip_while(|line| !line.contains("[parts]"))
        .skip(1)
        .collect();
    let v = lines
        .iter()
        .map(|l| l.split_terminator("\t"))
        .map(|parts| parts.collect::<Vec<_>>())
        .map(|parts| Element::from_raw_row(&parts))
        .collect();

    System(v)
}

fn get_subsystem(system: &System, size: usize, horizontal: HorizontalPosition, vertical: VerticalPosition) -> System {
    let original_size = ((system.0.len() / 5) as f32).sqrt() as usize;
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
            out_system.extend((0..5)
                .map(|i| Element { id: (new_index + i) as i32, ..system.0[index + i].clone() })
            );
        }
    }
    System(out_system)
}

fn save_system(filename: impl AsRef<Path>, system: &System) {
    let mut buffer = String::new();
    let system = &system.0;
    writeln!(buffer, "[header]").expect("Error");
    writeln!(buffer, "dimensions=2").expect("Error");
    writeln!(buffer, "size={}", system.len()).expect("Error");
    let state = system.iter()
        .map(|r| if r.state { "1" } else { "0" })
        .fold(String::new(), |acc, part| acc + part);
    writeln!(buffer, "state={}", state).expect("Error");
    writeln!(buffer, "[parts]").expect("Error");
    for row in system {
        let state = if row.state { "1" } else { "0" };
        writeln!(buffer, "{}\t{:.16}\t{:.16}\t{:.16}\t{:.16}\t{:.16}\t{:.16}\t{}", row.id, row.pos.x, row.pos.y, row.pos.z, row.m.x, row.m.y, row.m.z, state).expect("Error");
    }

    fs::write(filename, buffer).expect("Error on write to file");
}

fn main() {
    let opt: Opt = Opt::from_args();

    match &opt {
        Opt::Corner { input, output, size, corner } => {
            let rows = load_from_file(input);
            let subsystem = get_subsystem(&rows, *size, corner.horizontal, corner.vertical);
            save_system(output, &subsystem);
        }
        Opt::Energy { input, output, sizes, corners, } => {
            let system = load_from_file(input);
            let mut file = File::create(output).expect("Error on open output file");
            for size in sizes {
                let mut se = 0.0;
                let mut es = Vec::with_capacity(corners.len());
                let n = corners.len() as f64;
                for corner in corners.iter() {
                    let e = get_subsystem(&system, *size, corner.horizontal, corner.vertical).energy();
                    se += e;
                    es.push(e);
                }
                let ae = se / n;
                let qe = (es.iter().fold(0.0, |acc, e| acc + ((e - ae) * (e - ae)) / n)).sqrt();
                writeln!(file, "{}\t{}\t#{}", ae, qe, size);
            }
        }
    }
}
