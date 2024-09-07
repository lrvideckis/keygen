use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
/// Methods for calculating the penalty of a keyboard layout given an input
/// corpus string.
use std::vec::Vec;

use layout::KeyPress;
use layout::Layout;
use layout::LayoutPosMap;
use layout::KP_NONE;

pub struct KeyPenalty<'a> {
    name: &'a str,
}

#[derive(Clone)]
pub struct KeyPenaltyResult<'a> {
    pub name: &'a str,
    pub total: f64,
    pub high_keys: HashMap<&'a str, f64>,
}

pub struct QuartadList<'a>(HashMap<&'a str, usize>);

impl<'a> fmt::Display for KeyPenaltyResult<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.total)
    }
}

pub fn init<'a>() -> Vec<KeyPenalty<'a>> {
    let mut penalties = Vec::new();

    // Base penalty.
    penalties.push(KeyPenalty { name: "base" });

    penalties
}

pub fn prepare_quartad_list<'a>(
    string: &'a str,
    position_map: &'a LayoutPosMap,
) -> QuartadList<'a> {
    let mut range: Range<usize> = 0..0;
    let mut quartads: HashMap<&str, usize> = HashMap::new();
    for (i, c) in string.chars().enumerate() {
        match *position_map.get_key_position(c) {
            Some(_) => {
                range.end = i + 1;
                if range.end > 3 && range.start < range.end - 4 {
                    range.start = range.end - 4;
                }
                let quartad = &string[range.clone()];
                let entry = quartads.entry(quartad).or_insert(0);
                *entry += 1;
            }
            None => {
                range = (i + 1)..(i + 1);
            }
        }
    }

    QuartadList(quartads)
}

pub fn calculate_penalty<'a>(
    quartads: &QuartadList<'a>,
    len: usize,
    layout: &Layout,
    penalties: &'a Vec<KeyPenalty>,
    detailed: bool,
) -> (f64, f64, Vec<KeyPenaltyResult<'a>>) {
    let QuartadList(ref quartads) = *quartads;
    let mut result: Vec<KeyPenaltyResult> = Vec::new();
    let mut total = 0.0;

    if detailed {
        for penalty in penalties {
            result.push(KeyPenaltyResult {
                name: penalty.name,
                total: 0.0,
                high_keys: HashMap::new(),
            });
        }
    }

    let position_map = layout.get_position_map();
    for (string, count) in quartads {
        total += penalty_for_quartad(string, *count, &position_map, &mut result, detailed);
    }

    (total, total / (len as f64), result)
}

fn penalty_for_quartad<'a, 'b>(
    string: &'a str,
    count: usize,
    position_map: &'b LayoutPosMap,
    result: &'b mut Vec<KeyPenaltyResult<'a>>,
    detailed: bool,
) -> f64 {
    let mut chars = string.chars().into_iter().rev();
    let opt_curr = chars.next();
    let opt_old1 = chars.next();
    let opt_old2 = chars.next();
    let opt_old3 = chars.next();

    let curr = match opt_curr {
        Some(c) => match position_map.get_key_position(c) {
            &Some(ref kp) => kp,
            &None => return 0.0,
        },
        None => panic!("unreachable"),
    };
    let old1 = match opt_old1 {
        Some(c) => position_map.get_key_position(c),
        None => &KP_NONE,
    };
    let old2 = match opt_old2 {
        Some(c) => position_map.get_key_position(c),
        None => &KP_NONE,
    };
    let old3 = match opt_old3 {
        Some(c) => position_map.get_key_position(c),
        None => &KP_NONE,
    };

    penalize(string, count, &curr, old1, old2, old3, result, detailed)
}

// constants taken from https://www.exideas.com/ME/ICMI2003Paper.pdf

// Time (in seconds) taken to tap (finger down, then finger up)
pub static A: f64 = 0.127;
// assuming each key is a 1-unit by 1-unit square, this is the distance a swipe takes (also 1-unit)
// here, I assume each swipe is the same distance, independent of direction
pub static D_SWIPE: f64 = 1.0;

// if your thumb starts at position (x_start,y_start), and needs to travel to a button (with the
// given width) at (x_end,y_end) where dist=sqrt((x_start-x_end)^2 + (y_start-y_end)^2)
//
// then this function returns the amount of time needed to travel and press (finger down, then
// finger up) the button
fn fitts_law(dist: f64, width: f64) -> f64 {
    1.0 / 4.9 * f64::log2(dist / width + 1.0)
}

fn distance(point0: (f64, f64), point1: (f64, f64)) -> f64 {
    f64::sqrt(
        (point0.0 - point1.0) * (point0.0 - point1.0)
            + (point0.1 - point1.1) * (point0.1 - point1.1),
    )
}

fn get_coordinates(key: &KeyPress) -> (f64, f64) {
    let pos = key.pos / 9;
    ((pos / 5) as f64, (pos % 5) as f64)
}

fn penalize<'a, 'b>(
    string: &'a str,
    count: usize,
    curr: &KeyPress,
    old1: &Option<KeyPress>,
    _: &Option<KeyPress>,
    _: &Option<KeyPress>,
    result: &'b mut Vec<KeyPenaltyResult<'a>>,
    detailed: bool,
) -> f64 {
    let len = string.len();
    let count = count as f64;
    let mut penalty = 0.0;

    // Two key penalties.
    let old1 = match *old1 {
        Some(ref o) => o,
        None => return penalty,
    };

    let slice2 = &string[(len - 2)..len];
    for c in slice2.chars() {
        if c == ' ' {
            return penalty;
        }
    }

    // previous key is a tap
    if old1.pos % 9 == 0 {
        if old1.pos == curr.pos {
            penalty = A;
        } else {
            let dist = distance(get_coordinates(old1), get_coordinates(curr));
            penalty = fitts_law(dist, 1.0);
        }
    } else { // previous key is a swipe
    }

    penalty *= count;

    if detailed {
        *result[0].high_keys.entry(slice2).or_insert(0.0) += penalty;
        result[0].total += penalty;
    }
    penalty
}

//   8   1   2 |  17  10  11 |  26  19  20 |  35  28  29 |  44  37  38
//   7   0   3 |  16   9  12 |  25  18  21 |  34  27  30 |  43  36  39
//   6   5   4 |  15  14  13 |  24  23  22 |  33  32  31 |  42  41  40
// ------------ ------------- ------------- ------------- -------------
//  53  46  47 |  62  55  56 |  71  64  65 |  80  73  74 |  89  82  83
//  52  45  48 |  61  54  57 |  70  63  66 |  79  72  75 |  88  81  84
//  51  50  49 |  60  59  58 |  69  68  67 |  78  77  76 |  87  86  85
// ------------ ------------- ------------- ------------- -------------
//             | 107 100 101 | 116 109 110 | 125 118 119 |
//    shift    | 106  99 102 | 115 108 111 | 124 117 120 |  backspace
//             | 105 104 103 | 114 113 112 | 123 122 121 |
