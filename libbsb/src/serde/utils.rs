use itertools::{Itertools, TupleWindows};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{digit1, multispace0},
    combinator::map_res,
    number::streaming::double,
    sequence::separated_pair,
    IResult, InputLength,
};
use regex::Regex;
use tracing::{error, warn};

use crate::image::header::{Polynomial, Ref};

fn unpack_ires<T>((_, param): (&str, T)) -> T {
    param
}

pub(super) fn handle_ires<'a, NomFunc, T>(f: NomFunc, input: &'a str) -> T
where
    NomFunc: FnOnce(&'a str) -> IResult<&'a str, T>,
    T: 'a,
{
    f(input).map_or_else(
        |e| {
            error!("Received Nom error for required value:\n{:?}", e);
            panic!()
        },
        unpack_ires,
    )
}

pub(super) fn handle_opt_ires<'a, NomFunc, T>(f: NomFunc, input: &'a str) -> Option<T>
where
    NomFunc: FnOnce(&'a str) -> IResult<&'a str, T>,
    T: 'a,
{
    f(input).map_or_else(
        |e| {
            warn!("Received Nom error:\n{:?}", e);
            None
        },
        |r| Some(unpack_ires(r)),
    )
}

pub(super) fn handle_owned_opt_ires<'a, NomFunc>(f: NomFunc, input: &'a str) -> Option<String>
where
    NomFunc: FnOnce(&'a str) -> IResult<&'a str, &'a str>,
{
    handle_opt_ires(f, input).map(str::to_owned)
}

pub(super) fn get_boundaries<'reg, 'hay>(
    re: &'reg Regex,
    input: &'hay str,
) -> impl Iterator<Item = (usize, usize)> + 'hay
where
    'reg: 'hay,
{
    let last = re.find_iter(input).last();
    let caps = re.find_iter(input);
    let windows: TupleWindows<regex::Matches<'reg, 'hay>, (regex::Match<'_>, regex::Match<'_>)> =
        caps.tuple_windows();
    windows
        .map(|(m1, m2)| (m1.start(), m2.start()))
        .chain(last.map(|m| (m.start(), input.input_len())))
}

pub fn comma_or_multispace(input: &str) -> IResult<&str, &str> {
    alt((tag(","), multispace0))(input)
}

pub fn parse_till_comma_or_newline(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == ',' || c == '\n' || c == '\r')(input)
}

pub fn _parse_till_comma(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == ',')(input)
}

pub fn parse_num_tuple_u16(input: &str) -> IResult<&str, (u16, u16)> {
    let (input, tuple) = map_res(
        separated_pair(digit1, nom::character::complete::char(','), digit1),
        |tuple: (&str, &str)| {
            tuple
                .0
                .parse::<u16>()
                .and_then(|t0| tuple.1.parse::<u16>().map(|t1| (t0, t1)))
        },
    )(input)?;
    Ok((input, tuple))
}

pub fn parse_num_tuple_usize(input: &str) -> IResult<&str, (usize, usize)> {
    let (input, tuple) = map_res(
        separated_pair(digit1, nom::character::complete::char(','), digit1),
        |tuple: (&str, &str)| {
            tuple
                .0
                .parse::<usize>()
                .and_then(|t0| tuple.1.parse::<usize>().map(|t1| (t0, t1)))
        },
    )(input)?;
    Ok((input, tuple))
}

#[inline]
pub fn parse_index(input: &str) -> IResult<&str, usize> {
    let (input, i) = map_res(digit1, |d: &str| d.parse::<usize>())(input)?;
    let (input, _) = tag(",")(input)?;
    Ok((input, i))
}
pub fn parse_index_rgb(input: &str) -> IResult<&str, (u8, u8, u8)> {
    let (input, _index) = parse_index(input)?;
    parse_rgb(input)
}

pub fn parse_index_coords(input: &str) -> IResult<&str, (f64, f64)> {
    let (input, _index) = parse_index(input)?;
    parse_coords(input)
}

pub fn parse_index_poly(input: &str) -> IResult<&str, Polynomial> {
    let (input, corner) = parse_index(input)?;
    let (input, poly) = parse_polynomial(input)?;
    Ok((input, Polynomial::new(corner, poly)))
}

pub fn parse_index_err(input: &str) -> IResult<&str, [f64; 4]> {
    let (input, _index) = parse_index(input)?;
    parse_err(input)
}

pub fn parse_rgb(input: &str) -> IResult<&str, (u8, u8, u8)> {
    let (input, r) = map_res(digit1, |d: &str| d.parse::<u8>())(input)?;
    let (input, _) = comma_or_multispace(input)?;
    let (input, g) = map_res(digit1, |d: &str| d.parse::<u8>())(input)?;
    let (input, _) = comma_or_multispace(input)?;
    let (input, b) = map_res(digit1, |d: &str| d.parse::<u8>())(input)?;
    Ok((input, (r, g, b)))
}

pub fn parse_ref(input: &str) -> IResult<&str, Ref> {
    let (input, _i) = map_res(digit1, |d: &str| d.parse::<usize>())(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, pixels) = parse_num_tuple_usize(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, coords) = parse_coords(input)?;
    Ok((input, Ref { pixels, coords }))
}

pub fn parse_coords(input: &str) -> IResult<&str, (f64, f64)> {
    let (input, lat) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, lon) = double(input)?;
    let coords = (lat, lon);
    Ok((input, coords))
}

pub fn parse_polynomial(input: &str) -> IResult<&str, [f64; 6]> {
    let (input, v1) = double(input)?;
    let (input, _) = comma_or_multispace(input)?;

    let (input, v2) = double(input)?;
    let (input, _) = comma_or_multispace(input)?;

    let (input, v3) = double(input)?;
    let (input, _) = comma_or_multispace(input)?;

    let (input, v4) = double(input)?;
    let (input, _) = comma_or_multispace(input)?;

    let (input, v5) = double(input)?;
    let (input, _) = comma_or_multispace(input)?;

    let (input, v6) = double(input)?;
    Ok((input, [v1, v2, v3, v4, v5, v6]))
}

pub fn parse_err(input: &str) -> IResult<&str, [f64; 4]> {
    let (input, v1) = double(input)?;
    let (input, _) = tag(",")(input)?;

    let (input, v2) = double(input)?;
    let (input, _) = tag(",")(input)?;

    let (input, v3) = double(input)?;
    let (input, _) = tag(",")(input)?;

    let (input, v4) = double(input)?;
    Ok((input, [v1, v2, v3, v4]))
}
