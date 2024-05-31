#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum Mode {
    Normal(Normal),
}

impl Mode {
    fn consume_char(&mut self, ch: char) -> Option<Command> {
        match self {
            Mode::Normal(normal) => normal.consume_char(ch),
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal(Normal::default())
    }
}

#[derive(Default, Debug, Clone)]
pub struct Normal {
    minor: NormalMinor,
}

impl Normal {
    fn consume_char(&mut self, ch: char) -> Option<Command> {
        self.minor.consume_char(ch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalMinor {
    Initial(usize),
    OperatorPending(usize, Operator),
    TextObjectPending(usize, Operator, ObjRange),
}

impl NormalMinor {
    fn consume_char(&mut self, ch: char) -> Option<Command> {
        match self {
            NormalMinor::Initial(repeat) => {
                if let Some(digit) = digit_from_char(ch) {
                    if *repeat == 0 {
                        *self = NormalMinor::Initial(digit);
                    } else {
                        *self = NormalMinor::Initial(*repeat * 10 + digit);
                    }
                    return None;
                }
                if let Some(operator) = Operator::from_char(ch) {
                    *self = NormalMinor::OperatorPending(*repeat, operator);
                    return None;
                }
                if let Some(motion) = Motion::from_char(ch) {
                    let cmd = Some(Command::Motion(
                        *repeat,
                        motion,
                        Direction::from_char(ch).expect("invalid direction"),
                    ));
                    *self = Default::default();
                    return cmd;
                }

                // If we haven't succesfully found a command by now, then we need to reset the
                // minor mode and return None.
                *self = Default::default();
                None
            }
            NormalMinor::OperatorPending(repeat, operator) => {
                if let Some(motion) = Motion::from_char(ch) {
                    let cmd = Some(Command::Operation(Operation::Motion(
                        *repeat,
                        *operator,
                        motion,
                        Direction::from_char(ch).expect("invalid direction"),
                    )));
                    *self = Default::default();
                    return cmd;
                }
                if let Some(obj_range) = ObjRange::from_char(ch) {
                    *self = NormalMinor::TextObjectPending(*repeat, *operator, obj_range);
                    return None;
                }

                // If we haven't succesfully found a command by now, then we need to reset the
                // minor mode and return None.
                *self = Default::default();
                None
            }
            NormalMinor::TextObjectPending(repeat, operator, range) => {
                if let Some(obj_motion) = ObjMotion::from_char(ch) {
                    let cmd = Some(Command::Operation(Operation::TextObject(
                        *repeat,
                        *operator,
                        TextObject(*range, obj_motion),
                    )));
                    *self = Default::default();
                    return cmd;
                }

                // If we haven't succesfully found a command by now, then we need to reset the
                // minor mode and return None.
                *self = Default::default();
                None
            }
        }
    }

    fn repeat(&self) -> usize {
        match self {
            NormalMinor::Initial(repeat) => *repeat,
            NormalMinor::OperatorPending(repeat, _) => *repeat,
            NormalMinor::TextObjectPending(repeat, _, _) => *repeat,
        }
        .max(1)
    }
}

impl Default for NormalMinor {
    fn default() -> Self {
        NormalMinor::Initial(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Motion(usize, Motion, Direction),
    Operation(Operation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Motion(usize, Operator, Motion, Direction),
    TextObject(usize, Operator, TextObject),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Change,
    Delete,
    Yank,
    Visual,
}

impl Operator {
    fn from_char(ch: char) -> Option<Operator> {
        match ch {
            'c' => Some(Operator::Change),
            'd' => Some(Operator::Delete),
            'y' => Some(Operator::Yank),
            'v' => Some(Operator::Visual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjRange {
    Inner,
    Outer,
}

impl ObjRange {
    fn from_char(ch: char) -> Option<ObjRange> {
        match ch {
            'i' => Some(ObjRange::Inner),
            'a' => Some(ObjRange::Outer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjMotion {
    Word,
}

impl ObjMotion {
    fn from_char(ch: char) -> Option<ObjMotion> {
        match ch {
            'w' => Some(ObjMotion::Word),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextObject(ObjRange, ObjMotion);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Char,
    Word,
    Line,
}

impl Motion {
    fn from_char(ch: char) -> Option<Motion> {
        match ch {
            'h' => Some(Motion::Char),
            'l' => Some(Motion::Char),
            'j' => Some(Motion::Line),
            'k' => Some(Motion::Line),
            'w' => Some(Motion::Word),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

impl Direction {
    fn from_char(ch: char) -> Option<Direction> {
        match ch {
            'h' => Some(Direction::Backward),
            'j' => Some(Direction::Forward),
            'k' => Some(Direction::Backward),
            'l' => Some(Direction::Forward),
            'w' => Some(Direction::Forward),
            _ => None,
        }
    }
}

fn digit_from_char(ch: char) -> Option<usize> {
    match ch {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normal_parse(input: &str) -> Option<Command> {
        let mut mode = Mode::default();
        for ch in input.chars() {
            if let Some(cmd) = mode.consume_char(ch) {
                return Some(cmd);
            }
        }
        None
    }

    #[test]
    fn test_word_motion() {
        let expectations = [
            (
                "w",
                Some(Command::Motion(1, Motion::Word, Direction::Forward)),
            ),
            (
                "2w",
                Some(Command::Motion(2, Motion::Word, Direction::Forward)),
            ),
            (
                "340w",
                Some(Command::Motion(340, Motion::Word, Direction::Forward)),
            ),
        ];
        for (input, expected) in expectations.iter() {
            assert_eq!(normal_parse(input), *expected);
        }
    }
}
