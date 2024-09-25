/// Data structures and methods for creating and shuffling keyboard layouts.
extern crate rand;

use self::rand::random;
use penalty::D_SWIPE;
use std::fmt;

/* ----- *
 * TYPES *
 * ----- */

// KeyMap format:
//
//          col 0    col 1    col 2    col 3    col 4    col 5
//
//          2   3 |  7   8 | 12  13 | 17  18 | 22  23 | 27  28
// row 0      4   |    9   |   14   |   19   |   24   |   29
//          1   0 |  6   5 | 11  10 | 16  15 | 21  20 | 26  25
//        -------- -------- -------- -------- -------- --------
//         32  33 | 37  38 | 42  43 | 47  48 | 52  53 | 57  58
// row 1     34   |   39   |   44   |   49   |   54   |   59
//         31  30 | 36  35 | 41  40 | 46  45 | 51  50 | 56  55
//        -------- -------- -------- -------- -------- --------
//                | 67  68 | 72  73 | 77  78 | 82  83 |
// row 2    SHIFT |   69   |   74   |   79   |   84   | BACKSPACE
//                | 66  65 | 71  70 | 76  75 | 81  80 |
//
// index%5 = relative location in key
// (index/5)/6 = row
// (index/5)%6 = col
//
// (row*6+col)*5+(relative location in key)=index
//
// if relative location in key != 4, then it's an integer in [0,4) representing a swipe such that:
// ((relative location in key) / 4.0 + 1.0 / 8.0) * 2PI = angle in radians of swipe.
//
// note the standard programming grid has the y-axis flipped compared to the standard x-y euclidean
// plane in math, formally:
// - programming-grid-col = euclidean-plane-x
// - programming-grid-row = -1 * euclidean-plane-y

// Not including shift,backspace, there are 80 valid locations.
//
// This function maps a number in [0,80) to a valid location, preserving order.
fn to_index(mut orig: usize) -> usize {
    assert!(orig < 80);
    if orig >= 60 {
        orig += 5;
    }
    orig
}

pub struct KeyMap<T>(pub [T; 85]);

impl<T: Copy> Clone for KeyMap<T> {
    fn clone(&self) -> KeyMap<T> {
        KeyMap(self.0)
    }
}

#[derive(Clone)]
pub struct Layout(KeyMap<char>);

pub struct LayoutPermutations {
    orig_layout: Layout,
    swaps: Vec<(usize, usize)>,
    index: usize,
}

pub struct LayoutPosMap([Option<KeyPress>; 128]);

#[derive(Clone, Copy)]
pub struct KeyPress {
    pub pos: usize,
}

/* ------- *
 * STATICS *
 * ------- */

//all letters,symbols except ()[]{}<>
//I will put these in manually at the end
#[rustfmt::skip]
pub static INIT_LAYOUT: Layout = Layout(KeyMap([
//row 0
'b','\0',';','\0','l',
'z','\0','*','\0','c',
'?','\0','#','\0','m',
'\0','@','\0','|','p',
'\0','$','\0','/','u',
'\0','\"','\0','%','o',
//row 1
'j','\0','+','\0','d',
'g','\0','\\','\0','n',
'v','\0','x','\0','r',
'\0','\'','\0','!','a',
'\0','k','\0','-','i',
'\0','~','\0','_','f',
//row 2
'\0','\0','\0','\0','\0',
'=','\0','q','\0','s',
',','&','w','\0','t',
'^','y','`','.','e',
'\0',':','\0','\0','h',
]));

pub static KP_NONE: Option<KeyPress> = None;

/* ------- *
 * HELPERS *
 * ------- */

pub fn is_tap(key: &KeyPress) -> bool {
    key.pos % 5 == 4
}

pub fn get_coordinates(key: &KeyPress) -> (usize, usize) {
    assert!(!is_space(key));
    let spot = key.pos / 5;
    (spot / 6, spot % 6)
}

pub fn get_coordinates_float(key: &KeyPress) -> (f64, f64) {
    if is_space(key) {
        (3.0, 2.5)
    } else {
        let (row, col) = get_coordinates(key);
        (row as f64, col as f64)
    }
}

// is typed with left thumb
fn is_left(key: &KeyPress) -> bool {
    assert!(!is_space(key));
    let column = get_coordinates(key).1;
    column < 3
}

// is a good swipe for the left thumb: i.e. either north-west or south-east
fn is_good_for_left(key: &KeyPress) -> bool {
    assert!(!is_tap(key));
    (key.pos % 5) % 2 == 0
}

pub fn swipe_is_good_for_hand(key: &KeyPress) -> bool {
    assert!(!is_space(key));
    is_left(key) == is_good_for_left(key)
}

pub fn same_hand(key1: &KeyPress, key2: &KeyPress) -> bool {
    assert!(!is_space(key1));
    assert!(!is_space(key2));
    is_left(key1) == is_left(key2)
}

fn get_swipe_angle_radians(key: &KeyPress) -> f64 {
    assert!(!is_tap(key));
    ((key.pos % 5) as f64 / 4.0 + 1.0 / 8.0) * 2.0 * std::f64::consts::PI
}

// returns coordinates of end of swipe
pub fn get_end_of_swipe_coords(key: &KeyPress) -> (f64, f64) {
    assert!(!is_tap(key));
    let mut end_of_swipe_coords = get_coordinates_float(key);
    let (sin, cos) = f64::sin_cos(get_swipe_angle_radians(key));
    end_of_swipe_coords.0 += D_SWIPE * sin;
    end_of_swipe_coords.1 += D_SWIPE * cos;
    end_of_swipe_coords
}

pub fn is_space(key: &KeyPress) -> bool {
    key.pos == 109
}

/* ----- *
 * IMPLS *
 * ----- */

pub static ALL_CHARS: &str = "`~!@#$%^&*-_=+\\|;:\'\",./?qwertyuiopasdfghjklzxcvbnm";

impl Layout {
    pub fn shuffle(&mut self, times: usize) {
        for _ in 0..times {
            let i = to_index(random::<usize>() % 80);
            let j = to_index(random::<usize>() % 80);
            let KeyMap(ref mut layer) = self.0;
            layer.swap(i, j);
        }
    }

    pub fn get_position_map(&self) -> LayoutPosMap {
        let KeyMap(ref layer) = self.0;
        let mut map = [None; 128];
        map[' ' as usize] = Some(KeyPress { pos: 109 });
        for (pos, c) in layer.into_iter().enumerate() {
            if *c < (128 as char) {
                map[*c as usize] = Some(KeyPress { pos });
            }
        }

        for c in ALL_CHARS.chars() {
            assert!(map[c as usize].is_some(), "missing char: {}", c);
        }

        LayoutPosMap(map)
    }
}

impl LayoutPosMap {
    pub fn get_key_position(&self, kc: char) -> &Option<KeyPress> {
        let LayoutPosMap(ref map) = *self;
        if kc < (128 as char) {
            &map[kc as usize]
        } else {
            &KP_NONE
        }
    }
}

impl LayoutPermutations {
    // for now, I will ignore the num_swaps/depth variable; and always search adjacent layouts
    // which are 1 swap away
    pub fn new(layout: &Layout, _: usize) -> LayoutPermutations {
        let mut swaps = Vec::new();
        for i in 0..80 {
            for j in (i + 1)..80 {
                swaps.push((to_index(i), to_index(j)));
            }
        }
        LayoutPermutations {
            orig_layout: layout.clone(),
            swaps,
            index: 0,
        }
    }
}

impl Iterator for LayoutPermutations {
    type Item = Layout;

    fn next(&mut self) -> Option<Layout> {
        if self.index == self.swaps.len() {
            None
        } else {
            let mut current_layout = self.orig_layout.clone();
            let KeyMap(ref mut layer) = current_layout.0;

            let (i, j) = self.swaps[self.index];
            layer.swap(i, j);

            self.index += 1;
            return Some(current_layout);
        }
    }
}

fn convert_for_printing(c: char) -> char {
    match c {
        '\0' => ' ',
        ' ' => 'S',
        _ => c,
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let KeyMap(ref layer) = self.0;
        for row in 0..3 {
            for col in 0..6 {
                if row == 2 && col == 5 {
                    continue;
                }
                let loc = row * 6 + col;
                write!(
                    f,
                    " {}   {} |",
                    convert_for_printing(layer[loc * 5 + 2]),
                    convert_for_printing(layer[loc * 5 + 3]),
                )?;
            }
            writeln!(f, "")?;
            for col in 0..6 {
                if row == 2 && col == 5 {
                    continue;
                }
                let loc = row * 6 + col;
                write!(f, "   {}   |", convert_for_printing(layer[loc * 5 + 4]))?;
            }
            writeln!(f, "")?;
            for col in 0..6 {
                if row == 2 && col == 5 {
                    continue;
                }
                let loc = row * 6 + col;
                write!(
                    f,
                    " {}   {} |",
                    convert_for_printing(layer[loc * 5 + 1]),
                    convert_for_printing(layer[loc * 5 + 0]),
                )?;
            }
            writeln!(f, "")?;
            writeln!(f, "------- ------- ------- ------- ------- -------")?;
        }
        Ok(())
    }
}
