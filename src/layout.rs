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
//          5   6   7 |  14  15  16 |  23  24  25 |  32  33  34 |  41  42  43
// row 0    4   8   0 |  13  17   9 |  22  26  18 |  31  35  27 |  40  44  36
//          3   2   1 |  12  11  10 |  21  20  19 |  30  29  28 |  39  38  37
//        ------------ ------------- ------------- ------------- -------------
//         50  51  52 |  59  60  61 |  68  69  70 |  77  78  79 |  86  87  88
// row 1   49  53  45 |  58  62  54 |  67  71  63 |  76  80  72 |  85  89  81
//         48  47  46 |  57  56  55 |  66  65  64 |  75  74  73 |  84  83  82
//        ------------ ------------- ------------- ------------- -------------
//                    | 104 105 106 | 113 114 115 | 122 123 124 |
// row 2     shift    | 103 107  99 | 112 116 108 | 121 125 117 |  backspace
//                    | 102 101 100 | 111 110 109 | 120 119 118 |
//
// index%9 = relative location in key
// (index/9)/5 = row
// (index/9)%5 = col
//
// (row*5+col)*9+(relative location in key)=index

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

// skipping symbols ()[]{}<> for now
#[rustfmt::skip]
pub static INIT_LAYOUT: Layout = Layout(KeyMap([
//row 0
' ',' ',' ',' ',' ',' ',' ',' ',' ',
'e',' ','y','i','o',' ',' ',' ',' ',
'r',' ','k','d','f',' ',' ',' ',' ',
' ',' ',' ',' ',' ',' ',' ',' ',' ',
'w',' ',' ',' ',' ',' ',' ',' ',' ',
//row 1
'm','z',' ','x','c',' ',' ',' ',' ',
'a',' ','u','j',' ',' ',' ',' ',' ',
' ',' ',' ',' ',' ',' ',' ',' ',' ',
's',' ','l',' ','p',' ',' ',' ',' ',
'v',' ',' ',' ',' ',' ',' ',' ',' ',
//row 2
' ',' ',' ',' ',' ',' ',' ',' ',' ',
'n',' ',' ','g','q',' ',' ',' ',' ',
't',' ',' ','b','h',' ',' ',' ',' ',
' ',' ',' ',' ',' ',' ',' ',' ',' ',
]));

#[rustfmt::skip]
pub static MESSAGEASE_LAYOUT: Layout = Layout(KeyMap([
//row 0
' ',' ',' ',' ',' ',' ',' ',' ',' ',
' ','v',' ',' ',' ',' ',' ',' ','a',
' ',' ','l',' ',' ',' ',' ',' ','n',
' ',' ',' ','x',' ',' ',' ',' ','i',
' ',' ',' ',' ',' ',' ',' ',' ',' ',
//row 1
' ',' ',' ',' ',' ',' ',' ',' ',' ',
'k',' ',' ',' ',' ',' ',' ',' ','h',
'b','j','d','g','c','q','u','p','o',
' ',' ',' ',' ','m',' ',' ',' ','r',
' ',' ',' ',' ',' ',' ',' ',' ',' ',
//row 2
' ',' ',' ',' ',' ',' ',' ',' ',' ',
' ',' ',' ',' ',' ',' ',' ','y','t',
'z',' ',' ',' ',' ',' ','w',' ','e',
' ',' ',' ',' ',' ','f',' ',' ','s',
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
                write!(
                    f,
                    " {} {} {} |",
                    layer[loc * 9 + 5],
                    layer[loc * 9 + 6],
                    layer[loc * 9 + 7]
                )?;
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
                    layer[loc * 9 + 4],
                    layer[loc * 9 + 8],
                    layer[loc * 9 + 0]
                )?;
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
                    layer[loc * 9 + 3],
                    layer[loc * 9 + 2],
                    layer[loc * 9 + 1]
                )?;
            }
            writeln!(f, "")?;
            writeln!(f, "------- ------- ------- ------- -------")?;
        }
        Ok(())
    }
}
