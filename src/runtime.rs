// Laser Bridges & Doors: the pure beam model.
//
// This file is both the standalone Cargo library and the injected
// `crate::game::laserbridges` module; it must not name any game type.
//
// The behaviour is `ISourceBlock.generateBridge` (Laser Bridges & Doors 6.3,
// MIT): a powered source walks a line of cells away from itself and turns each
// one into its laser block until something solid stops it; unpowered, the same
// walk clears the laser blocks it finds. `length` and the step geometry are
// upstream's, including its truncated `(max_length / 15) * power` length and
// its exclusive bound (`i < length`).

/// `LaserBridgesConfig.max_length`, upstream's shipped default.
pub const MAX_LENGTH: i32 = 15;
/// A redstone signal is 0 to 15.
pub const FULL_POWER: u8 = 15;

/// `BlockStateProperties.ATTACH_FACE`: where the source is mounted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Floor,
    Wall,
    Ceiling,
}

impl Face {
    /// The state value, as the content file and the blockstate spell it.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "floor" => Face::Floor,
            "wall" => Face::Wall,
            "ceiling" => Face::Ceiling,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Face::Floor => "floor",
            Face::Wall => "wall",
            Face::Ceiling => "ceiling",
        }
    }
}

/// `BlockStateProperties.HORIZONTAL_FACING`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    North,
    South,
    East,
    West,
}

impl Facing {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "north" => Facing::North,
            "south" => Facing::South,
            "east" => Facing::East,
            "west" => Facing::West,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Facing::North => "north",
            Facing::South => "south",
            Facing::East => "east",
            Facing::West => "west",
        }
    }

    /// The horizontal unit vector, minecraft's axes: east `+x`, south `+z`.
    pub fn vector(self) -> [i32; 3] {
        match self {
            Facing::North => [0, 0, -1],
            Facing::South => [0, 0, 1],
            Facing::East => [1, 0, 0],
            Facing::West => [-1, 0, 0],
        }
    }
}

/// A block's `face`, `facing` and `color`: what decides the beam's line and
/// which laser block it writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beam {
    pub face: Face,
    pub facing: Facing,
    /// `ISourceBlock.COLOR`: a dye id, 0 to 15.
    pub colour: u8,
}

impl Beam {
    /// `generateBridge`'s offset for step `i`, from the source cell: a floor
    /// source goes up, a ceiling source down, a wall source along `facing`.
    pub fn offset(self, i: i32) -> [i32; 3] {
        match self.face {
            Face::Floor => [0, i, 0],
            Face::Ceiling => [0, -i, 0],
            Face::Wall => {
                let [x, y, z] = self.facing.vector();
                [x * i, y, z * i]
            }
        }
    }
}

/// `generateBridge`'s beam length: `max_length / 15 * power`, truncated.
///
/// The shipped `max_length` is 15, so the length is the signal strength; the
/// generator then places only while `i < length`, so a full signal of 15 fills
/// 14 cells and a signal of 1 fills none.
pub fn length(max_length: i32, power: u8) -> i32 {
    let min_length = max_length as f32 / FULL_POWER as f32;
    (min_length * f32::from(power)) as i32
}

/// What the generator finds in one cell of the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// The laser block this beam would write, in exactly the state it wants.
    Same,
    /// Air (or cave/void air): nothing to cut, nothing to stop.
    Air,
    /// A block `blocks_cut_through_by_lasers` names: the beam cuts it.
    Cuttable,
    /// Anything else, including a laser block of another state: upstream's
    /// exact-state test fails, so the line stops before it.
    Solid,
}

/// One write the generator asks for, at step `at` from the source cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Write {
    /// `Level.destroyBlock(pos, true)`: the cut block drops as a player's break
    /// would.
    Cut { at: i32 },
    /// `Level.setBlock(pos, beam, 3)`.
    Place { at: i32 },
    /// `Level.setBlock(pos, air, 3)`: this beam's block was there.
    Clear { at: i32 },
}

/// The writes `generateBridge` would make along the line, in order.
///
/// `look` answers what is in the cell at step `i` (`1..max_length`). Upstream
/// walks every step while the cell is the beam's own state or cuttable, so an
/// unpowered beam clears itself all the way to the first solid block, and a
/// shorter one clears its tail.
pub fn plan(
    _beam: Beam,
    power: u8,
    max_length: i32,
    mut look: impl FnMut(i32) -> Cell,
) -> Vec<Write> {
    let length = length(max_length, power);
    let mut writes = Vec::new();
    let mut ended = false;
    // `for (int i = 1; i < LaserBridgesConfig.max_length; i++)`.
    for at in 1..max_length {
        let cell = look(at);
        let cuttable = cell == Cell::Cuttable;
        if cell == Cell::Solid {
            break;
        }
        if cuttable && power != 0 {
            writes.push(Write::Cut { at });
        }
        if !ended && at < length {
            writes.push(Write::Place { at });
        } else {
            ended = true;
            if cell == Cell::Same {
                writes.push(Write::Clear { at });
            }
        }
    }
    writes
}

/// One side of a cell: the six directions a receiver is asked about, and the six its
/// `face`/`facing` resolve to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Down,
    Up,
    North,
    South,
    East,
    West,
}

impl Side {
    pub fn opposite(self) -> Side {
        match self {
            Side::Down => Side::Up,
            Side::Up => Side::Down,
            Side::North => Side::South,
            Side::South => Side::North,
            Side::East => Side::West,
            Side::West => Side::East,
        }
    }

    /// The unit vector, minecraft's axes: east `+x`, south `+z`, up `+y`.
    pub fn vector(self) -> [i32; 3] {
        match self {
            Side::Down => [0, -1, 0],
            Side::Up => [0, 1, 0],
            Side::North => [0, 0, -1],
            Side::South => [0, 0, 1],
            Side::East => [1, 0, 0],
            Side::West => [-1, 0, 0],
        }
    }
}

impl Facing {
    /// The [`Side`] of the same name.
    pub fn side(self) -> Side {
        match self {
            Facing::North => Side::North,
            Facing::South => Side::South,
            Facing::East => Side::East,
            Facing::West => Side::West,
        }
    }
}

/// The cell a mount looks into: `IReceiverBlock.updateState`'s `dir` — up from a
/// floor mount, down from a ceiling one, along `facing` for a wall one.
pub fn look(face: Face, facing: Facing) -> Side {
    match face {
        Face::Floor => Side::Up,
        Face::Ceiling => Side::Down,
        Face::Wall => facing.side(),
    }
}

/// What a receiver finds in the cell it looks into, as far as its rule cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Front {
    /// A laser block (`AbstractLaserBlock`), whatever its colour.
    Laser { colour: u8 },
    /// A source with its `powered` state set.
    PoweredSource { colour: u8 },
    /// Anything else, including an unpowered source.
    Other,
}

/// The `(powered, color)` a receiver should carry for what it sees: powered by a
/// laser or a powered source, carrying that block's colour; otherwise unpowered and
/// keeping the colour it already had (upstream only assigns `color` inside its match).
pub fn receiver_state(powered: bool, colour: u8, front: Front) -> (bool, u8) {
    let _ = powered;
    match front {
        Front::Laser { colour } | Front::PoweredSource { colour } => (true, colour),
        // Upstream never clears `color` when the laser leaves: it writes
        // `POWERED=false` with the colour the state already had.
        Front::Other => (false, colour),
    }
}

/// The state write a receiver needs, or `None` when it already carries that state:
/// `IReceiverBlock.updateState` writes only when `POWERED` moves.
pub fn receiver_update(powered: bool, colour: u8, front: Front) -> Option<(bool, u8)> {
    let next = receiver_state(powered, colour, front);
    (next != (powered, colour)).then_some(next)
}

/// `IReceiverBlock.handleGetSignal` and `getDirectSignal`: a powered receiver gives 15,
/// except on the face it hangs from (a normal signal) and everywhere but the face it
/// looks into (a direct one).
pub fn receiver_signal(
    powered: bool,
    face: Face,
    facing: Facing,
    asked: Side,
    direct: bool,
) -> i32 {
    if !powered {
        return 0;
    }
    if direct && asked != look(face, facing) {
        return 0;
    }
    if asked == facing.side().opposite() {
        return 0;
    }
    i32::from(FULL_POWER)
}

/// One property of a serialized block state (`laserbridges:x[face=wall,powered=true]`).
///
/// A block part's signal moment is handed a `BlockState` and a level, but no metadata
/// service, so the state's own `level.state_string(..)` is the only way to read
/// `powered`/`face`/`facing` there. Pure, so the parse is tested without a game.
pub fn state_property<'a>(state: &'a str, name: &str) -> Option<&'a str> {
    let (_, properties) = state.split_once('[')?;
    let properties = properties.strip_suffix(']')?;
    properties.split(',').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then_some(value)
    })
}

/// The face a player clicked to place a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clicked {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

/// `AbstractSourceBlock.getStateForPlacement`: the nearest looking direction is
/// the clicked face's opposite, so a vertical click mounts the source on the
/// floor or ceiling and a horizontal one on the wall, emitting away from it.
pub fn placement(clicked: Clicked, yaw: f32) -> (Face, Facing) {
    match clicked {
        Clicked::Up => (Face::Floor, horizontal(yaw)),
        Clicked::Down => (Face::Ceiling, horizontal(yaw)),
        Clicked::North => (Face::Wall, Facing::South),
        Clicked::South => (Face::Wall, Facing::North),
        Clicked::East => (Face::Wall, Facing::West),
        Clicked::West => (Face::Wall, Facing::East),
    }
}

/// `Direction.fromYRot`: the direction the player is looking.
pub fn horizontal(yaw: f32) -> Facing {
    let quarter = ((f64::from(yaw) / 90.0 + 0.5).floor() as i64 & 3) as u8;
    match quarter {
        0 => Facing::South,
        1 => Facing::West,
        2 => Facing::North,
        _ => Facing::East,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line reader over `cells`, indexed by step; missing steps are air.
    fn line(cells: &[Cell]) -> impl FnMut(i32) -> Cell + '_ {
        move |i| cells.get(i as usize).copied().unwrap_or(Cell::Air)
    }

    fn wall(facing: Facing) -> Beam {
        Beam {
            face: Face::Wall,
            facing,
            colour: 0,
        }
    }

    fn places(first: i32, last: i32) -> Vec<Write> {
        (first..=last).map(|at| Write::Place { at }).collect()
    }

    #[test]
    fn length_is_the_signal_strength_at_the_default_maximum() {
        assert_eq!(length(MAX_LENGTH, 0), 0);
        assert_eq!(length(MAX_LENGTH, 1), 1);
        assert_eq!(length(MAX_LENGTH, 7), 7);
        assert_eq!(length(MAX_LENGTH, 15), 15);
    }

    #[test]
    fn length_scales_with_a_configured_maximum_and_truncates() {
        assert_eq!(length(30, 15), 30);
        assert_eq!(length(30, 7), 14);
        assert_eq!(length(16, 7), 7);
    }

    #[test]
    fn a_floor_source_beams_up_and_a_ceiling_source_down() {
        let floor = Beam {
            face: Face::Floor,
            facing: Facing::South,
            colour: 0,
        };
        assert_eq!(floor.offset(1), [0, 1, 0]);
        assert_eq!(floor.offset(4), [0, 4, 0]);
        let ceiling = Beam {
            face: Face::Ceiling,
            facing: Facing::North,
            colour: 0,
        };
        assert_eq!(ceiling.offset(3), [0, -3, 0]);
    }

    #[test]
    fn a_wall_source_beams_along_its_facing() {
        assert_eq!(wall(Facing::East).offset(2), [2, 0, 0]);
        assert_eq!(wall(Facing::West).offset(2), [-2, 0, 0]);
        assert_eq!(wall(Facing::South).offset(2), [0, 0, 2]);
        assert_eq!(wall(Facing::North).offset(2), [0, 0, -2]);
    }

    #[test]
    fn a_full_signal_fills_every_cell_but_the_last() {
        // `i < length` with length 15 places steps 1..=14, upstream's bound.
        assert_eq!(
            plan(wall(Facing::East), 15, MAX_LENGTH, line(&[])),
            places(1, 14)
        );
    }

    #[test]
    fn a_partial_signal_stops_at_its_length() {
        assert_eq!(
            plan(wall(Facing::East), 7, MAX_LENGTH, line(&[])),
            places(1, 6)
        );
    }

    #[test]
    fn a_signal_of_one_fills_nothing() {
        assert!(plan(wall(Facing::East), 1, MAX_LENGTH, line(&[])).is_empty());
        assert!(plan(wall(Facing::East), 0, MAX_LENGTH, line(&[])).is_empty());
    }

    #[test]
    fn a_solid_block_stops_the_line() {
        let cells = [Cell::Air, Cell::Air, Cell::Air, Cell::Solid];
        assert_eq!(
            plan(wall(Facing::East), 15, MAX_LENGTH, line(&cells)),
            places(1, 2)
        );
    }

    #[test]
    fn a_cuttable_block_is_cut_and_beamed_over_when_powered() {
        let cells = [Cell::Air, Cell::Air, Cell::Cuttable];
        let mut expected = vec![
            Write::Place { at: 1 },
            Write::Cut { at: 2 },
            Write::Place { at: 2 },
        ];
        expected.extend(places(3, 14));
        assert_eq!(
            plan(wall(Facing::East), 15, MAX_LENGTH, line(&cells)),
            expected
        );
    }

    #[test]
    fn unpowered_the_beam_clears_itself_up_to_the_first_solid_block() {
        let cells = [
            Cell::Air,
            Cell::Same,
            Cell::Same,
            Cell::Same,
            Cell::Solid,
            Cell::Same,
        ];
        assert_eq!(
            plan(wall(Facing::East), 0, MAX_LENGTH, line(&cells)),
            vec![
                Write::Clear { at: 1 },
                Write::Clear { at: 2 },
                Write::Clear { at: 3 },
            ]
        );
    }

    #[test]
    fn unpowered_a_cuttable_block_is_left_alone_but_the_line_continues() {
        let cells = [Cell::Air, Cell::Cuttable, Cell::Same];
        assert_eq!(
            plan(wall(Facing::East), 0, MAX_LENGTH, line(&cells)),
            vec![Write::Clear { at: 2 }]
        );
    }

    #[test]
    fn a_shorter_signal_clears_the_tail_it_no_longer_fills() {
        let cells = [
            Cell::Air,
            Cell::Same,
            Cell::Same,
            Cell::Same,
            Cell::Same,
            Cell::Same,
            Cell::Same,
            Cell::Same,
            Cell::Same,
        ];
        let mut expected = places(1, 3);
        expected.extend((4..=8).map(|at| Write::Clear { at }));
        assert_eq!(
            plan(wall(Facing::East), 4, MAX_LENGTH, line(&cells)),
            expected
        );
    }

    #[test]
    fn another_laser_state_is_solid_to_the_line() {
        // A laser block carrying another colour is not `equals(bs)`, so the
        // walk stops there and never reaches the `clear` branch.
        let cells = [Cell::Air, Cell::Air, Cell::Solid, Cell::Same];
        assert_eq!(
            plan(wall(Facing::East), 0, MAX_LENGTH, line(&cells)),
            Vec::new()
        );
    }

    #[test]
    fn a_horizontal_yaw_gives_the_direction_the_player_faces() {
        assert_eq!(horizontal(0.0), Facing::South);
        assert_eq!(horizontal(90.0), Facing::West);
        assert_eq!(horizontal(180.0), Facing::North);
        assert_eq!(horizontal(270.0), Facing::East);
        assert_eq!(horizontal(-90.0), Facing::East);
    }

    #[test]
    fn a_vertical_click_mounts_on_floor_or_ceiling_and_a_wall_click_on_the_wall() {
        assert_eq!(placement(Clicked::Up, 0.0), (Face::Floor, Facing::South));
        assert_eq!(
            placement(Clicked::Down, 90.0),
            (Face::Ceiling, Facing::West)
        );
        assert_eq!(placement(Clicked::North, 0.0), (Face::Wall, Facing::South));
        assert_eq!(placement(Clicked::South, 0.0), (Face::Wall, Facing::North));
        assert_eq!(placement(Clicked::East, 0.0), (Face::Wall, Facing::West));
        assert_eq!(placement(Clicked::West, 0.0), (Face::Wall, Facing::East));
    }

    #[test]
    fn a_floor_receiver_looks_up_and_a_ceiling_one_down() {
        assert_eq!(look(Face::Floor, Facing::North), Side::Up);
        assert_eq!(look(Face::Ceiling, Facing::East), Side::Down);
    }

    #[test]
    fn a_wall_receiver_looks_along_its_facing() {
        assert_eq!(look(Face::Wall, Facing::North), Side::North);
        assert_eq!(look(Face::Wall, Facing::South), Side::South);
        assert_eq!(look(Face::Wall, Facing::East), Side::East);
        assert_eq!(look(Face::Wall, Facing::West), Side::West);
    }

    #[test]
    fn every_side_has_an_opposite_and_a_unit_vector() {
        for side in [
            Side::Down,
            Side::Up,
            Side::North,
            Side::South,
            Side::East,
            Side::West,
        ] {
            assert_eq!(side.opposite().opposite(), side, "{side:?}");
            let [x, y, z] = side.vector();
            let [ox, oy, oz] = side.opposite().vector();
            assert_eq!([x + ox, y + oy, z + oz], [0, 0, 0], "{side:?}");
        }
        assert_eq!(Side::Up.vector(), [0, 1, 0]);
        assert_eq!(Side::South.vector(), [0, 0, 1]);
    }

    #[test]
    fn a_receiver_powers_on_a_laser_and_copies_its_colour() {
        assert_eq!(
            receiver_state(false, 0, Front::Laser { colour: 7 }),
            (true, 7)
        );
        assert_eq!(
            receiver_update(false, 0, Front::Laser { colour: 7 }),
            Some((true, 7))
        );
    }

    #[test]
    fn a_powered_source_in_front_powers_a_receiver_but_an_unpowered_one_does_not() {
        assert_eq!(
            receiver_state(false, 0, Front::PoweredSource { colour: 3 }),
            (true, 3)
        );
        assert_eq!(receiver_state(false, 0, Front::Other), (false, 0));
    }

    #[test]
    fn a_receiver_that_already_carries_the_state_is_not_written() {
        assert_eq!(receiver_update(true, 7, Front::Laser { colour: 7 }), None);
        assert_eq!(receiver_update(false, 0, Front::Other), None);
    }

    #[test]
    fn a_colour_change_on_a_powered_receiver_is_copied() {
        // Upstream's own write is gated on POWERED moving, so a colour change
        // under a powered receiver used to be lost; the port recomputes the pair.
        assert_eq!(
            receiver_update(true, 3, Front::Laser { colour: 9 }),
            Some((true, 9))
        );
    }

    #[test]
    fn an_unpowered_receiver_keeps_the_last_colour_it_copied() {
        assert_eq!(receiver_state(false, 5, Front::Other), (false, 5));
        assert_eq!(receiver_update(false, 5, Front::Other), None);
    }

    #[test]
    fn a_receiver_powers_down_when_the_laser_goes_away() {
        assert_eq!(receiver_update(true, 5, Front::Other), Some((false, 5)));
        assert_eq!(
            receiver_update(true, 5, Front::PoweredSource { colour: 2 }),
            Some((true, 2))
        );
    }

    #[test]
    fn a_powered_receiver_gives_15_on_every_face_but_the_one_it_hangs_from() {
        // A wall receiver facing west hangs on its east face; upstream's
        // `getSignal` answers 0 there and 15 everywhere else.
        let powered = receiver_signal(true, Face::Wall, Facing::West, Side::West, false);
        assert_eq!(powered, 15);
        assert_eq!(
            receiver_signal(true, Face::Wall, Facing::West, Side::Up, false),
            15
        );
        assert_eq!(
            receiver_signal(true, Face::Wall, Facing::West, Side::East, false),
            0,
            "the face it hangs from gives nothing"
        );
    }

    #[test]
    fn a_receiver_gives_a_direct_signal_only_on_the_face_it_looks_into() {
        assert_eq!(
            receiver_signal(true, Face::Wall, Facing::West, Side::West, true),
            15
        );
        assert_eq!(
            receiver_signal(true, Face::Wall, Facing::West, Side::Up, true),
            0
        );
        assert_eq!(
            receiver_signal(true, Face::Floor, Facing::North, Side::Up, true),
            15
        );
        assert_eq!(
            receiver_signal(true, Face::Floor, Facing::North, Side::North, true),
            0
        );
    }

    #[test]
    fn an_unpowered_receiver_gives_no_signal_on_any_face() {
        for side in [
            Side::Down,
            Side::Up,
            Side::North,
            Side::South,
            Side::East,
            Side::West,
        ] {
            assert_eq!(
                receiver_signal(false, Face::Wall, Facing::West, side, false),
                0,
                "{side:?}"
            );
            assert_eq!(
                receiver_signal(false, Face::Wall, Facing::West, side, true),
                0,
                "{side:?}"
            );
        }
    }

    #[test]
    fn a_serialized_state_gives_its_properties() {
        let state =
            "laserbridges:laser_attached_receiver[color=0,face=wall,facing=west,powered=true]";
        assert_eq!(state_property(state, "face"), Some("wall"));
        assert_eq!(state_property(state, "facing"), Some("west"));
        assert_eq!(state_property(state, "powered"), Some("true"));
        assert_eq!(state_property(state, "color"), Some("0"));
        assert_eq!(state_property(state, "missing"), None);
        assert_eq!(state_property("minecraft:stone", "face"), None);
        assert_eq!(
            state_property("minecraft:x[facex=north,face=north]", "face"),
            Some("north"),
            "a longer key must not answer for the one asked"
        );
    }
}
