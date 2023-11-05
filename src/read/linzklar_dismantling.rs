use anyhow::anyhow;
use serde_derive::Deserialize as De;
use std::fs::File;

use super::char_pronunciation::Linzklar;

#[derive(Debug, De)]
struct Record {
    linzklar: String,
    dismantling: String,
}

/// a lookup table from a linzklar to a dismantling
pub type DismantlingTable = Vec<(Linzklar, DismantlingTree)>;

#[derive(Clone, Eq, PartialEq)]
pub enum DismantlingTree {
    Leaf(Linzklar),
    Unary(CustomUnaryOperator, Box<DismantlingTree>),
    Binary(
        IdsBinaryOperator,
        Box<DismantlingTree>,
        Box<DismantlingTree>,
    ),
    Trinary(
        IdsTrinaryOperator,
        Box<DismantlingTree>,
        Box<DismantlingTree>,
        Box<DismantlingTree>,
    ),
}

impl DismantlingTree {
    /// # Errors
    /// Fails when the input is not a correct IDS sequence (special unary operators 💥 and ↺ are permitted)
    pub fn parse(input: &str) -> anyhow::Result<Self> {
        let mut stack: Vec<Self> = vec![];
        for c in input.chars().rev() {
            match c {
                '💥' => {
                    let Some(t) = stack.pop() else {
                        return Err(anyhow!(format!(
                            "In {input}:\n  Encountered a unary operator '💥' but there is nothing to apply the operator onto."
                        )));
                    };
                    stack.push(Self::Unary(CustomUnaryOperator::Explosion, Box::new(t)));
                }
                '↺' => {
                    let Some(t) = stack.pop() else {
                        return Err(anyhow!(format!(
                            "In {input}:\n  Encountered a unary operator '↺' but there is nothing to apply the operator onto."
                        )));
                    };
                    stack.push(Self::Unary(CustomUnaryOperator::Explosion, Box::new(t)));
                }
                '⿲' | '⿳' => {
                    let t1 = stack.pop();
                    let t2 = stack.pop();
                    let t3 = stack.pop();

                    let (Some(t1), Some(t2), Some(t3)) = (t1, t2, t3) else {
                        return Err(anyhow!(format!(
                            "In {input}:\n  Encountered a trinary operator '{c}' but cannot pop three elements from the stack."
                        )));
                    };

                    stack.push(Self::Trinary(
                        IdsTrinaryOperator::Unit,
                        Box::new(t1),
                        Box::new(t2),
                        Box::new(t3),
                    ));
                }
                '⿰' | '⿱' | '⿴' | '⿵' | '⿶' | '⿷' | '⿸' | '⿹' | '⿺' | '⿻' => {
                    let t1 = stack.pop();
                    let t2 = stack.pop();

                    let (Some(t1), Some(t2)) = (t1, t2) else {
                        return Err(anyhow!(format!(
                            "In {input}:\n  Encountered a binary operator '{c}' but cannot pop two elements from the stack."
                        )));
                    };

                    stack.push(Self::Binary(
                        IdsBinaryOperator::Unit,
                        Box::new(t1),
                        Box::new(t2),
                    ));
                }
                lin => {
                    stack.push(Self::Leaf(Linzklar::from_char(lin)?));
                }
            }
        }

        if stack.len() != 1 {
            return Err(anyhow!(format!(
                "In {input}:\n  The stack should only have a single element at the end of the operation, but the stack has length {}", stack.len()
            )));
        }

        Ok(stack[0].clone())
    }
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub enum CustomUnaryOperator {
    Rotation,
    Explosion,
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub enum IdsBinaryOperator {
    /*LeftToRight,
    AboveToBelow,
    FullSurround,
    SurroundFromAbove,
    SurroundFromBelow,
    SurroundFromLeft,
    SurroundFromUpperLeft,
    SurroundFromUpperRight,
    SurroundFromLowerLeft,
    Overlaid,*/
    Unit,
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub enum IdsTrinaryOperator {
    Unit, /*LeftToMiddleAndRight,
          AboveToMiddleAndBelow,*/
}

#[allow(clippy::tabs_in_doc_comments)]
/// Parses "raw/linzklar-dismantling.tsv" to obtain a table converting a linzklar to how it should be dismantled.
/// The tsv used for the input should be of the following form:
/// ```text
/// linzklar	dismantling
/// 我	⿰人己
/// 汝	⿰人物
/// 此	⿱己口
/// 其	⿱物口
/// 彼	💥⿱物己
/// 何	⿵心無
/// 或	FALSE
/// 全	⿴周物
/// 無	FALSE
/// ```
/// Each of the first column must be a linzklar.
/// Each of the second column must be either "FALSE" or an IDS sequence, with two additional unary operators 💥 and ↺.
/// # Errors
/// Gives errors if:
/// - IO fails
/// - "raw/linzklar-dismantling.tsv" does not conform to an expected format
/// - the IDS Sequence is unparsable
///
pub fn parse() -> anyhow::Result<DismantlingTable> {
    fn convert(record: &Record) -> anyhow::Result<(Linzklar, DismantlingTree)> {
        let linzklar = Linzklar::from_str(&record.linzklar)?;
        if record.dismantling == "FALSE" {
            Ok((linzklar, DismantlingTree::Leaf(linzklar)))
        } else {
            match DismantlingTree::parse(&record.dismantling) {
                Err(e) => Err(e),
                Ok(a) => Ok((linzklar, a)),
            }
        }
    }

    let f = File::open("raw/linzklar-dismantling.tsv")?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(f);
    let mut ans = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        ans.push(record);
    }

    ans.iter().map(convert).collect::<anyhow::Result<_>>()
}
