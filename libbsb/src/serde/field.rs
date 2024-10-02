use super::{AMERICAN_DATE_FORMAT, DATE_FORMAT};
use crate::image::header::ImageHeader;
use strum::{EnumString, IntoStaticStr};

#[derive(Debug, IntoStaticStr, EnumString, PartialEq, Eq)]
pub enum Field {
    NA,
    NU,
    RA,
    DU,
    SC,
    // FIXME: how to handle two different GD fields? (in both KNQ & KNP)
    GD,
    PR,
    PP,
    PI,
    SK,
    TA,
    UN,
    SD,
    DX,
    DY,
    SE,
    RE,
    ED,
    NE,
    ND,
    BF,
    BD,
    // unknowns
    SP,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    P8,
    EC,
    VC,
    PC,
    GC,
    RM,
}
use core::fmt::Write;
use Field::{
    BD, BF, DU, DX, DY, EC, ED, GC, GD, NA, ND, NE, NU, P1, P2, P3, P4, P5, P6, P7, P8, PC, PI, PP,
    PR, RA, RE, RM, SC, SD, SE, SK, SP, TA, UN, VC,
};

fn write_optional_value<T: std::fmt::Display>(buf: &mut String, val: Option<T>) {
    if let Some(n) = val {
        let _ = write!(buf, "{n}");
    }
}

// TODO: remove
#[allow(clippy::option_option)]
fn write_nested_optional_value<T: std::fmt::Display>(buf: &mut String, val: Option<Option<T>>) {
    if let Some(n) = val {
        write_optional_value(buf, n);
    }
}

impl Field {
    pub fn as_str(&self) -> &'static str {
        Into::<&'static str>::into(self)
    }

    // Serializes the 'other' GD parameter which doesn't correspond to the GD implementation
    pub(crate) fn serialize_additional_gd(header: &ImageHeader) -> String {
        let mut f = format!("{}=", GD.as_str());
        write_nested_optional_value(
            &mut f,
            header
                .additional_parameters
                .as_ref()
                .map(|ap| ap.gd.as_ref()),
        );
        f
    }

    pub(crate) fn serialize_additional_sc(header: &ImageHeader) -> String {
        let mut f = format!("{}=", SC.as_str());
        write_nested_optional_value(
            &mut f,
            header
                .additional_parameters
                .as_ref()
                .map(|ap| ap.sc.as_ref()),
        );
        f
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn serialize_field(&self, header: &ImageHeader) -> String {
        // We cannot begin to construct the string in advance, unfortunately, because
        // some field names (i.e. P5-P8) are not placed at all if empty.
        // This is in contrary to the behavior with other field types,
        // which print the key but leave the value empty (i.e. 'SP=')
        let o = match self {
            NA => {
                let name = header.general_parameters.chart_name.as_ref();
                format!(
                    "{}={}",
                    self.as_str(),
                    name.as_ref().map_or_else(|| "", |s| s)
                )
            }
            NU => {
                let mut f = format!("{}=", self.as_str());
                write_optional_value(&mut f, header.general_parameters.chart_number.as_ref());
                f
            }
            RA => {
                let (w, h) = header.general_parameters.image_width_height;
                format!("{}={},{}", self.as_str(), w, h)
            }
            DU => {
                let mut f = format!("{}=", self.as_str());
                write_optional_value(&mut f, header.general_parameters.drawing_units.as_ref());
                f
            }
            SC => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .detailed_parameters
                        .as_ref()
                        .map(|dp| dp.chart_scale.as_ref()),
                );
                f
            }
            GD => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .detailed_parameters
                        .as_ref()
                        .map(|dp| dp.geodetic_datum_name.as_ref()),
                );
                f
            }
            PR => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .detailed_parameters
                        .as_ref()
                        .map(|dp| dp.projection_name.as_ref()),
                );
                f
            }
            PP => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .detailed_parameters
                        .as_ref()
                        .map(|dp| dp.projection_parameter.as_ref()),
                );
                f
            }
            PI => {
                let mut f = format!("{}=", self.as_str());
                // TODO: special write
                if let Some(Some(n)) = header
                    .detailed_parameters
                    .as_ref()
                    .map(|dp| dp.projection_interval.as_ref())
                {
                    let _ = write!(f, "{n:.3}");
                }
                f
            }
            SP => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header.detailed_parameters.as_ref().map(|dp| dp.sp.as_ref()),
                );
                f
            }
            SK => {
                let mut f = format!("{}=", self.as_str());
                // TODO: special write
                if let Some(Some(n)) = header
                    .detailed_parameters
                    .as_ref()
                    .map(|dp| dp.skew_angle.as_ref())
                {
                    let _ = write!(f, "{n:.7}");
                }
                f
            }
            TA => {
                let mut f = format!("{}=", self.as_str());
                // TODO: special write
                if let Some(Some(n)) = header
                    .detailed_parameters
                    .as_ref()
                    .map(|dp| dp.text_angle.as_ref())
                {
                    let _ = write!(f, "{n:.7}");
                }
                f
            }
            UN => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .detailed_parameters
                        .as_ref()
                        .map(|dp| dp.depth_units.as_ref()),
                );
                f
            }
            SD => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .detailed_parameters
                        .as_ref()
                        .map(|dp| dp.sounding_datum.as_ref()),
                );
                f
            }
            DX => {
                let mut f = format!("{}=", self.as_str());
                // TODO: special write
                if let Some(Some(n)) = header
                    .detailed_parameters
                    .as_ref()
                    .map(|dp| dp.x_resolution.as_ref())
                {
                    let _ = write!(f, "{n:.2}");
                }
                f
            }
            DY => {
                let mut f = format!("{}=", self.as_str());
                // TODO: special write
                if let Some(Some(n)) = header
                    .detailed_parameters
                    .as_ref()
                    .map(|dp| dp.y_resolution.as_ref())
                {
                    let _ = write!(f, "{n:.2}");
                }
                f
            }
            EC => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.ec.as_ref()),
                );
                f
            }
            VC => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.vc.as_ref()),
                );
                f
            }
            PC => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.pc.as_ref()),
                );
                f
            }
            P1 => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.p1.as_ref()),
                );
                f
            }
            P2 => {
                let mut f = format!("{}=", self.as_str());
                // TODO: special write
                if let Some(Some(n)) = header
                    .additional_parameters
                    .as_ref()
                    .map(|ap| ap.p2.as_ref())
                {
                    let _ = write!(f, "{n:.3}");
                }
                f
            }
            P3 => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.p3.as_ref()),
                );
                f
            }
            P4 => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.p4.as_ref()),
                );
                f
            }
            P5 => {
                if let Some(Some(n)) = header
                    .additional_parameters
                    .as_ref()
                    .map(|ap| ap.p5.as_ref())
                {
                    let mut f = format!("{}=", self.as_str());
                    let _ = write!(f, "{n}");
                    f
                } else {
                    String::new()
                }
            }
            P6 => {
                if let Some(Some(n)) = header
                    .additional_parameters
                    .as_ref()
                    .map(|ap| ap.p6.as_ref())
                {
                    let mut f = format!("{}=", self.as_str());
                    let _ = write!(f, "{n}");
                    f
                } else {
                    String::new()
                }
            }
            P7 => {
                if let Some(Some(n)) = header
                    .additional_parameters
                    .as_ref()
                    .map(|ap| ap.p7.as_ref())
                {
                    format!("{}={n}", self.as_str())
                } else {
                    String::new()
                }
            }
            P8 => {
                if let Some(Some(n)) = header
                    .additional_parameters
                    .as_ref()
                    .map(|ap| ap.p8.as_ref())
                {
                    let mut f = format!("{}=", self.as_str());
                    let _ = write!(f, "{n}");
                    f
                } else {
                    String::new()
                }
            }
            GC => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.gc.as_ref()),
                );
                f
            }
            RM => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header
                        .additional_parameters
                        .as_ref()
                        .map(|ap| ap.rm.as_ref()),
                );
                f
            }
            SE => {
                let mut f = format!("{}=", self.as_str());
                write_nested_optional_value(
                    &mut f,
                    header.ced.as_ref().map(|c| c.source_edition.as_ref()),
                );
                f
            }
            RE => {
                let mut f = format!("{}=", self.as_str());
                if let Some(Some(n)) = header.ced.as_ref().map(|ced| ced.raster_edition.as_ref()) {
                    let _ = write!(f, "{n:02}");
                }
                f
            }
            ED => {
                let mut f = format!("{}=", self.as_str());
                if let Some(Some(n)) = header.ced.as_ref().map(|ced| ced.edition_date.as_ref()) {
                    let _ = write!(f, "{}", n.format(DATE_FORMAT));
                }
                f
            }
            NE => {
                let mut f = format!("{}=", self.as_str());
                if let Some(Some(n)) = header.ntm.as_ref().map(|ntm| ntm.edition.as_ref()) {
                    let _ = write!(f, "{n:.2}");
                }
                f
            }
            ND => {
                let mut f = format!("{}=", self.as_str());
                if let Some(Some(n)) = header.ntm.as_ref().map(|ntm| ntm.date.as_ref()) {
                    let _ = write!(f, "{}", n.format(AMERICAN_DATE_FORMAT));
                }
                f
            }
            BF => {
                let mut f = format!("{}=", self.as_str());
                if let Some(Some(n)) = header.ntm.as_ref().map(|ntm| ntm.base_flag.as_ref()) {
                    let _ = write!(f, "{n}");
                }
                f
            }
            BD => {
                let mut f = format!("{}=", self.as_str());
                if let Some(Some(n)) = header.ntm.as_ref().map(|ntm| ntm.adn_record.as_ref()) {
                    let _ = write!(f, "{}", n.format(AMERICAN_DATE_FORMAT));
                }
                f
            }
        };
        o
    }
}
