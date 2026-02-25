use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map_res, opt, recognize},
};
use std::str::FromStr;
#[cfg(test)]
mod tests;

/// `\d+`
pub fn parse_integer<T>(input: &str) -> IResult<&str, T>
where
    T: FromStr,
{
    map_res(recognize(digit1), str::parse).parse(input)
}
/// Parses an integer with optional `px` suffix.
pub fn parse_i_px_maybe<T>(input: &str) -> IResult<&str, T>
where
    T: FromStr,
{
    let (rest, (i, _)) = (parse_integer, opt(tag("px"))).parse(input)?;
    Ok((rest, i))
}

/// `\d+\.\d+`
pub fn parse_f32(input: &str) -> IResult<&str, f32> {
    let float1 = (digit1, opt((tag("."), digit1)));
    map_res(recognize(float1), str::parse).parse(input)
}
/// Parses a float followed by percent sign, returns numeric part.
pub fn parse_f_percent(input: &str) -> IResult<&str, f32> {
    let (rest, (f, _)) = (parse_f32, char('%')).parse(input)?;
    Ok((rest, f))
}

/// `\d+\/\d+`
pub fn parse_fraction(input: &str) -> IResult<&str, (usize, usize)> {
    let (rest, (a, _, b)) = (parse_integer, tag("/"), parse_integer).parse(input)?;
    Ok((rest, (a, b)))
}

/// 100(/50)?
#[inline]
pub fn parse_fraction_maybe(input: &str) -> IResult<&str, (usize, Option<usize>)> {
    let (rest, (a, b)) = (parse_integer, opt((tag("/"), parse_integer))).parse(input)?;
    Ok((rest, (a, b.map(|s| s.1))))
}

/// #50d71e
pub fn parser_color_hex() {}

// fn hex3() {
//
// }
//
// fn hex6() {
//
// }
//
// fn hex8() {
//
// }
