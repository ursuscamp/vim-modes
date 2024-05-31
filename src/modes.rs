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
    Initial(Option<usize>),
    OperatorPending(usize, Operator),
    TextObjectPending(usize, Operator, ObjRange),
}

impl NormalMinor {
    fn consume_char(&mut self, ch: char) -> Option<Command> {
        match self {
            NormalMinor::Initial(repeat) => {
                if let Some(digit) = digit_from_char(ch) {
                    *self = NormalMinor::Initial(Some(repeat.unwrap_or_default() * 10 + digit));
                    return None;
                }
                if let Some(operator) = Operator::from_char(ch) {
                    *self = NormalMinor::OperatorPending(repeat.unwrap_or(1), operator);
                    return None;
                }
                if let Some(motion) = Motion::from_char(ch) {
                    let cmd = Some(Command::Motion(
                        repeat.unwrap_or(1),
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
}

impl Default for NormalMinor {
    fn default() -> Self {
        NormalMinor::Initial(None)
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
    Subword,
    SubwordEnd,
    Word,
    WordEnd,
    Line,
}

impl Motion {
    fn from_char(ch: char) -> Option<Motion> {
        match ch {
            'h' => Some(Motion::Char),
            'l' => Some(Motion::Char),
            'j' => Some(Motion::Line),
            'k' => Some(Motion::Line),
            'w' => Some(Motion::Subword),
            'W' => Some(Motion::Word),
            'e' => Some(Motion::SubwordEnd),
            'E' => Some(Motion::WordEnd),
            'b' => Some(Motion::Subword),
            'B' => Some(Motion::Word),
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
            'W' => Some(Direction::Forward),
            'e' => Some(Direction::Forward),
            'E' => Some(Direction::Forward),
            'b' => Some(Direction::Backward),
            'B' => Some(Direction::Backward),
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
    fn test_subword_motion() {
        let expectations = [
            (
                "w",
                Some(Command::Motion(1, Motion::Subword, Direction::Forward)),
            ),
            (
                "2w",
                Some(Command::Motion(2, Motion::Subword, Direction::Forward)),
            ),
            (
                "340w",
                Some(Command::Motion(340, Motion::Subword, Direction::Forward)),
            ),
        ];
        for (input, expected) in expectations.iter() {
            assert_eq!(normal_parse(input), *expected);
        }
    }

    #[test]
    fn test_word_motion() {
        let expectations = [
            (
                "W",
                Some(Command::Motion(1, Motion::Word, Direction::Forward)),
            ),
            (
                "2W",
                Some(Command::Motion(2, Motion::Word, Direction::Forward)),
            ),
            (
                "340W",
                Some(Command::Motion(340, Motion::Word, Direction::Forward)),
            ),
        ];
        for (input, expected) in expectations.iter() {
            assert_eq!(normal_parse(input), *expected);
        }
    }

    #[test]
    fn test_hjkl_motion() {
        let expectations = [
            (
                "h",
                Some(Command::Motion(1, Motion::Char, Direction::Backward)),
            ),
            (
                "j",
                Some(Command::Motion(1, Motion::Line, Direction::Forward)),
            ),
            (
                "k",
                Some(Command::Motion(1, Motion::Line, Direction::Backward)),
            ),
            (
                "l",
                Some(Command::Motion(1, Motion::Char, Direction::Forward)),
            ),
        ];
        for (input, expected) in expectations.iter() {
            assert_eq!(normal_parse(input), *expected);
        }
    }

    #[test]
    fn test_word_end_motion() {
        let expectations = [
            (
                "e",
                Some(Command::Motion(1, Motion::SubwordEnd, Direction::Forward)),
            ),
            (
                "E",
                Some(Command::Motion(1, Motion::WordEnd, Direction::Forward)),
            ),
            (
                "2e",
                Some(Command::Motion(2, Motion::SubwordEnd, Direction::Forward)),
            ),
            (
                "2E",
                Some(Command::Motion(2, Motion::WordEnd, Direction::Forward)),
            ),
            (
                "340e",
                Some(Command::Motion(340, Motion::SubwordEnd, Direction::Forward)),
            ),
            (
                "340E",
                Some(Command::Motion(340, Motion::WordEnd, Direction::Forward)),
            ),
        ];
        for (input, expected) in expectations.iter() {
            assert_eq!(normal_parse(input), *expected);
        }
    }

    #[test]
    fn test_word_back_motion() {
        let expectations = [
            (
                "b",
                Some(Command::Motion(1, Motion::Subword, Direction::Backward)),
            ),
            (
                "B",
                Some(Command::Motion(1, Motion::Word, Direction::Backward)),
            ),
            (
                "2b",
                Some(Command::Motion(2, Motion::Subword, Direction::Backward)),
            ),
            (
                "2B",
                Some(Command::Motion(2, Motion::Word, Direction::Backward)),
            ),
            (
                "340b",
                Some(Command::Motion(340, Motion::Subword, Direction::Backward)),
            ),
            (
                "340B",
                Some(Command::Motion(340, Motion::Word, Direction::Backward)),
            ),
        ];
        for (input, expected) in expectations.iter() {
            assert_eq!(normal_parse(input), *expected);
        }
    }
}
