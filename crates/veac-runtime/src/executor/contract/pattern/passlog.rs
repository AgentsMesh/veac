use std::collections::{HashSet, VecDeque};

#[derive(Clone, Copy)]
enum Matcher {
    Byte(u8),
    Digit,
    Any,
}

#[derive(Default)]
struct State {
    accept: bool,
    transitions: Vec<(Matcher, usize)>,
    epsilon: Vec<usize>,
}

struct Nfa {
    states: Vec<State>,
}

pub(in crate::executor) fn accepts(prefix: &[u8], value: &[u8]) -> bool {
    let Some(rest) = value
        .strip_prefix(prefix)
        .and_then(|value| value.strip_prefix(b"-"))
    else {
        return false;
    };
    let digits = rest.iter().take_while(|byte| byte.is_ascii_digit()).count();
    if digits == 0 || !rest[digits..].starts_with(b".log") {
        return false;
    }
    let tail = &rest[digits + 4..];
    tail.is_empty() || (tail.len() > 1 && tail[0] == b'.')
}

pub(in crate::executor) fn overlap(left: &[u8], right: &[u8]) -> bool {
    intersects(&passlog(left), &passlog(right))
}

pub(in crate::executor) fn overlap_pattern(
    passlog_prefix: &[u8],
    pattern_prefix: &[u8],
    pattern_suffix: &[u8],
    minimum_width: Option<usize>,
) -> bool {
    intersects(
        &passlog(passlog_prefix),
        &number_pattern(pattern_prefix, pattern_suffix, minimum_width.unwrap_or(1)),
    )
}

fn number_pattern(prefix: &[u8], suffix: &[u8], minimum_width: usize) -> Nfa {
    let mut states = vec![State::default()];
    let current = fixed(&mut states, 0, prefix);
    let current = digits(&mut states, current, minimum_width);
    let current = fixed(&mut states, current, suffix);
    states[current].accept = true;
    Nfa { states }
}

fn passlog(prefix: &[u8]) -> Nfa {
    let mut states = vec![State::default()];
    let current = fixed(&mut states, 0, prefix);
    let current = fixed(&mut states, current, b"-");
    let current = digits(&mut states, current, 1);
    let current = fixed(&mut states, current, b".log");
    states[current].accept = true;
    let dot = push(&mut states);
    states[current].transitions.push((Matcher::Byte(b'.'), dot));
    let tail = push(&mut states);
    states[dot].transitions.push((Matcher::Any, tail));
    states[tail].transitions.push((Matcher::Any, tail));
    states[tail].accept = true;
    Nfa { states }
}

fn fixed(states: &mut Vec<State>, mut current: usize, bytes: &[u8]) -> usize {
    for byte in bytes {
        let next = push(states);
        states[current]
            .transitions
            .push((Matcher::Byte(*byte), next));
        current = next;
    }
    current
}

fn digits(states: &mut Vec<State>, mut current: usize, minimum: usize) -> usize {
    for _ in 0..minimum {
        let digit = push(states);
        states[current].transitions.push((Matcher::Digit, digit));
        current = digit;
    }
    states[current].transitions.push((Matcher::Digit, current));
    let next = push(states);
    states[current].epsilon.push(next);
    next
}

fn push(states: &mut Vec<State>) -> usize {
    states.push(State::default());
    states.len() - 1
}

fn intersects(left: &Nfa, right: &Nfa) -> bool {
    let start = (closure(left, vec![0]), closure(right, vec![0]));
    let mut pending = VecDeque::from([start.clone()]);
    let mut seen = HashSet::from([start]);
    while let Some((left_set, right_set)) = pending.pop_front() {
        if accepting(left, &left_set) && accepting(right, &right_set) {
            return true;
        }
        for byte in 0_u8..=u8::MAX {
            let next_left = advance(left, &left_set, byte);
            let next_right = advance(right, &right_set, byte);
            if next_left.is_empty() || next_right.is_empty() {
                continue;
            }
            let next = (next_left, next_right);
            if seen.insert(next.clone()) {
                pending.push_back(next);
            }
        }
    }
    false
}

fn advance(nfa: &Nfa, current: &[usize], byte: u8) -> Vec<usize> {
    let mut next = Vec::new();
    for state in current {
        for (matcher, target) in &nfa.states[*state].transitions {
            if matches(matcher, byte) {
                next.push(*target);
            }
        }
    }
    closure(nfa, next)
}

fn closure(nfa: &Nfa, mut values: Vec<usize>) -> Vec<usize> {
    let mut index = 0;
    while index < values.len() {
        for target in &nfa.states[values[index]].epsilon {
            if !values.contains(target) {
                values.push(*target);
            }
        }
        index += 1;
    }
    values.sort_unstable();
    values.dedup();
    values
}

fn accepting(nfa: &Nfa, states: &[usize]) -> bool {
    states.iter().any(|state| nfa.states[*state].accept)
}

fn matches(matcher: &Matcher, byte: u8) -> bool {
    match matcher {
        Matcher::Byte(value) => *value == byte,
        Matcher::Digit => byte.is_ascii_digit(),
        Matcher::Any => true,
    }
}
