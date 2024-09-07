/// Data structures and methods for creating and shuffling keyboard layouts.
extern crate rand;

use self::rand::random;
use std::fmt;

/* ----- *
 * TYPES *
 * ----- */

// KeyMap format:
//
//            col 0         col 1         col 2         col 3         col 4
//
//          8   1   2 |  17  10  11 |  26  19  20 |  35  28  29 |  44  37  38
// row 0    7   0   3 |  16   9  12 |  25  18  21 |  34  27  30 |  43  36  39
//          6   5   4 |  15  14  13 |  24  23  22 |  33  32  31 |  42  41  40
//        ------------ ------------- ------------- ------------- -------------
//         53  46  47 |  62  55  56 |  71  64  65 |  80  73  74 |  89  82  83
// row 1   52  45  48 |  61  54  57 |  70  63  66 |  79  72  75 |  88  81  84
//         51  50  49 |  60  59  58 |  69  68  67 |  78  77  76 |  87  86  85
//        ------------ ------------- ------------- ------------- -------------
//                    | 107 100 101 | 116 109 110 | 125 118 119 |
// row 2     shift    | 106  99 102 | 115 108 111 | 124 117 120 |  backspace
//                    | 105 104 103 | 114 113 112 | 123 122 121 |
//
// index%9 = relative location in key
// (index/9)/5 = row
// (index/9)%5 = col

// convert number in range [0,117) to [0,90) union [99,126)
pub fn shift_index(mut i: usize) -> usize {
    if i >= 90 {
        i += 9;
    }
    i
}

pub struct KeyMap<T>(pub [T; 126]);

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
' ',' ',' ',' ',' ',' ',' ',' ',' ',
'e',' ','y','i','o',' ',' ',' ',' ',
'r',' ','k','d','f',' ',' ',' ',' ',
'.','\"','\'','-',',','?','!','*',' ',
'w',' ',' ',' ',' ',' ',' ',' ',' ',
//row 1
'm','z',' ','x','c',' ',' ',' ',' ',
'a',' ','u','j',' ',' ',' ',' ',' ',
' ',' ',' ',' ',' ',' ',' ',' ',' ',
's',' ','l','_','p',' ',' ',' ',' ',
'v',' ',' ',' ',' ',' ',' ',' ',' ',
//row 2
' ',' ',' ',' ',' ',' ',' ',' ',' ',
'n',' ',' ','g','q',' ',' ',' ',' ',
't',' ',' ','b','h',' ',' ',' ',' ',
' ',' ',' ',' ',' ',' ',' ',' ',' ',
]));

pub static KP_NONE: Option<KeyPress> = None;

/* ----- *
 * IMPLS *
 * ----- */

impl Layout {
    pub fn shuffle(&mut self, times: usize) {
        for _ in 0..times {
            let i = shift_index(random::<usize>() % 117);
            let j = shift_index(random::<usize>() % 117);
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
        for i in 0..117 {
            for j in (i + 1)..117 {
                swaps.push((shift_index(i), shift_index(j)));
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
            for col in 0..5 {
                if row == 2 && col == 4 {
                    break;
                }
                let loc = row * 5 + col;
                if row == 0 && col == 4 {
                    write!(f, " {}   {} |", '{', '}')?;
                } else {
                    write!(f, "   {}   |", layer[loc * 5 + 1])?;
                }
            }
            writeln!(f, "")?;
            for col in 0..5 {
                if row == 2 && col == 4 {
                    break;
                }
                let loc = row * 5 + col;
                write!(
                    f,
                    " {} {} {} |",
                    layer[loc * 5 + 4],
                    layer[loc * 5],
                    layer[loc * 5 + 3]
                )?;
            }
            writeln!(f, "")?;
            for col in 0..5 {
                if row == 2 && col == 4 {
                    break;
                }
                let loc = row * 5 + col;
                if row == 0 && col == 4 {
                    write!(f, " <   > |")?;
                } else {
                    write!(f, "   {}   |", layer[loc * 5 + 2])?;
                }
            }
            writeln!(f, "")?;
            writeln!(f, "------- ------- ------- ------- -------")?;
        }
        Ok(())
    }
}
