use chrono::NaiveDate;
use itertools::Itertools;
use nom::{
    bytes::complete::take_till,
    character::complete::digit1,
    combinator::map_res,
    number::complete::{double, float},
};
use std::str::FromStr;
use strum::{EnumString, IntoStaticStr};
use tracing::warn;

use crate::{
    image::header::{
        AdditionalParameters, ChartEditionParameters, DetailedParameters, GeneralParameters,
        ImageHeader, NTMRecord,
    },
    serde::AMERICAN_DATE_FORMAT,
    CRLF,
};

use super::{
    error::Error, field::Field, get_boundaries, handle_ires, handle_opt_ires,
    handle_owned_opt_ires, parse_coords, parse_index_coords, parse_index_err, parse_index_poly,
    parse_index_rgb, parse_num_tuple_u16, parse_ref, parse_till_comma_or_newline, DATE_FORMAT,
    FIELD_REGEX,
};

#[derive(IntoStaticStr, EnumString, PartialEq, Eq, Debug, Copy, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum Record {
    VER,
    CRR,
    BSB,
    KNP,
    KNQ,
    CED,
    NTM,
    OST,
    IFM,
    RGB,
    DAY,
    DSK,
    NGT,
    NGR,
    REF,
    WPX,
    PWX,
    WPY,
    PWY,
    ERR,
    PLY,
    DTM,
    // unknowns
    GRY,
    PRC,
    PRG,
    CPH,
    #[strum(serialize = "!")]
    Comment,
}

impl Record {
    pub fn as_str(self) -> &'static str {
        Into::<&'static str>::into(self)
    }
}

fn get_record_data<'a>(input: &'a str, record_name: &'a Record) -> &'a str {
    if *record_name == Record::Comment {
        &input[1..]
    } else {
        &input[4..]
    }
}

#[allow(clippy::too_many_lines)]
pub fn parse(
    input: &str,
    record_name: Record,
    image_header: &mut ImageHeader,
) -> Result<(), Error> {
    let record_data = get_record_data(input, &record_name);
    match record_name {
        Record::VER => {
            image_header.version = handle_opt_ires(float, record_data);
        }
        Record::CRR => {
            let (first_line, rest) = record_data.split_once(['\r', '\n']).expect("split CRR");
            let mut crr = String::new();
            crr.extend([first_line, CRLF, &rest.split_whitespace().join(" ")]);
            image_header.copyright_record = Some(crr);
        }
        Record::BSB => {
            image_header.general_parameters = parse_general_parameters(record_data)?;
        }
        Record::KNP => {
            image_header.detailed_parameters = Some(parse_detailed_parameters(record_data));
        }
        Record::KNQ => {
            image_header.additional_parameters = Some(parse_additional_parameters(record_data));
        }
        Record::CED => {
            image_header.ced = Some(parse_chart_edition_parameters(record_data));
        }
        Record::NTM => {
            image_header.ntm = Some(parse_ntm(record_data));
        }
        Record::OST => {
            let ost = handle_opt_ires(map_res(digit1, |s: &str| s.parse::<usize>()), record_data);
            image_header.ost = ost;
        }
        Record::IFM => {
            image_header.ifm = handle_ires(map_res(digit1, |s: &str| s.parse::<u8>()), record_data)
                .try_into()
                .map_err(|_| Error::MissingDepth)?;
        }
        Record::RGB => {
            let rgb = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = rgb {
                if let Some(rgbs) = image_header.rgb.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.rgb = Some(Vec::from([rgb]));
                };
            }
        }
        Record::DAY => {
            let day = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = day {
                if let Some(rgbs) = image_header.day.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.day = Some(Vec::from([rgb]));
                };
            }
        }
        Record::DSK => {
            let dsk = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = dsk {
                if let Some(rgbs) = image_header.dsk.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.dsk = Some(Vec::from([rgb]));
                };
            }
        }
        Record::NGT => {
            let ngt = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = ngt {
                if let Some(rgbs) = image_header.ngt.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.ngt = Some(Vec::from([rgb]));
                };
            }
        }
        Record::NGR => {
            let ngr = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = ngr {
                if let Some(rgbs) = image_header.ngr.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.ngr = Some(Vec::from([rgb]));
                };
            }
        }

        Record::GRY => {
            let gry = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = gry {
                if let Some(rgbs) = image_header.gry.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.gry = Some(Vec::from([rgb]));
                };
            }
        }

        Record::PRC => {
            let prc = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = prc {
                if let Some(rgbs) = image_header.prc.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.prc = Some(Vec::from([rgb]));
                };
            }
        }

        Record::PRG => {
            let prg = handle_opt_ires(parse_index_rgb, record_data);
            if let Some(rgb) = prg {
                if let Some(rgbs) = image_header.prg.as_mut() {
                    rgbs.push(rgb);
                } else {
                    image_header.prg = Some(Vec::from([rgb]));
                };
            }
        }
        Record::REF => {
            let ref_ = handle_opt_ires(parse_ref, record_data);
            if let Some(ref_) = ref_ {
                if let Some(refs) = image_header.reference_point_record.as_mut() {
                    refs.push(ref_);
                } else {
                    image_header.reference_point_record = Some(vec![ref_]);
                };
            }
        }
        Record::WPX => {
            image_header.wpx = handle_opt_ires(parse_index_poly, record_data);
        }
        Record::PWX => {
            image_header.pwx = handle_opt_ires(parse_index_poly, record_data);
        }
        Record::WPY => {
            image_header.wpy = handle_opt_ires(parse_index_poly, record_data);
        }
        Record::PWY => {
            image_header.pwy = handle_opt_ires(parse_index_poly, record_data);
        }
        Record::ERR => {
            let err = handle_opt_ires(parse_index_err, record_data);
            if let Some(err) = err {
                if let Some(errs) = image_header.err.as_mut() {
                    errs.push(err);
                } else {
                    image_header.err = Some(vec![err]);
                }
            }
        }
        Record::PLY => {
            let coords = handle_opt_ires(parse_index_coords, record_data);
            if let Some(coords) = coords {
                if let Some(errs) = image_header.ply.as_mut() {
                    errs.push(coords);
                } else {
                    image_header.ply = Some(vec![coords]);
                }
            }
        }
        Record::DTM => {
            image_header.dtm = handle_opt_ires(parse_coords, record_data);
        }
        Record::Comment => {
            let comment = handle_ires(take_till(|c: char| c == '\n'), record_data);
            if let Some(v) = image_header.comments.as_mut() {
                v.push(comment.to_owned());
            } else {
                image_header.comments = Some(vec![comment.to_owned()]);
            }
        }
        Record::CPH => {
            image_header.phase_shift = handle_opt_ires(double, record_data);
        }
    }
    Ok(())
}

pub fn parse_general_parameters(input: &str) -> Result<GeneralParameters, Error> {
    let starts = get_boundaries(&FIELD_REGEX, input);
    let mut bsb = GeneralParameters::default();
    for (start, next) in starts {
        let field_name = &input[start..start + 2];
        let Ok(field_name) = Field::from_str(field_name) else {
            warn!("Unrecognized field name: {field_name}");
            continue;
        };
        let field_data = &input[start + 3..next];
        match field_name {
            Field::NA => {
                bsb.chart_name = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::NU => {
                bsb.chart_number = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::RA => {
                bsb.image_width_height = parse_num_tuple_u16(field_data)
                    .map(|(_, width_height)| width_height)
                    .map_err(|_| Error::MissingWidthHeight)?;
            }
            Field::DU => {
                bsb.drawing_units =
                    handle_opt_ires(map_res(digit1, |s: &str| s.parse::<usize>()), field_data);
            }
            _ => {
                warn!(
                    "Field '{}' should not exist in current context. Skipping",
                    Into::<&str>::into(&field_name)
                );
                continue;
            }
        }
    }
    Ok(bsb)
}

pub fn parse_detailed_parameters(input: &str) -> DetailedParameters {
    let mut knp = DetailedParameters::default();
    let starts = get_boundaries(&FIELD_REGEX, input);
    for (start, next) in starts {
        let field_name = &input[start..start + 2];
        let Ok(field_name) = Field::from_str(field_name) else {
            warn!("Unrecognized field name: {field_name}");
            dbg!("Unrecognized field name", &field_name);
            continue;
        };
        let field_data = &input[start + 3..next];
        match field_name {
            Field::SC => {
                knp.chart_scale =
                    handle_opt_ires(map_res(digit1, |s: &str| s.parse::<usize>()), field_data);
            }
            Field::GD => {
                knp.geodetic_datum_name =
                    handle_opt_ires(parse_till_comma_or_newline, field_data).map(str::to_owned);
            }
            Field::PR => {
                knp.projection_name =
                    handle_opt_ires(parse_till_comma_or_newline, field_data).map(str::to_owned);
            }
            Field::PP => {
                knp.projection_parameter = handle_opt_ires(float, field_data);
            }
            Field::PI => {
                // TODO: should this be a string or a float?
                knp.projection_interval = handle_opt_ires(float, field_data);
                // knp.projection_interval = handle_opt_ires(float, field_data);
            }
            Field::SP => {
                knp.sp =
                    handle_opt_ires(parse_till_comma_or_newline, field_data).map(str::to_owned);
            }
            Field::SK => {
                // TODO: check and change name
                knp.skew_angle = handle_opt_ires(float, field_data);
            }
            Field::TA => {
                knp.text_angle = handle_opt_ires(float, field_data);
            }
            Field::UN => {
                // TODO: enum
                knp.depth_units =
                    handle_opt_ires(parse_till_comma_or_newline, field_data).map(str::to_owned);
            }
            Field::SD => {
                // TODO: enum
                knp.sounding_datum =
                    handle_opt_ires(parse_till_comma_or_newline, field_data).map(str::to_owned);
            }
            Field::DX => {
                knp.x_resolution = handle_opt_ires(float, field_data);
            }
            Field::DY => {
                knp.y_resolution = handle_opt_ires(float, field_data);
            }
            _ => {
                warn!(
                    "Field '{}' should not exist in current context. Skipping",
                    Into::<&str>::into(&field_name)
                );
                continue;
            }
        }
    }

    knp
}

pub fn parse_additional_parameters(input: &str) -> AdditionalParameters {
    let mut knq = AdditionalParameters::default();
    let boundaries = get_boundaries(&FIELD_REGEX, input);
    for (start, end) in boundaries {
        let field_name = &input[start..start + 2];
        let Ok(field_name) = Field::from_str(field_name) else {
            warn!("Unrecognized field name: {field_name}");
            dbg!("Unrecognized field name", &field_name);
            continue;
        };
        let field_data = &input[start + 3..end];
        match field_name {
            Field::P1 => {
                knq.p1 = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::P2 => {
                knq.p2 = handle_opt_ires(float, field_data);
            }
            Field::P3 => {
                knq.p3 = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::P4 => {
                knq.p4 = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::P5 => {
                knq.p5 = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::P6 => {
                knq.p6 = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::P7 => {
                // TODO: float?
                knq.p7 = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::P8 => {
                // TODO: float?
                knq.p8 = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::EC => {
                knq.ec = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::GD => {
                // NOTE: NOT the same GD as the other
                knq.gd = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::VC => {
                knq.vc = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::SC => {
                knq.sc = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::PC => {
                knq.pc = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }

            Field::RM => {
                knq.rm = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::GC => {
                knq.gc = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            _ => {
                warn!(
                    "Field '{}' should not exist in current context. Skipping",
                    Into::<&str>::into(&field_name)
                );
                continue;
            }
        }
    }
    knq
}

pub fn parse_chart_edition_parameters(input: &str) -> ChartEditionParameters {
    let mut ced = ChartEditionParameters::default();
    let boundaries = get_boundaries(&FIELD_REGEX, input);
    for (start, end) in boundaries {
        let field_name = &input[start..start + 2];
        let Ok(field_name) = Field::from_str(field_name) else {
            warn!("Unrecognized field name: {field_name}");
            dbg!("Unrecognized field name", &field_name);
            continue;
        };
        let field_data = &input[start + 3..end];
        match field_name {
            Field::SE => {
                // TODO: can be either number or date
                ced.source_edition =
                    handle_opt_ires(map_res(digit1, |s: &str| s.parse::<usize>()), field_data);
            }

            Field::RE => {
                ced.raster_edition =
                    handle_opt_ires(map_res(digit1, |s: &str| s.parse::<usize>()), field_data);
            }

            Field::ED => {
                // TODO: either number or date
                ced.edition_date = handle_opt_ires(
                    map_res(parse_till_comma_or_newline, |s| {
                        NaiveDate::parse_from_str(s, DATE_FORMAT).map_err(|e| {
                            warn!("Received {e} for date format. Trying American format.");
                            NaiveDate::parse_from_str(s, AMERICAN_DATE_FORMAT)
                        })
                    }),
                    field_data,
                );
            }
            _ => {
                warn!(
                    "Field '{}' should not exist in current context. Skipping",
                    Into::<&str>::into(&field_name)
                );
                continue;
            }
        }
    }
    ced
}

pub fn parse_ntm(input: &str) -> NTMRecord {
    let mut ntm = NTMRecord::default();
    let boundaries = get_boundaries(&FIELD_REGEX, input);
    for (start, end) in boundaries {
        let field_name = &input[start..start + 2];
        let Ok(field_name) = Field::from_str(field_name) else {
            warn!("Unrecognized field name: {field_name}");
            continue;
        };
        let field_data = &input[start + 3..end];
        match field_name {
            Field::NE => {
                ntm.edition = handle_opt_ires(float, field_data);
            }
            Field::ND => {
                ntm.date = handle_opt_ires(
                    map_res(parse_till_comma_or_newline, |s| {
                        NaiveDate::parse_from_str(s, AMERICAN_DATE_FORMAT)
                    }),
                    field_data,
                );
            }
            Field::BF => {
                ntm.base_flag = handle_owned_opt_ires(parse_till_comma_or_newline, field_data);
            }
            Field::BD => {
                // TODO: check naming
                ntm.adn_record = handle_opt_ires(
                    map_res(parse_till_comma_or_newline, |s| {
                        NaiveDate::parse_from_str(s.trim(), AMERICAN_DATE_FORMAT)
                    }),
                    field_data,
                );
            }
            _ => {
                warn!(
                    "Field '{}' should not exist in current context. Skipping",
                    Into::<&str>::into(&field_name)
                );
                continue;
            }
        }
    }
    ntm
}
