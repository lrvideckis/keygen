/// Data structures and methods for creating and shuffling keyboard layouts.
extern crate rand;

use self::rand::random;
use std::fmt;

/* ----- *
 * TYPES *
 * ----- */

// KeyMap format:
//
//            col 0   col 1     col 2
//
//         5  6  7 |         |
// row 0   4  8  0 |         |
//         3  2  1 |         |
//        --------- --------- ---------
//                 |         |
// row 1           |         |
//                 |         |
//        --------- --------- ---------
//                 |         |
// row 2           |         |
//                 |         |
//        --------- --------- ---------
//                 |         |
// row 3           |  space  |
//                 |         |
//
// index%9 = relative location in key
// (index/9)/3 = row
// (index/9)%3 = col
//
// (row*3+col)*9+(relative location in key)=index
//
// if relative location in key != 8, then it's an integer in [0,8) representing a swipe such that:
// (relative location in key) / 8.0 * 2PI = angle in radians of swipe.
//
// note the standard programming grid has the y-axis flipped compared to the standard x-y euclidean
// plane in math

pub struct KeyMap<T>(pub [T; 108]);

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

#[rustfmt::skip]
pub static INIT_LAYOUT: Layout = Layout(KeyMap([
//row 0
'\0','m','\0','\0','\0','\0','\0','\0','r',
'\0','\0','d','\0','\0','\0','\0','\0','i',
'\0','\0','\0','g','\0','\0','\0','\0','n',
//row 1
'u','\0','\0','\0','\0','\0','\0','\0','o',
'k','x','y','p','j','q','v','z','e',
'\0','\0','\0','\0','l','\0','\0','\0','a',
//row 2
'\0','\0','\0','\0','\0','\0','\0','w','s',
'f','\0','\0','\0','\0','\0','b','\0','t',
'\0','\0','\0','\0','\0','c','\0','\0','h',
//row 3
'\0','\0','\0','\0','\0','\0','\0','\0','\0',
'\0','\0','\0','\0','\0','\0','\0','\0',' ',
'\0','\0','\0','\0','\0','\0','\0','\0','\0',
]));

#[rustfmt::skip]
pub static MESSAGEASE_LAYOUT: Layout = Layout(KeyMap([
//row 0
'\0','v','\0','\0','\0','\0','\0','\0','a',
'\0','\0','l','\0','\0','\0','\0','\0','n',
'\0','\0','\0','x','\0','\0','\0','\0','i',
//row 1
'k','\0','\0','\0','\0','\0','\0','\0','h',
'b','j','d','g','c','q','u','p','o',
'\0','\0','\0','\0','m','\0','\0','\0','r',
//row 2
'\0','\0','\0','\0','\0','\0','\0','y','t',
'z','\0','\0','\0','\0','\0','w','\0','e',
'\0','\0','\0','\0','\0','f','\0','\0','s',
//row 3
'\0','\0','\0','\0','\0','\0','\0','\0','\0',
'\0','\0','\0','\0','\0','\0','\0','\0',' ',
'\0','\0','\0','\0','\0','\0','\0','\0','\0',
]));

#[rustfmt::skip]
pub static THUMB_KEY_LAYOUT: Layout = Layout(KeyMap([
//row 0
'\0','w','\0','\0','\0','\0','\0','\0','s',
'\0','\0','g','\0','\0','\0','\0','\0','r',
'\0','\0','\0','u','\0','\0','\0','\0','o',
//row 1
'm','\0','\0','\0','\0','\0','\0','\0','n',
'p','y','x','v','k','j','q','b','h',
'\0','\0','\0','\0','l','\0','\0','\0','a',
//row 2
'\0','\0','\0','\0','\0','\0','\0','c','t',
'z','\0','\0','\0','\0','\0','f','\0','i',
'\0','\0','\0','\0','\0','d','\0','\0','e',
//row 3
'\0','\0','\0','\0','\0','\0','\0','\0','\0',
'\0','\0','\0','\0','\0','\0','\0','\0',' ',
'\0','\0','\0','\0','\0','\0','\0','\0','\0',
]));

pub static KP_NONE: Option<KeyPress> = None;

/* ----- *
 * IMPLS *
 * ----- */

impl Layout {
    pub fn shuffle(&mut self, times: usize) {
        for _ in 0..times {
            let i = random::<usize>() % 81;
            let j = random::<usize>() % 81;
            let KeyMap(ref mut layer) = self.0;
            layer.swap(i, j);
        }
    }

    pub fn get_position_map(&self) -> LayoutPosMap {
        let KeyMap(ref layer) = self.0;
        let mut map = [None; 128];
        for (pos, c) in layer.into_iter().enumerate() {
            if *c < (128 as char) {
                map[*c as usize] = Some(KeyPress { pos });
            }
        }

        LayoutPosMap(map)
    }

    pub fn get(&self, pos: usize) -> char {
        let KeyMap(ref layer) = self.0;
        layer[pos]
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
        for i in 0..81 {
            for j in (i + 1)..81 {
                swaps.push((i, j));
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
        let len = self.swaps.len();
        if self.index == len * len {
            None
        } else {
            let mut current_layout = self.orig_layout.clone();
            let KeyMap(ref mut layer) = current_layout.0;

            let swaps: [usize; 2] = [self.index % len, (self.index / len) % len];

            for swap in swaps {
                let (i, j) = self.swaps[swap];
                layer.swap(i, j);
            }

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
        for row in 0..4 {
            for col in 0..3 {
                let loc = row * 3 + col;
                write!(
                    f,
                    " {} {} {} |",
                    convert_for_printing(layer[loc * 9 + 5]),
                    convert_for_printing(layer[loc * 9 + 6]),
                    convert_for_printing(layer[loc * 9 + 7]),
                )?;
            }
            writeln!(f, "")?;
            for col in 0..3 {
                let loc = row * 3 + col;
                write!(
                    f,
                    " {} {} {} |",
                    convert_for_printing(layer[loc * 9 + 4]),
                    convert_for_printing(layer[loc * 9 + 8]),
                    convert_for_printing(layer[loc * 9 + 0]),
                )?;
            }
            writeln!(f, "")?;
            for col in 0..3 {
                let loc = row * 3 + col;
                write!(
                    f,
                    " {} {} {} |",
                    convert_for_printing(layer[loc * 9 + 3]),
                    convert_for_printing(layer[loc * 9 + 2]),
                    convert_for_printing(layer[loc * 9 + 1]),
                )?;
            }
            writeln!(f, "")?;
            writeln!(f, "------- ------- -------")?;
        }
        Ok(())
    }
}
