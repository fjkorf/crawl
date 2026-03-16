//! SUBST, NSUBST, and SHUFFLE processing for .des vault maps.
//!
//! These transform map glyphs after Lua execution but before terrain conversion.

use rand::Rng;

/// Apply all substitutions to map lines.
pub fn apply_substitutions(
    map_lines: &mut Vec<String>,
    substs: &[String],
    nsubsts: &[String],
    shuffles: &[String],
) {
    // Apply SHUFFLEs first (permute glyph groups)
    for spec in shuffles {
        apply_shuffle(map_lines, spec);
    }

    // Apply SUBSTs (simple replacements)
    for spec in substs {
        apply_subst(map_lines, spec);
    }

    // Apply NSUBSTs (counted replacements)
    for spec in nsubsts {
        apply_nsubst(map_lines, spec);
    }
}

/// SUBST: A = x:5 .
/// Replace all occurrences of glyph A with weighted random choice from alternatives.
fn apply_subst(map_lines: &mut Vec<String>, spec: &str) {
    let Some((glyph_str, rhs)) = spec.split_once('=').or_else(|| spec.split_once(':')) else {
        return;
    };
    let glyph = glyph_str.trim().chars().next().unwrap_or('?');
    let choices = parse_weighted_choices(rhs.trim());
    if choices.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    for line in map_lines.iter_mut() {
        let mut new_line = String::with_capacity(line.len());
        for ch in line.chars() {
            if ch == glyph {
                new_line.push(pick_weighted(&choices, &mut rng));
            } else {
                new_line.push(ch);
            }
        }
        *line = new_line;
    }
}

/// NSUBST: A = 2:1 / 1:2 / * = .
/// Replace counted occurrences: first 2 A's become 1, next 1 becomes 2, rest become .
fn apply_nsubst(map_lines: &mut Vec<String>, spec: &str) {
    let Some((glyph_str, rhs)) = spec.split_once('=') else {
        return;
    };
    let glyph = glyph_str.trim().chars().next().unwrap_or('?');

    // Collect all positions of the glyph
    let mut positions: Vec<(usize, usize)> = Vec::new();
    for (y, line) in map_lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == glyph {
                positions.push((y, x));
            }
        }
    }

    // Shuffle positions for randomness
    let mut rng = rand::rng();
    for i in (1..positions.len()).rev() {
        let j = rng.random_range(0..=i);
        positions.swap(i, j);
    }

    // Parse the replacement specs: "2:1 / 1:2 / * = ."
    let parts: Vec<&str> = rhs.split('/').collect();
    let mut pos_idx = 0;

    for part in &parts {
        let part = part.trim();
        if pos_idx >= positions.len() {
            break;
        }

        let (count, replacement) = if let Some((count_str, repl)) = part.split_once(':') {
            let count_str = count_str.trim();
            let count = if count_str == "*" {
                positions.len() - pos_idx
            } else {
                count_str.parse::<usize>().unwrap_or(1)
            };
            let choices = parse_weighted_choices(repl.trim());
            (count, choices)
        } else if let Some((count_str, repl)) = part.split_once('=') {
            let count_str = count_str.trim();
            let count = if count_str == "*" {
                positions.len() - pos_idx
            } else {
                count_str.parse::<usize>().unwrap_or(1)
            };
            let choices = parse_weighted_choices(repl.trim());
            (count, choices)
        } else {
            continue;
        };

        if replacement.is_empty() {
            continue;
        }

        for _ in 0..count {
            if pos_idx >= positions.len() {
                break;
            }
            let (y, x) = positions[pos_idx];
            let ch = pick_weighted(&replacement, &mut rng);
            let line = &mut map_lines[y];
            let mut new_line = String::with_capacity(line.len());
            for (i, c) in line.chars().enumerate() {
                new_line.push(if i == x { ch } else { c });
            }
            *line = new_line;
            pos_idx += 1;
        }
    }
}

/// SHUFFLE: 1234
/// Randomly permute the listed glyphs across the map.
fn apply_shuffle(map_lines: &mut Vec<String>, spec: &str) {
    let spec = spec.trim();
    let glyphs: Vec<char> = spec.chars().filter(|c| !c.is_whitespace() && *c != ',').collect();
    if glyphs.len() < 2 {
        return;
    }

    // Create a random permutation
    let mut rng = rand::rng();
    let mut permuted = glyphs.clone();
    for i in (1..permuted.len()).rev() {
        let j = rng.random_range(0..=i);
        permuted.swap(i, j);
    }

    // Build mapping
    let mapping: std::collections::HashMap<char, char> =
        glyphs.iter().copied().zip(permuted.iter().copied()).collect();

    for line in map_lines.iter_mut() {
        let new_line: String = line
            .chars()
            .map(|ch| *mapping.get(&ch).unwrap_or(&ch))
            .collect();
        *line = new_line;
    }
}

/// Parse weighted choices like "x:5 ." → [(x, 5), (., 1)]
fn parse_weighted_choices(spec: &str) -> Vec<(char, u32)> {
    let mut choices = Vec::new();
    let mut chars = spec.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        chars.next();

        // Check for :weight suffix
        if chars.peek() == Some(&':') {
            chars.next(); // skip ':'
            let mut weight_str = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    weight_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            let weight = weight_str.parse::<u32>().unwrap_or(1);
            choices.push((ch, weight));
        } else {
            choices.push((ch, 1));
        }
    }

    choices
}

fn pick_weighted(choices: &[(char, u32)], rng: &mut impl Rng) -> char {
    let total: u32 = choices.iter().map(|(_, w)| w).sum();
    if total == 0 {
        return choices.first().map(|(c, _)| *c).unwrap_or('.');
    }
    let mut roll = rng.random_range(0..total);
    for (ch, weight) in choices {
        if roll < *weight {
            return *ch;
        }
        roll -= weight;
    }
    choices.last().map(|(c, _)| *c).unwrap_or('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_weighted_basic() {
        let choices = parse_weighted_choices("x:5 .");
        assert_eq!(choices, vec![('x', 5), ('.', 1)]);
    }

    #[test]
    fn parse_weighted_no_weights() {
        let choices = parse_weighted_choices("abc");
        assert_eq!(choices, vec![('a', 1), ('b', 1), ('c', 1)]);
    }

    #[test]
    fn subst_basic() {
        let mut lines = vec!["xAxAx".into()];
        apply_subst(&mut lines, "A = .");
        assert_eq!(lines[0], "x.x.x");
    }

    #[test]
    fn shuffle_permutes() {
        let mut lines = vec!["1234".into()];
        apply_shuffle(&mut lines, "1234");
        // Result should contain exactly the same chars, just permuted
        let mut sorted: Vec<char> = lines[0].chars().collect();
        sorted.sort();
        assert_eq!(sorted, vec!['1', '2', '3', '4']);
    }

    #[test]
    fn nsubst_counted() {
        let mut lines = vec!["AAAAAAA".into()];
        apply_nsubst(&mut lines, "A = 2:1 / 2:2 / *:.");
        let result = &lines[0];
        assert_eq!(result.len(), 7);
        assert_eq!(result.chars().filter(|c| *c == '1').count(), 2);
        assert_eq!(result.chars().filter(|c| *c == '2').count(), 2);
        assert_eq!(result.chars().filter(|c| *c == '.').count(), 3);
    }
}
