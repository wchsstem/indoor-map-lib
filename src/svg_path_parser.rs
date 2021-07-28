use svg::node::element::path;
use svg::node::element::path::{Command as RawCommand, Position};

/// Represents a command. First component is the absolute x destination, second is the absolute y
/// destination. Does not store any information besides the destination.
#[derive(Copy, Clone, Debug)]
pub struct Command(pub f32, pub f32);

impl Command {
    pub fn from_raw_command(raw_command: &RawCommand, last_command: Command) -> Vec<Self> {
        match raw_command {
            RawCommand::Close => vec![], // Ignored
            RawCommand::HorizontalLine(position, parameters) => {
                let chunks = parameters.chunks_exact(1);
                let coords: Vec<_> = match position {
                    Position::Absolute => chunks.map(|chunk| (chunk[0], last_command.0)).collect(),
                    Position::Relative => chunks.map(|chunk| (chunk[0], 0.0)).collect(),
                };
                Self::from_coords_position(coords.into_iter(), *position, last_command)
            }
            RawCommand::VerticalLine(position, parameters) => {
                let chunks = parameters.chunks_exact(1);
                let coords: Vec<_> = match position {
                    Position::Absolute => chunks.map(|chunk| (last_command.0, chunk[0])).collect(),
                    Position::Relative => chunks.map(|chunk| (0.0, chunk[0])).collect(),
                };
                Self::from_coords_position(coords.into_iter(), *position, last_command)
            }
            RawCommand::SmoothQuadraticCurve(position, parameters)
            | RawCommand::Move(position, parameters)
            | RawCommand::Line(position, parameters) => {
                let coords = parameters.chunks_exact(2).map(|chunk| (chunk[0], chunk[1]));
                Self::from_coords_position(coords, *position, last_command)
            }
            RawCommand::SmoothCubicCurve(position, parameters)
            | RawCommand::QuadraticCurve(position, parameters) => {
                let coords = parameters.chunks_exact(4).map(|chunk| (chunk[2], chunk[3]));
                Self::from_coords_position(coords, *position, last_command)
            }
            RawCommand::CubicCurve(position, parameters) => {
                let coords = parameters.chunks_exact(6).map(|chunk| (chunk[4], chunk[5]));
                Self::from_coords_position(coords, *position, last_command)
            }
            RawCommand::EllipticalArc(position, parameters) => {
                let coords = parameters.chunks_exact(7).map(|chunk| (chunk[5], chunk[6]));
                Self::from_coords_position(coords, *position, last_command)
            }
        }
    }

    fn from_coords_position<T>(coords: T, position: Position, last_command: Command) -> Vec<Command>
    where
        T: Iterator<Item = (f32, f32)>,
    {
        match position {
            Position::Absolute => coords.map(|(x, y)| Command(x, y)).collect(),
            Position::Relative => {
                coords
                    .fold((vec![], last_command), |(mut acc, last_command), (x, y)| {
                        let command = Command(last_command.0 + x, last_command.1 + y);
                        acc.push(command);
                        (acc, command)
                    })
                    .0
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Path {
    path: Vec<Command>,
}

impl From<&path::Data> for Path {
    fn from(raw_commands: &path::Data) -> Self {
        let path = raw_commands
            .iter()
            .fold(
                (vec![], Command(0.0, 0.0)),
                |(mut acc, last_command), raw_command| {
                    let command = Command::from_raw_command(raw_command, last_command);
                    let last = command.last().copied();
                    match last {
                        Some(new_last_command) => {
                            acc.extend(command.into_iter());
                            (acc, new_last_command)
                        }
                        None => (acc, last_command),
                    }
                },
            )
            .0;

        Self { path }
    }
}

impl IntoIterator for Path {
    type Item = Command;
    type IntoIter = <Vec<Command> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.path.into_iter()
    }
}
