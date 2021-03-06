//! Defines types used to construct representations of chords and notes.
//!
//! These can be generated manually, but often one would want to use the
//! `parser` module into order to generate these from a string.
//!
//! ```
//! use quartic::chord::{Chord, ChordStructure, Note, NoteClass, PitchClass};
//!
//! // Manual construction of a A#Maj13(#5)(#11)
//! let root = Note::new(NoteClass::A, 1);
//! let structure =
//!     ChordStructure::new()
//!         .insert_many(&[
//!             (PitchClass::N3, 0),
//!             (PitchClass::N5, 1),
//!             (PitchClass::N7, 1),
//!             (PitchClass::N9, 0),
//!             (PitchClass::N11, 1),
//!             (PitchClass::N13, 0),
//!         ]);
//!
//! let chord1 = Chord::new(root, structure);
//!
//! // Shorthand construction
//! let chord2 = Chord::from_shorthand("A#Maj13(#5,#11)").unwrap();
//! ```
//!
//! All `Chord`'s have an implicit root pitch class.

use std::fmt::{self, Write};
use std::iter;

use combine::Parser;
use parser::{parse_chord, parse_polychord};

/// Represents an error which may occur when parsing shorthand chord forms.
pub use combine::ParseError;

/// Represents a result returned by values which perform parsing.
pub type ParseResult<'a, T> = Result<T, ParseError<&'a str>>;

/// A single note without accidentals.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NoteClass {
    A, B, C, D, E, F, G
}

/// The total number of `NoteClass` elements.
pub const NOTE_CLASS_COUNT: usize = 7;

impl NoteClass {
    /// Construct a `Note` from a char representation.
    pub fn from_char(input: char) -> Option<NoteClass> {
        use self::NoteClass::*;

        match input {
            'A' => Some(A),
            'B' => Some(B),
            'C' => Some(C),
            'D' => Some(D),
            'E' => Some(E),
            'F' => Some(F),
            'G' => Some(G),
            _   => None
        }
    }

    /// Construct a `Note` from a small integer value.
    pub fn from_int(input: usize) -> Option<NoteClass> {
        use self::NoteClass::*;

        match input {
            0 => Some(A),
            1 => Some(B),
            2 => Some(C),
            3 => Some(D),
            4 => Some(E),
            5 => Some(F),
            6 => Some(G),
            _   => None
        }
    }

    /// Returns the order set of `NoteClass` for indexing.
    pub fn to_int(&self) -> usize {
        use self::NoteClass::*;

        match *self {
            A => 0,
            B => 1,
            C => 2,
            D => 3,
            E => 4,
            F => 5,
            G => 6
        }
    }

    /// Compute the semi-tonal difference between two base `NoteClass`'s.
    ///
    /// The value returned will be less than 12.
    pub fn difference(&self, other: &NoteClass) -> usize {
        const OFFSETS: [usize; NOTE_CLASS_COUNT] = [
            0, 2, 3, 5, 7, 8, 10,
        ];

        let upper = OFFSETS[other.to_int()] + 12;
        let lower = OFFSETS[self.to_int()];
        (upper - lower) % 12
    }
}

impl fmt::Display for NoteClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Relative pitch compared to some base note as part of a chord.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PitchClass {
    N1, N2, N3, N4, N5, N6, N7, N9, N11, N13
}

/// The total number of `PitchClass` elements.
pub const PITCH_CLASS_COUNT: usize = 10;

impl PitchClass {
    /// Construct a pitch-class from an index.
    pub fn from_int(input: usize) -> Option<Self> {
        use self::PitchClass::*;

        match input {
            0 => Some(N1),
            1 => Some(N2),
            2 => Some(N3),
            3 => Some(N4),
            4 => Some(N5),
            5 => Some(N6),
            6 => Some(N7),
            7 => Some(N9),
            8 => Some(N11),
            9 => Some(N13),
            _ => None,
        }
    }

    /// Allows `PitchClass` to be used as an indexable element.
    pub fn index(&self) -> usize {
        use self::PitchClass::*;

        match *self {
            N1  => 0,
            N2  => 1,
            N3  => 2,
            N4  => 3,
            N5  => 4,
            N6  => 5,
            N7  => 6,
            N9  => 7,
            N11 => 8,
            N13 => 9,
        }
    }

    /// Returns the absolute offset for a natural `PitchClass` with no
    /// alterations.
    pub fn to_int(&self) -> usize {
        use self::PitchClass::*;

        match *self {
            N1       => 0,
            N2 | N9  => 1,
            N3       => 2,
            N4 | N11 => 3,
            N5       => 4,
            N6 | N13 => 5,
            N7       => 6,
        }
    }

    /// Returns the number of semi-tones difference this `PitchClass` represents.
    pub fn to_relative_difference(&self) -> usize {
        use self::PitchClass::*;

        match *self {
            N1  => 0,
            N2  => 2,
            N3  => 4,
            N4  => 5,
            N5  => 7,
            N6  => 9,
            N7  => 11,
            N9  => 14,
            N11 => 17,
            N13 => 21,
        }
    }

    /// Returns a slice over the extended chord intervals for the specified
    /// pitch class.
    ///
    /// For example, a `C11` chord implicitly includes the lower extended
    /// intervals of the `7`th and `9`th within its representation. Passing
    /// `PitchClass::N11` would return a slice with `7`, `9` and `11` values.
    pub fn extended_intervals(&self) -> &'static [ChordComponent] {
        use self::PitchClass::*;

        static CLASSES: [ChordComponent; 4] = [
            (N7, 0), (N9, 0), (N11, 0), (N13, 0),
        ];

        match *self {
            N7  => &CLASSES[..1],
            N9  => &CLASSES[..2],
            N11 => &CLASSES[..3],
            N13 => &CLASSES[..4],
            _   => &[]
        }
    }
}

/// Represents an alteration of a base `NoteClass`.
///
/// This is analagous to accidentals, with positive values representing
/// repeated sharps and negative values representing repeated flats.
pub type PitchOffset = i8;

/// A single note which may have applied accidentals.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Note {
    /// The base note
    pub root: NoteClass,

    /// The resulting shift from accidentals
    pub offset: PitchOffset
}

impl Note {
    /// Construct and return a new `Note`.
    pub fn new(root: NoteClass, offset: PitchOffset) -> Note {
        Note { root, offset }
    }

    /// Return the relative `Note` based on the given pitch-class.
    pub fn get_relative(&self, (class, offset): ChordComponent) -> Note {
        let root_val = (self.root.to_int() + class.to_int()) % NOTE_CLASS_COUNT;
        let root_note = NoteClass::from_int(root_val).unwrap();
        let rel_offset = class.to_relative_difference() as i8
                         - self.root.difference(&root_note) as i8;

        Note {
            root: root_note,
            offset: self.offset + offset + rel_offset
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.root)?;
        let accidental = if self.offset.is_positive() { '#' } else { 'b' };
        for _ in 0..self.offset.abs() {
            f.write_char(accidental)?;
        }

        Ok(())
    }
}

/// A relative note within a chord by its intervallic representation.
///
/// For example, a (`PitchClass::n7`, 1) would represent a sharpened seventh,
/// as found in a major seventh chord.
pub type ChordComponent = (PitchClass, PitchOffset);

/// Represents the intervallic structure of a chord.
///
/// This is relative to a root note so a transposition is very cheap.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChordStructure([Option<PitchOffset>; PITCH_CLASS_COUNT]);

impl ChordStructure {
    /// Construct and return a new `ChordStructure`.
    ///
    /// This will always have the root note (`PitchClass::n1`) by default.
    pub fn new() -> ChordStructure {
        let mut classes = [None; PITCH_CLASS_COUNT];
        classes[PitchClass::N1.index()] = Some(0);

        ChordStructure(classes)
    }

    /// Construct a `ChordStructure` from a single component.
    ///
    /// This is provided to simplify merging alterations into a core chord and
    /// may be removed at some point.
    pub fn from_component(component: ChordComponent) -> ChordStructure {
        let mut classes = [None; PITCH_CLASS_COUNT];
        classes[component.0.index()] = Some(component.1);

        ChordStructure(classes)
    }

    /// Insert a single `ChordComponent` into this `ChordStructure`.
    ///
    /// This will overwrite any existing component if the interval was already
    /// set.
    pub fn insert(mut self, component: ChordComponent) -> ChordStructure {
        self.0[component.0.index()] = Some(component.1);
        self
    }

    /// Insert a slice of `ChordComponent` values into the `ChordStructure`.
    ///
    /// This will overwrite any existing components for each of the intervals
    /// which are already present in the input slice.
    pub fn insert_many(mut self, components: &[ChordComponent]) -> ChordStructure {
        for &component in components {
            self.0[component.0.index()] = Some(component.1);
        }
        self
    }

    /// Merge two `ChordStructure`'s together with preference for elements
    /// within the `other` structure.
    pub fn merge(mut self, other: &ChordStructure) -> ChordStructure {
        for i in 0..PITCH_CLASS_COUNT {
            if other.0[i].is_some() {
                self.0[i] = other.0[i];
            }
        }
        self
    }
}

/// A single simple chord comprised of many notes.
///
/// The chord representation used internally is based on tertian harmony.
#[derive(Clone, Debug, PartialEq)]
pub struct Chord {
    /// Slash chord base note
    pub slash_root: Option<Note>,

    /// The base note
    pub root: Note,

    /// The relative intervallic structure of this chord
    pub structure: ChordStructure
}

impl Chord {
    /// Construct and return a new `Chord`.
    pub fn new(root: Note, structure: ChordStructure) -> Chord {
        Chord { slash_root: None, root, structure }
    }

    /// Construct and return a new slash-chord.
    pub fn new_slash(slash_root: Note, root: Note, structure: ChordStructure)
        -> Chord
    {
        Chord { slash_root: Some(slash_root), root, structure }
    }

    /// Construct a chord from a shorthand string.
    pub fn from_shorthand(input: &str) -> ParseResult<Chord> {
        parse_chord().parse(input).map(|c| c.0)
    }

    /// Return an iterator over each of all notes this chord is comprised of.
    ///
    /// Notes are returned from lowest pitch to highest, in order.
    pub fn iter(&self) -> NoteIterator {
        NoteIterator {
            chord: self,
            state: NoteIteratorState::Slash,
        }
    }
}

/// An iterator over notes in a chord.
///
/// Notes are returned lowest to highest in pitch.
#[derive(Clone, Debug)]
pub struct NoteIterator<'a> {
    pub chord: &'a Chord,
    state: NoteIteratorState,
}

#[derive(Clone, Copy, Debug)]
enum NoteIteratorState {
    Slash,
    Structure(usize),
    Exhausted,
}

impl<'a> Iterator for NoteIterator<'a> {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        use self::NoteIteratorState::*;

        'retry: loop {
            return match self.state {
                Slash => {
                    self.state = Structure(0);

                    if let Some(note) = self.chord.slash_root {
                        Some(note)
                    } else {
                        continue 'retry;
                    }
                },

                Structure(ii) => {
                    let mut i = ii;

                    while i < PITCH_CLASS_COUNT {
                        if let Some(offset) = self.chord.structure.0[i] {
                            let pc = PitchClass::from_int(i).unwrap();

                            // Next time around we need to be looking at the
                            // next element from the beginning.
                            self.state = Structure(i + 1);
                            return Some(self.chord.root.get_relative((pc, offset)));
                        }

                        i += 1;
                    }

                    self.state = Exhausted;
                    None
                },

                Exhausted => {
                    None
                },
            };
        }
    }
}

/// A single polychord which is comprised of an upper chord stacked atop a
/// lower chord.
#[derive(Clone, Debug, PartialEq)]
pub struct PolyChord {
    /// Upper chord structure
    pub upper: Chord,

    /// Lower chord structure
    pub lower: Chord
}

impl PolyChord {
    /// Construct and return a new `PolyChord`.
    pub fn new(upper: Chord, lower: Chord) -> PolyChord {
        PolyChord { upper, lower }
    }

    /// Construct a chord from a shorthand string.
    pub fn from_shorthand(input: &str) -> ParseResult<PolyChord> {
        parse_polychord().parse(input).map(|c| c.0)
    }

    /// Return an iterator over each of all notes this chord is comprised of.
    ///
    /// Notes are returned from lowest pitch to highest, in order.
    pub fn iter(&self) -> iter::Chain<NoteIterator, NoteIterator> {
        self.lower.iter().chain(self.upper.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chord::NoteClass::*;
    use chord::PitchClass::*;

    #[test]
    fn offset_calculation() {
        assert_eq!(Note::new(A, 0).get_relative((N5, 0)), Note::new(E, 0));
        assert_eq!(Note::new(A, 0).get_relative((N5, -1)), Note::new(E, -1));
        assert_eq!(Note::new(F, -1).get_relative((N2, -2)), Note::new(G, -3));
        assert_eq!(Note::new(D, 0).get_relative((N3, 0)), Note::new(F, 1));
        assert_eq!(Note::new(A, 0).get_relative((N3, 0)), Note::new(C, 1));
    }

    #[test]
    fn chord_notes() {
        // A/C#
        let chord = Chord::new_slash(
            Note::new(C, 1),
            Note::new(A, 0),
            ChordStructure::new()
                .insert_many(&[(N3, 0), (N5, 0)])
        );

        let notes = vec![
            Note::new(C, 1),
            Note::new(A, 0),
            Note::new(C, 1),
            Note::new(E, 0),
        ];

        assert_eq!(chord.iter().collect::<Vec<_>>(), notes);
    }

    #[test]
    fn polychord_notes() {
        // F#(#5)|Bm
        let chord = PolyChord::new(
            Chord::new(
                Note::new(F, 1),
                ChordStructure::new()
                    .insert_many(&[(N3, 0), (N5, 1)])
            ),
            Chord::new(
                Note::new(B, 0),
                ChordStructure::new()
                    .insert_many(&[(N3, -1), (N5, 0)])
            )
        );

        let notes = vec![
            Note::new(B, 0),
            Note::new(D, 0),
            Note::new(F, 1),
            Note::new(F, 1),
            Note::new(A, 1),
            Note::new(C, 2),
        ];

        assert_eq!(chord.iter().collect::<Vec<_>>(), notes);
    }

    #[test]
    fn chord_from_shorthand() {
        let chord = Chord::from_shorthand("C").unwrap();

        let expected = Chord::new(
            Note::new(C, 0),
            ChordStructure::new()
                .insert_many(&[(N3, 0), (N5, 0)])
        );

        assert_eq!(chord, expected);
    }

    #[test]
    fn polychord_from_shorthand() {
        let chord = PolyChord::from_shorthand("C|Am").unwrap();

        let upper = Chord::new(
            Note::new(C, 0),
            ChordStructure::new()
                .insert_many(&[(N3, 0), (N5, 0)])
        );

        let lower = Chord::new(
            Note::new(A, 0),
            ChordStructure::new()
                .insert_many(&[(N3, -1), (N5, 0)])
        );

        let expected = PolyChord::new(upper, lower);
        assert_eq!(chord, expected);
    }
}
