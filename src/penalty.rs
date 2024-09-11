use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
/// Methods for calculating the penalty of a keyboard layout given an input
/// corpus string.
use std::vec::Vec;

use layout::convert_for_printing;
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
    penalties.push(KeyPenalty {
        name: "base penalty",
    });

    // Swipe penalty.
    penalties.push(KeyPenalty {
        name: "swipe penalty",
    });

    // thumb travel distance penalty
    penalties.push(KeyPenalty {
        name: "thumb travel distance penalty",
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
        total += penalty_for_quartad(string, *count, &position_map, &mut result, detailed, layout);
    }

    (total, total / (len as f64), result)
}

fn penalty_for_quartad<'a, 'b>(
    string: &'a str,
    count: usize,
    position_map: &'b LayoutPosMap,
    result: &'b mut Vec<KeyPenaltyResult<'a>>,
    detailed: bool,
    layout: &Layout,
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

    penalize(
        string, count, &curr, old1, old2, old3, result, detailed, layout,
    )
}

// https://github.com/dessalines/thumb-key?tab=readme-ov-file#thumb-key-letter-positions
// Prioritize bottom, and right side of keyboard. So EAO should be on the right side, and bottom to
// top, while TNS is on the left side.
//
// this penalty is only applied to taps, as the reason for this penalty is you type the frequent
// key then press enter (on left/bottom) side. And applying this reason to swipes, the direction
// now matters too. But I just don't feel like implementing that, and it might just add noise.
#[rustfmt::skip]
pub static BASE_TAP_PENALTY: [[f64; 3]; 4] = [
    [0.2, 0.15, 0.1],
    [0.15, 0.1, 0.05],
    [0.1, 0.05, 0.0],
    [0.0, 0.0, 0.0], // 0 penalty for space key
];

// Time (in seconds) penalized for each swipe
pub static SWIPE_PENALTY: f64 = 0.5;

// constants taken from https://www.exideas.com/ME/ICMI2003Paper.pdf
// Time (in seconds) taken to tap (finger down, then finger up)
pub static A: f64 = 0.127;
// assuming each key is a 1-unit by 1-unit square, this is the distance a swipe takes (slightly
// longer than a side length)
// here, I assume each swipe is the same distance, independent of direction
pub static D_SWIPE: f64 = 1.3;

// Time (in seconds) gained back for typing 3,4 keystrokes in a row with alternating thumbs
pub static LENGTH_3_ALTERNATION_BONUS: f64 = 0.2;
pub static LENGTH_4_ALTERNATION_BONUS: f64 = 0.3;

// if your thumb starts at position (x_start,y_start), and needs to travel to a button (with the
// given width) at (x_end,y_end) where dist=sqrt((x_start-x_end)^2 + (y_start-y_end)^2)
//
// then this function returns the amount of time needed to travel and press (finger down, then
// finger up) the button
fn fitts_law(dist: f64, width: f64) -> f64 {
    A.max(1.0 / 4.9 * f64::log2(dist / width + 1.0))
}

fn square(x: f64) -> f64 {
    x * x
}

fn distance(point0: (f64, f64), point1: (f64, f64)) -> f64 {
    f64::sqrt(square(point0.0 - point1.0) + square(point0.1 - point1.1))
}

fn get_coordinates(key: &KeyPress) -> (f64, f64) {
    let spot = key.pos / 9;
    ((spot / 3) as f64, (spot % 3) as f64)
}

fn get_base_tap_penalty(key: &KeyPress) -> f64 {
    let spot = key.pos / 9;
    BASE_TAP_PENALTY[spot / 3][spot % 3]
}

fn is_tap(key: &KeyPress) -> bool {
    key.pos % 9 == 8
}

fn get_column(key: &KeyPress) -> usize {
    (key.pos / 9) % 3
}

// is typed with left thumb
fn is_left(column: usize) -> bool {
    column == 0 || column == 1
}

// is typed with right thumb
fn is_right(column: usize) -> bool {
    column == 2 || column == 1
}

// returns coordinate of end of swipe, and width
fn get_swipe_details(old1: &KeyPress, layout: &Layout) -> ((f64, f64), f64) {
    let spot = old1.pos / 9;
    let dir = old1.pos % 9;

    let mut end_of_swipe_coords = get_coordinates(old1);
    let (sin, cos) = f64::sin_cos((dir as f64) / 8.0 * 2.0 * std::f64::consts::PI);
    end_of_swipe_coords.0 += D_SWIPE * sin;
    end_of_swipe_coords.1 += D_SWIPE * cos;

    // get next and previous swipe-letters (if they exist) to determine how precise the
    // swipe-direction has to be
    let width = {
        let next_delta = if layout.get(spot * 9 + ((dir + 1) % 8)) != '\0' {
            1.0
        } else {
            4.0
        };
        let prev_delta = if layout.get(spot * 9 + ((dir + 7) % 8)) != '\0' {
            -1.0
        } else {
            -4.0
        };
        (next_delta - prev_delta) / 8.0
    };

    if false {
        println!(" ------- ");
        println!(
            "| {} {} {} |",
            convert_for_printing(layout.get(spot * 9 + 5)),
            convert_for_printing(layout.get(spot * 9 + 6)),
            convert_for_printing(layout.get(spot * 9 + 7)),
        );
        println!(
            "| {} {} {} |",
            convert_for_printing(layout.get(spot * 9 + 4)),
            convert_for_printing(layout.get(spot * 9 + 8)),
            convert_for_printing(layout.get(spot * 9 + 0)),
        );
        println!(
            "| {} {} {} |",
            convert_for_printing(layout.get(spot * 9 + 3)),
            convert_for_printing(layout.get(spot * 9 + 2)),
            convert_for_printing(layout.get(spot * 9 + 1)),
        );
        println!(" ------- ");

        println!(
            "dir {} swipe delta {:.4} {:.4} width {}",
            dir, sin, cos, width,
        );
        println!();
    }

    (end_of_swipe_coords, width)
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
    layout: &Layout,
) -> f64 {
    let len = string.len();
    let count = count as f64;
    let mut total = 0.0;

    // One key penalties.
    let slice1 = &string[(len - 1)..len];

    if is_tap(curr) {
        // Base tap penalty
        let base_tap_penalty = get_base_tap_penalty(curr) * count;
        if detailed {
            *result[0].high_keys.entry(slice1).or_insert(0.0) += base_tap_penalty;
            result[0].total += base_tap_penalty;
        }
        total += base_tap_penalty;
    } else {
        // Swipe penalty
        let swipe_penalty = SWIPE_PENALTY * count;
        if detailed {
            *result[1].high_keys.entry(slice1).or_insert(0.0) += swipe_penalty;
            result[1].total += swipe_penalty;
        }
        total += swipe_penalty;
    }

    // Two key penalties.
    let old1 = match *old1 {
        Some(ref o) => o,
        None => return total,
    };

    let slice2 = &string[(len - 2)..len];

    {
        let mut penalty = 0.0;

        // previous key is a tap
        if is_tap(old1) {
            penalty += fitts_law(distance(get_coordinates(old1), get_coordinates(curr)), 1.0);
        } else {
            // previous key is a swipe
            let (end_of_swipe_coords, width) = get_swipe_details(old1, layout);
            penalty += fitts_law(D_SWIPE, width) - A;
            penalty += fitts_law(distance(end_of_swipe_coords, get_coordinates(curr)), 1.0);
        }

        penalty *= count;

        if detailed {
            *result[2].high_keys.entry(slice2).or_insert(0.0) += penalty;
            result[2].total += penalty;
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

        let col_old2 = get_column(old2);
        let col_old1 = get_column(old1);
        let col_curr = get_column(curr);

        if col_old2 != col_old1 && col_old1 != col_curr {
            if is_right(col_old2) && is_left(col_old1) && is_right(col_curr) {
                penalty -= LENGTH_3_ALTERNATION_BONUS;
            }
            if is_left(col_old2) && is_right(col_old1) && is_left(col_curr) {
                penalty -= LENGTH_3_ALTERNATION_BONUS;
            }
        }

        penalty *= count;

        if detailed {
            *result[3].high_keys.entry(slice3).or_insert(0.0) += penalty;
            result[3].total += penalty;
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

        let col_old3 = get_column(old3);
        let col_old2 = get_column(old2);
        let col_old1 = get_column(old1);
        let col_curr = get_column(curr);

        if col_old3 != col_old2 && col_old2 != col_old1 && col_old1 != col_curr {
            if is_left(col_old3) && is_right(col_old2) && is_left(col_old1) && is_right(col_curr) {
                penalty -= LENGTH_4_ALTERNATION_BONUS;
            }
            if is_right(col_old3) && is_left(col_old2) && is_right(col_old1) && is_left(col_curr) {
                penalty -= LENGTH_4_ALTERNATION_BONUS;
            }
        }

        penalty *= count;

        if detailed {
            *result[4].high_keys.entry(slice4).or_insert(0.0) += penalty;
            result[4].total += penalty;
        }
        total += penalty;
    }

    total
}
