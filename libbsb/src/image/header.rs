use bon::Builder;
use chrono::NaiveDate;

use crate::image::Depth;

/// Raw image header, holding all possible records and fields for KAP/BSB image files
///
/// Based on Maptech BSB File Format
/// Test Dataset Instructions for Raster Navigational Chart (RNC) dated 25 July 2001
// See the research materials [readme](../../../../research/readme.md) for more details
#[derive(Default, Debug, PartialEq, PartialOrd, Builder)]
#[non_exhaustive]
pub struct ImageHeader {
    /// Comments
    ///
    ///  
    pub comments: Option<Vec<String>>,

    /// CRR Copyright Record
    ///
    ///  
    pub copyright_record: Option<String>,

    /// VER Format Version (number)
    ///
    /// Version number of BSB format e.g. 1, 2.0, 3.0, 3.07, 4.0
    ///
    ///  
    pub version: Option<f32>,

    /// BSB General Parameters
    ///
    /// BSB    (or NOS for older GEO/NOS or GEO/NO1 files)
    ///  RA=width,height - width and height of raster image data in pixels
    ///  NA=Name given to the BSB chart (can represent more than one .KAP)
    ///  NU=Number of chart (especially when more than one chart is grouped or tiled together)
    ///  DU=Drawing Units in pixels/inch (same as DPI resolution) e.g. 50, 150, 175, 254, 300   
    ///
    ///  
    pub general_parameters: GeneralParameters,

    /// KNP Detailed Parameters
    ///
    /// KNP
    ///   SC=Scale e.g. 25000
    ///   GD=Geodetic Datum e.g. NAD83, WGS84
    ///   PR=Projection e.g. LAMBERT CONFORMAL CONIC, MERCATOR
    ///   PP=Projection Parameter (value depends upon Projection) e.g. 135.0
    ///   PI=? e.g. 0.0, 0.033333, 0.083333, 2.0
    ///   SP=?
    ///   SK=Skew angle? e.g. 0.0
    ///   TA=? e.g. 90
    ///   UN=Units (for DX, DY and others) e.g. METRES, FATHOMS
    ///   SD=Sounding Datum e.g. MEAN LOWER LOW WATER, HHWLT
    ///   DX=distance (approx.) covered by one pixel in X direction
    ///   DY=distance (approx.) covered by one pixel in Y direction   
    ///
    ///
    pub detailed_parameters: Option<DetailedParameters>,

    /// KNQ
    ///  P1=...,P2=...
    ///  P3=...,P4=...
    ///  P5=...,P6=...
    pub additional_parameters: Option<AdditionalParameters>,

    /// identifier: CED
    pub ced: Option<ChartEditionParameters>,

    /// identifier: NTM
    pub ntm: Option<NTMRecord>,

    /// OST Offset values section
    ///
    /// OST Offset Strip image lines (number of image rows per entry in the index table) e.g. 1
    pub ost: Option<usize>,

    /// IFM Compression type
    ///
    /// Depth of the colormap (bits per pixel). BSB supports 1 through 7 (2 through 127 max colors)?
    pub ifm: Depth,

    /// RGB Default color palette
    ///
    /// IFM Entries in the raster colormap of the form index,red,green,blue (index 0 is not used in BSB)
    pub rgb: Option<Vec<(u8, u8, u8)>>,

    /// identifier: DAY
    pub day: Option<Vec<(u8, u8, u8)>>,

    /// identifier: DSK
    pub dsk: Option<Vec<(u8, u8, u8)>>,

    /// identifier: NGT
    /// Night Color palette
    pub ngt: Option<Vec<(u8, u8, u8)>>,

    /// identifier: NGR
    pub ngr: Option<Vec<(u8, u8, u8)>>,

    /// identifier: GRY
    pub gry: Option<Vec<(u8, u8, u8)>>,

    /// Optional palette
    /// identifier: PRC
    pub prc: Option<Vec<(u8, u8, u8)>>,

    /// Optional Grey palette
    /// identifier: PRG
    pub prg: Option<Vec<(u8, u8, u8)>>,

    /// REF Mechanism to allow geographical positions to be converted to RNC (pixel) coordinates
    /// REF - Registration reference points (at least 3 points)
    pub reference_point_record: Option<Vec<Ref>>,

    /// CPH Phase shift value
    pub phase_shift: Option<f64>,

    /// WPX Polynomial L to X
    pub wpx: Option<Polynomial>,

    /// PWX Polynomial X to L
    pub pwx: Option<Polynomial>,

    /// WPY Polynomial L to Y
    pub wpy: Option<Polynomial>,

    /// PWY Polynomial Y to L
    pub pwy: Option<Polynomial>,

    /// ERR Error record
    pub err: Option<Vec<[f64; 4]>>,

    /// PLY Border Polygon Record
    /// PLY - Border polygon of the map within the raster image, given in chart datum lat/long
    pub ply: Option<Vec<(f64, f64)>>,

    /// DTM Data Shift Record
    /// DTM - Datum's northing and easting in floating point seconds (appears to be optional for many charts)
    pub dtm: Option<(f64, f64)>,
}

/// Represents a Polynomial used to map L <-> X || L <-> Y
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Polynomial {
    /// values appear to be 1-4 going from 1=SW 2=NW 3=NE 4=SE
    // TODO: unclear what this is - perhaps the corner of the chart?
    pub corner: usize,

    /// The coordinates used by the polynomial
    pub poly: [f64; 6],
}

impl Polynomial {
    pub(crate) const fn new(corner: usize, poly: [f64; 6]) -> Self {
        Self { corner, poly }
    }
}

impl ImageHeader {
    pub(crate) fn empty() -> Self {
        Self::default()
    }
}

/// identifier: BSB
#[derive(Builder, Default, Debug, Clone, Eq, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct GeneralParameters {
    /// identifier: NA
    /// RNC name
    pub chart_name: Option<String>,

    /// identifier: NU
    /// RNC number
    pub chart_number: Option<String>,

    /// identifier: RA
    pub image_width_height: (u16, u16),

    /// identifier: DU
    /// Pixel resolution of the image file
    pub drawing_units: Option<usize>,
}

/// identifier: KNP
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct DetailedParameters {
    /// identifier: SC
    /// Chart scale
    pub chart_scale: Option<usize>,

    /// identifier: GD
    /// Geodetic Datum name
    pub geodetic_datum_name: Option<String>,

    /// identifier: PR
    /// Projection name
    pub projection_name: Option<String>,

    /// identifier: PP
    pub projection_parameter: Option<f32>,

    /// identifier: PI
    pub projection_interval: Option<f32>,

    /// identifier: SP
    // Unknown
    pub sp: Option<String>,

    /// identifier: SK
    /// Orientation of the north
    /// Skew Angel in the original [?sic]
    pub skew_angle: Option<f32>,

    /// identifier: TA
    pub text_angle: Option<f32>,

    /// identifier: UN
    /// Depth and height units
    pub depth_units: Option<String>,

    /// identifier: SD
    /// Vertical datums
    pub sounding_datum: Option<String>,

    /// identifier: DX
    pub x_resolution: Option<f32>,

    /// identifier: DY
    pub y_resolution: Option<f32>,
}

/// identifier: KNQ
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct AdditionalParameters {
    /// Optional * P1=?
    pub p1: Option<String>,
    /// P2 - the same as KNP PP for Mercator and Transverse Mercator projection
    pub p2: Option<f32>,
    /// P3= scale factor for Transverse Mercator; 1st standard parallel for lambert conic
    pub p3: Option<String>,
    /// P4= 0 for Transverse Mercator; 2nd standard parallel for lambert conic
    pub p4: Option<String>,
    /// P5= Central meridian for Transverse Mercator and lambert conic
    pub p5: Option<String>,
    /// P6= origin of latitude for Transverse Mercator and lambert conic
    pub p6: Option<String>,
    /// P7 = `+x_0` for Transverse Mercator and lambert conic
    pub p7: Option<String>,
    /// P8 = `+y_0` for Transverse Mercator and lambert conic
    pub p8: Option<String>,
    /// Unknown
    // TODO: often seen with the following value: 'EC=?'
    pub ec: Option<String>,
    /// Unknown
    //  NOTE: not same GD between KNP and KNQ
    pub gd: Option<String>,
    /// Unknown
    pub vc: Option<String>,
    /// Unknown
    pub sc: Option<String>,
    /// Unknown
    pub gc: Option<String>,
    /// Unknown
    pub rm: Option<String>,
    /// PC=?. Set to TC for Transverse Mercator
    pub pc: Option<String>,
}

/// identifier: CED
#[derive(Builder, Default, Debug, Clone, Eq, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct ChartEditionParameters {
    /// identifier: SE
    pub source_edition: Option<usize>,
    /// identifier: RE
    pub raster_edition: Option<usize>,
    /// identifier: ED
    pub edition_date: Option<NaiveDate>,
}

/// identifier: NTM
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct NTMRecord {
    /// identifier: NE
    /// NTM edition
    pub edition: Option<f32>,
    /// identifier: ND
    /// NTM date
    pub date: Option<NaiveDate>,
    /// identifier: BF
    /// Base flag
    pub base_flag: Option<String>,
    /// identifier: BD
    /// ADN Record
    pub adn_record: Option<NaiveDate>,
}

/// identifier: REF
/// Used to map pixel coordinates to geographical coordinates
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct Ref {
    /// The given pixel coordinates
    pub pixels: (usize, usize),
    /// The corresponding geographical coordinates
    pub coords: (f64, f64),
}
