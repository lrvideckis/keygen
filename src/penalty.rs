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
    penalties.push(KeyPenalty { name: "base" });

    // Penalize for swiping
    penalties.push(KeyPenalty {
        name: "swipe penalty",
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

// constants taken from https://www.exideas.com/ME/ICMI2003Paper.pdf

// Time (in seconds) taken to tap (finger down, then finger up)
pub static A: f64 = 0.127;
// assuming each key is a 1-unit by 1-unit square, this is the distance a swipe takes (slightly
// longer than a side length)
// here, I assume each swipe is the same distance, independent of direction
pub static D_SWIPE: f64 = 1.3;
// extra time (in seconds) penalized for each swipe
pub static SWIPE_PENALTY: f64 = 0.3;

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
    _: &Option<KeyPress>,
    _: &Option<KeyPress>,
    result: &'b mut Vec<KeyPenaltyResult<'a>>,
    detailed: bool,
    layout: &Layout,
) -> f64 {
    let len = string.len();
    let count = count as f64;
    let mut total = 0.0;

    // One key penalties.
    let slice1 = &string[(len - 1)..len];

    // 0: Base penalty.
    let base = (if curr.pos % 9 == 8 {
        0.0
    } else {
        SWIPE_PENALTY
    }) * count;
    if detailed {
        *result[0].high_keys.entry(slice1).or_insert(0.0) += base;
        result[0].total += base;
    }
    total += base;

    // Two key penalties.
    let old1 = match *old1 {
        Some(ref o) => o,
        None => return total,
    };

    let slice2 = &string[(len - 2)..len];

    let mut penalty = 0.0;

    // previous key is a tap
    if old1.pos % 9 == 8 {
        let dist = distance(get_coordinates(old1), get_coordinates(curr));
        penalty += fitts_law(dist, 1.0);
    } else {
        // previous key is a swipe
        let (end_of_swipe_coords, width) = get_swipe_details(old1, layout);
        penalty += fitts_law(D_SWIPE, width) - A;
        penalty += fitts_law(distance(end_of_swipe_coords, get_coordinates(curr)), 1.0);
    }

    penalty *= count;

    if detailed {
        *result[1].high_keys.entry(slice2).or_insert(0.0) += penalty;
        result[1].total += penalty;
    }
    total += penalty;
    total
}
