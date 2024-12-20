pub mod error;
pub mod field;
pub mod record;
pub mod utils;

use crate::{
    image::header::{ImageHeader, Polynomial, Ref},
    serde::field::Field,
    CRLF,
};
use record::{parse, Record};
use regex::Regex;
use std::fmt::Write;
use std::{str::FromStr, sync::LazyLock};
use tracing::{trace, warn};
use utils::{
    get_boundaries, handle_ires, handle_opt_ires, handle_owned_opt_ires, parse_coords,
    parse_index_coords, parse_index_err, parse_index_poly, parse_index_rgb, parse_num_tuple_u16,
    parse_ref, parse_till_comma_or_newline,
};
use Field::{
    BD, BF, DU, DX, DY, EC, ED, GC, GD, NA, ND, NE, NU, P1, P2, P3, P4, PC, PI, PP, PR, RA, RE, RM,
    SC, SD, SE, SK, SP, TA, UN, VC,
};

const AMERICAN_DATE_FORMAT: &str = "%m/%d/%Y";
const DATE_FORMAT: &str = "%d/%m/%Y";

static RECORD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Z]{3}/|!").expect("Regex for record identifier must be valid")
});

static FIELD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Z][A-Z1-9]=").expect("Regex for field identifier must be valid")
});

impl ImageHeader {
    // This text was taken from the libbsb c crate:
    //
    // Although described as proprietary by NOAA, much of the information in the ASCII text header is self explanatory.
    // Each line begins with a 3 character token followed by a slash "/". The remainder of the line depends upon the token,
    // but in general consists of a list of comma separated values. If the list of values is too long for 80 columns,
    // the values for the token are continued on the next line by prefixing the new line with 4 spaces. Comments are indicated with a leading "!".

    /// Serializes [`Self`] into the ASCII header format of image kap files
    #[must_use]
    pub fn into_header_format(&self) -> String {
        let mut serialized = String::new();
        // TODO: Serialize comments? Can comments be placed anywhere or exclusively at the
        // beginning?
        // place_optional_string(&mut serialized, self.serialize_comment(0));
        place_optional_string(&mut serialized, self.serialize_crr());
        serialized.extend([CRLF]);
        place_optional_string(&mut serialized, self.serialize_ver());
        serialized.extend([CRLF]);
        serialized.extend([self.serialize_general_parameters()]);
        serialized.extend([CRLF]);
        serialized.extend([self.serialize_detailed_parameters()]);
        serialized.extend([CRLF]);
        serialized.extend([self.serialize_additional_parameters()]);
        serialized.extend([CRLF]);
        serialized.extend([self.serialize_chart_edition_parameters()]);
        serialized.extend([CRLF]);
        serialized.extend([self.serialize_ntm_parameters()]);
        serialized.extend([CRLF]);
        place_optional_string(&mut serialized, self.serialize_ost());
        serialized.extend([CRLF]);
        serialized.extend([self.serialize_ifm()]);
        serialized.extend([CRLF]);

        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::RGB, self.rgb.as_deref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::DAY, self.day.as_deref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::DSK, self.dsk.as_deref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::NGT, self.ngt.as_deref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::NGR, self.ngr.as_deref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::GRY, self.gry.as_deref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::PRC, self.prc.as_deref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_rgb(Record::PRG, self.prg.as_deref()),
        );

        place_optional_string(
            &mut serialized,
            Self::serialize_refs(self.reference_point_record.as_deref()),
        );
        place_optional_string(&mut serialized, Self::serialize_plys(self.ply.as_deref()));
        place_optional_string(&mut serialized, Self::serialize_dtm(self.dtm.as_ref()));
        place_optional_string(
            &mut serialized,
            Self::serialize_cph(self.phase_shift.as_ref()),
        );
        place_optional_string(
            &mut serialized,
            Self::serialize_polys(Record::WPX, self.wpx.as_ref()),
        );
        serialized.extend([CRLF]);
        place_optional_string(
            &mut serialized,
            Self::serialize_polys(Record::WPY, self.wpy.as_ref()),
        );
        serialized.extend([CRLF]);
        place_optional_string(
            &mut serialized,
            Self::serialize_polys(Record::PWX, self.pwx.as_ref()),
        );
        serialized.extend([CRLF]);
        place_optional_string(
            &mut serialized,
            Self::serialize_polys(Record::PWY, self.pwy.as_ref()),
        );
        serialized.extend([CRLF]);
        place_optional_string(&mut serialized, self.serialize_errs());
        serialized
    }

    fn serialize_crr(&self) -> Option<String> {
        self.copyright_record.as_ref().map(|crr| {
            let mut out = String::new();
            let (first, rest) = crr.split_once(CRLF)?;

            let _ = write!(out, "{}/{first}{CRLF}", Record::CRR.as_str());
            place_crr_string(&mut out, rest);
            Some(out)
        })?
    }

    fn serialize_ver(&self) -> Option<String> {
        self.version
            .map(|v| format!("{}/{v:?}", Record::VER.as_str()))
    }

    fn serialize_general_parameters(&self) -> String {
        use Record::BSB;

        let mut out = Vec::new();
        out.extend([format!("{}/", BSB.as_str())]);
        out.extend([
            NA.serialize_field(self),
            NU.serialize_field(self),
            RA.serialize_field(self),
            DU.serialize_field(self),
        ]);
        format_field_names(out)
    }
    fn serialize_detailed_parameters(&self) -> String {
        use Record::KNP;

        let mut out = Vec::new();
        out.extend([format!("{}/", KNP.as_str())]);
        out.extend([
            SC.serialize_field(self),
            GD.serialize_field(self),
            PR.serialize_field(self),
            PP.serialize_field(self),
            PI.serialize_field(self),
            SP.serialize_field(self),
            SK.serialize_field(self),
            TA.serialize_field(self),
            UN.serialize_field(self),
            SD.serialize_field(self),
            DX.serialize_field(self),
            DY.serialize_field(self),
        ]);
        format_field_names(out)
    }

    fn serialize_additional_parameters(&self) -> String {
        use Record::KNQ;

        let mut out = Vec::new();
        out.extend([format!("{}/", KNQ.as_str())]);
        out.extend([
            EC.serialize_field(self),
            Field::serialize_additional_gd(self),
            VC.serialize_field(self),
            Field::serialize_additional_sc(self),
            PC.serialize_field(self),
            P1.serialize_field(self),
            P2.serialize_field(self),
            P3.serialize_field(self),
            P4.serialize_field(self),
            GC.serialize_field(self),
            RM.serialize_field(self),
        ]);

        // TODO:
        // Should these be inserted before IF some?
        //  P5.serialize_field(self),
        //  P6.serialize_field(self),
        //  P7.serialize_field(self),
        //  P8.serialize_field(self),
        format_field_names(out)
    }

    fn serialize_chart_edition_parameters(&self) -> String {
        use Record::CED;

        let mut out = Vec::new();
        out.extend([format!("{}/", CED.as_str())]);
        out.extend([
            SE.serialize_field(self),
            RE.serialize_field(self),
            ED.serialize_field(self),
        ]);

        format_field_names(out)
    }

    fn serialize_ntm_parameters(&self) -> String {
        use Record::NTM;

        let mut out = Vec::new();
        out.extend([format!("{}/", NTM.as_str())]);
        out.extend([
            NE.serialize_field(self),
            ND.serialize_field(self),
            BF.serialize_field(self),
            BD.serialize_field(self),
        ]);

        format_field_names(out)
    }
    fn serialize_ost(&self) -> Option<String> {
        self.ost.map(|v| format!("{}/{}", Record::OST.as_str(), v))
    }

    fn serialize_ifm(&self) -> String {
        format!("{}/{}", Record::IFM.as_str(), self.ifm)
    }
    fn serialize_rgb(rec: Record, rgbs: Option<&[(u8, u8, u8)]>) -> Option<String> {
        let rgbs = rgbs?;
        let mut out = String::new();
        rgbs.iter().enumerate().for_each(|(i, (r, g, b))| {
            let _ = write!(&mut out, "{}/{},{},{},{}\r\n", rec.as_str(), i + 1, r, g, b);
        });

        (!out.is_empty()).then_some(out)
    }

    fn serialize_refs(refs: Option<&[Ref]>) -> Option<String> {
        let refs = refs?;
        let mut out = String::new();
        refs.iter()
            .enumerate()
            .for_each(|(i, Ref { pixels, coords })| {
                let _ = write!(
                    &mut out,
                    "{}/{},{},{},{:.10},{:.10}\r\n",
                    Record::REF.as_str(),
                    i + 1,
                    pixels.0,
                    pixels.1,
                    coords.0,
                    coords.1,
                );
            });

        (!out.is_empty()).then_some(out)
    }
    fn serialize_plys(plys: Option<&[(f64, f64)]>) -> Option<String> {
        let plys = plys?;
        let mut out = String::new();
        plys.iter().enumerate().for_each(|(i, ply)| {
            let _ = write!(
                &mut out,
                "{}/{},{:.10},{:.10}\r\n",
                Record::PLY.as_str(),
                i + 1,
                ply.0,
                ply.1,
            );
        });

        (!out.is_empty()).then_some(out)
    }

    fn serialize_dtm(dtm: Option<&(f64, f64)>) -> Option<String> {
        let dtm = dtm?;
        let mut out = String::new();
        let _ = write!(
            &mut out,
            "{}/{:.10},{:.10}\r\n",
            Record::DTM.as_str(),
            dtm.0,
            dtm.1,
        );

        (!out.is_empty()).then_some(out)
    }

    fn serialize_cph(cph: Option<&f64>) -> Option<String> {
        let cph = cph?;
        let mut out = String::new();
        let _ = write!(&mut out, "{}/{:.10}\r\n", Record::CPH.as_str(), cph,);
        (!out.is_empty()).then_some(out)
    }

    // https://stackoverflow.com/questions/65264069/alignment-of-floating-point-numbers-printed-in-scientific-notation
    fn fmt_f64(num: f64, width: usize, precision: usize, exp_pad: usize) -> String {
        if num.abs() > 0.01 {
            return format!("{num:width$}");
        }

        let mut num = format!("{num:.precision$e}");
        // Safe to `unwrap` as `num` is guaranteed to contain `'e'`
        let exp = num.split_off(num.find('e').unwrap());
        let (sign, exp) = exp
            .strip_prefix("e-")
            .map_or_else(|| ('+', &exp[1..]), |prefix| ('-', prefix));
        let _ = write!(num, "e{sign}{exp:0>exp_pad$}");

        format!("{num:>width$}")
    }

    fn serialize_polys(rec: Record, poly: Option<&Polynomial>) -> Option<String> {
        let Polynomial { corner, poly } = poly?;
        let mut out = Vec::new();
        out.extend([format!("{}/", rec.as_str())]);
        out.extend([format!("{corner}")]);
        let iter = poly.iter().map(|&num| Self::fmt_f64(num, 10, 9, 3));
        out.extend(iter);
        format_field_names(out).into()
    }

    fn serialize_errs(&self) -> Option<String> {
        let errs = self.err.as_deref()?;
        let mut out = String::new();
        errs.iter().enumerate().for_each(|(i, err)| {
            let _ = write!(
                &mut out,
                "{}/{},{:.10},{:.10},{:.10},{:.10}\r\n",
                Record::ERR.as_str(),
                i + 1,
                err[0],
                err[1],
                err[2],
                err[3],
            );
        });

        (!out.is_empty()).then_some(out)
    }

    fn _serialize_comment(&self, index: usize) -> Option<String> {
        let comments = self.comments.as_ref()?;
        comments.get(index).cloned()
    }
}

impl FromStr for ImageHeader {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut image_header = Self::empty();
        let starts = get_boundaries(&RECORD_REGEX, input);
        for (start, next) in starts {
            trace!(input = &input[start..start + 3]);
            let Ok(record_name) = (if input[start..].starts_with('!') {
                Ok(Record::Comment)
            } else {
                Record::from_str(&input[start..start + 3])
            }) else {
                warn!(
                    "Unrecognized record identifier: {}",
                    &input[start..start + 3]
                );
                continue;
            };
            parse(&input[start..next], record_name, &mut image_header)?;
        }
        Ok(image_header)
    }
}

fn format_field_names(v: Vec<String>) -> String {
    use core::fmt::Write;
    let mut out = String::new();
    // NOTE: we start at false since the record does not require a comma
    let mut first = false;
    for (i, field) in v.into_iter().enumerate() {
        let len = out
            .split_terminator('\n')
            .last()
            .map_or(out.len(), str::len);

        if len + field.len() + 1 /* accounts for the comma */ >= 80 {
            // if the line would be over 80 characters, add newline and 4 spaces
            let _ = write!(out, "\r\n    ");
            // NOTE: we skipped a line, so first is true
            first = true;
        }
        if first || i == 0 {
            out.extend([field]);
            first = false;
        } else {
            let _ = write!(out, ",{field}");
        }
        if i == 0 {
            first = true;
        }
    }
    out
}

fn place_optional_string(header: &mut impl Extend<String>, mut r: Option<String>) {
    if let Some(s) = r.take() {
        header.extend([s]);
    }
}

fn place_crr_string(header: &mut String, rec: &str) {
    let mut first = true;
    for word in rec.split_whitespace() {
        let len = header
            .split_terminator('\n')
            .last()
            .map_or(header.len(), str::len);

        if first {
            header.extend(["    "]);
        } else if len + word.len() + 1 /* accounts for the space */ >= 80 {
            // if the line will be over 80 characters, add newline and 4 spaces
            header.extend(["\r\n    "]);
            first = true;
        }
        if first {
            header.extend([word]);
            first = false;
        } else {
            // Two spaces after dot?
            if header.ends_with('.') {
                header.extend([' ']);
            }
            header.extend([" ", word]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_header_works() {
        let result: Result<ImageHeader, _> = TEST_1.parse();
        assert!(result.is_ok());
        let result: Result<ImageHeader, _> = HEADER_12221_1_KAP.parse();
        assert!(result.is_ok());
        dbg!(&result);
    }

    const TEST_1: &str = r"VER/3.0
    BSB/NA=Australia 3000000
        NU=,RA=625,480,DU=50
    KNP/SC=3000000,GD=,PR=LAMBERT CONFORMAL CONIC,PP=145.0
        PI=0.0,SP=Unknown,0,SK=0.0
        UN=METRES,SD=,DX=6000.0,DY=6000.0
    OST/1
    IFM/4
    RGB/1,199,231,252
    RGB/2,174,234,84
    RGB/3,255,254,206
    RGB/4,226,65,6
    DTM/0.0,0.0
    ";

    // Inconsistencies with header file means that sometimes there are two spaces
    // after the dot (in the CERTIFICATE OF AUTHENTICITY), and sometimes only one.
    // In these tests I've standardizes to two spaces
    // since that appears to be the default case
    const HEADER_12221_1_KAP: &str = r"!Copyright 1999, Maptech Inc.  All Rights Reserved.
CRR/CERTIFICATE OF AUTHENTICITY
    This electronic chart was produced under the authority of the National
    Oceanic and Atmospheric Administration (NOAA).  NOAA is the hydrographic
    office for the United States of America.  The digital data provided by NOAA
    from which this electronic chart was produced has been certified by NOAA
    for navigation.  'NOAA' and the NOAA emblem are registered trademarks of
    the National Oceanic and Atmospheric Administration.  'Maptech' and the
    Maptech emblem are registered trademarks of Maptech, Inc.  Copyright 1999
    Maptech, Inc.  All rights reserved.
VER/3.0
BSB/NA=CHESAPEAKE BAY ENTRANCE,NU=558,RA=11547,9767,DU=254
KNP/SC=80000,GD=NAD83,PR=MERCATOR,PP=37.083,PI=10.000,SP=,SK=0.0000000
    TA=90.0000000,UN=FEET,SD=MEAN LOWER LOW WATER,DX=8.00,DY=8.00
KNQ/EC=RF,GD=NARC,VC=UNKNOWN,SC=MLLW,PC=MC,P1=UNKNOWN,P2=37.083
    P3=NOT_APPLICABLE,P4=NOT_APPLICABLE,GC=NOT_APPLICABLE,RM=POLYNOMIAL
CED/SE=70,RE=01,ED=09/12/1998
NTM/NE=70.00,ND=10/30/1999,BF=ON,BD=10/26/1999
OST/1
IFM/4
RGB/1,0,0,0
RGB/2,255,255,255
RGB/3,209,221,239
RGB/4,221,234,247
RGB/5,244,232,193
RGB/6,214,219,201
RGB/7,219,181,242
RGB/8,114,114,114
RGB/9,188,188,188
RGB/10,150,176,155
RGB/11,94,153,193
RGB/12,219,73,150
DAY/1,0,0,0
DAY/2,255,255,255
DAY/3,185,210,240
DAY/4,214,227,245
DAY/5,238,223,161
DAY/6,181,181,123
DAY/7,219,181,242
DAY/8,114,114,114
DAY/9,188,188,188
DAY/10,38,212,84
DAY/11,37,138,191
DAY/12,219,73,150
DSK/1,0,0,0
DSK/2,128,128,128
DSK/3,93,105,120
DSK/4,107,114,123
DSK/5,119,112,81
DSK/6,91,91,62
DSK/7,110,91,121
DSK/8,57,57,57
DSK/9,94,94,94
DSK/10,19,106,42
DSK/11,19,69,96
DSK/12,110,37,75
NGT/1,55,55,55
NGT/2,0,0,0
NGT/3,0,0,38
NGT/4,0,0,28
NGT/5,30,21,13
NGT/6,0,23,12
NGT/7,17,0,0
NGT/8,35,35,35
NGT/9,25,25,25
NGT/10,1,50,1
NGT/11,0,55,55
NGT/12,64,0,64
NGR/1,0,0,0
NGR/2,255,0,0
NGR/3,204,0,0
NGR/4,230,0,0
NGR/5,220,0,0
NGR/6,175,0,0
NGR/7,213,0,0
NGR/8,114,0,0
NGR/9,188,0,0
NGR/10,120,0,0
NGR/11,104,0,0
NGR/12,145,0,0
GRY/1,0,0,0
GRY/2,255,255,255
GRY/3,199,199,199
GRY/4,226,226,226
GRY/5,215,215,215
GRY/6,175,175,175
GRY/7,203,203,203
GRY/8,114,114,114
GRY/9,188,188,188
GRY/10,120,120,120
GRY/11,104,104,104
GRY/12,138,138,138
PRC/1,0,0,0
PRC/2,255,255,255
PRC/3,181,206,240
PRC/4,213,230,250
PRC/5,247,239,181
PRC/6,181,191,123
PRC/7,219,181,242
PRC/8,114,114,114
PRC/9,188,188,188
PRC/10,38,212,84
PRC/11,37,138,191
PRC/12,219,73,150
PRG/1,0,0,0
PRG/2,255,255,255
PRG/3,204,204,204
PRG/4,230,230,230
PRG/5,222,222,222
PRG/6,175,175,175
PRG/7,213,213,213
PRG/8,114,114,114
PRG/9,188,188,188
PRG/10,120,120,120
PRG/11,104,104,104
PRG/12,145,145,145
REF/1,374,8790,36.8166861111,-76.4500000000
REF/2,374,695,37.4000111111,-76.4500000000
REF/3,4505,695,37.4000111111,-76.0783222222
REF/4,4505,579,37.4083444444,-76.0783222222
REF/5,4912,579,37.4083444444,-76.0416638889
REF/6,4912,695,37.4000111111,-76.0416638889
REF/7,5209,695,37.4000111111,-76.0149944444
REF/8,5209,668,37.4019444444,-76.0149944444
REF/9,5283,668,37.4019444444,-76.0083222222
REF/10,5283,695,37.4000111111,-76.0083222222
REF/11,7042,695,37.4000111111,-75.8499972222
REF/12,7042,490,37.4147250000,-75.8499972222
REF/13,7413,490,37.4147250000,-75.8166555556
REF/14,7413,695,37.4000111111,-75.8166555556
REF/15,11118,695,37.4000111111,-75.4833222222
REF/16,11118,8790,36.8166861111,-75.4833222222
REF/17,8080,8790,36.8166861111,-75.7566444444
REF/18,8080,8813,36.8150138889,-75.7566444444
REF/19,8006,8813,36.8150138889,-75.7633166667
REF/20,8006,8790,36.8166861111,-75.7633166667
REF/21,5153,8790,36.8166861111,-76.0199777778
REF/22,5153,8836,36.8133416667,-76.0199777778
REF/23,5060,8836,36.8133416667,-76.0283194444
REF/24,5060,8790,36.8166861111,-76.0283194444
REF/25,10933,8790,36.8166861111,-75.4999833333
REF/26,10933,8560,36.8333500000,-75.4999833333
REF/27,10933,6253,37.0000111111,-75.4999833333
REF/28,10933,3941,37.1666694444,-75.4999833333
REF/29,10933,1624,37.3333416667,-75.4999833333
REF/30,10933,695,37.4000111111,-75.4999833333
REF/31,9080,8790,36.8166861111,-75.6666500000
REF/32,9080,8560,36.8333500000,-75.6666500000
REF/33,9080,6253,37.0000111111,-75.6666500000
REF/34,9080,3941,37.1666694444,-75.6666500000
REF/35,9080,1624,37.3333416667,-75.6666500000
REF/36,9080,695,37.4000111111,-75.6666500000
REF/37,7228,8790,36.8166861111,-75.8333138889
REF/38,7228,8560,36.8333500000,-75.8333138889
REF/39,7228,6253,37.0000111111,-75.8333138889
REF/40,7228,3941,37.1666694444,-75.8333138889
REF/41,7228,1624,37.3333416667,-75.8333138889
REF/42,7228,490,37.4147250000,-75.8333138889
REF/43,5375,8790,36.8166861111,-75.9999805556
REF/44,5375,8560,36.8333500000,-75.9999805556
REF/45,5375,6253,37.0000111111,-75.9999805556
REF/46,5375,3941,37.1666694444,-75.9999805556
REF/47,5375,1624,37.3333416667,-75.9999805556
REF/48,5375,695,37.4000111111,-75.9999805556
REF/49,3523,8790,36.8166861111,-76.1666472222
REF/50,3523,8560,36.8333500000,-76.1666472222
REF/51,3523,6253,37.0000111111,-76.1666472222
REF/52,3523,3941,37.1666694444,-76.1666472222
REF/53,3523,1624,37.3333416667,-76.1666472222
REF/54,3523,695,37.4000111111,-76.1666472222
REF/55,1671,8790,36.8166861111,-76.3333138889
REF/56,1671,8560,36.8333500000,-76.3333138889
REF/57,1671,6253,37.0000111111,-76.3333138889
REF/58,1671,3941,37.1666694444,-76.3333138889
REF/59,1671,1624,37.3333416667,-76.3333138889
REF/60,1671,695,37.4000111111,-76.3333138889
REF/61,11118,8560,36.8333500000,-75.4833222222
REF/62,374,8560,36.8333500000,-76.4500000000
REF/63,11118,6253,37.0000111111,-75.4833222222
REF/64,374,6253,37.0000111111,-76.4500000000
REF/65,11118,3941,37.1666694444,-75.4833222222
REF/66,374,3941,37.1666694444,-76.4500000000
REF/67,11118,1624,37.3333416667,-75.4833222222
REF/68,374,1624,37.3333416667,-76.4500000000
PLY/1,36.8166666667,-76.4500000000
PLY/2,37.4000000000,-76.4500000000
PLY/3,37.4000000000,-76.0783333333
PLY/4,37.4083333333,-76.0783333333
PLY/5,37.4083333333,-76.0416666667
PLY/6,37.4000000000,-76.0416666667
PLY/7,37.4000000000,-76.0150000000
PLY/8,37.4019444444,-76.0150000000
PLY/9,37.4019444444,-76.0083333333
PLY/10,37.4000000000,-76.0083333333
PLY/11,37.4000000000,-75.8500000000
PLY/12,37.4147222222,-75.8500000000
PLY/13,37.4147222222,-75.8166666667
PLY/14,37.4000000000,-75.8166666667
PLY/15,37.4000000000,-75.4833333333
PLY/16,36.8166666667,-75.4833333333
PLY/17,36.8166666667,-75.7566666667
PLY/18,36.8150000000,-75.7566666667
PLY/19,36.8150000000,-75.7633333333
PLY/20,36.8166666667,-75.7633333333
PLY/21,36.8166666667,-76.0200000000
PLY/22,36.8133333333,-76.0200000000
PLY/23,36.8133333333,-76.0283333333
PLY/24,36.8166666667,-76.0283333333
DTM/0.0000000000,0.0000000000
CPH/0.0000000000
WPX/2,863264.4957,11420.23114,-85.46756208,1.913941167,-0.4081181078
    0.7362163163
WPY/2,390032.0953,69.56409751,-6745.589267,0.4669253601,0.0367153316
    -96.0547565
PWX/2,-76.48368342,8.999135076e-005,5.758392982e-009,-1.392859319e-012
    -2.377189159e-013,-3.432372134e-013
PWY/2,37.44988807,-3.111799225e-009,-7.171936009e-005,2.694372983e-013
    -1.725045227e-014,-3.594145418e-011
ERR/1,0.0395099814,0.1453734568,0.0000106128,0.0000035393
ERR/2,0.2568631181,0.1909729033,0.0000135084,0.0000230797
ERR/3,0.2741345061,0.0861261497,0.0000060346,0.0000246567
ERR/4,0.2686635828,0.0312145515,0.0000025324,0.0000241637
ERR/5,0.1452865095,0.0345549325,0.0000027703,0.0000130843
ERR/6,0.1399402606,0.0827745526,0.0000057959,0.0000126025
ERR/7,0.4574537708,0.0811248175,0.0000056780,0.0000411483
ERR/8,0.4562435934,0.1430947875,0.0000100925,0.0000410389
ERR/9,0.3011454875,0.1427864003,0.0000100706,0.0000270834
ERR/10,0.3023504002,0.0808159566,0.0000056561,0.0000271924
ERR/11,0.3723051299,0.0856845822,0.0000060026,0.0000335090
ERR/12,0.3806629821,0.0522721431,0.0000033386,0.0000342629
ERR/13,0.0373658667,0.0562993191,0.0000036259,0.0000033487
ERR/14,0.0455235020,0.0896937462,0.0000062887,0.0000040845
ERR/15,0.0838977644,0.1868453183,0.0000132139,0.0000075364
ERR/16,0.0966772515,0.1205425621,0.0000088179,0.0000086710
ERR/17,0.0824056347,0.0390765628,0.0000030177,0.0000074121
ERR/18,0.0818353321,0.1509363346,0.0000111398,0.0000073614
ERR/19,0.2449596538,0.1498203351,0.0000110606,0.0000220367
ERR/20,0.2455254028,0.0379601536,0.0000029384,0.0000220870
ERR/21,0.0428919326,0.0265733996,0.0000021336,0.0000038726
ERR/22,0.0436772242,0.2497880776,0.0000183422,0.0000039423
ERR/23,0.3327634719,0.2504511931,0.0000183899,0.0000299530
ERR/24,0.3319895661,0.0272354908,0.0000021812,0.0000298842
ERR/25,0.1121062566,0.1135798831,0.0000083225,0.0000101088
ERR/26,0.1193099468,0.2279827579,0.0000166458,0.0000107518
ERR/27,0.1688627289,0.1144423425,0.0000074904,0.0000151927
ERR/28,0.1775175278,0.1594268132,0.0000118409,0.0000159778
ERR/29,0.1452711190,0.3247700367,0.0000227542,0.0000130831
ERR/30,0.1209193080,0.1795258089,0.0000126926,0.0000108875
ERR/31,0.2409699573,0.0581966085,0.0000043778,0.0000216726
ERR/32,0.2348997396,0.2834680027,0.0000205979,0.0000211309
ERR/33,0.1966831937,0.0579372615,0.0000034646,0.0000177062
ERR/34,0.1993644420,0.1019019128,0.0000077412,0.0000179396
ERR/35,0.2429478428,0.3833148414,0.0000269279,0.0000218549
ERR/36,0.2718344884,0.1205730390,0.0000084892,0.0000244597
ERR/37,0.2687489734,0.0287539061,0.0000022840,0.0000241722
ERR/38,0.2736857376,0.3130126734,0.0000226990,0.0000246127
ERR/39,0.3005662362,0.0273727714,0.0000012898,0.0000270217
ERR/40,0.2865491295,0.0703176204,0.0000054925,0.0000257704
ERR/41,0.2316289257,0.4159190210,0.0000292507,0.0000208351
ERR/42,0.1899498524,0.0541574746,0.0000034733,0.0000170764
ERR/43,0.2969855982,0.0252507946,0.0000020389,0.0000267342
ERR/44,0.2931823065,0.3166177549,0.0000229515,0.0000263951
ERR/45,0.2776380441,0.0227478536,0.0000009636,0.0000250023
ERR/46,0.3029911981,0.0646728832,0.0000050923,0.0000272720
ERR/47,0.3692483939,0.4225836626,0.0000297248,0.0000332279
ERR/48,0.4072046331,0.0804882944,0.0000056329,0.0000366510
ERR/49,0.0309497656,0.0476879809,0.0000036427,0.0000027738
ERR/50,0.0336195848,0.2942825386,0.0000213551,0.0000030116
ERR/51,0.0378276110,0.0440632335,0.0000024863,0.0000033887
ERR/52,0.0011384097,0.0849684439,0.0000065412,0.0000001012
ERR/53,0.0764557781,0.4033080063,0.0000283500,0.0000068748
ERR/54,0.1189468519,0.0993559855,0.0000069781,0.0000107069
ERR/55,0.2525550645,0.0960654650,0.0000070947,0.0000227270
ERR/56,0.2540914112,0.2460070246,0.0000179103,0.0000228636
ERR/57,0.2469632012,0.0913189113,0.0000058574,0.0000222250
ERR/58,0.1989379527,0.1312043023,0.0000098384,0.0000179196
ERR/59,0.1100067729,0.3580920522,0.0000251268,0.0000099235
ERR/60,0.0629808645,0.1441639745,0.0000101715,0.0000056825
ERR/61,0.0893602518,0.2210098854,0.0000161497,0.0000080179
ERR/62,0.0387671976,0.1967704237,0.0000143974,0.0000034736
ERR/63,0.0386742240,0.1215171647,0.0000079940,0.0000034756
ERR/64,0.0538320955,0.1398415078,0.0000093186,0.0000048235
ERR/65,0.0288861982,0.1666035833,0.0000123517,0.0000025889
ERR/66,0.1097938997,0.1790129063,0.0000132479,0.0000098418
ERR/67,0.0599992857,0.3174913101,0.0000222359,0.0000053816
ERR/68,0.2066622965,0.3109975002,0.0000217692,0.0000185522
";
    // FIXME: ERR doesn't parse when there's no newline break at the end, WHY?
}
