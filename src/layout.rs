/// Data structures and methods for creating and shuffling keyboard layouts.
extern crate rand;

use self::rand::random;
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
    swaps: Vec<(usize, usize, usize)>,
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
'a','b','c','d','e',
'f','g','h','i','j',
'k','l','m','n','o',
'p','q','r','s','t',
'u','v','w','x','y',
'z','`','~','!','@',
//row 1
'#','\0','$','%','^',
'&','*','-','_','+',
'=','\\','|',';',':',
'\'','\"',',','.','/',
'?','\0','\0','\0','\0',
'\0','\0','\0','\0','\0',
//row 2
'\0','\0','\0','\0','\0',
'\0','\0','\0','\0','\0',
'\0','\0','\0','\0','\0',
'\0','\0','\0','\0','\0',
'\0','\0','\0','\0','\0',
]));

pub static KP_NONE: Option<KeyPress> = None;

/* ----- *
 * IMPLS *
 * ----- */

fn is_tap(key: usize) -> bool {
    key % 5 == 4
}

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
                if is_tap(i) == is_tap(j) {
                    for k in j..81 {
                        if is_tap(j) == is_tap(k) {
                            swaps.push((i, j, k));
                        }
                    }
                }
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

            let (i, j, k) = self.swaps[self.index];
            layer.swap(i, j);
            layer.swap(j, k);

            self.index += 1;
            return Some(current_layout);
        }
    }
}

pub fn convert_for_printing(c: char) -> char {
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
                write!(f, "   {}   |", convert_for_printing(layer[loc * 5 + 4]),)?;
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
