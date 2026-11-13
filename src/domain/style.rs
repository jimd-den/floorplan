//! A house style and the bias it carries over the plan.

/// A house style. Each one is a real bias over footprint, rooms,
/// circulation and windows — not a label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    Ranch,
    Colonial,
    Craftsman,
    Modern,
    Cottage,
    Townhouse,
}

impl Style {
    pub fn all() -> [Style; 6] {
        [
            Style::Ranch,
            Style::Colonial,
            Style::Craftsman,
            Style::Modern,
            Style::Cottage,
            Style::Townhouse,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Style::Ranch => "Ranch",
            Style::Colonial => "Colonial",
            Style::Craftsman => "Craftsman",
            Style::Modern => "Modern",
            Style::Cottage => "Cottage",
            Style::Townhouse => "Townhouse",
        }
    }

    /// The machine id: lowercase, used in JSON and on the CLI.
    pub fn id(&self) -> &'static str {
        match self {
            Style::Ranch => "ranch",
            Style::Colonial => "colonial",
            Style::Craftsman => "craftsman",
            Style::Modern => "modern",
            Style::Cottage => "cottage",
            Style::Townhouse => "townhouse",
        }
    }

    pub fn from_name(name: &str) -> Option<Style> {
        Style::all()
            .into_iter()
            .find(|s| s.name().eq_ignore_ascii_case(name))
    }

    /// The style's bias, as data. The generator reads only this.
    pub fn spec(&self) -> StyleSpec {
        match self {
            Style::Ranch => StyleSpec {
                stories: 1,
                width: (14.0, 20.0),
                depth: (9.0, 12.0),
                rooms: (6, 8),
                upper_rooms: (0, 0),
                window: (1.2, 1.8),
                shapes: &[FootprintShape::Rectangle, FootprintShape::L, FootprintShape::U],
                porch: PorchKind::Optional,
                garage: true,
            },
            Style::Colonial => StyleSpec {
                stories: 2,
                width: (12.0, 16.0),
                depth: (10.0, 13.0),
                rooms: (7, 9),
                upper_rooms: (4, 5),
                window: (1.0, 1.5),
                shapes: &[FootprintShape::Rectangle, FootprintShape::L],
                porch: PorchKind::Optional,
                garage: true,
            },
            Style::Craftsman => StyleSpec {
                stories: 1,
                width: (13.0, 18.0),
                depth: (10.0, 13.0),
                rooms: (6, 8),
                upper_rooms: (0, 0),
                window: (1.0, 1.4),
                shapes: &[
                    FootprintShape::Rectangle,
                    FootprintShape::L,
                    FootprintShape::T,
                ],
                porch: PorchKind::Wide,
                garage: true,
            },
            Style::Modern => StyleSpec {
                stories: 1,
                width: (14.0, 22.0),
                depth: (9.0, 12.0),
                rooms: (5, 7),
                upper_rooms: (0, 0),
                window: (1.8, 2.4),
                shapes: &[FootprintShape::Rectangle, FootprintShape::L, FootprintShape::U],
                porch: PorchKind::None,
                garage: false,
            },
            Style::Cottage => StyleSpec {
                stories: 1,
                width: (8.0, 11.0),
                depth: (7.0, 10.0),
                rooms: (4, 5),
                upper_rooms: (0, 0),
                window: (0.8, 1.0),
                shapes: &[FootprintShape::Rectangle, FootprintShape::L],
                porch: PorchKind::Optional,
                garage: false,
            },
            Style::Townhouse => StyleSpec {
                stories: 2,
                width: (6.0, 8.0),
                depth: (12.0, 16.0),
                // Six is the floor: entry + living + kitchen + hall +
                // bedroom + bath is the smallest two-storey ground floor.
                rooms: (6, 7),
                upper_rooms: (4, 5),
                window: (1.0, 1.6),
                shapes: &[FootprintShape::Rectangle],
                porch: PorchKind::None,
                garage: false,
            },
        }
    }
}

/// The numbers a style biases. Ranges are inclusive, in metres.
#[derive(Debug, Clone, Copy)]
pub struct StyleSpec {
    pub stories: u8,
    pub width: (f64, f64),
    pub depth: (f64, f64),
    pub rooms: (usize, usize),
    pub upper_rooms: (usize, usize),
    pub window: (f64, f64),
    pub shapes: &'static [FootprintShape],
    pub porch: PorchKind,
    pub garage: bool,
}

/// Footprint outline shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootprintShape {
    Rectangle,
    L,
    U,
    T,
}

/// Whether the style carries a porch, and how wide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PorchKind {
    None,
    Optional,
    Wide,
}
