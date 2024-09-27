use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
/// Methods for calculating the penalty of a keyboard layout given an input
/// corpus string.
use std::vec::Vec;

use layout::get_coordinates;
use layout::get_coordinates_float;
use layout::get_end_of_swipe_coords;
use layout::is_space;
use layout::is_tap;
use layout::same_hand;
use layout::swipe_is_good_for_hand;
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
    // thumb can move more naturally from top-side to bottom-middle
    // https://github.com/Julow/Unexpected-Keyboard/issues/740#issuecomment-2350971805
    penalties.push(KeyPenalty {
        name: "base penalty",
    });

    // Swipe penalty.
    // Also includes extra penalty depending on direction, and which thumb
    // https://github.com/Julow/Unexpected-Keyboard/issues/740#issuecomment-2361848821
    penalties.push(KeyPenalty {
        name: "swipe penalty",
    });

    // single thumb travel distance penalty
    penalties.push(KeyPenalty {
        name: "single thumb travel distance penalty",
    });

    // both thumbs travel distance penalty
    // but 0 keystrokes are typed with other thumb
    penalties.push(KeyPenalty {
        name: "both thumbs travel distance penalty, 0 in between",
    });

    // both thumbs travel distance penalty
    // but 1 keystroke is typed with other thumb
    penalties.push(KeyPenalty {
        name: "both thumbs travel distance penalty, 1 in between",
    });

    // both thumbs travel distance penalty
    // but 2 keystrokes are typed with other thumb
    penalties.push(KeyPenalty {
        name: "both thumbs travel distance penalty, 2 in between",
    });

    // Bonus for alternating thumbs for 2 keys
    penalties.push(KeyPenalty {
        name: "length 2 alternation bonus",
    });

    // Bonus for alternating thumbs for 3 keys
    penalties.push(KeyPenalty {
        name: "length 3 alternation bonus",
    });

    // Bonus for alternating thumbs for 4 keys
    penalties.push(KeyPenalty {
        name: "length 4 alternation bonus",
    });

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

// https://github.com/Julow/Unexpected-Keyboard/issues/740#issuecomment-2350971805
#[rustfmt::skip]
pub static BASE_PENALTY: [[f64; 6]; 3] = [
    [0.0, 0.0, 0.1, 0.1, 0.0, 0.0],
    [0.5, 0.0, 0.0, 0.0, 0.0, 0.5],
    [0.0, 0.5, 0.0, 0.0, 0.5, 0.0],
];

// penalty for each swipe
pub static SWIPE_PENALTY: f64 = 2.0;
pub static EXTRA_SWIPE_PENALTY: f64 = 2.0;

// constants taken from https://www.exideas.com/ME/ICMI2003Paper.pdf
// Time (in seconds) taken to tap (finger down, then finger up)
pub static A: f64 = 0.127;
// assuming each key is a 1-unit by 1-unit square, this is the distance a swipe takes (slightly
// longer than a side length)
// here, I assume each swipe is the same distance, independent of direction
pub static D_SWIPE: f64 = 1.5;

// penalty gained back for typing 23,4 keystrokes in a row with alternating thumbs
pub static LENGTH_2_ALTERNATION_BONUS: f64 = -0.15;
pub static LENGTH_3_ALTERNATION_BONUS: f64 = -0.3;
pub static LENGTH_4_ALTERNATION_BONUS: f64 = -0.7;

pub static TWO_THUMB_3_4_ALTERNATION_WEIGHT: f64 = 0.5;

// if your thumb starts at position (x_start,y_start), and needs to travel to a button (with the
// given width) at (x_end,y_end) where dist=sqrt((x_start-x_end)^2 + (y_start-y_end)^2)
//
// then this function returns the amount of time needed to travel and press (finger down, then
// finger up) the button
// width is assumed to be 1.0 for all swipes, since we only have diagonal swipes
fn fitts_law(dist: f64) -> f64 {
    A.max(1.0 / 4.9 * f64::log2(dist / 1.0 + 1.0))
}

fn square(x: f64) -> f64 {
    x * x
}

fn distance(point0: (f64, f64), point1: (f64, f64)) -> f64 {
    f64::sqrt(square(point0.0 - point1.0) + square(point0.1 - point1.1))
}

fn thumb_travel_penalty(old: &KeyPress, curr: &KeyPress) -> f64 {
    if is_tap(old) {
        // previous key is a tap
        fitts_law(distance(
            get_coordinates_float(old),
            get_coordinates_float(curr),
        ))
    } else {
        // previous key is a swipe
        let end_of_swipe_coords = get_end_of_swipe_coords(old);
        fitts_law(D_SWIPE) - A
            + fitts_law(distance(end_of_swipe_coords, get_coordinates_float(curr)))
    }
}

fn penalize<'a, 'b>(
    string: &'a str,
    count: usize,
    curr: &KeyPress,
    old1: &Option<KeyPress>,
    old2: &Option<KeyPress>,
    old3: &Option<KeyPress>,
    result: &'b mut Vec<KeyPenaltyResult<'a>>,
    detailed: bool,
) -> f64 {
    let len = string.len();
    let count = count as f64;
    let mut total = 0.0;

    // One key penalties.
    let slice1 = &string[(len - 1)..len];

    if !is_space(curr) {
        {
            let (row, col) = get_coordinates(curr);
            let base_penalty = BASE_PENALTY[row][col] * count;
            if detailed {
                *result[0].high_keys.entry(slice1).or_insert(0.0) += base_penalty;
                result[0].total += base_penalty;
            }
            total += base_penalty;
        }

        if !is_tap(curr) {
            let mut swipe_penalty = SWIPE_PENALTY * count;
            if !swipe_is_good_for_hand(curr) {
                swipe_penalty += EXTRA_SWIPE_PENALTY * count;
            }
            if detailed {
                *result[1].high_keys.entry(slice1).or_insert(0.0) += swipe_penalty;
                result[1].total += swipe_penalty;
            }
            total += swipe_penalty;
        }
    }
    // Two key penalties.
    let old1 = match *old1 {
        Some(ref o) => o,
        None => return total,
    };

    let slice2 = &string[(len - 2)..len];

    {
        let penalty = thumb_travel_penalty(old1, curr) * count;

        if detailed {
            *result[2].high_keys.entry(slice2).or_insert(0.0) += penalty;
            result[2].total += penalty;
        }
        total += penalty;
    }

    for c in slice2.chars() {
        if c == ' ' {
            return total;
        }
    }

    if same_hand(old1, curr) {
        let penalty = thumb_travel_penalty(old1, curr) * count;
        if detailed {
            *result[3].high_keys.entry(slice2).or_insert(0.0) += penalty;
            result[3].total += penalty;
        }
        total += penalty;
    } else {
        let penalty = LENGTH_2_ALTERNATION_BONUS * count;
        if detailed {
            *result[6].high_keys.entry(slice2).or_insert(0.0) += penalty;
            result[6].total += penalty;
        }
        total += penalty;
    }

    // Three key penalties.
    let old2 = match *old2 {
        Some(ref o) => o,
        None => return total,
    };

    let slice3 = &string[(len - 3)..len];
    for c in slice3.chars() {
        if c == ' ' {
            return total;
        }
    }

    {
        let mut penalty = 0.0;

        if !same_hand(old2, old1) && !same_hand(old1, curr) {
            penalty = LENGTH_3_ALTERNATION_BONUS * count;
        }

        if detailed {
            *result[7].high_keys.entry(slice3).or_insert(0.0) += penalty;
            result[7].total += penalty;
        }
        total += penalty;
    }

    if same_hand(old2, curr) && !same_hand(old2, old1) {
        let penalty = TWO_THUMB_3_4_ALTERNATION_WEIGHT * thumb_travel_penalty(old2, curr) * count;
        if detailed {
            *result[4].high_keys.entry(slice3).or_insert(0.0) += penalty;
            result[4].total += penalty;
        }
        total += penalty;
    }

    // Four key penalties.
    let old3 = match *old3 {
        Some(ref o) => o,
        None => return total,
    };

    let slice4 = &string[(len - 4)..len];
    for c in slice4.chars() {
        if c == ' ' {
            return total;
        }
    }
    {
        let mut penalty = 0.0;

        if !same_hand(old3, old2) && !same_hand(old2, old1) && !same_hand(old1, curr) {
            penalty = LENGTH_4_ALTERNATION_BONUS * count;
        }

        if detailed {
            *result[8].high_keys.entry(slice4).or_insert(0.0) += penalty;
            result[8].total += penalty;
        }
        total += penalty;
    }

    if same_hand(old3, curr) && !same_hand(old3, old1) && !same_hand(old3, old2) {
        let penalty = TWO_THUMB_3_4_ALTERNATION_WEIGHT * thumb_travel_penalty(old3, curr) * count;
        if detailed {
            *result[5].high_keys.entry(slice4).or_insert(0.0) += penalty;
            result[5].total += penalty;
        }
        total += penalty;
    }

    total
}
