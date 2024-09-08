/// Data structures and methods for creating and shuffling keyboard layouts.
extern crate rand;

use self::rand::random;
use std::fmt;

/* ----- *
 * TYPES *
 * ----- */

// KeyMap format:
//
//            col 0      col 1       col 2
//
//         5  6  7 |             |
// row 0   4  8  0 |             |
//         3  2  1 |             |
//        --------- ------------- -------------
//                 |             |
// row 1           |             |
//                 |             |
//        --------- ------------- -------------
//                 |             |
// row 2           |             |
//                 |             |
//
// index%9 = relative location in key
// (index/9)/3 = row
// (index/9)%3 = col
//
// (row*3+col)*9+(relative location in key)=index

pub struct KeyMap<T>(pub [T; 81]);

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

pub static LETTER_SPOTS: [usize; 26] = [
    1, 8, 11, 17, 21, 26, 27, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 49, 53, 61, 62, 63, 69, 71,
    77, 80,
];

/* ------- *
 * STATICS *
 * ------- */

#[rustfmt::skip]
pub static INIT_LAYOUT: Layout = Layout(KeyMap([
//row 0
' ','c',' ',' ',' ',' ',' ',' ','t',
' ',' ','p',' ',' ',' ',' ',' ','h',
' ',' ',' ','b',' ',' ',' ',' ','s',
//row 1
'l',' ',' ',' ',' ',' ',' ',' ','i',
'j','v','f','q','z','x','k','y','a',
' ',' ',' ',' ','w',' ',' ',' ','e',
//row 2
' ',' ',' ',' ',' ',' ',' ','g','n',
'd',' ',' ',' ',' ',' ','u',' ','o',
' ',' ',' ',' ',' ','m',' ',' ','r',
]));

#[rustfmt::skip]
pub static MESSAGEASE_LAYOUT: Layout = Layout(KeyMap([
//row 0
' ','v',' ',' ',' ',' ',' ',' ','a',
' ',' ','l',' ',' ',' ',' ',' ','n',
' ',' ',' ','x',' ',' ',' ',' ','i',
//row 1
'k',' ',' ',' ',' ',' ',' ',' ','h',
'b','j','d','g','c','q','u','p','o',
' ',' ',' ',' ','m',' ',' ',' ','r',
//row 2
' ',' ',' ',' ',' ',' ',' ','y','t',
'z',' ',' ',' ',' ',' ','w',' ','e',
' ',' ',' ',' ',' ','f',' ',' ','s',
]));

#[rustfmt::skip]
pub static THUMB_KEY_LAYOUT: Layout = Layout(KeyMap([
//row 0
' ','w',' ',' ',' ',' ',' ',' ','s',
' ',' ','g',' ',' ',' ',' ',' ','r',
' ',' ',' ','u',' ',' ',' ',' ','o',
//row 1
'm',' ',' ',' ',' ',' ',' ',' ','n',
'p','y','x','v','k','j','q','b','h',
' ',' ',' ',' ','l',' ',' ',' ','a',
//row 2
' ',' ',' ',' ',' ',' ',' ','c','t',
'z',' ',' ',' ',' ',' ','f',' ','i',
' ',' ',' ',' ',' ','d',' ',' ','e',
]));

pub static KP_NONE: Option<KeyPress> = None;

/* ----- *
 * IMPLS *
 * ----- */

impl Layout {
    pub fn shuffle(&mut self, times: usize) {
        for _ in 0..times {
            let i = LETTER_SPOTS[random::<usize>() % 26];
            let j = LETTER_SPOTS[random::<usize>() % 26];
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
        for i in 0..26 {
            for j in (i + 1)..26 {
                swaps.push((LETTER_SPOTS[i], LETTER_SPOTS[j]));
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

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let KeyMap(ref layer) = self.0;
        for row in 0..3 {
            for col in 0..3 {
                let loc = row * 3 + col;
                write!(
                    f,
                    " {} {} {} |",
                    layer[loc * 9 + 5],
                    layer[loc * 9 + 6],
                    layer[loc * 9 + 7]
                )?;
            }
            writeln!(f, "")?;
            for col in 0..3 {
                let loc = row * 3 + col;
                write!(
                    f,
                    " {} {} {} |",
                    layer[loc * 9 + 4],
                    layer[loc * 9 + 8],
                    layer[loc * 9 + 0]
                )?;
            }
            writeln!(f, "")?;
            for col in 0..3 {
                let loc = row * 3 + col;
                write!(
                    f,
                    " {} {} {} |",
                    layer[loc * 9 + 3],
                    layer[loc * 9 + 2],
                    layer[loc * 9 + 1]
                )?;
            }
            writeln!(f, "")?;
            writeln!(f, "------- ------- -------")?;
        }
        Ok(())
    }
}
